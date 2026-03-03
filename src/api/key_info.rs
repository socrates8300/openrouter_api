use crate::error::{Error, Result};
use crate::types::key_info::KeyInfoResponse;
use crate::utils::retry::operations::GET_KEY_INFO;
use crate::utils::{retry::execute_with_retry_builder, retry::handle_response_json};
use reqwest::Client;

/// API endpoint for key information.
pub struct KeyInfoApi {
    pub(crate) client: Client,
    pub(crate) config: crate::client::ApiConfig,
}

impl KeyInfoApi {
    /// Creates a new KeyInfoApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Retrieves information about the current API key.
    ///
    /// Returns credit limits, usage, rate limit info, and whether the key is on the free tier.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     let key_info = client.key_info()?.get_key_info().await?;
    ///
    ///     if let Some(remaining) = key_info.limit_remaining() {
    ///         println!("Remaining credits: ${:.2}", remaining);
    ///     }
    ///     println!("Free tier: {}", key_info.is_free_tier());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_key_info(&self) -> Result<KeyInfoResponse> {
        let url = self
            .config
            .base_url
            .join("auth/key")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for key info endpoint: {e}"),
                metadata: None,
            })?;

        let response = execute_with_retry_builder(&self.config.retry_config, GET_KEY_INFO, || {
            self.client
                .get(url.clone())
                .headers((*self.config.headers).clone())
        })
        .await?;

        handle_response_json::<KeyInfoResponse>(response, GET_KEY_INFO).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_info_api_new() {
        use crate::tests::test_helpers::test_client_config;

        let config = test_client_config();
        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();
        assert!(!api.config.headers.is_empty());
        assert!(api.config.headers.contains_key("authorization"));
    }
}
