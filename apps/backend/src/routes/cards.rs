use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use std::sync::Arc;

use crate::models::{Card, CreateCard, UpdateCard};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_cards).post(create_card))
        .route("/{id}", get(get_card).patch(update_card).delete(delete_card))
}

async fn list_cards(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Card>>, StatusCode> {
    let cards = sqlx::query_as::<_, Card>(
        "SELECT * FROM cards ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch cards: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(cards))
}

async fn create_card(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCard>,
) -> Result<(StatusCode, Json<Card>), StatusCode> {
    let card = Card::new(payload.front, payload.back, payload.source_id);

    sqlx::query(
        r#"
        INSERT INTO cards (id, front, back, source_id, ease_factor, interval, repetitions, next_review, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&card.id)
    .bind(&card.front)
    .bind(&card.back)
    .bind(&card.source_id)
    .bind(card.ease_factor)
    .bind(card.interval)
    .bind(card.repetitions)
    .bind(&card.next_review)
    .bind(&card.created_at)
    .bind(&card.updated_at)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create card: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(card)))
}

async fn get_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Card>, StatusCode> {
    let card = sqlx::query_as::<_, Card>(
        "SELECT * FROM cards WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch card: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(card))
}

async fn update_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateCard>,
) -> Result<Json<Card>, StatusCode> {
    // First fetch existing card
    let existing = sqlx::query_as::<_, Card>(
        "SELECT * FROM cards WHERE id = ?"
    )
    .bind(&id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch card: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::NOT_FOUND)?;

    let front = payload.front.unwrap_or(existing.front);
    let back = payload.back.unwrap_or(existing.back);
    let updated_at = chrono::Utc::now();

    sqlx::query(
        "UPDATE cards SET front = ?, back = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&front)
    .bind(&back)
    .bind(&updated_at)
    .bind(&id)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update card: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let card = sqlx::query_as::<_, Card>(
        "SELECT * FROM cards WHERE id = ?"
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch updated card: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(card))
}

async fn delete_card(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM cards WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete card: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}
