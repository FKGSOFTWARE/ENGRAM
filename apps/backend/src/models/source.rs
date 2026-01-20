use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Manual,
    Text,
    Pdf,
    Url,
}

impl From<String> for SourceType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "manual" => SourceType::Manual,
            "text" => SourceType::Text,
            "pdf" => SourceType::Pdf,
            "url" => SourceType::Url,
            _ => SourceType::Manual,
        }
    }
}

impl From<SourceType> for String {
    fn from(value: SourceType) -> Self {
        match value {
            SourceType::Manual => "manual".to_string(),
            SourceType::Text => "text".to_string(),
            SourceType::Pdf => "pdf".to_string(),
            SourceType::Url => "url".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Source {
    pub id: String,
    pub source_type: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub content_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Source {
    pub fn new(source_type: SourceType, title: Option<String>, url: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_type: source_type.into(),
            title,
            url,
            content_hash: None,
            created_at: Utc::now(),
        }
    }
}
