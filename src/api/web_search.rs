/// Web Search API implementation
use crate::{
    error::{Error, Result},
    types::web_search::{WebSearchRequest, WebSearchResponse},
    utils::retry::operations::WEB_SEARCH,
    utils::{
        retry::execute_with_retry_builder, retry::handle_response_json,
        validation::validate_web_search_request,
    },
};
use reqwest::Client;

/// API endpoint for web search integration.
pub struct WebSearchApi {
    pub client: Client,
    pub config: crate::client::ApiConfig,
}

impl WebSearchApi {
    /// Creates a new WebSearchApi with the given reqwest client and configuration.
    #[must_use = "returns an API client that should be used for API calls"]
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Performs a web search with the given request and returns a structured response.
    pub async fn search(&self, request: WebSearchRequest) -> Result<WebSearchResponse> {
        // Validate the request using the validation module
        validate_web_search_request(&request)?;

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

        // Execute request with retry logic
        let response = execute_with_retry_builder(&self.config.retry_config, WEB_SEARCH, || {
            self.client
                .post(url.clone())
                .headers((*self.config.headers).clone())
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
