use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Card {
    pub id: String,
    pub front: String,
    pub back: String,
    pub source_id: Option<String>,
    // Legacy SM-2 field (kept for backward compatibility)
    pub ease_factor: f64,
    pub interval: i32,
    pub repetitions: i32,
    // FSRS fields
    pub stability: f64,
    pub difficulty: f64,
    pub lapses: i32,
    // Scheduling
    pub next_review: DateTime<Utc>,
    pub last_review: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCard {
    pub front: String,
    pub back: String,
    pub source_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCard {
    pub front: Option<String>,
    pub back: Option<String>,
}

impl Card {
    pub fn new(front: String, back: String, source_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            front,
            back,
            source_id,
            ease_factor: 2.5,  // Legacy SM-2 field
            interval: 0,
            repetitions: 0,
            stability: 0.0,   // FSRS: initial stability
            difficulty: 5.0,  // FSRS: initial difficulty (middle of 1-10 scale)
            lapses: 0,
            next_review: now,
            last_review: None,
            created_at: now,
            updated_at: now,
        }
    }
}
