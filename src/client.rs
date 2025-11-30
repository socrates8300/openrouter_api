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

pub mod config;
pub use config::*;



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
                // Default to 10MB limit
                max_response_bytes: 10 * 1024 * 1024,
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

    /// Sets the maximum response size in bytes.
    /// Defaults to 10MB (10 * 1024 * 1024 bytes).
    pub fn with_max_response_bytes(mut self, bytes: usize) -> Self {
        self.config.max_response_bytes = bytes;
        self
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

    /// Sets the maximum response size in bytes.
    /// Defaults to 10MB (10 * 1024 * 1024 bytes).
    pub fn with_max_response_bytes(mut self, bytes: usize) -> Self {
        self.config.max_response_bytes = bytes;
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
        crate::api::chat::ChatApi::new(client, &self.config)
    }

    /// Provides access to the completions endpoint.
    pub fn completions(&self) -> Result<crate::api::completion::CompletionApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::completion::CompletionApi::new(client, &self.config)
    }

    /// Provides access to the models endpoint.
    pub fn models(&self) -> Result<crate::api::models::ModelsApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::models::ModelsApi::new(client, &self.config)
    }

    /// Provides access to the structured output endpoint.
    pub fn structured(&self) -> Result<crate::api::structured::StructuredApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::structured::StructuredApi::new(client, &self.config)
    }

    /// Provides access to the web search endpoint.
    pub fn web_search(&self) -> Result<crate::api::web_search::WebSearchApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::web_search::WebSearchApi::new(client, &self.config)
    }

    /// Provides access to the credits endpoint.
    pub fn credits(&self) -> Result<crate::api::credits::CreditsApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::credits::CreditsApi::new(client, &self.config)
    }

    /// Provides access to the analytics endpoint.
    /// Get analytics API instance
    pub fn analytics(&self) -> Result<crate::api::analytics::AnalyticsApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::analytics::AnalyticsApi::new(client, &self.config)
    }

    /// Provides access to the providers endpoint.
    pub fn providers(&self) -> Result<crate::api::providers::ProvidersApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::providers::ProvidersApi::new(client, &self.config)
    }

    /// Provides access to the generation endpoint.
    pub fn generation(&self) -> Result<crate::api::generation::GenerationApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        crate::api::generation::GenerationApi::new(client, &self.config)
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

        // Check Content-Length header first if available
        if let Some(content_length) = response.content_length() {
            if content_length > self.config.max_response_bytes as u64 {
                return Err(Error::ResponseTooLarge(
                    content_length as usize,
                    self.config.max_response_bytes,
                ));
            }
        }

        // Read body with limit
        let body_bytes = response.bytes().await?;
        if body_bytes.len() > self.config.max_response_bytes {
            return Err(Error::ResponseTooLarge(
                body_bytes.len(),
                self.config.max_response_bytes,
            ));
        }

        // Convert to string (lossy is fine here as we expect JSON/text)
        let body = String::from_utf8_lossy(&body_bytes).to_string();
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(&body, "API error"),
                metadata: Some(serde_json::json!({
                    "response_text_length": body.len(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status_code": status.as_u16(),
                    "has_structured_error": false
                })),
            });
        }
        if body.trim().is_empty() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: "Empty response body".into(),
                metadata: Some(serde_json::json!({
                    "response_text_length": 0,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "status_code": status.as_u16(),
                    "error_type": "empty_response"
                })),
            });
        }
        serde_json::from_str::<T>(&body).map_err(|e| Error::ApiError {
            code: status.as_u16(),
            message: create_safe_error_message(
                &format!("Failed to decode JSON: {e}"),
                "JSON parsing error",
            ),
            metadata: Some(serde_json::json!({
                "response_text_length": body.len(),
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "status_code": status.as_u16(),
                "error_type": "json_parsing",
                "parsing_error": e.to_string()
            })),
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
