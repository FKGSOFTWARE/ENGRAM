use std::sync::Arc;

use super::{
    anthropic::AnthropicProvider,
    gemini::GeminiProvider,
    openai::OpenAIProvider,
    provider::{
        CardEvaluation, EvaluationRequest, GeneratedCard, GenerationRequest, LlmError, LlmProvider,
    },
};

pub struct ProviderManager {
    providers: Vec<Arc<dyn LlmProvider>>,
}

impl ProviderManager {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Create a manager from environment variables
    pub fn from_env() -> Self {
        let mut manager = Self::new();

        // Add providers in priority order: Gemini -> OpenAI -> Anthropic
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            if !key.is_empty() {
                manager.add_provider(Arc::new(GeminiProvider::new(key)));
            }
        }

        if let Ok(key) = std::env::var("OPENAI_API_KEY") {
            if !key.is_empty() {
                manager.add_provider(Arc::new(OpenAIProvider::new(key)));
            }
        }

        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            if !key.is_empty() {
                manager.add_provider(Arc::new(AnthropicProvider::new(key)));
            }
        }

        manager
    }

    pub fn add_provider(&mut self, provider: Arc<dyn LlmProvider>) {
        self.providers.push(provider);
    }

    pub fn has_available_provider(&self) -> bool {
        self.providers.iter().any(|p| p.is_available())
    }

    pub fn available_providers(&self) -> Vec<&str> {
        self.providers
            .iter()
            .filter(|p| p.is_available())
            .map(|p| p.name())
            .collect()
    }

    /// Evaluate an answer using available providers with automatic fallback
    pub async fn evaluate_answer(
        &self,
        request: EvaluationRequest,
    ) -> Result<CardEvaluation, LlmError> {
        let mut last_error = LlmError::Unavailable("No providers configured".to_string());

        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }

            tracing::debug!("Trying provider: {}", provider.name());

            match provider.evaluate_answer(request.clone()).await {
                Ok(result) => {
                    tracing::debug!("Success with provider: {}", provider.name());
                    return Ok(result);
                }
                Err(LlmError::InvalidApiKey) => {
                    tracing::warn!("Invalid API key for provider: {}", provider.name());
                    last_error = LlmError::InvalidApiKey;
                    continue;
                }
                Err(LlmError::RateLimited { retry_after_secs }) => {
                    tracing::warn!(
                        "Rate limited by provider: {}, retry after {}s",
                        provider.name(),
                        retry_after_secs
                    );
                    last_error = LlmError::RateLimited { retry_after_secs };
                    continue;
                }
                Err(e) => {
                    tracing::error!("Error from provider {}: {}", provider.name(), e);
                    last_error = e;
                    continue;
                }
            }
        }

        Err(last_error)
    }

    /// Generate cards using available providers with automatic fallback
    pub async fn generate_cards(
        &self,
        request: GenerationRequest,
    ) -> Result<Vec<GeneratedCard>, LlmError> {
        let mut last_error = LlmError::Unavailable("No providers configured".to_string());

        for provider in &self.providers {
            if !provider.is_available() {
                continue;
            }

            tracing::debug!("Trying provider: {}", provider.name());

            match provider.generate_cards(request.clone()).await {
                Ok(result) => {
                    tracing::debug!("Success with provider: {}", provider.name());
                    return Ok(result);
                }
                Err(LlmError::InvalidApiKey) => {
                    tracing::warn!("Invalid API key for provider: {}", provider.name());
                    last_error = LlmError::InvalidApiKey;
                    continue;
                }
                Err(LlmError::RateLimited { retry_after_secs }) => {
                    tracing::warn!(
                        "Rate limited by provider: {}, retry after {}s",
                        provider.name(),
                        retry_after_secs
                    );
                    last_error = LlmError::RateLimited { retry_after_secs };
                    continue;
                }
                Err(e) => {
                    tracing::error!("Error from provider {}: {}", provider.name(), e);
                    last_error = e;
                    continue;
                }
            }
        }

        Err(last_error)
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}
