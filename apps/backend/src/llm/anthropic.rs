use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::provider::{
    CardEvaluation, EvaluationRequest, GeneratedCard, GenerationRequest, LlmError, LlmProvider,
    SuggestedRating, EVALUATION_SYSTEM_PROMPT, GENERATION_SYSTEM_PROMPT,
};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: "claude-3-5-haiku-latest".to_string(),
        }
    }

    /// Configure a custom model (builder pattern for future flexibility)
    #[allow(dead_code)]
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<AnthropicMessage>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
}

#[derive(Debug, Deserialize)]
struct EvaluationResponse {
    is_correct: bool,
    score: f32,
    feedback: String,
    suggested_rating: String,
}

#[derive(Debug, Deserialize)]
struct GenerationResponse {
    cards: Vec<GeneratedCardResponse>,
}

#[derive(Debug, Deserialize)]
struct GeneratedCardResponse {
    front: String,
    back: String,
    tags: Vec<String>,
}

fn extract_json(text: &str) -> &str {
    // Find JSON in the response (handle markdown code blocks)
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            return &text[start..=end];
        }
    }
    text
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &'static str {
        "Anthropic"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn evaluate_answer(&self, request: EvaluationRequest) -> Result<CardEvaluation, LlmError> {
        let prompt = format!(
            "Card Question: {}\nExpected Answer: {}\nStudent Answer: {}\n\nRespond with JSON only.",
            request.card_front, request.card_back, request.user_answer
        );

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            system: EVALUATION_SYSTEM_PROMPT.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post(format!("{}/messages", ANTHROPIC_API_BASE))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);
            return Err(LlmError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        if response.status() == 401 {
            return Err(LlmError::InvalidApiKey);
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::RequestFailed(error_text));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let text = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| LlmError::ParseError("No response content".to_string()))?;

        let json_text = extract_json(&text);
        let eval: EvaluationResponse =
            serde_json::from_str(json_text).map_err(|e| LlmError::ParseError(e.to_string()))?;

        let suggested_rating = match eval.suggested_rating.as_str() {
            "again" => SuggestedRating::Again,
            "hard" => SuggestedRating::Hard,
            "good" => SuggestedRating::Good,
            "easy" => SuggestedRating::Easy,
            _ => SuggestedRating::Good,
        };

        Ok(CardEvaluation {
            is_correct: eval.is_correct,
            score: eval.score,
            feedback: eval.feedback,
            suggested_rating,
        })
    }

    async fn generate_cards(&self, request: GenerationRequest) -> Result<Vec<GeneratedCard>, LlmError> {
        let prompt = format!(
            "Generate up to {} flashcards from this content:\n\n{}\n\nRespond with JSON only.",
            request.max_cards, request.content
        );

        let anthropic_request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            system: GENERATION_SYSTEM_PROMPT.to_string(),
            messages: vec![AnthropicMessage {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = self
            .client
            .post(format!("{}/messages", ANTHROPIC_API_BASE))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&anthropic_request)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if response.status() == 429 {
            return Err(LlmError::RateLimited { retry_after_secs: 60 });
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::RequestFailed(error_text));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let text = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| LlmError::ParseError("No response content".to_string()))?;

        let json_text = extract_json(&text);
        let gen: GenerationResponse =
            serde_json::from_str(json_text).map_err(|e| LlmError::ParseError(e.to_string()))?;

        Ok(gen
            .cards
            .into_iter()
            .map(|c| GeneratedCard {
                front: c.front,
                back: c.back,
                tags: c.tags,
            })
            .collect())
    }
}
