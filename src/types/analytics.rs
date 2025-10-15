use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Activity data for a specific request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityData {
    /// Unique identifier for the request
    pub id: String,
    /// Timestamp when the request was made
    pub created_at: DateTime<Utc>,
    /// Model used for the request
    pub model: String,
    /// Total cost of the request in USD
    pub total_cost: Option<f64>,
    /// Number of tokens in the prompt
    pub tokens_prompt: Option<u32>,
    /// Number of tokens in the completion
    pub tokens_completion: Option<u32>,
    /// Total number of tokens
    pub total_tokens: Option<u32>,
    /// Provider that handled the request
    pub provider: Option<String>,
    /// Whether the request was streamed
    pub streamed: Option<bool>,
    /// Whether the request was cancelled
    pub cancelled: Option<bool>,
    /// Whether web search was used
    pub web_search: Option<bool>,
    /// Whether media was included
    pub media: Option<bool>,
    /// Whether reasoning was used
    pub reasoning: Option<bool>,
    /// Finish reason for the generation
    pub finish_reason: Option<String>,
    /// Native finish reason from the provider
    pub native_finish_reason: Option<String>,
    /// Origin of the request
    pub origin: Option<String>,
    /// Request latency in milliseconds
    pub latency: Option<u64>,
    /// Generation time in milliseconds
    pub generation_time: Option<u64>,
    /// Moderation latency in milliseconds
    pub moderation_latency: Option<u64>,
    /// Cache discount applied
    pub cache_discount: Option<f64>,
    /// Effective cost after discounts
    pub effective_cost: Option<f64>,
    /// Upstream API request ID
    pub upstream_id: Option<String>,
    /// User ID associated with the request
    pub user_id: Option<String>,
    /// HTTP referer
    pub http_referer: Option<String>,
}

impl ActivityData {
    /// Returns the cost per token if both cost and token count are available
    pub fn cost_per_token(&self) -> Option<f64> {
        match (self.total_cost, self.total_tokens) {
            (Some(cost), Some(tokens)) if tokens > 0 => Some(cost / tokens as f64),
            _ => None,
        }
    }

    /// Returns the cost per million tokens if both cost and token count are available
    pub fn cost_per_million_tokens(&self) -> Option<f64> {
        self.cost_per_token().map(|cost| cost * 1_000_000.0)
    }

    /// Returns the latency in seconds if available
    pub fn latency_seconds(&self) -> Option<f64> {
        self.latency.map(|ms| ms as f64 / 1000.0)
    }

    /// Returns the generation time in seconds if available
    pub fn generation_time_seconds(&self) -> Option<f64> {
        self.generation_time.map(|ms| ms as f64 / 1000.0)
    }

    /// Returns true if the request was successful (not cancelled)
    pub fn is_successful(&self) -> bool {
        self.cancelled.unwrap_or(false) == false
    }

    /// Returns true if the request was streamed
    pub fn was_streamed(&self) -> bool {
        self.streamed.unwrap_or(false)
    }

    /// Returns true if the request used web search
    pub fn used_web_search(&self) -> bool {
        self.web_search.unwrap_or(false)
    }

    /// Returns true if the request included media
    pub fn included_media(&self) -> bool {
        self.media.unwrap_or(false)
    }

    /// Returns true if the request used reasoning
    pub fn used_reasoning(&self) -> bool {
        self.reasoning.unwrap_or(false)
    }

    /// Returns the total cost (prioritizing effective cost if available)
    pub fn final_cost(&self) -> Option<f64> {
        self.effective_cost.or(self.total_cost)
    }
}

/// Request parameters for activity data retrieval
#[derive(Debug, Clone, Serialize, Default)]
pub struct ActivityRequest {
    /// Start date for filtering (YYYY-MM-DD format)
    pub start_date: Option<String>,
    /// End date for filtering (YYYY-MM-DD format)
    pub end_date: Option<String>,
    /// Model name for filtering
    pub model: Option<String>,
    /// Provider name for filtering
    pub provider: Option<String>,
    /// Field to sort by
    pub sort: Option<String>,
    /// Sort order (asc or desc)
    pub order: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

impl ActivityRequest {
    /// Creates a new activity request with default parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the start date for filtering
    pub fn with_start_date(mut self, date: impl Into<String>) -> Self {
        self.start_date = Some(date.into());
        self
    }

    /// Sets the end date for filtering
    pub fn with_end_date(mut self, date: impl Into<String>) -> Self {
        self.end_date = Some(date.into());
        self
    }

    /// Sets the model filter
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the provider filter
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Sets the sort field
    pub fn with_sort(mut self, sort: impl Into<String>) -> Self {
        self.sort = Some(sort.into());
        self
    }

    /// Sets the sort order
    pub fn with_order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Sets the limit
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Validates the request parameters
    pub fn validate(&self) -> Result<(), String> {
        // Validate date format if provided
        if let Some(start_date) = &self.start_date {
            if !is_valid_date_format(start_date) {
                return Err("Invalid start_date format. Use YYYY-MM-DD".to_string());
            }
        }

        if let Some(end_date) = &self.end_date {
            if !is_valid_date_format(end_date) {
                return Err("Invalid end_date format. Use YYYY-MM-DD".to_string());
            }
        }

        // Validate date range if both are provided
        if let (Some(start_date), Some(end_date)) = (&self.start_date, &self.end_date) {
            if start_date > end_date {
                return Err("start_date cannot be after end_date".to_string());
            }
        }

        // Validate sort order
        if let Some(order) = &self.order {
            if order != "asc" && order != "desc" {
                return Err("order must be 'asc' or 'desc'".to_string());
            }
        }

        Ok(())
    }
}

/// Response from the activity endpoint
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityResponse {
    /// List of activity data entries
    pub data: Vec<ActivityData>,
    /// Total number of entries matching the query
    pub total_count: Option<u32>,
    /// Whether there are more results available
    pub has_more: Option<bool>,
}

impl ActivityResponse {
    /// Returns the total cost across all activity entries
    pub fn total_cost(&self) -> f64 {
        self.data.iter().filter_map(|d| d.final_cost()).sum()
    }

    /// Returns the total tokens across all activity entries
    pub fn total_tokens(&self) -> u32 {
        self.data.iter().filter_map(|d| d.total_tokens).sum()
    }

    /// Returns the total prompt tokens across all activity entries
    pub fn total_prompt_tokens(&self) -> u32 {
        self.data.iter().filter_map(|d| d.tokens_prompt).sum()
    }

    /// Returns the total completion tokens across all activity entries
    pub fn total_completion_tokens(&self) -> u32 {
        self.data.iter().filter_map(|d| d.tokens_completion).sum()
    }

    /// Returns the average cost per request
    pub fn average_cost_per_request(&self) -> Option<f64> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.total_cost() / self.data.len() as f64)
        }
    }

    /// Returns the average cost per million tokens
    pub fn average_cost_per_million_tokens(&self) -> Option<f64> {
        let total_tokens = self.total_tokens();
        if total_tokens == 0 {
            None
        } else {
            Some(self.total_cost() * 1_000_000.0 / total_tokens as f64)
        }
    }

    /// Returns the average latency in seconds
    pub fn average_latency_seconds(&self) -> Option<f64> {
        let latencies: Vec<f64> = self.data
            .iter()
            .filter_map(|d| d.latency_seconds())
            .collect();
        
        if latencies.is_empty() {
            None
        } else {
            Some(latencies.iter().sum::<f64>() / latencies.len() as f64)
        }
    }

    /// Returns the success rate (percentage of non-cancelled requests)
    pub fn success_rate(&self) -> f64 {
        if self.data.is_empty() {
            0.0
        } else {
            let successful = self.data.iter().filter(|d| d.is_successful()).count();
            successful as f64 / self.data.len() as f64 * 100.0
        }
    }

    /// Returns the streaming rate (percentage of streamed requests)
    pub fn streaming_rate(&self) -> f64 {
        if self.data.is_empty() {
            0.0
        } else {
            let streamed = self.data.iter().filter(|d| d.was_streamed()).count();
            streamed as f64 / self.data.len() as f64 * 100.0
        }
    }

    /// Groups activity data by model
    pub fn group_by_model(&self) -> HashMap<String, Vec<&ActivityData>> {
        let mut groups: HashMap<String, Vec<&ActivityData>> = HashMap::new();
        for activity in &self.data {
            groups.entry(activity.model.clone()).or_default().push(activity);
        }
        groups
    }

    /// Groups activity data by provider
    pub fn group_by_provider(&self) -> HashMap<String, Vec<&ActivityData>> {
        let mut groups: HashMap<String, Vec<&ActivityData>> = HashMap::new();
        for activity in &self.data {
            if let Some(provider) = &activity.provider {
                groups.entry(provider.clone()).or_default().push(activity);
            }
        }
        groups
    }

    /// Returns usage statistics for a specific model
    pub fn model_stats(&self, model: &str) -> ModelUsageStats {
        let model_activities: Vec<&ActivityData> = self.data
            .iter()
            .filter(|d| d.model == model)
            .collect();

        ModelUsageStats {
            model: model.to_string(),
            request_count: model_activities.len(),
            total_cost: model_activities.iter().filter_map(|d| d.final_cost()).sum(),
            total_tokens: model_activities.iter().filter_map(|d| d.total_tokens).sum(),
            average_cost_per_request: if model_activities.is_empty() {
                None
            } else {
                Some(
                    model_activities.iter().filter_map(|d| d.final_cost()).sum::<f64>()
                        / model_activities.len() as f64
                )
            },
            success_rate: if model_activities.is_empty() {
                0.0
            } else {
                let successful = model_activities.iter().filter(|d| d.is_successful()).count();
                successful as f64 / model_activities.len() as f64 * 100.0
            },
        }
    }

    /// Returns usage statistics for a specific provider
    pub fn provider_stats(&self, provider: &str) -> ProviderUsageStats {
        let provider_activities: Vec<&ActivityData> = self.data
            .iter()
            .filter(|d| d.provider.as_ref().map_or(false, |p| p == provider))
            .collect();

        ProviderUsageStats {
            provider: provider.to_string(),
            request_count: provider_activities.len(),
            total_cost: provider_activities.iter().filter_map(|d| d.final_cost()).sum(),
            total_tokens: provider_activities.iter().filter_map(|d| d.total_tokens).sum(),
            average_cost_per_request: if provider_activities.is_empty() {
                None
            } else {
                Some(
                    provider_activities.iter().filter_map(|d| d.final_cost()).sum::<f64>()
                        / provider_activities.len() as f64
                )
            },
            success_rate: if provider_activities.is_empty() {
                0.0
            } else {
                let successful = provider_activities.iter().filter(|d| d.is_successful()).count();
                successful as f64 / provider_activities.len() as f64 * 100.0
            },
        }
    }

    /// Returns the percentage of requests that used each feature
    pub fn feature_usage_percentages(&self) -> FeatureUsagePercentages {
        if self.data.is_empty() {
            return FeatureUsagePercentages {
                web_search: 0.0,
                media: 0.0,
                reasoning: 0.0,
                streaming: 0.0,
            };
        }

        let total = self.data.len() as f64;
        FeatureUsagePercentages {
            web_search: self.data.iter().filter(|d| d.used_web_search()).count() as f64 / total * 100.0,
            media: self.data.iter().filter(|d| d.included_media()).count() as f64 / total * 100.0,
            reasoning: self.data.iter().filter(|d| d.used_reasoning()).count() as f64 / total * 100.0,
            streaming: self.data.iter().filter(|d| d.was_streamed()).count() as f64 / total * 100.0,
        }
    }
}

/// Usage statistics for a specific model
#[derive(Debug, Clone)]
pub struct ModelUsageStats {
    pub model: String,
    pub request_count: usize,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub average_cost_per_request: Option<f64>,
    pub success_rate: f64,
}

/// Usage statistics for a specific provider
#[derive(Debug, Clone)]
pub struct ProviderUsageStats {
    pub provider: String,
    pub request_count: usize,
    pub total_cost: f64,
    pub total_tokens: u32,
    pub average_cost_per_request: Option<f64>,
    pub success_rate: f64,
}

/// Feature usage percentages
#[derive(Debug, Clone)]
pub struct FeatureUsagePercentages {
    pub web_search: f64,
    pub media: f64,
    pub reasoning: f64,
    pub streaming: f64,
}

/// Validates date format (YYYY-MM-DD)
fn is_valid_date_format(date: &str) -> bool {
    if date.len() != 10 {
        return false;
    }

    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    // Check year (4 digits)
    if parts[0].len() != 4 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Check month (2 digits, 01-12)
    if parts[1].len() != 2 || !parts[1].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Ok(month) = parts[1].parse::<u32>() {
        if month < 1 || month > 12 {
            return false;
        }
    } else {
        return false;
    }

    // Check day (2 digits, 01-31)
    if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    if let Ok(day) = parts[2].parse::<u32>() {
        if day < 1 || day > 31 {
            return false;
        }
    } else {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_request_validation() {
        // Valid request
        let request = ActivityRequest::new()
            .with_start_date("2024-01-01")
            .with_end_date("2024-01-31")
            .with_order("asc");
        assert!(request.validate().is_ok());

        // Invalid date format
        let request = ActivityRequest::new().with_start_date("2024/01/01");
        assert!(request.validate().is_err());

        // Start date after end date
        let request = ActivityRequest::new()
            .with_start_date("2024-02-01")
            .with_end_date("2024-01-31");
        assert!(request.validate().is_err());

        // Invalid order
        let request = ActivityRequest::new().with_order("invalid");
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_date_format_validation() {
        assert!(is_valid_date_format("2024-01-01"));
        assert!(is_valid_date_format("2024-12-31"));
        assert!(!is_valid_date_format("2024-1-1"));
        assert!(!is_valid_date_format("2024/01/01"));
        assert!(!is_valid_date_format("24-01-01"));
        assert!(!is_valid_date_format("2024-13-01"));
        assert!(!is_valid_date_format("2024-01-32"));
        assert!(!is_valid_date_format("invalid"));
    }

    #[test]
    fn test_activity_data_convenience_methods() {
        let activity = ActivityData {
            id: "test-123".to_string(),
            created_at: Utc::now(),
            model: "test-model".to_string(),
            total_cost: Some(0.001),
            tokens_prompt: Some(10),
            tokens_completion: Some(20),
            total_tokens: Some(30),
            provider: Some("test-provider".to_string()),
            streamed: Some(true),
            cancelled: Some(false),
            web_search: Some(true),
            media: Some(false),
            reasoning: Some(false),
            finish_reason: Some("stop".to_string()),
            native_finish_reason: None,
            origin: None,
            latency: Some(1000),
            generation_time: Some(500),
            moderation_latency: None,
            cache_discount: None,
            effective_cost: Some(0.0009),
            upstream_id: None,
            user_id: None,
            http_referer: None,
        };

        // Test that cost calculations return reasonable values
        assert!(activity.cost_per_token().is_some());
        assert!(activity.cost_per_million_tokens().is_some());
        assert!(activity.cost_per_token().unwrap() > 0.0);
        assert!(activity.cost_per_million_tokens().unwrap() > 0.0);
        assert_eq!(activity.latency_seconds(), Some(1.0));
        assert_eq!(activity.generation_time_seconds(), Some(0.5));
        assert!(activity.is_successful());
        assert!(activity.was_streamed());
        assert!(activity.used_web_search());
        assert!(!activity.included_media());
        assert!(!activity.used_reasoning());
        assert_eq!(activity.final_cost(), Some(0.0009));
    }

    #[test]
    fn test_activity_response_aggregations() {
        let activities = vec![
            ActivityData {
                id: "test-1".to_string(),
                created_at: Utc::now(),
                model: "model-a".to_string(),
                total_cost: Some(0.001),
                total_tokens: Some(100),
                cancelled: Some(false),
                streamed: Some(true),
                web_search: Some(true),
                media: Some(false),
                reasoning: Some(false),
                provider: Some("provider-x".to_string()),
                latency: Some(1000),
                ..Default::default()
            },
            ActivityData {
                id: "test-2".to_string(),
                created_at: Utc::now(),
                model: "model-b".to_string(),
                total_cost: Some(0.002),
                total_tokens: Some(200),
                cancelled: Some(true),
                streamed: Some(false),
                web_search: Some(false),
                media: Some(true),
                reasoning: Some(true),
                provider: Some("provider-y".to_string()),
                latency: Some(2000),
                ..Default::default()
            },
        ];

        let response = ActivityResponse {
            data: activities,
            total_count: Some(2),
            has_more: Some(false),
        };

        assert_eq!(response.total_cost(), 0.003);
        assert_eq!(response.total_tokens(), 300);
        assert_eq!(response.average_cost_per_request(), Some(0.0015));
        assert_eq!(response.success_rate(), 50.0);
        assert_eq!(response.streaming_rate(), 50.0);
        assert_eq!(response.average_latency_seconds(), Some(1.5));

        let feature_usage = response.feature_usage_percentages();
        assert_eq!(feature_usage.web_search, 50.0);
        assert_eq!(feature_usage.media, 50.0);
        assert_eq!(feature_usage.reasoning, 50.0);
        assert_eq!(feature_usage.streaming, 50.0);
    }
}

impl Default for ActivityData {
    fn default() -> Self {
        Self {
            id: String::new(),
            created_at: Utc::now(),
            model: String::new(),
            total_cost: None,
            tokens_prompt: None,
            tokens_completion: None,
            total_tokens: None,
            provider: None,
            streamed: None,
            cancelled: None,
            web_search: None,
            media: None,
            reasoning: None,
            finish_reason: None,
            native_finish_reason: None,
            origin: None,
            latency: None,
            generation_time: None,
            moderation_latency: None,
            cache_discount: None,
            effective_cost: None,
            upstream_id: None,
            user_id: None,
            http_referer: None,
        }
    }
}