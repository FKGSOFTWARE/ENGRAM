//! Unified Error Handling
//!
//! Provides a consistent error type hierarchy for the ENGRAM backend.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;

/// Application error type
#[derive(Error, Debug)]
pub enum AppError {
    /// Database errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Not found errors
    #[error("{resource} not found: {id}")]
    NotFound { resource: &'static str, id: String },

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// LLM provider errors
    #[error("LLM error: {0}")]
    Llm(String),

    /// External service errors
    #[error("External service error: {0}")]
    External(String),

    /// PDF processing errors
    #[error("PDF processing error: {0}")]
    PdfProcessing(String),

    /// Internal server errors
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Error response body
#[derive(Serialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Error code for client handling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("DATABASE_ERROR"),
                    "A database error occurred".to_string(),
                )
            }
            AppError::NotFound { resource, id } => (
                StatusCode::NOT_FOUND,
                Some("NOT_FOUND"),
                format!("{} with id '{}' not found", resource, id),
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                Some("VALIDATION_ERROR"),
                msg.clone(),
            ),
            AppError::Auth(msg) => (
                StatusCode::UNAUTHORIZED,
                Some("AUTH_ERROR"),
                msg.clone(),
            ),
            AppError::Llm(msg) => {
                tracing::error!("LLM error: {}", msg);
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Some("LLM_ERROR"),
                    msg.clone(),
                )
            }
            AppError::External(msg) => {
                tracing::error!("External service error: {}", msg);
                (
                    StatusCode::BAD_GATEWAY,
                    Some("EXTERNAL_ERROR"),
                    msg.clone(),
                )
            }
            AppError::PdfProcessing(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Some("PDF_ERROR"),
                msg.clone(),
            ),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Some("INTERNAL_ERROR"),
                    "An internal error occurred".to_string(),
                )
            }
        };

        let body = ErrorResponse {
            error: message,
            code: code.map(String::from),
            details: None,
        };

        (status, Json(body)).into_response()
    }
}

// Convenience conversion from anyhow::Error
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Convenience conversions for common error types
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::External(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Validation(format!("JSON parsing error: {}", err))
    }
}

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;

/// Helper trait for converting Option to AppError::NotFound
#[allow(dead_code)]
pub trait OptionExt<T> {
    fn ok_or_not_found(self, resource: &'static str, id: impl Into<String>) -> AppResult<T>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, resource: &'static str, id: impl Into<String>) -> AppResult<T> {
        self.ok_or_else(|| AppError::NotFound {
            resource,
            id: id.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_found_error() {
        let err = AppError::NotFound {
            resource: "Card",
            id: "abc-123".to_string(),
        };
        assert!(err.to_string().contains("Card"));
        assert!(err.to_string().contains("abc-123"));
    }

    #[test]
    fn test_option_ext() {
        let none: Option<i32> = None;
        let result = none.ok_or_not_found("Item", "test-id");
        assert!(matches!(result, Err(AppError::NotFound { .. })));

        let some = Some(42);
        let result = some.ok_or_not_found("Item", "test-id");
        assert_eq!(result.unwrap(), 42);
    }
}
