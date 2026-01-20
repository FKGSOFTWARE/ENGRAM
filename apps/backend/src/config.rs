//! Application Configuration
//!
//! Centralized configuration loaded from environment variables with defaults.

use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// CORS configuration
    pub cors: CorsConfig,
    /// LLM provider configuration
    pub llm: LlmConfig,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Host to bind to
    pub host: String,
    /// Port to listen on
    pub port: u16,
    /// Log level
    pub log_level: String,
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// SQLite database URL
    pub url: String,
    /// Maximum number of connections
    pub max_connections: u32,
}

/// CORS configuration
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (comma-separated, or "*" for any)
    pub allowed_origins: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
}

/// LLM provider configuration
#[derive(Debug, Clone)]
pub struct LlmConfig {
    /// Anthropic API key
    pub anthropic_api_key: Option<String>,
    /// OpenAI API key
    pub openai_api_key: Option<String>,
    /// Google (Gemini) API key
    pub google_api_key: Option<String>,
    /// Default model for evaluation
    pub default_model: String,
    /// Maximum tokens for generation
    pub max_tokens: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            server: ServerConfig::from_env(),
            database: DatabaseConfig::from_env(),
            cors: CorsConfig::from_env(),
            llm: LlmConfig::from_env(),
        }
    }
}

impl ServerConfig {
    fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3001),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "engram_backend=debug,tower_http=debug".to_string()),
        }
    }

    /// Get the bind address
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl DatabaseConfig {
    fn from_env() -> Self {
        Self {
            url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:engram.db?mode=rwc".to_string()),
            max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(5),
        }
    }
}

impl CorsConfig {
    fn from_env() -> Self {
        let origins = env::var("CORS_ORIGINS").unwrap_or_else(|_| "*".to_string());
        let allowed_origins = if origins == "*" {
            vec!["*".to_string()]
        } else {
            origins.split(',').map(|s| s.trim().to_string()).collect()
        };

        Self {
            allowed_origins,
            allow_credentials: env::var("CORS_ALLOW_CREDENTIALS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        }
    }

    /// Check if all origins are allowed
    pub fn is_any_origin(&self) -> bool {
        self.allowed_origins.len() == 1 && self.allowed_origins[0] == "*"
    }
}

impl LlmConfig {
    fn from_env() -> Self {
        Self {
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            google_api_key: env::var("GOOGLE_API_KEY")
                .or_else(|_| env::var("GEMINI_API_KEY"))
                .ok(),
            default_model: env::var("LLM_DEFAULT_MODEL")
                .unwrap_or_else(|_| "gemini-1.5-flash".to_string()),
            max_tokens: env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(2048),
        }
    }

    /// Check if any LLM provider is configured
    pub fn has_any_provider(&self) -> bool {
        self.anthropic_api_key.is_some()
            || self.openai_api_key.is_some()
            || self.google_api_key.is_some()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        // Clear any existing env vars that might interfere
        let config = Config::from_env();

        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
        assert!(!config.database.url.is_empty());
    }

    #[test]
    fn test_cors_any_origin() {
        let cors = CorsConfig {
            allowed_origins: vec!["*".to_string()],
            allow_credentials: false,
        };
        assert!(cors.is_any_origin());

        let cors = CorsConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allow_credentials: false,
        };
        assert!(!cors.is_any_origin());
    }
}
