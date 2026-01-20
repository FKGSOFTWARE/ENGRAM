use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::provider::{
    CardEvaluation, EvaluationRequest, GeneratedCard, GenerationRequest, LlmError, LlmProvider,
    SuggestedRating, EVALUATION_SYSTEM_PROMPT, GENERATION_SYSTEM_PROMPT,
};

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
    model: String,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
            model: "gemini-2.5-flash".to_string(),
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
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GeminiConfig,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiConfig {
    temperature: f32,
    #[serde(rename = "responseMimeType")]
    response_mime_type: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Debug, Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
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

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn name(&self) -> &'static str {
        "Gemini"
    }

    fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn evaluate_answer(&self, request: EvaluationRequest) -> Result<CardEvaluation, LlmError> {
        let prompt = format!(
            "Card Question: {}\nExpected Answer: {}\nStudent Answer: {}",
            request.card_front, request.card_back, request.user_answer
        );

        let gemini_request = GeminiRequest {
            contents: vec![
                GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart {
                        text: format!("{}\n\n{}", EVALUATION_SYSTEM_PROMPT, prompt),
                    }],
                },
            ],
            generation_config: GeminiConfig {
                temperature: 0.3,
                response_mime_type: "application/json".to_string(),
            },
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if response.status() == 429 {
            return Err(LlmError::RateLimited { retry_after_secs: 60 });
        }

        if response.status() == 401 || response.status() == 403 {
            return Err(LlmError::InvalidApiKey);
        }

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::RequestFailed(error_text));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| LlmError::ParseError("No response content".to_string()))?;

        let eval: EvaluationResponse =
            serde_json::from_str(&text).map_err(|e| LlmError::ParseError(e.to_string()))?;

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
            "Generate up to {} flashcards from this content:\n\n{}",
            request.max_cards, request.content
        );

        let gemini_request = GeminiRequest {
            contents: vec![
                GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart {
                        text: format!("{}\n\n{}", GENERATION_SYSTEM_PROMPT, prompt),
                    }],
                },
            ],
            generation_config: GeminiConfig {
                temperature: 0.7,
                response_mime_type: "application/json".to_string(),
            },
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            GEMINI_API_BASE, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&gemini_request)
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

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let text = gemini_response
            .candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| LlmError::ParseError("No response content".to_string()))?;

        let gen: GenerationResponse =
            serde_json::from_str(&text).map_err(|e| LlmError::ParseError(e.to_string()))?;

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
