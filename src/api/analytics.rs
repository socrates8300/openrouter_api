use crate::client::ClientConfig;
use crate::error::{Error, Result};
#[allow(dead_code, unused_imports)]
use crate::types::analytics::{ActivityRequest, ActivityResponse, SortField, SortOrder};
use crate::utils::retry::operations::GET_ACTIVITY;
use crate::utils::{retry::execute_with_retry_builder, retry::handle_response_json};
use reqwest::Client;
use urlencoding::encode;

/// API endpoint for analytics and activity data.
pub struct AnalyticsApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl AnalyticsApi {
    /// Creates a new AnalyticsApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Retrieves activity data for the authenticated user.
    ///
    /// This endpoint returns detailed usage and activity information including
    /// request costs, token usage, latency data, and feature usage statistics.
    ///
    /// # Arguments
    ///
    /// * `request` - ActivityRequest containing filtering and pagination parameters
    ///
    /// # Returns
    ///
    /// Returns an `ActivityResponse` containing:
    /// - List of activity data entries with detailed request information
    /// - Total count of entries matching the query
    /// - Whether more results are available
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The request parameters are invalid (bad date format, invalid sort order, etc.)
    /// - The API request fails (network issues, authentication, etc.)
    /// - The response cannot be parsed
    /// - The server returns an error status code
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    /// use openrouter_api::types::analytics::{ActivityRequest, SortField, SortOrder};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///
    ///     // Get activity for the last 7 days
    ///     let request = ActivityRequest::new()
    ///         .with_start_date("2024-01-01")
    ///         .with_end_date("2024-01-07")
    ///         .with_sort(SortField::CreatedAt)
    ///         .with_order(SortOrder::Descending)
    ///         .with_limit(100);
    ///
    ///     let response = client.analytics()?.get_activity(request).await?;
    ///
    ///     println!("Total requests: {}", response.data.len());
    ///     println!("Total cost: ${:.6}", response.total_cost());
    ///     println!("Total tokens: {}", response.total_tokens());
    ///     println!("Success rate: {:.1}%", response.success_rate());
    ///     println!("Streaming rate: {:.1}%", response.streaming_rate());
    ///
    ///     // Group by model
    ///     let model_groups = response.group_by_model();
    ///     for (model, activities) in model_groups {
    ///         let stats = response.model_stats(&model);
    ///         println!("Model {}: {} requests, ${:.6} total cost",
    ///                  model, stats.request_count, stats.total_cost);
    ///     }
    ///
    ///     // Feature usage percentages
    ///     let features = response.feature_usage_percentages();
    ///     println!("Web search usage: {:.1}%", features.web_search);
    ///     println!("Media usage: {:.1}%", features.media);
    ///     println!("Reasoning usage: {:.1}%", features.reasoning);
    ///     println!("Streaming usage: {:.1}%", features.streaming);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_activity(&self, request: ActivityRequest) -> Result<ActivityResponse> {
        // Validate the request parameters
        request.validate().map_err(|e| Error::ConfigError(e))?;

        // Build the URL with query parameters
        // Handle base_url that may or may not end with /
        let url = if self.config.base_url.path().ends_with('/') {
            self.config.base_url.join("api/v1/activity")
        } else {
            self.config.base_url.join("api/v1/activity")
        }
        .map_err(|e| Error::ApiError {
            code: 400,
            message: format!("Invalid URL for activity endpoint: {e}"),
            metadata: None,
        })?;

        // Build query parameters
        let mut query_params = Vec::new();

        if let Some(start_date) = &request.start_date {
            query_params.push(("start_date", encode(start_date).to_string()));
        }

        if let Some(end_date) = &request.end_date {
            query_params.push(("end_date", encode(end_date).to_string()));
        }

        if let Some(model) = &request.model {
            query_params.push(("model", encode(model).to_string()));
        }

        if let Some(provider) = &request.provider {
            query_params.push(("provider", encode(provider).to_string()));
        }

        if let Some(sort) = &request.sort {
            query_params.push(("sort", encode(sort.as_str()).to_string()));
        }

        if let Some(order) = &request.order {
            query_params.push(("order", encode(order.as_str()).to_string()));
        }

        if let Some(limit) = request.limit {
            query_params.push(("limit", limit.to_string()));
        }

        if let Some(offset) = request.offset {
            query_params.push(("offset", offset.to_string()));
        }

        // Build headers once to avoid closure issues
        let headers = self.config.build_headers()?;

        // Execute request with retry logic
        let response = execute_with_retry_builder(&self.config.retry_config, GET_ACTIVITY, || {
            let mut req_builder = self.client.get(url.clone()).headers(headers.clone());

            // Add query parameters if any
            if !query_params.is_empty() {
                req_builder = req_builder.query(&query_params);
            }

            req_builder
        })
        .await?;

        // Handle response with consistent error parsing
        handle_response_json::<ActivityResponse>(response, GET_ACTIVITY).await
    }

    /// Retrieves activity data for a specific date range with default parameters.
    ///
    /// This is a convenience method that creates an ActivityRequest with the specified
    /// date range and uses sensible defaults for other parameters.
    ///
    /// # Arguments
    ///
    /// * `start_date` - Start date in YYYY-MM-DD format
    /// * `end_date` - End date in YYYY-MM-DD format
    ///
    /// # Returns
    ///
    /// Returns an `ActivityResponse` containing activity data for the specified date range.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///
    ///     // Get activity for the last 7 days
    ///     let response = client.analytics()?
    ///         .get_activity_by_date_range("2024-01-01", "2024-01-07")
    ///         .await?;
    ///
    ///     println!("Found {} requests", response.data.len());
    ///     println!("Total cost: ${:.6}", response.total_cost());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_activity_by_date_range(
        &self,
        start_date: &str,
        end_date: &str,
    ) -> Result<ActivityResponse> {
        let request = ActivityRequest::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_sort(crate::types::analytics::SortField::CreatedAt)
            .with_order(crate::types::analytics::SortOrder::Descending);

        self.get_activity(request).await
    }

    /// Retrieves recent activity data (last 30 days) with default parameters.
    ///
    /// This is a convenience method that retrieves activity for the last 30 days
    /// with sensible defaults for sorting and pagination.
    ///
    /// # Returns
    ///
    /// Returns an `ActivityResponse` containing recent activity data.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///
    ///     // Get recent activity
    ///     let response = client.analytics()?.get_recent_activity().await?;
    ///
    ///     println!("Found {} recent requests", response.data.len());
    ///     println!("Success rate: {:.1}%", response.success_rate());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_recent_activity(&self) -> Result<ActivityResponse> {
        // Calculate date for recent activity
        let end_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let start_date = (chrono::Utc::now()
            - chrono::Duration::days(crate::types::analytics::constants::DEFAULT_RECENT_DAYS))
        .format("%Y-%m-%d")
        .to_string();

        let request = ActivityRequest::new()
            .with_start_date(start_date)
            .with_end_date(end_date)
            .with_sort(crate::types::analytics::SortField::CreatedAt)
            .with_order(crate::types::analytics::SortOrder::Descending)
            .with_limit(crate::types::analytics::constants::MAX_LIMIT);

        self.get_activity(request).await
    }

    /// Retrieves activity data for a specific model.
    ///
    /// This is a convenience method that filters activity data by model name.
    ///
    /// # Arguments
    ///
    /// * `model` - The model name to filter by
    /// * `start_date` - Optional start date in YYYY-MM-DD format
    /// * `end_date` - Optional end date in YYYY-MM-DD format
    ///
    /// # Returns
    ///
    /// Returns an `ActivityResponse` containing activity data for the specified model.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///
    ///     // Get activity for a specific model
    ///     let response = client.analytics()?
    ///         .get_activity_by_model("anthropic/claude-3-opus", None, None)
    ///         .await?;
    ///
    ///     let stats = response.model_stats("anthropic/claude-3-opus");
    ///     println!("Model {}: {} requests", stats.model, stats.request_count);
    ///     println!("Total cost: ${:.6}", stats.total_cost);
    ///     println!("Success rate: {:.1}%", stats.success_rate);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_activity_by_model(
        &self,
        model: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<ActivityResponse> {
        let mut request = ActivityRequest::new()
            .with_model(model)
            .with_sort(crate::types::analytics::SortField::CreatedAt)
            .with_order(crate::types::analytics::SortOrder::Descending);

        if let Some(start) = start_date {
            request = request.with_start_date(start);
        }

        if let Some(end) = end_date {
            request = request.with_end_date(end);
        }

        self.get_activity(request).await
    }

    /// Retrieves activity data for a specific provider.
    ///
    /// This is a convenience method that filters activity data by provider name.
    ///
    /// # Arguments
    ///
    /// * `provider` - The provider name to filter by
    /// * `start_date` - Optional start date in YYYY-MM-DD format
    /// * `end_date` - Optional end date in YYYY-MM-DD format
    ///
    /// # Returns
    ///
    /// Returns an `ActivityResponse` containing activity data for the specified provider.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openrouter_api::OpenRouterClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = OpenRouterClient::from_env()?;
    ///
    ///     // Get activity for a specific provider
    ///     let response = client.analytics()?
    ///         .get_activity_by_provider("Anthropic", None, None)
    ///         .await?;
    ///
    ///     let stats = response.provider_stats("Anthropic");
    ///     println!("Provider {}: {} requests", stats.provider, stats.request_count);
    ///     println!("Total cost: ${:.6}", stats.total_cost);
    ///     println!("Success rate: {:.1}%", stats.success_rate);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_activity_by_provider(
        &self,
        provider: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<ActivityResponse> {
        let mut request = ActivityRequest::new()
            .with_provider(provider)
            .with_sort(crate::types::analytics::SortField::CreatedAt)
            .with_order(crate::types::analytics::SortOrder::Descending);

        if let Some(start) = start_date {
            request = request.with_start_date(start);
        }

        if let Some(end) = end_date {
            request = request.with_end_date(end);
        }

        self.get_activity(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analytics_api_new() {
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use reqwest::Client;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(30),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };

        let client = Client::new();
        let analytics_api = AnalyticsApi::new(client, &config);

        assert!(analytics_api.config.api_key.is_some());
    }

    #[test]
    fn test_activity_request_builder() {
        let request = ActivityRequest::new()
            .with_start_date("2024-01-01")
            .with_end_date("2024-01-31")
            .with_model("test-model")
            .with_provider("test-provider")
            .with_sort(SortField::CreatedAt)
            .with_order(SortOrder::Descending)
            .with_limit(100)
            .with_offset(0);

        assert_eq!(request.start_date, Some("2024-01-01".to_string()));
        assert_eq!(request.end_date, Some("2024-01-31".to_string()));
        assert_eq!(request.model, Some("test-model".to_string()));
        assert_eq!(request.provider, Some("test-provider".to_string()));
        assert_eq!(request.sort, Some(SortField::CreatedAt));
        assert_eq!(request.order, Some(SortOrder::Descending));
        assert_eq!(request.limit, Some(100));
        assert_eq!(request.offset, Some(0));
    }

    #[test]
    fn test_query_parameter_encoding() {
        // Test that special characters in parameters are properly encoded
        let request = ActivityRequest::new()
            .with_model("model with spaces & symbols")
            .with_provider("provider/test");

        assert!(request.validate().is_ok());
        assert_eq!(
            request.model,
            Some("model with spaces & symbols".to_string())
        );
        assert_eq!(request.provider, Some("provider/test".to_string()));
    }
}
