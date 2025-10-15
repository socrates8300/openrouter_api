//! Analytics API types for OpenRouter activity data

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Activity data for a specific model and date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityData {
    /// UTC date in YYYY-MM-DD format
    pub date: String,
    /// Model name
    pub model: String,
    /// Model permanent slug
    pub model_permaslug: String,
    /// Endpoint ID
    pub endpoint_id: String,
    /// Provider name
    pub provider_name: String,
    /// Usage cost in USD
    pub usage: f64,
    /// BYOK usage for inference in USD
    pub byok_usage_inference: f64,
    /// Number of requests
    pub requests: f64,
    /// Number of prompt tokens
    pub prompt_tokens: f64,
    /// Number of completion tokens
    pub completion_tokens: f64,
    /// Number of reasoning tokens
    pub reasoning_tokens: f64,
}

impl ActivityData {
    /// Parse the date string into a NaiveDate
    pub fn parsed_date(&self) -> Result<NaiveDate, Box<dyn std::error::Error>> {
        Ok(NaiveDate::parse_from_str(&self.date, "%Y-%m-%d")?)
    }

    /// Get total tokens (prompt + completion + reasoning)
    pub fn total_tokens(&self) -> f64 {
        self.prompt_tokens + self.completion_tokens + self.reasoning_tokens
    }

    /// Get cost per request
    pub fn cost_per_request(&self) -> f64 {
        if self.requests > 0.0 {
            self.usage / self.requests
        } else {
            0.0
        }
    }

    /// Get cost per million tokens
    pub fn cost_per_million_tokens(&self) -> f64 {
        let total_tokens = self.total_tokens();
        if total_tokens > 0.0 {
            (self.usage / total_tokens) * 1_000_000.0
        } else {
            0.0
        }
    }

    /// Check if this activity includes reasoning tokens
    pub fn has_reasoning(&self) -> bool {
        self.reasoning_tokens > 0.0
    }

    /// Check if this activity uses BYOK (Bring Your Own Key)
    pub fn uses_byok(&self) -> bool {
        self.byok_usage_inference > 0.0
    }

    /// Get the percentage of usage that is BYOK
    pub fn byok_percentage(&self) -> f64 {
        if self.usage > 0.0 {
            (self.byok_usage_inference / self.usage) * 100.0
        } else {
            0.0
        }
    }
}

/// Request parameters for getting activity data
#[derive(Debug, Clone, Serialize, Default)]
pub struct ActivityRequest {
    /// Filter by a single UTC date in the last 30 days (YYYY-MM-DD format)
    pub date: Option<String>,
}

impl ActivityRequest {
    /// Create a new activity request
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the date filter
    pub fn date(mut self, date: impl Into<String>) -> Self {
        self.date = Some(date.into());
        self
    }

    /// Set the date filter from a NaiveDate
    pub fn date_from_naive_date(mut self, date: NaiveDate) -> Self {
        self.date = Some(date.format("%Y-%m-%d").to_string());
        self
    }

    /// Set the date filter from a DateTime<Utc>
    pub fn date_from_datetime(mut self, date: DateTime<Utc>) -> Self {
        self.date = Some(date.format("%Y-%m-%d").to_string());
        self
    }
}

/// Response from the activity endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityResponse {
    /// Array of activity data
    pub data: Vec<ActivityData>,
}

impl ActivityResponse {
    /// Get activity data for a specific date
    pub fn for_date(&self, date: &str) -> Vec<&ActivityData> {
        self.data.iter().filter(|item| item.date == date).collect()
    }

    /// Get activity data for a specific model
    pub fn for_model(&self, model: &str) -> Vec<&ActivityData> {
        self.data
            .iter()
            .filter(|item| item.model == model)
            .collect()
    }

    /// Get activity data for a specific provider
    pub fn for_provider(&self, provider: &str) -> Vec<&ActivityData> {
        self.data
            .iter()
            .filter(|item| item.provider_name == provider)
            .collect()
    }

    /// Get all unique dates in the response
    pub fn unique_dates(&self) -> Vec<String> {
        let mut dates: Vec<String> = self.data.iter().map(|item| item.date.clone()).collect();
        dates.sort();
        dates.dedup();
        dates
    }

    /// Get all unique models in the response
    pub fn unique_models(&self) -> Vec<String> {
        let mut models: Vec<String> = self.data.iter().map(|item| item.model.clone()).collect();
        models.sort();
        models.dedup();
        models
    }

    /// Get all unique providers in the response
    pub fn unique_providers(&self) -> Vec<String> {
        let mut providers: Vec<String> = self
            .data
            .iter()
            .map(|item| item.provider_name.clone())
            .collect();
        providers.sort();
        providers.dedup();
        providers
    }

    /// Calculate total usage across all activity
    pub fn total_usage(&self) -> f64 {
        self.data.iter().map(|item| item.usage).sum()
    }

    /// Calculate total requests across all activity
    pub fn total_requests(&self) -> f64 {
        self.data.iter().map(|item| item.requests).sum()
    }

    /// Calculate total prompt tokens across all activity
    pub fn total_prompt_tokens(&self) -> f64 {
        self.data.iter().map(|item| item.prompt_tokens).sum()
    }

    /// Calculate total completion tokens across all activity
    pub fn total_completion_tokens(&self) -> f64 {
        self.data.iter().map(|item| item.completion_tokens).sum()
    }

    /// Calculate total reasoning tokens across all activity
    pub fn total_reasoning_tokens(&self) -> f64 {
        self.data.iter().map(|item| item.reasoning_tokens).sum()
    }

    /// Calculate total tokens across all activity
    pub fn total_tokens(&self) -> f64 {
        self.data.iter().map(|item| item.total_tokens()).sum()
    }

    /// Get activity sorted by date (newest first)
    pub fn sorted_by_date_desc(&self) -> Vec<&ActivityData> {
        let mut sorted: Vec<&ActivityData> = self.data.iter().collect();
        sorted.sort_by(|a, b| b.date.cmp(&a.date));
        sorted
    }

    /// Get activity sorted by usage (highest first)
    pub fn sorted_by_usage_desc(&self) -> Vec<&ActivityData> {
        let mut sorted: Vec<&ActivityData> = self.data.iter().collect();
        sorted.sort_by(|a, b| {
            b.usage
                .partial_cmp(&a.usage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }

    /// Get activity sorted by requests (highest first)
    pub fn sorted_by_requests_desc(&self) -> Vec<&ActivityData> {
        let mut sorted: Vec<&ActivityData> = self.data.iter().collect();
        sorted.sort_by(|a, b| {
            b.requests
                .partial_cmp(&a.requests)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_activity_data_convenience_methods() {
        let data = ActivityData {
            date: "2024-01-01".to_string(),
            model: "test-model".to_string(),
            model_permaslug: "test-model".to_string(),
            endpoint_id: "test-endpoint".to_string(),
            provider_name: "test-provider".to_string(),
            usage: 1.23,
            byok_usage_inference: 0.23,
            requests: 10.0,
            prompt_tokens: 1000.0,
            completion_tokens: 500.0,
            reasoning_tokens: 100.0,
        };

        assert_eq!(data.total_tokens(), 1600.0);
        assert_eq!(data.cost_per_request(), 0.123);
        assert!((data.cost_per_million_tokens() - 768.75).abs() < 0.01);
        assert!(data.has_reasoning());
        assert!(data.uses_byok());
        assert!((data.byok_percentage() - 18.7).abs() < 0.1);
    }

    #[test]
    fn test_activity_data_date_parsing() {
        let data = ActivityData {
            date: "2024-01-01".to_string(),
            model: "test-model".to_string(),
            model_permaslug: "test-model".to_string(),
            endpoint_id: "test-endpoint".to_string(),
            provider_name: "test-provider".to_string(),
            usage: 1.23,
            byok_usage_inference: 0.0,
            requests: 10.0,
            prompt_tokens: 1000.0,
            completion_tokens: 500.0,
            reasoning_tokens: 0.0,
        };

        let parsed = data.parsed_date().unwrap();
        assert_eq!(parsed, NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    }

    #[test]
    fn test_activity_request() {
        let request = ActivityRequest::new().date("2024-01-01");
        assert_eq!(request.date, Some("2024-01-01".to_string()));

        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let request = ActivityRequest::new().date_from_naive_date(date);
        assert_eq!(request.date, Some("2024-01-01".to_string()));
    }

    #[test]
    fn test_activity_response_filtering() {
        let data = vec![
            ActivityData {
                date: "2024-01-01".to_string(),
                model: "model-a".to_string(),
                model_permaslug: "model-a".to_string(),
                endpoint_id: "endpoint-1".to_string(),
                provider_name: "provider-x".to_string(),
                usage: 1.0,
                byok_usage_inference: 0.0,
                requests: 10.0,
                prompt_tokens: 1000.0,
                completion_tokens: 500.0,
                reasoning_tokens: 0.0,
            },
            ActivityData {
                date: "2024-01-01".to_string(),
                model: "model-b".to_string(),
                model_permaslug: "model-b".to_string(),
                endpoint_id: "endpoint-2".to_string(),
                provider_name: "provider-y".to_string(),
                usage: 2.0,
                byok_usage_inference: 0.0,
                requests: 20.0,
                prompt_tokens: 2000.0,
                completion_tokens: 1000.0,
                reasoning_tokens: 0.0,
            },
            ActivityData {
                date: "2024-01-02".to_string(),
                model: "model-a".to_string(),
                model_permaslug: "model-a".to_string(),
                endpoint_id: "endpoint-1".to_string(),
                provider_name: "provider-x".to_string(),
                usage: 1.5,
                byok_usage_inference: 0.0,
                requests: 15.0,
                prompt_tokens: 1500.0,
                completion_tokens: 750.0,
                reasoning_tokens: 0.0,
            },
        ];

        let response = ActivityResponse { data };

        // Test filtering
        assert_eq!(response.for_date("2024-01-01").len(), 2);
        assert_eq!(response.for_date("2024-01-02").len(), 1);
        assert_eq!(response.for_model("model-a").len(), 2);
        assert_eq!(response.for_model("model-b").len(), 1);
        assert_eq!(response.for_provider("provider-x").len(), 2);
        assert_eq!(response.for_provider("provider-y").len(), 1);

        // Test aggregations
        assert_eq!(response.total_usage(), 4.5);
        assert_eq!(response.total_requests(), 45.0);
        assert_eq!(response.total_tokens(), 6750.0);

        // Test unique values
        let dates = response.unique_dates();
        assert_eq!(dates.len(), 2);
        assert!(dates.contains(&"2024-01-01".to_string()));
        assert!(dates.contains(&"2024-01-02".to_string()));

        let models = response.unique_models();
        assert_eq!(models.len(), 2);
        assert!(models.contains(&"model-a".to_string()));
        assert!(models.contains(&"model-b".to_string()));
    }

    #[test]
    fn test_activity_response_sorting() {
        let data = vec![
            ActivityData {
                date: "2024-01-01".to_string(),
                model: "model-a".to_string(),
                model_permaslug: "model-a".to_string(),
                endpoint_id: "endpoint-1".to_string(),
                provider_name: "provider-x".to_string(),
                usage: 1.0,
                byok_usage_inference: 0.0,
                requests: 10.0,
                prompt_tokens: 1000.0,
                completion_tokens: 500.0,
                reasoning_tokens: 0.0,
            },
            ActivityData {
                date: "2024-01-02".to_string(),
                model: "model-b".to_string(),
                model_permaslug: "model-b".to_string(),
                endpoint_id: "endpoint-2".to_string(),
                provider_name: "provider-y".to_string(),
                usage: 2.0,
                byok_usage_inference: 0.0,
                requests: 20.0,
                prompt_tokens: 2000.0,
                completion_tokens: 1000.0,
                reasoning_tokens: 0.0,
            },
        ];

        let response = ActivityResponse { data };

        // Test sorting by date
        let sorted_by_date = response.sorted_by_date_desc();
        assert_eq!(sorted_by_date[0].date, "2024-01-02");
        assert_eq!(sorted_by_date[1].date, "2024-01-01");

        // Test sorting by usage
        let sorted_by_usage = response.sorted_by_usage_desc();
        assert_eq!(sorted_by_usage[0].usage, 2.0);
        assert_eq!(sorted_by_usage[1].usage, 1.0);

        // Test sorting by requests
        let sorted_by_requests = response.sorted_by_requests_desc();
        assert_eq!(sorted_by_requests[0].requests, 20.0);
        assert_eq!(sorted_by_requests[1].requests, 10.0);
    }
}
