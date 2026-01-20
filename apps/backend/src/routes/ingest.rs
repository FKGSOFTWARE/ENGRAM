use axum::{
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::llm::{GenerationRequest, GenerationStyle};
use crate::models::{Card, Source, SourceType};
use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/text", post(ingest_text))
        .route("/url", post(ingest_url))
        .route("/confirm", post(confirm_cards))
}

// Staged cards awaiting confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedCard {
    pub temp_id: String,
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
    pub approved: bool,
}

#[derive(Debug, Deserialize)]
pub struct IngestTextRequest {
    pub content: String,
    pub title: Option<String>,
    pub max_cards: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub source_id: String,
    pub staged_cards: Vec<StagedCard>,
    pub error: Option<String>,
}

async fn ingest_text(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestTextRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    // Check LLM availability
    if !state.llm.has_available_provider() {
        return Ok(Json(IngestResponse {
            source_id: String::new(),
            staged_cards: Vec::new(),
            error: Some("No LLM provider available for card generation".to_string()),
        }));
    }

    // Create source record
    let source = Source::new(
        SourceType::Text,
        payload.title.clone(),
        None,
    );

    sqlx::query(
        "INSERT INTO sources (id, source_type, title, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&source.id)
    .bind(&source.source_type)
    .bind(&source.title)
    .bind(&source.created_at)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create source: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate cards using LLM
    let request = GenerationRequest {
        content: payload.content,
        max_cards: payload.max_cards.unwrap_or(10),
        style: GenerationStyle::QuestionAnswer,
    };

    match state.llm.generate_cards(request).await {
        Ok(cards) => {
            let staged: Vec<StagedCard> = cards
                .into_iter()
                .map(|c| StagedCard {
                    temp_id: Uuid::new_v4().to_string(),
                    front: c.front,
                    back: c.back,
                    tags: c.tags,
                    approved: true, // Default to approved
                })
                .collect();

            Ok(Json(IngestResponse {
                source_id: source.id,
                staged_cards: staged,
                error: None,
            }))
        }
        Err(e) => Ok(Json(IngestResponse {
            source_id: source.id,
            staged_cards: Vec::new(),
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct IngestUrlRequest {
    pub url: String,
    pub max_cards: Option<usize>,
}

async fn ingest_url(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<IngestUrlRequest>,
) -> Result<Json<IngestResponse>, StatusCode> {
    // Check LLM availability
    if !state.llm.has_available_provider() {
        return Ok(Json(IngestResponse {
            source_id: String::new(),
            staged_cards: Vec::new(),
            error: Some("No LLM provider available for card generation".to_string()),
        }));
    }

    // Fetch URL content
    let client = reqwest::Client::new();
    let response = client
        .get(&payload.url)
        .header("User-Agent", "Engram/1.0")
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch URL: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    let html = response.text().await.map_err(|e| {
        tracing::error!("Failed to read URL content: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    // Simple HTML to text conversion (strip tags)
    let text = strip_html_tags(&html);

    // Create source record
    let source = Source::new(
        SourceType::Url,
        Some(extract_title(&html).unwrap_or_else(|| payload.url.clone())),
        Some(payload.url),
    );

    sqlx::query(
        "INSERT INTO sources (id, source_type, title, url, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&source.id)
    .bind(&source.source_type)
    .bind(&source.title)
    .bind(&source.url)
    .bind(&source.created_at)
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create source: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Generate cards using LLM
    let request = GenerationRequest {
        content: text,
        max_cards: payload.max_cards.unwrap_or(10),
        style: GenerationStyle::QuestionAnswer,
    };

    match state.llm.generate_cards(request).await {
        Ok(cards) => {
            let staged: Vec<StagedCard> = cards
                .into_iter()
                .map(|c| StagedCard {
                    temp_id: Uuid::new_v4().to_string(),
                    front: c.front,
                    back: c.back,
                    tags: c.tags,
                    approved: true,
                })
                .collect();

            Ok(Json(IngestResponse {
                source_id: source.id,
                staged_cards: staged,
                error: None,
            }))
        }
        Err(e) => Ok(Json(IngestResponse {
            source_id: source.id,
            staged_cards: Vec::new(),
            error: Some(e.to_string()),
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmCardsRequest {
    pub source_id: String,
    pub cards: Vec<StagedCard>,
}

#[derive(Debug, Serialize)]
pub struct ConfirmCardsResponse {
    pub created_count: i32,
    pub cards: Vec<Card>,
}

async fn confirm_cards(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfirmCardsRequest>,
) -> Result<Json<ConfirmCardsResponse>, StatusCode> {
    let mut created_cards: Vec<Card> = Vec::new();

    for staged in payload.cards.iter().filter(|c| c.approved) {
        let card = Card::new(
            staged.front.clone(),
            staged.back.clone(),
            Some(payload.source_id.clone()),
        );

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

        created_cards.push(card);
    }

    Ok(Json(ConfirmCardsResponse {
        created_count: created_cards.len() as i32,
        cards: created_cards,
    }))
}

// Helper function to strip HTML tags
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let in_script = false;
    let in_style = false;

    for c in html.chars() {
        match c {
            '<' => {
                in_tag = true;
            }
            '>' => {
                in_tag = false;
            }
            _ if in_tag => {
                // Check for script/style tags
                continue;
            }
            _ if !in_script && !in_style => {
                result.push(c);
            }
            _ => {}
        }
    }

    // Clean up whitespace
    result
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

// Helper function to extract title from HTML
fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    if let Some(start) = lower.find("<title>") {
        if let Some(end) = lower.find("</title>") {
            let title_start = start + 7;
            if title_start < end {
                return Some(html[title_start..end].trim().to_string());
            }
        }
    }
    None
}
