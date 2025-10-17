use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::credits::CreditsResponse;
use crate::utils::retry::operations::GET_BALANCE;
use crate::utils::{retry::execute_with_retry_builder, retry::handle_response_json};
use reqwest::Client;

/// API endpoint for credits management.
pub struct CreditsApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl CreditsApi {
    /// Creates a new CreditsApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Retrieves the current credit balance and usage information.
    ///
    /// This endpoint returns the total credits purchased and used for the authenticated user.
    ///
    /// # Returns
    ///
    /// Returns a `CreditsResponse` containing:
    /// - `total_credits`: Total credits purchased by the user
    /// - `total_usage`: Total credits used by the user
    ///
    /// The response also provides convenience methods to calculate:
    /// - Remaining credits: `total_credits - total_usage`
    /// - Usage percentage: `total_usage / total_credits`
    /// - Whether credits are available: `remaining > 0`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails (network issues, authentication, etc.)
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
    ///     let credits = client.credits()?.get_balance().await?;
    ///
    ///     println!("Total credits: ${:.2}", credits.total_credits());
    ///     println!("Usage: ${:.2}", credits.total_usage());
    ///     println!("Remaining: ${:.2}", credits.remaining_credits());
    ///     println!("Usage: {:.1}%", credits.usage_percentage() * 100.0);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_balance(&self) -> Result<CreditsResponse> {
        // Build the URL.
        let url = self
            .config
            .base_url
            .join("api/v1/credits")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for credits endpoint: {e}"),
                metadata: None,
            })?;

        // Build headers once to avoid closure issues
        let headers = self.config.build_headers()?;

        // Execute request with retry logic
        let response = execute_with_retry_builder(&self.config.retry_config, GET_BALANCE, || {
            self.client.get(url.clone()).headers(headers.clone())
        })
        .await?;

        // Handle response with consistent error parsing
        handle_response_json::<CreditsResponse>(response, GET_BALANCE).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credits_api_new() {
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
        let credits_api = CreditsApi::new(client, &config);

        assert!(credits_api.config.api_key.is_some());
    }
}
