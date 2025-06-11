/// Web search API implementation
use crate::{
    client::ClientConfig,
    error::{Error, Result},
    types::web_search::{WebSearchRequest, WebSearchResponse},
    utils::security::create_safe_error_message,
};
use reqwest::Client;
use serde::de::DeserializeOwned;

pub struct WebSearchApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl WebSearchApi {
    /// Creates a new WebSearchApi instance given a reqwest client and a client configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Performs a web search with the given request and returns a structured response.
    pub async fn search(&self, request: WebSearchRequest) -> Result<WebSearchResponse> {
        // Join the base URL with the relative path "web/search".
        let url = self
            .config
            .base_url
            .join("web/search")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL for web search: {}", e),
                metadata: None,
            })?;

        let response = self
            .client
            .post(url)
            .headers(self.config.build_headers()?)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status_code = response.status().as_u16();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status_code,
                message: create_safe_error_message(&body, "Web search API request failed"),
                metadata: None,
            });
        }

        let search_response: WebSearchResponse = self.handle_response(response).await?;
        Ok(search_response)
    }

    /// Internal helper to deserialize a response while handling errors.
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(&body, "Web search response error"),
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
                &format!("Failed to decode JSON: {}. Body was: {}", e, body),
                "Web search JSON parsing error",
            ),
            metadata: None,
        })
    }
}
