use reqwest::Response;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// OpenRouter API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetails {
    /// Error code (e.g., "insufficient_quota")
    pub code: Option<String>,

    /// HTTP status code
    pub status: Option<u16>,

    /// Provider-specific error details
    pub provider: Option<serde_json::Value>,

    /// Additional error metadata
    pub metadata: Option<serde_json::Value>,
}

/// Centralized error type for the OpenRouter client library.
#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("API error (status {code}): {message}")]
    ApiError {
        code: u16,
        message: String,
        metadata: Option<Value>,
    },

    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Structured output not supported by the provider/model")]
    StructuredOutputNotSupported,

    #[error("Schema validation error: {0}")]
    SchemaValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    #[error("Missing required credential: {0}")]
    MissingCredential(String),

    #[error("Streaming error: {0}")]
    StreamingError(String),

    #[error("Context length exceeded for model {model}: {message}")]
    ContextLengthExceeded { model: String, message: String },

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Creates an API error from a given HTTP response.
    pub async fn from_response(response: Response) -> Result<Self> {
        let status = response.status().as_u16();
        let text = response.text().await.unwrap_or_default();

        // Try to parse structured API error response
        if let Ok(api_error) = serde_json::from_str::<ApiErrorDetails>(&text) {
            return Ok(Error::ApiError {
                code: status,
                message: text,
                metadata: Some(serde_json::to_value(api_error).unwrap_or_default()),
            });
        }

        // Handle rate limiting specifically
        if status == 429 {
            return Ok(Error::RateLimitExceeded(text));
        }

        Ok(Error::ApiError {
            code: status,
            message: text,
            metadata: None,
        })
    }
}

#[cfg(test)]
mod tests;
