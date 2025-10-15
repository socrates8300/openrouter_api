use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::models::{ModelInfo, ModelsRequest, ModelsResponse};
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

    /// Lists available models, optionally filtered by various criteria.
    ///
    /// # Arguments
    ///
    /// * `request` - Optional ModelsRequest containing filter and sort parameters
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `ModelsResponse` with model information
    /// or an `Error` if the request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// use openrouter_api::types::models::{ModelsRequest, ModelCapability, ModelSortOrder};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     let models_api = client.models()?;
    ///
    ///     // List all models
    ///     let all_models = models_api.list_models(None).await?;
    ///
    ///     // Filter by capability
    ///     let chat_models = models_api.list_models(Some(ModelsRequest {
    ///         capability: Some(ModelCapability::Chat),
    ///         provider: None,
    ///         model_name: None,
    ///         min_context_length: None,
    ///         max_context_length: None,
    ///         free_only: None,
    ///         supports_tools: None,
    ///         supports_vision: None,
    ///         supports_function_calling: None,
    ///         sort: Some(ModelSortOrder::Name),
    ///         limit: Some(10),
    ///     })).await?;
    ///
    ///     // Find free models with vision support
    ///     let free_vision_models = models_api.list_models(Some(ModelsRequest {
    ///         capability: None,
    ///         provider: None,
    ///         model_name: None,
    ///         min_context_length: None,
    ///         max_context_length: None,
    ///         free_only: Some(true),
    ///         supports_tools: None,
    ///         supports_vision: Some(true),
    ///         supports_function_calling: None,
    ///         sort: Some(ModelSortOrder::Name),
    ///         limit: None,
    ///     })).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
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

    /// Finds a specific model by its ID
    ///
    /// This is a convenience method that fetches all models and returns
    /// the one matching the specified ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the model to retrieve
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `ModelInfo` or an `Error`
    /// if the request fails or the model is not found.
    pub async fn get_model_by_id(&self, id: &str) -> Result<ModelInfo> {
        let models_response = self.list_models(None).await?;

        models_response
            .find_by_id(id)
            .cloned()
            .ok_or_else(|| Error::ModelNotAvailable(format!("Model with ID '{}' not found", id)))
    }

    /// Finds models by provider (case-insensitive)
    ///
    /// This is a convenience method that fetches all models and returns
    /// those matching the specified provider.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name to filter by
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `ModelInfo` instances or an `Error`
    /// if the request fails.
    pub async fn get_models_by_provider(&self, provider: &str) -> Result<Vec<ModelInfo>> {
        let models_response = self.list_models(None).await?;
        Ok(models_response
            .find_by_provider(provider)
            .into_iter()
            .cloned()
            .collect())
    }

    /// Finds free models
    ///
    /// This is a convenience method that fetches all models and returns
    /// only those that are free to use.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `ModelInfo` instances or an `Error`
    /// if the request fails.
    pub async fn get_free_models(&self) -> Result<Vec<ModelInfo>> {
        let models_response = self.list_models(None).await?;
        Ok(models_response
            .find_free_models()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Finds models that support tools
    ///
    /// This is a convenience method that fetches all models and returns
    /// only those that support tools/function calling.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `ModelInfo` instances or an `Error`
    /// if the request fails.
    pub async fn get_models_with_tools(&self) -> Result<Vec<ModelInfo>> {
        let models_response = self.list_models(None).await?;
        Ok(models_response
            .find_models_with_tools()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Finds models that support vision/multimodal
    ///
    /// This is a convenience method that fetches all models and returns
    /// only those that support vision/multimodal capabilities.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `ModelInfo` instances or an `Error`
    /// if the request fails.
    pub async fn get_models_with_vision(&self) -> Result<Vec<ModelInfo>> {
        let models_response = self.list_models(None).await?;
        Ok(models_response
            .find_models_with_vision()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Searches models by name or description
    ///
    /// This is a convenience method that fetches all models and returns
    /// those matching the search query.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query to match against model names, descriptions, and IDs
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `ModelInfo` instances or an `Error`
    /// if the request fails.
    pub async fn search_models(&self, query: &str) -> Result<Vec<ModelInfo>> {
        let models_response = self.list_models(None).await?;
        Ok(models_response.search(query).into_iter().cloned().collect())
    }

    /// Gets models with minimum context length
    ///
    /// This is a convenience method that fetches all models and returns
    /// only those with at least the specified context length.
    ///
    /// # Arguments
    ///
    /// * `min_context` - Minimum context length required
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a vector of `ModelInfo` instances or an `Error`
    /// if the request fails.
    pub async fn get_models_with_min_context(&self, min_context: u32) -> Result<Vec<ModelInfo>> {
        let models_response = self.list_models(None).await?;
        Ok(models_response
            .find_models_with_min_context(min_context)
            .into_iter()
            .cloned()
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::SecureApiKey;
    use std::time::Duration;

    #[test]
    fn test_models_api_creation() {
        let config = crate::client::ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: Duration::from_secs(30),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: crate::client::RetryConfig::default(),
        };
        let http_client = Client::new();

        let models_api = ModelsApi::new(http_client, &config);

        // Test that the API was created successfully
        assert_eq!(
            models_api.config.base_url.as_str(),
            "https://openrouter.ai/api/v1/"
        );
    }

    #[tokio::test]
    async fn test_models_api_network_error() {
        let config = crate::client::ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://invalid-url-that-does-not-exist.com/api/v1/")
                .unwrap(),
            timeout: Duration::from_secs(1),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: crate::client::RetryConfig::default(),
        };
        let http_client = Client::new();
        let models_api = ModelsApi::new(http_client, &config);

        // Test that network errors are properly handled
        let result = models_api.list_models(None).await;
        assert!(result.is_err());

        // Any error type is acceptable for network failure
        match result.unwrap_err() {
            Error::HttpError(_) | Error::RateLimitExceeded(_) | Error::ApiError { .. } => {
                // Expected error types for network failure
            }
            other => {
                panic!("Expected network error, got: {:?}", other);
            }
        }
    }
}
