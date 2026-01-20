use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewRating {
    Again = 0,  // Complete blackout
    Hard = 1,   // Struggled but recalled
    Good = 2,   // Correct with effort
    Easy = 3,   // Effortless recall
}

impl From<i32> for ReviewRating {
    fn from(value: i32) -> Self {
        match value {
            0 => ReviewRating::Again,
            1 => ReviewRating::Hard,
            2 => ReviewRating::Good,
            3 => ReviewRating::Easy,
            _ => ReviewRating::Good,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: String,
    pub card_id: String,
    pub rating: i32,
    pub user_answer: Option<String>,
    pub llm_evaluation: Option<String>,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SubmitReview {
    pub card_id: String,
    pub rating: ReviewRating,
    pub user_answer: Option<String>,
}

impl Review {
    pub fn new(card_id: String, rating: ReviewRating, user_answer: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            card_id,
            rating: rating as i32,
            user_answer,
            llm_evaluation: None,
            reviewed_at: Utc::now(),
        }
    }
}
