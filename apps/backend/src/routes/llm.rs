use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/generate", post(generate_text))
}

#[derive(Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    #[serde(default = "default_max_tokens")]
    #[allow(dead_code)]
    pub max_tokens: usize,
}

fn default_max_tokens() -> usize {
    256
}

#[derive(Serialize)]
pub struct GenerateResponse {
    pub text: String,
    pub error: Option<String>,
}

/// Generate text using the configured LLM provider.
/// Used by conversational mode for intro/outro/question generation.
async fn generate_text(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, StatusCode> {
    if !state.llm.has_available_provider() {
        return Ok(Json(GenerateResponse {
            text: String::new(),
            error: Some("No LLM providers configured".to_string()),
        }));
    }

    match generate_raw_text(&state, &payload.prompt).await {
        Ok(text) => Ok(Json(GenerateResponse { text, error: None })),
        Err(e) => Ok(Json(GenerateResponse {
            text: String::new(),
            error: Some(e),
        })),
    }
}

/// Generate raw text from a prompt using available LLM providers
async fn generate_raw_text(state: &AppState, prompt: &str) -> Result<String, String> {
    // Try each provider in order
    let providers = state.llm.available_providers();

    if providers.is_empty() {
        return Err("No providers available".to_string());
    }

    // Use Gemini's generate_content directly if available
    if let Ok(key) = std::env::var("GEMINI_API_KEY") {
        if !key.is_empty() {
            return generate_with_gemini(&key, prompt).await;
        }
    }

    // Fallback to OpenAI
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            return generate_with_openai(&key, prompt).await;
        }
    }

    // Fallback to Anthropic
    if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
        if !key.is_empty() {
            return generate_with_anthropic(&key, prompt).await;
        }
    }

    Err("No provider could generate text".to_string())
}

async fn generate_with_gemini(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key={}",
        api_key
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }]
    });

    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API error: {}", error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text)
}

async fn generate_with_openai(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "gpt-4o-mini",
        "messages": [
            {"role": "user", "content": prompt}
        ],
        "max_tokens": 256
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error: {}", error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let text = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text)
}

async fn generate_with_anthropic(api_key: &str, prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 256,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Anthropic API error: {}", error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let text = json["content"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(text)
}
