// api/completion.rs
use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::completion::{CompletionRequest, CompletionResponse};
use crate::utils::{
    retry::execute_with_retry_builder, retry::handle_response_json,
    retry::operations::TEXT_COMPLETION,
};
use reqwest::Client;

/// API endpoint for text completions.
pub struct CompletionApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl CompletionApi {
    /// Creates a new CompletionApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Calls the completions endpoint. The request payload includes at minimum the `model` and `prompt` fields,
    /// along with any additional generation parameters (temperature, top_p, and so on).
    pub async fn text_completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Validate the request
        validate_completion_request(&request)?;

        // Build the URL.
        let url = self
            .config
            .base_url
            .join("completions")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for completions: {e}"),
                metadata: None,
            })?;

        // Build headers once to avoid closure issues
        let headers = self.config.build_headers()?;

        // Execute request with retry logic
        let response =
            execute_with_retry_builder(&self.config.retry_config, TEXT_COMPLETION, || {
                self.client
                    .post(url.clone())
                    .headers(headers.clone())
                    .json(&request)
            })
            .await?;

        // Handle response with consistent error parsing
        handle_response_json::<CompletionResponse>(response, TEXT_COMPLETION).await
    }
}

/// Validates a completion request for common errors.
fn validate_completion_request(request: &CompletionRequest) -> Result<()> {
    // Validate model is not empty
    if request.model.trim().is_empty() {
        return Err(Error::ConfigError("Model ID cannot be empty".into()));
    }

    // Validate prompt is not empty
    if request.prompt.trim().is_empty() {
        return Err(Error::ConfigError("Prompt cannot be empty".into()));
    }

    // Validate extra parameters if present
    if let serde_json::Value::Object(params) = &request.extra_params {
        // Temperature: [0.0, 2.0]
        if let Some(temp) = params.get("temperature").and_then(|v| v.as_f64()) {
            if !(0.0..=2.0).contains(&temp) {
                return Err(Error::ConfigError(format!(
                    "Temperature must be between 0.0 and 2.0, got {}",
                    temp
                )));
            }
        }

        // Top P: (0.0, 1.0]
        if let Some(top_p) = params.get("top_p").and_then(|v| v.as_f64()) {
            if top_p <= 0.0 || top_p > 1.0 {
                return Err(Error::ConfigError(format!(
                    "Top P must be between 0.0 (exclusive) and 1.0 (inclusive), got {}",
                    top_p
                )));
            }
        }

        // Frequency Penalty: [-2.0, 2.0]
        if let Some(fp) = params.get("frequency_penalty").and_then(|v| v.as_f64()) {
            if !(-2.0..=2.0).contains(&fp) {
                return Err(Error::ConfigError(format!(
                    "Frequency penalty must be between -2.0 and 2.0, got {}",
                    fp
                )));
            }
        }

        // Presence Penalty: [-2.0, 2.0]
        if let Some(pp) = params.get("presence_penalty").and_then(|v| v.as_f64()) {
            if !(-2.0..=2.0).contains(&pp) {
                return Err(Error::ConfigError(format!(
                    "Presence penalty must be between -2.0 and 2.0, got {}",
                    pp
                )));
            }
        }
    }

    Ok(())
}
