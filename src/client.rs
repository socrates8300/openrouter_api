// openrouter_api/src/client.rs

use crate::error::{Error, Result};
#[allow(unused_imports)]
use crate::types;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use std::marker::PhantomData;
use std::time::Duration;
use url::Url;

/// Client configuration containing API key, base URL, and additional settings.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: Option<String>,
    pub base_url: Url,
    pub http_referer: Option<String>,
    pub site_title: Option<String>,
    pub timeout: Duration,
}

impl ClientConfig {
    /// Build HTTP headers required for making API calls.
    /// Returns an error if any header value cannot be constructed.
    pub fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if let Some(ref key) = self.api_key {
            let auth_header = HeaderValue::from_str(&format!("Bearer {}", key))
                .map_err(|e| Error::ConfigError(format!("Invalid API key header format: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_header);
        }
        // Content-Type header is always valid.
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Some(ref referer) = self.http_referer {
            let ref_value = HeaderValue::from_str(referer)
                .map_err(|e| Error::ConfigError(format!("Invalid Referer header: {}", e)))?;
            headers.insert("Referer", ref_value);
        }
        if let Some(ref title) = self.site_title {
            let title_value = HeaderValue::from_str(title)
                .map_err(|e| Error::ConfigError(format!("Invalid Title header: {}", e)))?;
            headers.insert("X-Title", title_value);
        }
        Ok(headers)
    }
}

// Type‑state markers.
pub struct Unconfigured;
pub struct NoAuth;
pub struct Ready;

/// Main OpenRouter client using a type‑state builder pattern.
pub struct OpenRouterClient<State = Unconfigured> {
    pub config: ClientConfig,
    pub http_client: Option<reqwest::Client>,
    pub _state: PhantomData<State>,
}

impl OpenRouterClient<Unconfigured> {
    /// Creates a new unconfigured client.
    pub fn new() -> Self {
        Self {
            config: ClientConfig {
                api_key: None,
                // Default base URL; can be overridden with with_base_url().
                base_url: "https://openrouter.ai/api/v1/".parse().unwrap(),
                http_referer: None,
                site_title: None,
                timeout: Duration::from_secs(30),
            },
            http_client: None,
            _state: PhantomData,
        }
    }

    /// Sets the base URL and transitions to the NoAuth state.
    /// The base URL must include a trailing slash.
    pub fn with_base_url(
        mut self,
        base_url: impl Into<String>,
    ) -> Result<OpenRouterClient<NoAuth>> {
        self.config.base_url = Url::parse(&base_url.into()).map_err(|e| Error::ApiError {
            code: 400,
            message: format!("Invalid base URL: {}", e),
            metadata: None,
        })?;
        Ok(self.transition_to_no_auth())
    }

    fn transition_to_no_auth(self) -> OpenRouterClient<NoAuth> {
        OpenRouterClient {
            config: self.config,
            http_client: None,
            _state: PhantomData,
        }
    }
}

impl OpenRouterClient<NoAuth> {
    /// Supplies the API key and transitions to the Ready state.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Result<OpenRouterClient<Ready>> {
        self.config.api_key = Some(api_key.into());
        self.transition_to_ready()
    }

    /// Optionally sets the request timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
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

    fn transition_to_ready(self) -> Result<OpenRouterClient<Ready>> {
        let headers = self.config.build_headers()?;
        let http_client = reqwest::Client::builder()
            .timeout(self.config.timeout)
            .default_headers(headers)
            .build()
            .map_err(|e| Error::ConfigError(format!("Failed to create HTTP client: {}", e)))?;
        Ok(OpenRouterClient {
            config: self.config,
            http_client: Some(http_client),
            _state: PhantomData,
        })
    }
}

impl OpenRouterClient<Ready> {
    /// Provides access to the chat endpoint.
    /// Returns an error if the HTTP client is missing.
    pub fn chat(&self) -> Result<crate::api::chat::ChatApi> {
        let client = self
            .http_client
            .clone()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;
        Ok(crate::api::chat::ChatApi::new(client, &self.config))
    }

    /// Returns a new request builder for the completions endpoint.
    pub fn completion_request(
        &self,
        messages: Vec<crate::types::chat::Message>,
    ) -> crate::api::request::RequestBuilder<serde_json::Value> {
        let extra_params = serde_json::json!({});
        crate::api::request::RequestBuilder::new("openai/gpt-4", messages, extra_params)
    }

    /// Provides access to the web search endpoint.
    /// Returns an error if the HTTP client is missing.
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

    /// Example chat completion method.
    pub async fn chat_completion(
        &self,
        request: crate::types::chat::ChatCompletionRequest,
    ) -> Result<crate::types::chat::ChatCompletionResponse> {
        // Build the full URL by joining relative path.
        let url = self
            .config
            .base_url
            .join("chat/completions")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("URL join error: {}", e),
                metadata: None,
            })?;

        let client = self
            .http_client
            .as_ref()
            .ok_or_else(|| Error::ConfigError("HTTP client is missing".into()))?;

        let response = client
            .post(url)
            .headers(self.config.build_headers()?)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::ApiError {
                code: response.status().as_u16(),
                message: response.text().await?,
                metadata: None,
            });
        }
        let chat_response: crate::types::chat::ChatCompletionResponse =
            self.handle_response(response).await?;
        // Validate any tool calls in the response.
        self.validate_tool_calls(&chat_response)?;
        Ok(chat_response)
    }

    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: body.clone(),
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
            message: format!("Failed to decode JSON: {}. Body was: {}", e, body),
            metadata: None,
        })
    }

    pub fn validate_tool_calls(
        &self,
        response: &crate::types::chat::ChatCompletionResponse,
    ) -> Result<()> {
        for choice in &response.choices {
            if let Some(tool_calls) = &choice.message.tool_calls {
                for tc in tool_calls {
                    // When streaming, the first chunk only will contain "function".
                    // See: https://platform.openai.com/docs/guides/function-calling?api-mode=chat#streaming
                    if let Some(kind) = &tc.kind {
                        if kind != "function" {
                            return Err(Error::SchemaValidationError(format!(
                                "Invalid tool call kind: {}. Expected 'function'",
                                kind
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
