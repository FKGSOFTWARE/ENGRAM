use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardEvaluation {
    pub is_correct: bool,
    pub score: f32, // 0.0 to 1.0
    pub feedback: String,
    pub suggested_rating: SuggestedRating,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedRating {
    Again,
    Hard,
    Good,
    Easy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedCard {
    pub front: String,
    pub back: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EvaluationRequest {
    pub card_front: String,
    pub card_back: String,
    pub user_answer: String,
}

#[derive(Debug, Clone)]
pub struct GenerationRequest {
    pub content: String,
    pub max_cards: usize,
    pub style: GenerationStyle,
}

#[derive(Debug, Clone, Default)]
pub enum GenerationStyle {
    #[default]
    QuestionAnswer,
    Cloze,
    Definition,
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Response parsing failed: {0}")]
    ParseError(String),

    #[error("Provider unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Get the provider name for logging/debugging
    fn name(&self) -> &'static str;

    /// Check if the provider is configured and ready
    fn is_available(&self) -> bool;

    /// Evaluate a user's answer against the expected answer
    async fn evaluate_answer(&self, request: EvaluationRequest) -> Result<CardEvaluation, LlmError>;

    /// Generate flashcards from content
    async fn generate_cards(&self, request: GenerationRequest) -> Result<Vec<GeneratedCard>, LlmError>;
}

/// System prompt for answer evaluation
pub const EVALUATION_SYSTEM_PROMPT: &str = r#"You are an expert flashcard tutor evaluating student answers.

Your task is to compare the student's answer to the expected answer and determine:
1. Whether the answer is correct (allow for reasonable variations in wording)
2. A score from 0.0 (completely wrong) to 1.0 (perfect)
3. Brief, encouraging feedback
4. A suggested rating: "again" (wrong), "hard" (struggled), "good" (correct), or "easy" (effortless)

Be flexible with:
- Different phrasings that convey the same meaning
- Minor spelling mistakes if the concept is clearly understood
- Partial answers that show understanding of the core concept

Be strict with:
- Factual errors
- Missing critical information
- Conceptual misunderstandings

Respond with JSON only:
{
  "is_correct": boolean,
  "score": number,
  "feedback": "string",
  "suggested_rating": "again" | "hard" | "good" | "easy"
}"#;

/// System prompt for card generation
pub const GENERATION_SYSTEM_PROMPT: &str = r#"You are an expert at creating effective flashcards for learning.

Given the content, create flashcards that:
1. Focus on one concept per card
2. Use clear, concise questions
3. Have specific, memorable answers
4. Follow spaced repetition best practices

For each card, provide:
- A question (front)
- An answer (back)
- Relevant tags for categorization

Respond with JSON only:
{
  "cards": [
    {
      "front": "question",
      "back": "answer",
      "tags": ["tag1", "tag2"]
    }
  ]
}"#;
