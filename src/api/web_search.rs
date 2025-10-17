/// Web search API implementation
use crate::{
    client::ClientConfig,
    error::{Error, Result},
    types::web_search::{WebSearchRequest, WebSearchResponse},
    utils::retry::operations::WEB_SEARCH,
    utils::{retry::execute_with_retry_builder, retry::handle_response_json},
};
use reqwest::Client;

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
                message: format!("Invalid URL for web search: {e}"),
                metadata: None,
            })?;

        // Build headers once to avoid closure issues
        let headers = self.config.build_headers()?;

        // Execute request with retry logic
        let response = execute_with_retry_builder(&self.config.retry_config, WEB_SEARCH, || {
            self.client
                .post(url.clone())
                .headers(headers.clone())
                .json(&request)
        })
        .await?;

        // Handle response with consistent error parsing
        let search_response: WebSearchResponse =
            handle_response_json::<WebSearchResponse>(response, WEB_SEARCH).await?;
        Ok(search_response)
    }

    // Note: The handle_response method has been replaced by the centralized
    // handle_response_json utility in utils::retry for consistency across all endpoints.
}
