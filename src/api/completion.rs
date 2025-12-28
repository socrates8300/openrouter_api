// api/completion.rs
use crate::error::{Error, Result};
use crate::types::completion::{CompletionRequest, CompletionResponse};
use crate::utils::{
    retry::execute_with_retry_builder, retry::handle_response_json,
    retry::operations::TEXT_COMPLETION, validation::validate_completion_request,
};
use reqwest::Client;

/// API endpoint for text completions.
pub struct CompletionApi {
    pub client: Client,
    pub config: crate::client::ApiConfig,
}

impl CompletionApi {
    /// Creates a new CompletionApi with the given reqwest client and configuration.
    #[must_use = "returns an API client that should be used for completion operations"]
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Calls the completions endpoint. The request payload includes at minimum the `model` and `prompt` fields,
    /// along with any additional generation parameters (temperature, top_p, and so on).
    #[must_use = "returns a completion response that should be processed"]
    pub async fn text_completion(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        // Validate the request using the new validation module
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

        // Execute request with retry logic
        let response =
            execute_with_retry_builder(&self.config.retry_config, TEXT_COMPLETION, || {
                self.client
                    .post(url.clone())
                    .headers((*self.config.headers).clone())
                    .json(&request)
            })
            .await?;

        // Handle response with consistent error parsing
        handle_response_json::<CompletionResponse>(response, TEXT_COMPLETION).await
    }
}

// Validation is now handled by the validation module
// The validate_completion_request function is imported from utils::validation
