/// Web search request and response types
use serde::{Deserialize, Serialize};

/// Request type for performing a web search.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct WebSearchRequest {
    /// The search query string.
    pub query: String,
    /// Optionally specify the number of results to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_results: Option<u32>,
    // Additional parameters can be added here in the future.
}

/// A single search result.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WebSearchResult {
    /// The title of the search result.
    pub title: String,
    /// The URL of the search result.
    pub url: String,
    /// An optional snippet or preview text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

/// Response type returned by the web search API.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WebSearchResponse {
    /// The original search query.
    pub query: String,
    /// The list of search results.
    pub results: Vec<WebSearchResult>,
    /// The total number of results available.
    pub total_results: u32,
}
