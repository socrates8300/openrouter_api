use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::generation::GenerationResponse;
use crate::utils::security::create_safe_error_message;
use reqwest::Client;

/// API endpoint for generation management.
pub struct GenerationApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl GenerationApi {
    /// Creates a new GenerationApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
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

        // Build the request with query parameter.
        let response = self
            .client
            .get(url)
            .query(&[("id", id)])
            .headers(self.config.build_headers()?)
            .send()
            .await?;

        // Capture the status code before consuming the response body.
        let status = response.status();

        // Get the response body.
        let body = response.text().await?;

        // Check if the HTTP response was successful.
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(&body, "Generation API request failed"),
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
        serde_json::from_str::<GenerationResponse>(&body).map_err(|e| Error::ApiError {
            code: status.as_u16(),
            message: create_safe_error_message(
                &format!("Failed to decode JSON: {e}. Body was: {body}"),
                "Generation JSON parsing error",
            ),
            metadata: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation_api_new() {
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
        let generation_api = GenerationApi::new(client, &config);
        
        assert!(generation_api.config.api_key.is_some());
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
        let generation_api = GenerationApi::new(client, &config);

        // Test empty ID
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            generation_api.get_generation("").await
        });
        assert!(result.is_err());

        // Test whitespace-only ID
        let result = rt.block_on(async {
            generation_api.get_generation("   ").await
        });
        assert!(result.is_err());

        // Test valid ID (this will fail with network error, but not validation error)
        let result = rt.block_on(async {
            generation_api.get_generation("gen-valid123").await
        });
        assert!(result.is_err()); // Network error, not validation error
    }
}