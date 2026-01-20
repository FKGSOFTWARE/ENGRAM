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
    pub ease_factor: f64,
    pub interval: i32,
    pub repetitions: i32,
    pub next_review: DateTime<Utc>,
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
            ease_factor: 2.5,  // Default SM-2 ease factor
            interval: 0,
            repetitions: 0,
            next_review: now,
            created_at: now,
            updated_at: now,
        }
    }
}
