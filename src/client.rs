// openrouter_api/src/client.rs

use crate::error::{Error, Result};

/// Note: These imports are used to implement the client builder pattern.
use crate::types::routing::{PredefinedModelCoverageProfile, RouterConfig};
use std::marker::PhantomData;
use std::time::Duration;
use url::Url;

pub mod config;
pub use config::*;

/// Routing shortcut for high-throughput.
pub const ROUTING_NITRO: &str = ":nitro";
/// Routing shortcut for lowest price.
pub const ROUTING_FLOOR: &str = ":floor";
/// Routing shortcut for web search.
pub const ROUTING_ONLINE: &str = ":online";

// Type‑state markers.
#[derive(Debug)]
pub struct Unconfigured;
#[derive(Debug)]
pub struct NoAuth;
#[derive(Debug)]
pub struct Ready;

/// Main OpenRouter client using a type‑state builder pattern.
#[derive(Debug)]
pub struct OpenRouterClient<State = Unconfigured> {
    pub(crate) config: ClientConfig,
    pub(crate) http_client: Option<reqwest::Client>,
    pub(crate) _state: PhantomData<State>,
    pub(crate) router_config: Option<RouterConfig>,
    pub(crate) cached_api_config: Option<ApiConfig>,
    /// Shared providers cache persisted across `.providers()` calls
    pub(crate) providers_cache: Option<
        std::sync::Arc<
            std::sync::Mutex<
                crate::utils::cache::Cache<String, crate::types::providers::ProvidersResponse>,
            >,
        >,
    >,
}

impl Default for OpenRouterClient<Unconfigured> {
    fn default() -> Self {
        Self::new()
    }
}

// Convenience constructors and methods
impl OpenRouterClient<Ready> {
    /// Creates a ready-to-use client from environment variables.
    /// Looks for OPENROUTER_API_KEY or OR_API_KEY in the environment.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// // Set OPENROUTER_API_KEY in your environment
    /// let client = OpenRouterClient::from_env()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use = "returns a configured client that should be used for API calls"]
    pub fn from_env() -> Result<Self> {
        let api_key = crate::utils::auth::load_api_key_from_env()?;
        OpenRouterClient::from_api_key(api_key)
    }

    /// Creates a client from environment with custom configuration.
    /// This is a convenience method for common configuration patterns.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// // Set OPENROUTER_API_KEY in your environment
    /// let client = OpenRouterClient::from_env_with_config(
    ///     Some("https://myapp.com"),
    ///     Some("My Application")
    /// )?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use = "returns a configured client that should be used for API calls"]
    pub fn from_env_with_config(
        referer: Option<impl Into<String>>,
        title: Option<impl Into<String>>,
    ) -> Result<Self> {
        let api_key = crate::utils::auth::load_api_key_from_env()?;
        OpenRouterClient::new()
            .skip_url_configuration()
            .configure(api_key, referer, title)
    }

    /// Creates a quick client for development/testing with minimal configuration.
    /// Uses environment API key and default settings.
    #[must_use = "returns a configured client that should be used for API calls"]
    pub fn quick() -> Result<Self> {
        Self::from_env()
    }

    /// Creates a production-ready client with recommended settings.
    /// Includes reasonable timeouts and retry configuration.
    #[must_use = "returns a configured client that should be used for API calls"]
    pub fn production(
        api_key: impl Into<String>,
        app_name: impl Into<String>,
        app_url: impl Into<String>,
    ) -> Result<Self> {
        OpenRouterClient::new()
            .skip_url_configuration()
            .with_timeout_secs(60) // 60 second timeout for production
            .with_retries(5, 1000) // 5 retries with 1s initial backoff
            .configure(api_key, Some(app_url), Some(app_name))
    }
}

impl OpenRouterClient<Unconfigured> {
    /// Creates a new unconfigured client with default settings.
    /// Uses the default OpenRouter base URL: <https://openrouter.ai/api/v1/>
    #[must_use = "returns a client builder that should be configured and used for API calls"]
    pub fn new() -> Self {
        Self {
            config: ClientConfig {
                api_key: None,
                // Default base URL; can be overridden with with_base_url().
                base_url: "https://openrouter.ai/api/v1/".parse().unwrap(),
                http_referer: None,
                site_title: None,
                user_id: None,
                timeout: Duration::from_secs(30),
                retry_config: RetryConfig::default(),
                // Default to 10MB limit
                max_response_bytes: 10 * 1024 * 1024,
            },
            http_client: None,
            _state: PhantomData,
            router_config: None,
            cached_api_config: None,
            providers_cache: None,
        }
    }

    /// Creates a client directly from an API key using default settings.
    /// This is a convenience method for the most common use case.
    #[must_use = "returns a configured client that should be used for API calls"]
    pub fn from_api_key(api_key: impl Into<String>) -> Result<OpenRouterClient<Ready>> {
        Self::new().skip_url_configuration().with_api_key(api_key)
    }

    /// Creates a client from an API key with a custom base URL.
    /// Convenience method for common configuration pattern.
    #[must_use = "returns a configured client that should be used for API calls"]
    pub fn from_api_key_and_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<OpenRouterClient<Ready>> {
        Self::new().with_base_url(base_url)?.with_api_key(api_key)
    }

    /// Skip URL configuration and use the default OpenRouter URL.
    /// Transitions directly to NoAuth state without requiring with_base_url().
    #[must_use = "returns the updated client that should be configured and used for API calls"]
    pub fn skip_url_configuration(self) -> OpenRouterClient<NoAuth> {
        self.transition_to_no_auth()
    }

    /// Sets a custom base URL and transitions to the NoAuth state.
    /// The base URL should include the protocol (https://) and path.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// let client = OpenRouterClient::new()
    ///     .with_base_url("https://custom-proxy.example.com/api/v1/")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_base_url(
        mut self,
        base_url: impl Into<String>,
    ) -> Result<OpenRouterClient<NoAuth>> {
        let url_str = base_url.into();
        self.config.base_url = Url::parse(&url_str).map_err(|e| {
            Error::ConfigError(format!(
                "Invalid base URL '{url_str}': {e}. Expected format: 'https://api.example.com/v1/'"
            ))
        })?;
        crate::utils::https::enforce_https(&self.config.base_url)?;
        Ok(self.transition_to_no_auth())
    }

    fn transition_to_no_auth(self) -> OpenRouterClient<NoAuth> {
        OpenRouterClient {
            config: self.config,
            http_client: None,
            _state: PhantomData,
            router_config: self.router_config,
            cached_api_config: None,
            providers_cache: None,
        }
    }

    /// Sets maximum response size in bytes.
    /// Defaults to 10MB (10 * 1024 * 1024 bytes).
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_max_response_bytes(mut self, bytes: usize) -> Self {
        self.config.max_response_bytes = bytes;
        self
    }

    /// Helper to append a routing shortcut to a model ID.
    #[must_use = "returns a formatted model ID string that should be used in requests"]
    pub fn model_with_shortcut(model: &str, shortcut: &str) -> String {
        format!("{}{}", model, shortcut)
    }
}

impl OpenRouterClient<NoAuth> {
    /// Supplies API key and transitions to Ready state.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// let client = OpenRouterClient::new()
    ///     .skip_url_configuration()
    ///     .with_api_key("sk-your-api-key-here")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Result<OpenRouterClient<Ready>> {
        self.config.api_key = Some(SecureApiKey::new(api_key)?);
        self.transition_to_ready()
    }

    /// Supplies a pre-validated SecureApiKey and transitions to the Ready state.
    /// This is useful when you already have a SecureApiKey instance.
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_secure_api_key(mut self, api_key: SecureApiKey) -> Result<OpenRouterClient<Ready>> {
        self.config.api_key = Some(api_key);
        self.transition_to_ready()
    }

    /// Configures the client with multiple options at once.
    /// This is a convenience method for setting common options together.
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn configure(
        mut self,
        api_key: impl Into<String>,
        referer: Option<impl Into<String>>,
        title: Option<impl Into<String>>,
    ) -> Result<OpenRouterClient<Ready>> {
        if let Some(ref_val) = referer {
            self.config.http_referer = Some(ref_val.into());
        }
        if let Some(title_val) = title {
            self.config.site_title = Some(title_val.into());
        }
        self.config.api_key = Some(SecureApiKey::new(api_key)?);
        self.transition_to_ready()
    }

    /// Sets the request timeout.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    /// use std::time::Duration;
    ///
    /// let client = OpenRouterClient::new()
    ///     .skip_url_configuration()
    ///     .with_timeout(Duration::from_secs(60))
    ///     .with_api_key("sk-your-api-key")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Sets request timeout in seconds (convenience method).
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_timeout_secs(mut self, seconds: u64) -> Self {
        self.config.timeout = Duration::from_secs(seconds);
        self
    }

    /// Optionally sets HTTP referer header.
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_http_referer(mut self, referer: impl Into<String>) -> Self {
        self.config.http_referer = Some(referer.into());
        self
    }

    /// Optionally sets site title header.
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_site_title(mut self, title: impl Into<String>) -> Self {
        self.config.site_title = Some(title.into());
        self
    }

    /// Optionally sets user ID header for tracking specific users.
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.config.user_id = Some(user_id.into());
        self
    }

    /// Configures retry behavior with a complete RetryConfig.
    #[must_use = "returns the updated client that should be used for API calls"]
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.config.retry_config = retry_config;
        self
    }

    /// Configures basic retry settings (convenience method).
    #[must_use = "returns updated client that should be used for API calls"]
    pub fn with_retries(mut self, max_retries: u32, initial_backoff_ms: u64) -> Self {
        self.config.retry_config.max_retries = max_retries;
        self.config.retry_config.initial_backoff_ms = initial_backoff_ms;
        self
    }

    /// Sets maximum response size in bytes.
    /// Defaults to 10MB (10 * 1024 * 1024 bytes).
    #[must_use = "returns updated client that should be used for API calls"]
    pub fn with_max_response_bytes(mut self, bytes: usize) -> Self {
        self.config.max_response_bytes = bytes;
        self
    }

    /// Disables automatic retries.
    #[must_use = "returns updated client that should be used for API calls"]
    pub fn without_retries(mut self) -> Self {
        self.config.retry_config.max_retries = 0;
        self
    }

    /// Configures Model Coverage Profile for model selection and routing.
    #[must_use = "returns updated client that should be used for API calls"]
    pub fn with_model_coverage_profile(mut self, profile: PredefinedModelCoverageProfile) -> Self {
        self.router_config = Some(RouterConfig {
            profile,
            provider_preferences: None,
        });
        self
    }

    /// Enables Zero Data Retention (ZDR) by setting data collection to "deny".
    #[must_use = "returns updated client that should be used for API calls"]
    pub fn with_zdr(mut self) -> Self {
        let router_config = self.router_config.get_or_insert(RouterConfig {
            profile: PredefinedModelCoverageProfile::LowestCost,
            provider_preferences: Some(crate::types::provider::ProviderPreferences::new()),
        });

        let prefs = router_config
            .provider_preferences
            .get_or_insert(crate::types::provider::ProviderPreferences::new());
        prefs.data_collection = Some("deny".to_string());
        self
    }

    fn transition_to_ready(self) -> Result<OpenRouterClient<Ready>> {
        let headers = self.config.build_headers()?;

        // Build a client with retry capabilities
        let client_builder = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .tcp_keepalive(Duration::from_secs(60))
            .default_headers(headers);

        let http_client = client_builder
            .build()
            .map_err(|e| Error::ConfigError(format!("Failed to create HTTP client: {e}")))?;

        // Cache the ApiConfig so accessor methods don't rebuild it each time
        let api_config = self.config.to_api_config()?;

        Ok(OpenRouterClient {
            config: self.config,
            http_client: Some(http_client),
            _state: PhantomData,
            router_config: self.router_config,
            cached_api_config: Some(api_config),
            providers_cache: Some(std::sync::Arc::new(std::sync::Mutex::new(
                crate::utils::cache::Cache::new(std::time::Duration::from_secs(300)),
            ))),
        })
    }
}

impl OpenRouterClient<Ready> {
    /// Returns the cached HTTP client, or an error if missing.
    fn get_client_and_config(&self) -> Result<(reqwest::Client, ApiConfig)> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        let api_config = self
            .cached_api_config
            .clone()
            .ok_or_else(|| Error::ConfigError("API config is missing".into()))?;
        Ok((client, api_config))
    }

    /// Provides access to the chat endpoint.
    pub fn chat(&self) -> Result<crate::api::chat::ChatApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::chat::ChatApi { client, config })
    }

    /// Provides access to the completions endpoint.
    pub fn completions(&self) -> Result<crate::api::completion::CompletionApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::completion::CompletionApi { client, config })
    }

    /// Provides access to the models endpoint.
    pub fn models(&self) -> Result<crate::api::models::ModelsApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::models::ModelsApi { client, config })
    }

    /// Provides access to the structured output endpoint.
    pub fn structured(&self) -> Result<crate::api::structured::StructuredApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::structured::StructuredApi { client, config })
    }

    /// Provides access to the web search endpoint.
    pub fn web_search(&self) -> Result<crate::api::web_search::WebSearchApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::web_search::WebSearchApi { client, config })
    }

    /// Provides access to the credits endpoint.
    pub fn credits(&self) -> Result<crate::api::credits::CreditsApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::credits::CreditsApi { client, config })
    }

    /// Provides access to the analytics endpoint.
    pub fn analytics(&self) -> Result<crate::api::analytics::AnalyticsApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::analytics::AnalyticsApi { client, config })
    }

    /// Provides access to the providers endpoint.
    /// The cache is shared across calls so repeated `.providers()?.get_providers()` hits cache.
    pub fn providers(&self) -> Result<crate::api::providers::ProvidersApi> {
        let (client, config) = self.get_client_and_config()?;
        let cache = self.providers_cache.clone().ok_or_else(|| {
            crate::error::Error::ConfigError("Providers cache not initialized".into())
        })?;
        Ok(crate::api::providers::ProvidersApi {
            client,
            config,
            cache,
        })
    }

    /// Provides access to the key info endpoint.
    pub fn key_info(&self) -> Result<crate::api::key_info::KeyInfoApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::key_info::KeyInfoApi { client, config })
    }

    /// Provides access to the embeddings endpoint.
    pub fn embeddings(&self) -> Result<crate::api::embeddings::EmbeddingsApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::embeddings::EmbeddingsApi { client, config })
    }

    /// Provides access to the generation endpoint.
    pub fn generation(&self) -> Result<crate::api::generation::GenerationApi> {
        let (client, config) = self.get_client_and_config()?;
        Ok(crate::api::generation::GenerationApi { client, config })
    }

    /// Returns a new request builder for chat completions that supports MCP.
    pub fn chat_request_builder(
        &self,
        messages: Vec<crate::types::chat::Message>,
    ) -> crate::api::request::RequestBuilder<serde_json::Value> {
        // Apply the model coverage profile if available
        let primary_model = if let Some(router_config) = &self.router_config {
            match &router_config.profile {
                PredefinedModelCoverageProfile::Custom(profile) => profile.primary.clone(),
                PredefinedModelCoverageProfile::LowestLatency => "openai/gpt-3.5-turbo".to_string(),
                PredefinedModelCoverageProfile::LowestCost => "openai/gpt-3.5-turbo".to_string(),
                PredefinedModelCoverageProfile::HighestQuality => {
                    "anthropic/claude-3-opus-20240229".to_string()
                }
            }
        } else {
            "openai/gpt-4o".to_string()
        };

        // Set up basic params
        let mut extra_params = serde_json::json!({});

        // Add provider preferences if set
        if let Some(router_config) = &self.router_config {
            if let Some(provider_prefs) = &router_config.provider_preferences {
                // Convert to Value and handle errors
                if let Ok(prefs_value) = serde_json::to_value(provider_prefs) {
                    extra_params["provider"] = prefs_value;
                }
            }

            // Add fallback models if present in custom profile
            if let PredefinedModelCoverageProfile::Custom(profile) = &router_config.profile {
                if let Some(fallbacks) = &profile.fallbacks {
                    if let Ok(fallbacks_value) = serde_json::to_value(fallbacks) {
                        extra_params["models"] = fallbacks_value;
                    }
                }
            }
        }

        crate::api::request::RequestBuilder::new(primary_model, messages, extra_params)
    }

    /// Validates tool calls in a chat completion response.
    ///
    /// Note: With the introduction of `ToolType` enum, tool call kind is now
    /// type-safe at compile time. The only valid variant is `ToolType::Function`,
    /// so this validation is redundant. This function is kept for API compatibility
    /// but always returns `Ok(())`.
    pub fn validate_tool_calls(
        &self,
        _response: &crate::types::chat::ChatCompletionResponse,
    ) -> Result<()> {
        // Type system guarantees tool calls are valid
        Ok(())
    }
}

#[cfg(test)]
mod tests;
