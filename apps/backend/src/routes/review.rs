use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::llm::{CardEvaluation, EvaluationRequest};
use crate::models::{Card, Review, SubmitReview};
use crate::services::spaced_repetition::{self, FSRSState};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/next", get(get_next_cards))
        .route("/submit", post(submit_review))
        .route("/evaluate", post(evaluate_answer))
}

#[derive(Deserialize)]
pub struct NextCardsQuery {
    limit: Option<i32>,
}

async fn get_next_cards(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<NextCardsQuery>,
) -> Result<Json<Vec<Card>>, StatusCode> {
    let limit = query.limit.unwrap_or(10);
    let now = Utc::now();

    let cards = sqlx::query_as::<_, Card>(
        r#"
        SELECT * FROM cards
        WHERE next_review <= ?
        ORDER BY next_review ASC
        LIMIT ?
        "#,
    )
    .bind(&now)
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch review queue: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(cards))
}

#[derive(Deserialize)]
pub struct EvaluateRequest {
    pub card_id: String,
    pub user_answer: String,
}

#[derive(Serialize)]
pub struct EvaluateResponse {
    pub evaluation: Option<CardEvaluation>,
    pub error: Option<String>,
}

async fn evaluate_answer(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<EvaluateRequest>,
) -> Result<Json<EvaluateResponse>, StatusCode> {
    // Fetch the card
    let card = sqlx::query_as::<_, Card>("SELECT * FROM cards WHERE id = ?")
        .bind(&payload.card_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch card: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Check if LLM is available
    if !state.llm.has_available_provider() {
        return Ok(Json(EvaluateResponse {
            evaluation: None,
            error: Some("No LLM providers configured".to_string()),
        }));
    }

    // Evaluate the answer
    let request = EvaluationRequest {
        card_front: card.front,
        card_back: card.back,
        user_answer: payload.user_answer,
    };

    match state.llm.evaluate_answer(request).await {
        Ok(evaluation) => Ok(Json(EvaluateResponse {
            evaluation: Some(evaluation),
            error: None,
        })),
        Err(e) => Ok(Json(EvaluateResponse {
            evaluation: None,
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Serialize)]
pub struct SubmitResponse {
    pub card: Card,
    pub evaluation: Option<CardEvaluation>,
}

async fn submit_review(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SubmitReview>,
) -> Result<Json<SubmitResponse>, StatusCode> {
    // Fetch the card
    let card = sqlx::query_as::<_, Card>("SELECT * FROM cards WHERE id = ?")
        .bind(&payload.card_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch card: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Optionally evaluate with LLM if user provided an answer
    let mut evaluation: Option<CardEvaluation> = None;
    let mut llm_evaluation_json: Option<String> = None;

    if let Some(ref user_answer) = payload.user_answer {
        if state.llm.has_available_provider() && !user_answer.is_empty() {
            let request = EvaluationRequest {
                card_front: card.front.clone(),
                card_back: card.back.clone(),
                user_answer: user_answer.clone(),
            };

            match state.llm.evaluate_answer(request).await {
                Ok(eval) => {
                    llm_evaluation_json = serde_json::to_string(&eval).ok();
                    evaluation = Some(eval);
                }
                Err(e) => {
                    tracing::warn!("LLM evaluation failed: {}", e);
                }
            }
        }
    }

    // Determine rating (use LLM suggestion if available and no rating provided)
    let rating = payload.rating;

    // Build FSRS state from card
    let fsrs_state = FSRSState {
        stability: card.stability,
        difficulty: card.difficulty,
        reps: card.repetitions,
        lapses: card.lapses,
        last_review: card.last_review,
    };

    // Calculate new scheduling using FSRS directly
    let (new_interval, new_stability, new_difficulty, new_reps, new_lapses, next_review) =
        spaced_repetition::calculate_fsrs(&fsrs_state, rating, None);

    // Map difficulty back to ease_factor for backward compatibility
    let new_ease_factor = (2.5 - (new_difficulty - 1.0) * 0.17).clamp(1.3, 3.0);

    // Update card with new scheduling (both FSRS and legacy fields)
    let updated_at = Utc::now();
    sqlx::query(
        r#"
        UPDATE cards
        SET interval = ?, ease_factor = ?, repetitions = ?,
            stability = ?, difficulty = ?, lapses = ?,
            next_review = ?, last_review = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(new_interval)
    .bind(new_ease_factor)
    .bind(new_reps)
    .bind(new_stability)
    .bind(new_difficulty)
    .bind(new_lapses)
    .bind(&next_review)
    .bind(&updated_at) // last_review is now
    .bind(&updated_at) // updated_at
    .bind(&card.id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update card scheduling: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Record the review
    let review = Review::new(payload.card_id.clone(), rating, payload.user_answer.clone());
    sqlx::query(
        r#"
        INSERT INTO reviews (id, card_id, rating, user_answer, llm_evaluation, reviewed_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&review.id)
    .bind(&review.card_id)
    .bind(review.rating)
    .bind(&review.user_answer)
    .bind(&llm_evaluation_json)
    .bind(&review.reviewed_at)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to record review: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Return updated card
    let updated_card = sqlx::query_as::<_, Card>("SELECT * FROM cards WHERE id = ?")
        .bind(&payload.card_id)
        .fetch_one(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch updated card: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(SubmitResponse {
        card: updated_card,
        evaluation,
    }))
}
