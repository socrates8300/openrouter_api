// api/completion.rs
use crate::client::ClientConfig;
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

// Validation is now handled by the validation module
// The validate_completion_request function is imported from utils::validation
