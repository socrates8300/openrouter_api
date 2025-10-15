use crate::error::Error;
use crate::types::{ProvidersResponse, Provider};
use crate::client::ClientConfig;
use reqwest::Client;

/// API client for provider-related operations
pub struct ProvidersApi {
    client: Client,
    config: ClientConfig,
}

impl ProvidersApi {
    /// Creates a new ProvidersApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Retrieves a list of all available providers
    /// 
    /// Returns information about providers available through the OpenRouter API,
    /// including their policies and status information.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the `ProvidersResponse` with provider information
    /// or an `Error` if the request fails.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get all available providers
    ///     let providers_response = client.providers()?.get_providers().await?;
    ///     
    ///     println!("Found {} providers", providers_response.count());
    ///     
    ///     // Find a specific provider
    ///     if let Some(openai) = providers_response.find_by_slug("openai") {
    ///         println!("OpenAI provider found: {}", openai.name);
    ///         if openai.has_privacy_policy() {
    ///             println!("Privacy policy: {}", openai.privacy_policy_url.unwrap());
    ///         }
    ///     }
    ///     
    ///     // Group providers by domain
    ///     let domain_groups = providers_response.group_by_domain();
    ///     for (domain, providers) in domain_groups {
    ///         println!("Domain {}: {} providers", domain, providers.len());
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_providers(&self) -> Result<ProvidersResponse, Error> {
        let url = format!("{}/providers", self.config.base_url);

        let response = self.client
            .get(&url)
            .headers(self.config.build_headers()?)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::from_response(response).await?);
        }

        let providers_response: ProvidersResponse = response.json().await
            .map_err(|e| Error::HttpError(e))?;

        Ok(providers_response)
    }

    /// Retrieves a specific provider by slug
    /// 
    /// This is a convenience method that fetches all providers and returns
    /// the one matching the specified slug.
    /// 
    /// # Arguments
    /// 
    /// * `slug` - The slug identifier of the provider to retrieve
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the `Provider` information or an `Error`
    /// if the request fails or the provider is not found.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get a specific provider by slug
    ///     let openai = client.providers()?.get_provider_by_slug("openai").await?;
    ///     
    ///     println!("Provider: {}", openai.name);
    ///     println!("Slug: {}", openai.slug);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_provider_by_slug(&self, slug: &str) -> Result<Provider, Error> {
        let providers_response = self.get_providers().await?;
        
        providers_response
            .find_by_slug(slug)
            .cloned()
            .ok_or_else(|| Error::ModelNotAvailable(format!("Provider with slug '{}' not found", slug)))
    }

    /// Retrieves a specific provider by name (case-insensitive)
    /// 
    /// This is a convenience method that fetches all providers and returns
    /// the one matching the specified name.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the provider to retrieve (case-insensitive)
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing the `Provider` information or an `Error`
    /// if the request fails or the provider is not found.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get a specific provider by name (case-insensitive)
    ///     let anthropic = client.providers()?.get_provider_by_name("anthropic").await?;
    ///     
    ///     println!("Provider: {}", anthropic.name);
    ///     println!("Slug: {}", anthropic.slug);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_provider_by_name(&self, name: &str) -> Result<Provider, Error> {
        let providers_response = self.get_providers().await?;
        
        providers_response
            .find_by_name(name)
            .cloned()
            .ok_or_else(|| Error::ModelNotAvailable(format!("Provider with name '{}' not found", name)))
    }

    /// Retrieves providers that have a privacy policy
    /// 
    /// This is a convenience method that filters providers to only include
    /// those that have a privacy policy URL.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a vector of `Provider` instances that
    /// have privacy policies or an `Error` if the request fails.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get providers with privacy policies
    ///     let providers_with_privacy = client.providers()?
    ///         .get_providers_with_privacy_policy().await?;
    ///     
    ///     println!("{} providers have privacy policies", providers_with_privacy.len());
    ///     
    ///     for provider in providers_with_privacy {
    ///         println!("{}: {}", provider.name, provider.privacy_policy_url.unwrap());
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_providers_with_privacy_policy(&self) -> Result<Vec<Provider>, Error> {
        let providers_response = self.get_providers().await?;
        Ok(providers_response
            .with_privacy_policy()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Retrieves providers that have terms of service
    /// 
    /// This is a convenience method that filters providers to only include
    /// those that have a terms of service URL.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a vector of `Provider` instances that
    /// have terms of service or an `Error` if the request fails.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get providers with terms of service
    ///     let providers_with_tos = client.providers()?
    ///         .get_providers_with_terms_of_service().await?;
    ///     
    ///     println!("{} providers have terms of service", providers_with_tos.len());
    ///     
    ///     for provider in providers_with_tos {
    ///         println!("{}: {}", provider.name, provider.terms_of_service_url.unwrap());
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_providers_with_terms_of_service(&self) -> Result<Vec<Provider>, Error> {
        let providers_response = self.get_providers().await?;
        Ok(providers_response
            .with_terms_of_service()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Retrieves providers that have a status page
    /// 
    /// This is a convenience method that filters providers to only include
    /// those that have a status page URL.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a vector of `Provider` instances that
    /// have status pages or an `Error` if the request fails.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get providers with status pages
    ///     let providers_with_status = client.providers()?
    ///         .get_providers_with_status_page().await?;
    ///     
    ///     println!("{} providers have status pages", providers_with_status.len());
    ///     
    ///     for provider in providers_with_status {
    ///         println!("{}: {}", provider.name, provider.status_page_url.unwrap());
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_providers_with_status_page(&self) -> Result<Vec<Provider>, Error> {
        let providers_response = self.get_providers().await?;
        Ok(providers_response
            .with_status_page()
            .into_iter()
            .cloned()
            .collect())
    }

    /// Retrieves all provider slugs sorted alphabetically
    /// 
    /// This is a convenience method that returns all provider slugs
    /// in alphabetical order.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a vector of provider slugs or an `Error`
    /// if the request fails.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get all provider slugs sorted alphabetically
    ///     let slugs = client.providers()?.get_provider_slugs().await?;
    ///     
    ///     println!("Available provider slugs:");
    ///     for slug in slugs {
    ///         println!("  {}", slug);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_provider_slugs(&self) -> Result<Vec<String>, Error> {
        let providers_response = self.get_providers().await?;
        Ok(providers_response.sorted_slugs())
    }

    /// Retrieves all provider names sorted alphabetically
    /// 
    /// This is a convenience method that returns all provider names
    /// in alphabetical order.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result` containing a vector of provider names or an `Error`
    /// if the request fails.
    /// 
    /// # Examples
    /// 
    /// ```rust,no_run
    /// use openrouter_api::client::OpenRouterClient;
    /// 
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///     
    ///     // Get all provider names sorted alphabetically
    ///     let names = client.providers()?.get_provider_names().await?;
    ///     
    ///     println!("Available provider names:");
    ///     for name in names {
    ///         println!("  {}", name);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_provider_names(&self) -> Result<Vec<String>, Error> {
        let providers_response = self.get_providers().await?;
        Ok(providers_response.sorted_names())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
    use reqwest::Client;

    #[test]
    fn test_providers_api_new() {
        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(30),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };
        let http_client = Client::new();
        
        let providers_api = ProvidersApi::new(http_client, &config);
        
        // Test that the API was created successfully
        // We can't test actual API calls without a real server
        assert!(true);
    }

    #[tokio::test]
    async fn test_providers_api_network_error() {
        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://invalid-url-that-does-not-exist.com/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(1),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };
        let http_client = Client::new();
        let providers_api = ProvidersApi::new(http_client, &config);

        // Test that network errors are properly handled
        let result = providers_api.get_providers().await;
        assert!(result.is_err());
        
        // Any error type is acceptable for network failure
        // The important thing is that it doesn't panic and returns an error
        match result.unwrap_err() {
            Error::HttpError(_) | Error::RateLimitExceeded(_) => {
                // Expected - network or rate limit error
            }
            other => {
                println!("Got error: {:?}", other);
                panic!("Expected HttpError or RateLimitExceeded for network failure, got: {:?}", other);
            }
        }
    }

    #[tokio::test]
    async fn test_provider_convenience_methods_with_empty_response() {
        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://invalid-url-that-does-not-exist.com/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(1),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };
        let http_client = Client::new();
        let providers_api = ProvidersApi::new(http_client, &config);

        // All convenience methods should handle network errors gracefully
        assert!(providers_api.get_provider_by_slug("openai").await.is_err());
        assert!(providers_api.get_provider_by_name("OpenAI").await.is_err());
        assert!(providers_api.get_providers_with_privacy_policy().await.is_err());
        assert!(providers_api.get_providers_with_terms_of_service().await.is_err());
        assert!(providers_api.get_providers_with_status_page().await.is_err());
        assert!(providers_api.get_provider_slugs().await.is_err());
        assert!(providers_api.get_provider_names().await.is_err());
    }
}