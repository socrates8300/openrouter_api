use crate::error::{Error, Result};
use crate::types::generation::GenerationResponse;
use crate::utils::{
    retry::execute_with_retry_builder, retry::handle_response_json,
    retry::operations::GET_GENERATION,
};
use reqwest::Client;

/// API endpoint for generation management.
/// API endpoint for generation information.
pub struct GenerationApi {
    pub client: Client,
    pub config: crate::client::ApiConfig,
}

impl GenerationApi {
    /// Creates a new GenerationApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Retrieves metadata about a specific generation request.
    ///
    /// This endpoint returns detailed information about a generation including
    /// cost, token usage, latency, provider information, and more.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the generation to retrieve
    ///
    /// # Returns
    ///
    /// Returns a `GenerationResponse` containing comprehensive metadata about the generation:
    /// - Basic info: id, model, created_at, origin
    /// - Cost info: total_cost, cache_discount, effective_cost
    /// - Token usage: tokens_prompt, tokens_completion, total_tokens
    /// - Performance: latency, generation_time, moderation_latency
    /// - Provider details: provider_name, upstream_id
    /// - Features: streamed, cancelled, web_search, media, reasoning
    /// - Finish reasons: finish_reason, native_finish_reason
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The generation ID is empty or invalid
    /// - The API request fails (network issues, authentication, etc.)
    /// - The generation is not found
    /// - The response cannot be parsed
    /// - The server returns an error status code
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     let generation = client.generation()?.get_generation("gen-123456789").await?;
    ///
    ///     println!("Generation ID: {}", generation.id());
    ///     println!("Model: {}", generation.model());
    ///     println!("Total cost: ${:.6}", generation.total_cost());
    ///     println!("Effective cost: ${:.6}", generation.effective_cost());
    ///
    ///     if let Some(tokens) = generation.total_tokens() {
    ///         println!("Total tokens: {}", tokens);
    ///         if let Some(cost_per_token) = generation.cost_per_token() {
    ///             println!("Cost per token: ${:.8}", cost_per_token);
    ///         }
    ///     }
    ///
    ///     if let Some(latency) = generation.latency_seconds() {
    ///         println!("Latency: {:.2}s", latency);
    ///     }
    ///
    ///     println!("Successful: {}", generation.is_successful());
    ///     println!("Streamed: {}", generation.was_streamed());
    ///     println!("Used web search: {}", generation.used_web_search());
    ///     println!("Included media: {}", generation.included_media());
    ///     println!("Used reasoning: {}", generation.used_reasoning());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_generation(&self, id: &str) -> Result<GenerationResponse> {
        // Validate the generation ID
        if id.trim().is_empty() {
            return Err(Error::ConfigError(
                "Generation ID cannot be empty".to_string(),
            ));
        }

        // Build the URL with query parameter.
        let url = self
            .config
            .base_url
            .join("api/v1/generation")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for generation endpoint: {e}"),
                metadata: None,
            })?;

        // Use pre-built headers from config
        let headers = self.config.headers.clone();

        // Execute request with retry logic
        let response =
            execute_with_retry_builder(&self.config.retry_config, GET_GENERATION, || {
                self.client
                    .get(url.clone())
                    .query(&[("id", id)])
                    .headers(headers.clone())
            })
            .await?;

        // Handle response with consistent error parsing
        handle_response_json::<GenerationResponse>(response, GET_GENERATION).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_api_new() {
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use reqwest::Client;
        use url::Url;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: Url::parse("https://openrouter.ai/api/v1").unwrap(),
            timeout: std::time::Duration::from_secs(30),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };

        let client = Client::new();
        let generation_api = GenerationApi::new(client, &config).unwrap();

        // Verify that the API config was created successfully
        // The API key should NOT be stored in the API config for security reasons
        assert!(!generation_api.config.headers.is_empty());
        assert!(generation_api.config.headers.contains_key("authorization"));
    }

    #[test]
    fn test_generation_id_validation() {
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use reqwest::Client;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(30),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };

        let client = Client::new();
        let generation_api = GenerationApi::new(client, &config).unwrap();

        // Test empty ID
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { generation_api.get_generation("").await });
        assert!(result.is_err());

        // Test whitespace-only ID
        let result = rt.block_on(async { generation_api.get_generation("   ").await });
        assert!(result.is_err());

        // Test valid ID (this will fail with network error, but not validation error)
        let result = rt.block_on(async { generation_api.get_generation("gen-valid123").await });
        assert!(result.is_err()); // Network error, not validation error
    }
}
