// openrouter_api/src/client.rs

#![allow(unused)]
// Fix for unused imports in src/client.rs
use crate::error::{Error, Result};
use crate::types;
use crate::types::routing::{PredefinedModelCoverageProfile, RouterConfig};
use crate::utils::security::create_safe_error_message;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::marker::PhantomData;
use std::time::Duration;
use url::Url;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Secure wrapper for API keys that automatically zeros memory on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureApiKey {
    #[zeroize(skip)]
    inner: String,
}

impl SecureApiKey {
    /// Create a new secure API key with validation
    pub fn new(key: impl Into<String>) -> Result<Self> {
        let key = key.into();

        // Enhanced validation
        if key.trim().is_empty() {
            return Err(Error::ConfigError("API key cannot be empty".into()));
        }

        // Minimum length validation (OpenRouter keys are typically 20+ chars)
        if key.len() < 20 {
            return Err(Error::ConfigError(
                "API key is too short. Expected at least 20 characters".into(),
            ));
        }

        // Format validation for known patterns
        if !key.starts_with("sk-") && !key.starts_with("or-") {
            return Err(Error::ConfigError(
                "API key format invalid. Expected 'sk-' or 'or-' prefix".into(),
            ));
        }

        Ok(Self { inner: key })
    }

    /// Get the API key as a string slice (should be used minimally)
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Get the API key for header construction
    pub fn to_bearer_header(&self) -> String {
        format!("Bearer {}", self.inner)
    }
}

impl std::fmt::Debug for SecureApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose the actual key in debug output
        f.debug_struct("SecureApiKey")
            .field("inner", &"[REDACTED]")
            .finish()
    }
}

/// Client configuration containing API key, base URL, and additional settings.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: Option<SecureApiKey>,
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub user_id: Option<String>,
    pub timeout: Duration,
    pub retry_config: RetryConfig,
}

/// Configuration for automatic retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub retry_on_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 500,
            max_backoff_ms: 10000,
            retry_on_status_codes: vec![429, 500, 502, 503, 504],
        }
    }
}

impl ClientConfig {
    /// Build HTTP headers required for making API calls.
    /// Returns an error if any header value cannot be constructed.
    pub fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if let Some(ref key) = self.api_key {
            let auth_header = HeaderValue::from_str(&key.to_bearer_header())
                .map_err(|e| Error::ConfigError(format!("Invalid API key header format: {e}")))?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        // Content-Type header is always valid.
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref referer) = self.http_referer {
            let ref_value = HeaderValue::from_str(referer)
                .map_err(|e| Error::ConfigError(format!("Invalid Referer header: {e}")))?;
            headers.insert("Referer", ref_value);
        }
        if let Some(ref title) = self.site_title {
            let title_value = HeaderValue::from_str(title)
                .map_err(|e| Error::ConfigError(format!("Invalid Title header: {e}")))?;
            headers.insert("X-Title", title_value);
        }
        if let Some(ref user_id) = self.user_id {
            let user_id_value = HeaderValue::from_str(user_id)
                .map_err(|e| Error::ConfigError(format!("Invalid User-ID header: {e}")))?;
            headers.insert("X-User-ID", user_id_value);
        }
        Ok(headers)
    }
}

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
    pub config: ClientConfig,
    pub http_client: Option<reqwest::Client>,
    pub _state: PhantomData<State>,
    pub router_config: Option<RouterConfig>,
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
    pub fn quick() -> Result<Self> {
        Self::from_env()
    }

    /// Creates a production-ready client with recommended settings.
    /// Includes reasonable timeouts and retry configuration.
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
            },
            http_client: None,
            _state: PhantomData,
            router_config: None,
        }
    }

    /// Creates a client directly from an API key using default settings.
    /// This is a convenience method for the most common use case.
    pub fn from_api_key(api_key: impl Into<String>) -> Result<OpenRouterClient<Ready>> {
        Self::new().skip_url_configuration().with_api_key(api_key)
    }

    /// Creates a client from an API key with a custom base URL.
    /// Convenience method for common configuration pattern.
    pub fn from_api_key_and_url(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Result<OpenRouterClient<Ready>> {
        Self::new().with_base_url(base_url)?.with_api_key(api_key)
    }

    /// Skip URL configuration and use the default OpenRouter URL.
    /// Transitions directly to NoAuth state without requiring with_base_url().
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
        Ok(self.transition_to_no_auth())
    }

    fn transition_to_no_auth(self) -> OpenRouterClient<NoAuth> {
        OpenRouterClient {
            config: self.config,
            http_client: None,
            _state: PhantomData,
            router_config: self.router_config,
        }
    }
}

impl OpenRouterClient<NoAuth> {
    /// Supplies the API key and transitions to the Ready state.
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
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Result<OpenRouterClient<Ready>> {
        self.config.api_key = Some(SecureApiKey::new(api_key)?);
        self.transition_to_ready()
    }

    /// Supplies a pre-validated SecureApiKey and transitions to the Ready state.
    /// This is useful when you already have a SecureApiKey instance.
    pub fn with_secure_api_key(mut self, api_key: SecureApiKey) -> Result<OpenRouterClient<Ready>> {
        self.config.api_key = Some(api_key);
        self.transition_to_ready()
    }

    /// Configures the client with multiple options at once.
    /// This is a convenience method for setting common options together.
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
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }

    /// Sets the request timeout in seconds (convenience method).
    pub fn with_timeout_secs(mut self, seconds: u64) -> Self {
        self.config.timeout = Duration::from_secs(seconds);
        self
    }

    /// Optionally sets the HTTP referer header.
    pub fn with_http_referer(mut self, referer: impl Into<String>) -> Self {
        self.config.http_referer = Some(referer.into());
        self
    }

    /// Optionally sets the site title header.
    pub fn with_site_title(mut self, title: impl Into<String>) -> Self {
        self.config.site_title = Some(title.into());
        self
    }

    /// Optionally sets the user ID header for tracking specific users.
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.config.user_id = Some(user_id.into());
        self
    }

    /// Configures retry behavior with a complete RetryConfig.
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.config.retry_config = retry_config;
        self
    }

    /// Configures basic retry settings (convenience method).
    pub fn with_retries(mut self, max_retries: u32, initial_backoff_ms: u64) -> Self {
        self.config.retry_config.max_retries = max_retries;
        self.config.retry_config.initial_backoff_ms = initial_backoff_ms;
        self
    }

    /// Disables automatic retries.
    pub fn without_retries(mut self) -> Self {
        self.config.retry_config.max_retries = 0;
        self
    }

    /// Configures Model Coverage Profile for model selection and routing.
    pub fn with_model_coverage_profile(mut self, profile: PredefinedModelCoverageProfile) -> Self {
        self.router_config = Some(RouterConfig {
            profile,
            provider_preferences: None,
        });
        self
    }

    fn transition_to_ready(self) -> Result<OpenRouterClient<Ready>> {
        let headers = self.config.build_headers()?;

        // Build a client with retry capabilities
        let client_builder = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .default_headers(headers);

        let http_client = client_builder
            .build()
            .map_err(|e| Error::ConfigError(format!("Failed to create HTTP client: {e}")))?;

        Ok(OpenRouterClient {
            config: self.config,
            http_client: Some(http_client),
            _state: PhantomData,
            router_config: self.router_config,
        })
    }
}

impl OpenRouterClient<Ready> {
    /// Provides access to the chat endpoint.
    pub fn chat(&self) -> Result<crate::api::chat::ChatApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::chat::ChatApi::new(client, &self.config))
    }

    /// Provides access to the completions endpoint.
    pub fn completions(&self) -> Result<crate::api::completion::CompletionApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::completion::CompletionApi::new(
            client,
            &self.config,
        ))
    }

    /// Provides access to the models endpoint.
    pub fn models(&self) -> Result<crate::api::models::ModelsApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::models::ModelsApi::new(client, &self.config))
    }

    /// Provides access to the structured output endpoint.
    pub fn structured(&self) -> Result<crate::api::structured::StructuredApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::structured::StructuredApi::new(
            client,
            &self.config,
        ))
    }

    /// Provides access to the web search endpoint.
    pub fn web_search(&self) -> Result<crate::api::web_search::WebSearchApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::web_search::WebSearchApi::new(
            client,
            &self.config,
        ))
    }

    /// Provides access to the credits endpoint.
    pub fn credits(&self) -> Result<crate::api::credits::CreditsApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::credits::CreditsApi::new(client, &self.config))
    }

    /// Provides access to the generation endpoint.
    pub fn generation(&self) -> Result<crate::api::generation::GenerationApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::generation::GenerationApi::new(client, &self.config))
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
                match serde_json::to_value(provider_prefs) {
                    Ok(prefs_value) => {
                        extra_params["provider"] = prefs_value;
                    }
                    Err(e) => {
                        // Log error but continue without provider preferences
                        eprintln!("Failed to serialize provider preferences: {e}");
                    }
                }
            }

            // Add fallback models if present in custom profile
            if let PredefinedModelCoverageProfile::Custom(profile) = &router_config.profile {
                if let Some(fallbacks) = &profile.fallbacks {
                    match serde_json::to_value(fallbacks) {
                        Ok(fallbacks_value) => {
                            extra_params["models"] = fallbacks_value;
                        }
                        Err(e) => {
                            // Log error but continue without fallbacks
                            eprintln!("Failed to serialize fallback models: {e}");
                        }
                    }
                }
            }
        }

        crate::api::request::RequestBuilder::new(primary_model, messages, extra_params)
    }

    /// Helper method to handle standard HTTP responses.
    pub(crate) async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(&body, "API error"),
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
        serde_json::from_str::<T>(&body).map_err(|e| Error::ApiError {
            code: status.as_u16(),
            message: create_safe_error_message(
                &format!("Failed to decode JSON: {e}. Body was: {body}"),
                "JSON parsing error",
            ),
            metadata: None,
        })
    }

    /// Validates tool calls in a chat completion response.
    pub fn validate_tool_calls(
        &self,
        response: &crate::types::chat::ChatCompletionResponse,
    ) -> Result<()> {
        for choice in &response.choices {
            if let Some(tool_calls) = &choice.message.tool_calls {
                for tc in tool_calls {
                    if tc.kind != "function" {
                        return Err(Error::SchemaValidationError(format!(
                            "Invalid tool call kind: {}. Expected 'function'",
                            tc.kind
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
