use crate::error::{Error, Result};
use crate::types::models::{ModelsRequest, ModelsResponse};
use crate::utils::retry::operations::LIST_MODELS;
use crate::utils::{retry::execute_with_retry_builder, retry::handle_response_json};
use reqwest::Client;

/// API endpoint for model management.
/// API endpoint for model information.
pub struct ModelsApi {
    pub client: Client,
    pub config: crate::client::ApiConfig,
}

impl ModelsApi {
    /// Creates a new ModelsApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Lists available models, optionally filtered by capability or provider.
    pub async fn list_models(&self, request: Option<ModelsRequest>) -> Result<ModelsResponse> {
        // Build the URL.
        let url = self
            .config
            .base_url
            .join("models")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for models endpoint: {e}"),
                metadata: None,
            })?;

        // Use pre-built headers from config
        let headers = self.config.headers.clone();

        // Execute request with retry logic
        let response = execute_with_retry_builder(&self.config.retry_config, LIST_MODELS, || {
            let mut req_builder = self.client.get(url.clone()).headers(headers.clone());

            if let Some(ref req) = request {
                req_builder = req_builder.query(req);
            }

            req_builder
        })
        .await?;

        // Handle response with consistent error parsing
        handle_response_json::<ModelsResponse>(response, LIST_MODELS).await
    }
}
