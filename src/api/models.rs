use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::models::{ModelsRequest, ModelsResponse};
use crate::utils::security::create_safe_error_message;
use reqwest::Client;

/// API endpoint for model management.
pub struct ModelsApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl ModelsApi {
    /// Creates a new ModelsApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
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

        // Build the request with optional query parameters.
        let mut req_builder = self.client.get(url).headers(self.config.build_headers()?);

        if let Some(req) = request {
            req_builder = req_builder.query(&req);
        }

        // Send the request.
        let response = req_builder.send().await?;

        // Capture the status code before consuming the response body.
        let status = response.status();

        // Get the response body.
        let body = response.text().await?;

        // Check if the HTTP response was successful.
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(&body, "Models API request failed"),
                metadata: None,
            });
        }

        if body.trim().is_empty() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: "Empty response body".into(),
                metadata: None,
            });
        }

        // Deserialize the body.
        serde_json::from_str::<ModelsResponse>(&body).map_err(|e| Error::ApiError {
            code: status.as_u16(),
            message: create_safe_error_message(
                &format!("Failed to decode JSON: {e}. Body was: {body}"),
                "Models JSON parsing error",
            ),
            metadata: None,
        })
    }
}
