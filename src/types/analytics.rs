use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::ids::ActivityId;
use crate::types::status::{CancellationStatus, StreamingStatus};

/// Constants for analytics validation and defaults
pub mod constants {
    /// Standard date format length (YYYY-MM-DD)
    pub const DATE_FORMAT_LENGTH: usize = 10;

    /// Default limit for activity queries
    pub const DEFAULT_LIMIT: u32 = 100;

    /// Maximum limit for activity queries
    pub const MAX_LIMIT: u32 = 1000;

    /// Default recent activity days
    pub const DEFAULT_RECENT_DAYS: i64 = 30;

    /// Milliseconds per second for time conversions
    pub const MS_PER_SECOND: f64 = 1000.0;

    /// Tokens per million for cost calculations
    pub const TOKENS_PER_MILLION: f64 = 1_000_000.0;
}

/// Sort order for activity queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl SortOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "asc",
            SortOrder::Descending => "desc",
        }
    }
}

/// Sort field for activity queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortField {
    CreatedAt,
    Cost,
    Latency,
    Tokens,
}

impl SortField {
    pub fn as_str(&self) -> &'static str {
        match self {
            SortField::CreatedAt => "created_at",
            SortField::Cost => "total_cost",
            SortField::Latency => "latency",
            SortField::Tokens => "total_tokens",
        }
    }
}

/// Activity data for a specific request
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActivityData {
    /// Unique identifier for the request
    pub id: ActivityId,
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
    pub streamed: StreamingStatus,
    /// Whether the request was cancelled
    pub cancelled: CancellationStatus,
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
            (Some(cost), Some(tokens)) if tokens > 0 => {
                // Check for valid cost value
                if cost.is_finite() && cost >= 0.0 {
                    Some(cost / tokens as f64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns the cost per million tokens if both cost and token count are available
    pub fn cost_per_million_tokens(&self) -> Option<f64> {
        self.cost_per_token()
            .map(|cost| cost * constants::TOKENS_PER_MILLION)
    }

    /// Returns the latency in seconds if available
    pub fn latency_seconds(&self) -> Option<f64> {
        self.latency.map(|ms| ms as f64 / constants::MS_PER_SECOND)
    }

    /// Returns the generation time in seconds if available
    pub fn generation_time_seconds(&self) -> Option<f64> {
        self.generation_time
            .map(|ms| ms as f64 / constants::MS_PER_SECOND)
    }

    /// Returns true if request was successful (not cancelled)
    pub fn is_successful(&self) -> bool {
        !self.cancelled.is_cancelled()
    }

    /// Returns true if request was streamed
    pub fn was_streamed(&self) -> bool {
        self.streamed.is_active()
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
    pub sort: Option<SortField>,
    /// Sort order
    pub order: Option<SortOrder>,
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
    pub fn with_sort(mut self, sort: SortField) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Sets the sort order
    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
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

        // Sort order is now type-safe, no validation needed

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
            let total_cost = self.total_cost();
            if total_cost.is_finite() && total_cost >= 0.0 {
                Some(total_cost * constants::TOKENS_PER_MILLION / total_tokens as f64)
            } else {
                None
            }
        }
    }

    /// Returns the average latency in seconds
    pub fn average_latency_seconds(&self) -> Option<f64> {
        let latencies: Vec<f64> = self
            .data
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
            groups
                .entry(activity.model.clone())
                .or_default()
                .push(activity);
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
        let model_activities: Vec<&ActivityData> =
            self.data.iter().filter(|d| d.model == model).collect();

        ModelUsageStats {
            model: model.to_string(),
            request_count: model_activities.len(),
            total_cost: model_activities.iter().filter_map(|d| d.final_cost()).sum(),
            total_tokens: model_activities.iter().filter_map(|d| d.total_tokens).sum(),
            average_cost_per_request: if model_activities.is_empty() {
                None
            } else {
                Some(
                    model_activities
                        .iter()
                        .filter_map(|d| d.final_cost())
                        .sum::<f64>()
                        / model_activities.len() as f64,
                )
            },
            success_rate: if model_activities.is_empty() {
                0.0
            } else {
                let successful = model_activities
                    .iter()
                    .filter(|d| d.is_successful())
                    .count();
                successful as f64 / model_activities.len() as f64 * 100.0
            },
        }
    }

    /// Returns usage statistics for a specific provider
    pub fn provider_stats(&self, provider: &str) -> ProviderUsageStats {
        let provider_activities: Vec<&ActivityData> = self
            .data
            .iter()
            .filter(|d| d.provider.as_ref().is_some_and(|p| p == provider))
            .collect();

        ProviderUsageStats {
            provider: provider.to_string(),
            request_count: provider_activities.len(),
            total_cost: provider_activities
                .iter()
                .filter_map(|d| d.final_cost())
                .sum(),
            total_tokens: provider_activities
                .iter()
                .filter_map(|d| d.total_tokens)
                .sum(),
            average_cost_per_request: if provider_activities.is_empty() {
                None
            } else {
                Some(
                    provider_activities
                        .iter()
                        .filter_map(|d| d.final_cost())
                        .sum::<f64>()
                        / provider_activities.len() as f64,
                )
            },
            success_rate: if provider_activities.is_empty() {
                0.0
            } else {
                let successful = provider_activities
                    .iter()
                    .filter(|d| d.is_successful())
                    .count();
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
            web_search: self.data.iter().filter(|d| d.used_web_search()).count() as f64 / total
                * 100.0,
            media: self.data.iter().filter(|d| d.included_media()).count() as f64 / total * 100.0,
            reasoning: self.data.iter().filter(|d| d.used_reasoning()).count() as f64 / total
                * 100.0,
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

/// Validates date format (YYYY-MM-DD) with proper calendar validation
fn is_valid_date_format(date: &str) -> bool {
    if date.len() != constants::DATE_FORMAT_LENGTH {
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
    let month = if let Ok(m) = parts[1].parse::<u32>() {
        if !(1..=12).contains(&m) {
            return false;
        }
        m
    } else {
        return false;
    };

    // Check day (2 digits, 01-31)
    if parts[2].len() != 2 || !parts[2].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let day = if let Ok(d) = parts[2].parse::<u32>() {
        if !(1..=31).contains(&d) {
            return false;
        }
        d
    } else {
        return false;
    };

    // Validate day against month (including leap years)
    let year = parts[0].parse::<u32>().ok();
    is_valid_day_for_month(day, month, year)
}

/// Validates that a day is valid for a given month and year
fn is_valid_day_for_month(day: u32, month: u32, year: Option<u32>) -> bool {
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            // February: check for leap year
            if let Some(y) = year {
                if is_leap_year(y) {
                    29
                } else {
                    28
                }
            } else {
                28 // Default to non-leap year if year not provided
            }
        }
        _ => return false,
    };

    day <= max_day
}

/// Checks if a year is a leap year
#[allow(clippy::manual_is_multiple_of)]
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

impl Default for ActivityData {
    fn default() -> Self {
        Self {
            id: ActivityId::new(""),
            created_at: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            model: String::new(),
            total_cost: None,
            tokens_prompt: None,
            tokens_completion: None,
            total_tokens: None,
            provider: None,
            streamed: StreamingStatus::default(),
            cancelled: CancellationStatus::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_request_validation() {
        // Valid request
        let request = ActivityRequest::new()
            .with_start_date("2024-01-01")
            .with_end_date("2024-01-31")
            .with_order(SortOrder::Ascending);
        assert!(request.validate().is_ok());

        // Invalid date format
        let request = ActivityRequest::new().with_start_date("2024/01/01");
        assert!(request.validate().is_err());

        // Start date after end date
        let request = ActivityRequest::new()
            .with_start_date("2024-02-01")
            .with_end_date("2024-01-31");
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
            id: ActivityId::new("test-123"),
            created_at: Utc::now(),
            model: "test-model".to_string(),
            total_cost: Some(0.001),
            tokens_prompt: Some(10),
            tokens_completion: Some(20),
            total_tokens: Some(30),
            provider: Some("test-provider".to_string()),
            streamed: StreamingStatus::Complete,
            cancelled: CancellationStatus::NotCancelled,
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
                id: ActivityId::new("test-1"),
                created_at: Utc::now(),
                model: "model-a".to_string(),
                total_cost: Some(0.001),
                total_tokens: Some(100),
                cancelled: CancellationStatus::NotCancelled,
                streamed: StreamingStatus::Complete,
                web_search: Some(true),
                media: Some(false),
                reasoning: Some(false),
                provider: Some("provider-x".to_string()),
                latency: Some(1000),
                ..Default::default()
            },
            ActivityData {
                id: ActivityId::new("test-2"),
                created_at: Utc::now(),
                model: "model-b".to_string(),
                total_cost: Some(0.002),
                total_tokens: Some(200),
                cancelled: CancellationStatus::Completed,
                streamed: StreamingStatus::NotStarted,
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

    #[test]
    fn test_activity_response_edge_cases() {
        // Test empty response
        let empty_response = ActivityResponse {
            data: vec![],
            total_count: Some(0),
            has_more: Some(false),
        };

        assert_eq!(empty_response.total_cost(), 0.0);
        assert_eq!(empty_response.total_tokens(), 0);
        assert_eq!(empty_response.average_cost_per_request(), None);
        assert_eq!(empty_response.success_rate(), 0.0);
        assert_eq!(empty_response.streaming_rate(), 0.0);
        assert_eq!(empty_response.average_latency_seconds(), None);

        let feature_usage = empty_response.feature_usage_percentages();
        assert_eq!(feature_usage.web_search, 0.0);
        assert_eq!(feature_usage.media, 0.0);
        assert_eq!(feature_usage.reasoning, 0.0);
        assert_eq!(feature_usage.streaming, 0.0);

        // Test response with missing optional fields
        let partial_activities = vec![ActivityData {
        id: ActivityId::new("partial-1"),
        created_at: Utc::now(),
        model: "model-partial".to_string(),
        total_cost: None,   // Missing cost
        total_tokens: None, // Missing tokens
        cancelled: CancellationStatus::default(),    // Missing cancelled status
        streamed: StreamingStatus::default(),     // Missing streamed status
            web_search: None,   // Missing web search status
            media: None,        // Missing media status
            reasoning: None,    // Missing reasoning status
            provider: None,     // Missing provider
            latency: None,      // Missing latency
            ..Default::default()
        }];

        let partial_response = ActivityResponse {
            data: partial_activities,
            total_count: Some(1),
            has_more: Some(false),
        };

        // Should handle missing fields gracefully
        assert_eq!(partial_response.total_cost(), 0.0);
        assert_eq!(partial_response.total_tokens(), 0);
        assert_eq!(partial_response.success_rate(), 100.0); // Default to successful
        assert_eq!(partial_response.streaming_rate(), 0.0); // Default to not streamed
    }

    #[test]
    fn test_activity_data_edge_cases() {
        // Test with zero values
        let zero_activity = ActivityData {
            id: ActivityId::new("zero-test"),
            created_at: Utc::now(),
            model: "test-model".to_string(),
            total_cost: Some(0.0),
            total_tokens: Some(0),
            latency: Some(0),
            generation_time: Some(0),
            ..Default::default()
        };

        assert_eq!(zero_activity.cost_per_token(), None);
        assert_eq!(zero_activity.cost_per_million_tokens(), None);
        assert_eq!(zero_activity.latency_seconds(), Some(0.0));
        assert_eq!(zero_activity.generation_time_seconds(), Some(0.0));
        assert!(zero_activity.is_successful());

        // Test with negative values (should be handled gracefully)
        let negative_activity = ActivityData {
            id: ActivityId::new("negative-test"),
            created_at: Utc::now(),
            model: "test-model".to_string(),
            total_cost: Some(-0.001), // Negative cost
            total_tokens: Some(100),
            ..Default::default()
        };

        // Should return None for negative costs (invalid data)
        assert_eq!(negative_activity.cost_per_token(), None);
    }

    #[test]
    fn test_leap_year_validation() {
        // Test leap year dates
        assert!(is_valid_date_format("2024-02-29")); // Valid leap year
        assert!(!is_valid_date_format("2023-02-29")); // Invalid non-leap year
        assert!(is_valid_date_format("2000-02-29")); // Valid leap year (divisible by 400)
        assert!(!is_valid_date_format("1900-02-29")); // Invalid leap year (divisible by 100 but not 400)
    }

    #[test]
    fn test_model_and_provider_stats_edge_cases() {
        let response = ActivityResponse {
            data: vec![],
            total_count: Some(0),
            has_more: Some(false),
        };

        // Test stats for non-existent model/provider
        let model_stats = response.model_stats("non-existent-model");
        assert_eq!(model_stats.request_count, 0);
        assert_eq!(model_stats.total_cost, 0.0);
        assert_eq!(model_stats.total_tokens, 0);
        assert_eq!(model_stats.average_cost_per_request, None);
        assert_eq!(model_stats.success_rate, 0.0);

        let provider_stats = response.provider_stats("non-existent-provider");
        assert_eq!(provider_stats.request_count, 0);
        assert_eq!(provider_stats.total_cost, 0.0);
        assert_eq!(provider_stats.total_tokens, 0);
        assert_eq!(provider_stats.average_cost_per_request, None);
        assert_eq!(provider_stats.success_rate, 0.0);
    }

    #[test]
    fn test_activity_id_serialization() {
        let activity = ActivityData {
            id: ActivityId::new("activity-12345"),
            created_at: Utc::now(),
            model: "test-model".to_string(),
            ..Default::default()
        };

        // Test that ActivityId serializes as a plain string
        let json = serde_json::to_string(&activity).unwrap();
        assert!(json.contains("\"activity-12345\""));

        // Test deserialization roundtrip
        let deserialized: ActivityData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id.as_str(), "activity-12345");
        assert_eq!(deserialized.streamed, StreamingStatus::NotStarted);
        assert_eq!(deserialized.cancelled, CancellationStatus::NotCancelled);
    }

    #[test]
    fn test_activity_id_from_string() {
        let id: ActivityId = "string-id".into();
        assert_eq!(id.as_str(), "string-id");

        let id2: ActivityId = String::from("string-id-2").into();
        assert_eq!(id2.as_str(), "string-id-2");
    }

    #[test]
    fn test_activity_id_display() {
        let id = ActivityId::new("test-display");
        assert_eq!(format!("{}", id), "test-display");
    }

    #[test]
    fn test_activity_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ActivityId::new("id-1"));
        set.insert(ActivityId::new("id-2"));
        set.insert(ActivityId::new("id-1")); // Duplicate

        assert_eq!(set.len(), 2); // Should only have 2 unique IDs
    }
}
