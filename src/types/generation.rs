//! Types for OpenRouter Generation API responses.

use serde::{Deserialize, Serialize};

use crate::types::ids::GenerationId;
use crate::types::status::{CancellationStatus, StreamingStatus};

/// Generation data returned by the OpenRouter API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationData {
    /// Unique identifier for the generation
    pub id: GenerationId,
    /// Upstream API identifier (if available)
    pub upstream_id: Option<String>,
    /// Total cost of the generation in credits
    pub total_cost: f64,
    /// Cache discount applied (if any)
    pub cache_discount: Option<f64>,
    /// Upstream inference cost (if available)
    pub upstream_inference_cost: Option<f64>,
    /// Timestamp when the generation was created
    pub created_at: String,
    /// Model used for the generation
    pub model: String,
    /// Application ID (if applicable)
    pub app_id: Option<i64>,
    /// Whether the generation was streamed
    pub streamed: StreamingStatus,
    /// Whether the generation was cancelled
    pub cancelled: CancellationStatus,
    /// Name of the provider that handled the generation
    pub provider_name: Option<String>,
    /// Latency in milliseconds (if available)
    pub latency: Option<i64>,
    /// Moderation latency in milliseconds (if available)
    pub moderation_latency: Option<i64>,
    /// Generation time in milliseconds (if available)
    pub generation_time: Option<i64>,
    /// Finish reason for the generation
    pub finish_reason: Option<String>,
    /// Native finish reason from the provider
    pub native_finish_reason: Option<String>,
    /// Number of prompt tokens used
    pub tokens_prompt: Option<i64>,
    /// Number of completion tokens used
    pub tokens_completion: Option<i64>,
    /// Number of native prompt tokens used
    pub native_tokens_prompt: Option<i64>,
    /// Number of native completion tokens used
    pub native_tokens_completion: Option<i64>,
    /// Number of native reasoning tokens used
    pub native_tokens_reasoning: Option<i64>,
    /// Number of media items in prompt
    pub num_media_prompt: Option<i64>,
    /// Number of media items in completion
    pub num_media_completion: Option<i64>,
    /// Number of search results used
    pub num_search_results: Option<i64>,
    /// Origin of the generation request
    pub origin: String,
    /// Usage amount (deprecated, use total_cost)
    pub usage: f64,
    /// Whether this is a BYOK (Bring Your Own Key) generation
    pub is_byok: bool,
}

impl GenerationData {
    /// Get total tokens used (prompt + completion).
    pub fn total_tokens(&self) -> Option<i64> {
        match (self.tokens_prompt, self.tokens_completion) {
            (Some(prompt), Some(completion)) => Some(prompt + completion),
            (Some(prompt), None) => Some(prompt),
            (None, Some(completion)) => Some(completion),
            (None, None) => None,
        }
    }

    /// Get total native tokens used (prompt + completion + reasoning).
    pub fn total_native_tokens(&self) -> Option<i64> {
        let prompt = self.native_tokens_prompt.unwrap_or(0);
        let completion = self.native_tokens_completion.unwrap_or(0);
        let reasoning = self.native_tokens_reasoning.unwrap_or(0);

        if prompt == 0 && completion == 0 && reasoning == 0 {
            None
        } else {
            Some(prompt + completion + reasoning)
        }
    }

    /// Check if the generation was successful.
    pub fn is_successful(&self) -> bool {
        !self.cancelled.is_cancelled()
    }

    /// Check if the generation was streamed.
    pub fn was_streamed(&self) -> bool {
        self.streamed.is_active()
    }

    /// Check if the generation was cancelled.
    pub fn was_cancelled(&self) -> bool {
        self.cancelled.is_cancelled()
    }

    /// Get the effective cost (total cost minus cache discount).
    pub fn effective_cost(&self) -> f64 {
        self.total_cost - self.cache_discount.unwrap_or(0.0)
    }

    /// Get cost per token (if token count is available).
    pub fn cost_per_token(&self) -> Option<f64> {
        self.total_tokens()
            .map(|tokens| self.total_cost / tokens as f64)
    }

    /// Get latency in seconds (if available).
    pub fn latency_seconds(&self) -> Option<f64> {
        self.latency.map(|ms| ms as f64 / 1000.0)
    }

    /// Get generation time in seconds (if available).
    pub fn generation_time_seconds(&self) -> Option<f64> {
        self.generation_time.map(|ms| ms as f64 / 1000.0)
    }

    /// Check if this generation used web search.
    pub fn used_web_search(&self) -> bool {
        self.num_search_results.unwrap_or(0) > 0
    }

    /// Check if this generation included media.
    pub fn included_media(&self) -> bool {
        (self.num_media_prompt.unwrap_or(0) + self.num_media_completion.unwrap_or(0)) > 0
    }

    /// Check if this generation used reasoning tokens.
    pub fn used_reasoning(&self) -> bool {
        self.native_tokens_reasoning.unwrap_or(0) > 0
    }
}

/// Response from the generation endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenerationResponse {
    /// Generation data
    pub data: GenerationData,
}

impl GenerationResponse {
    /// Get reference to generation data.
    pub fn generation(&self) -> &GenerationData {
        &self.data
    }

    /// Get generation ID.
    pub fn id(&self) -> &str {
        self.data.id.as_str()
    }

    /// Get model used.
    pub fn model(&self) -> &str {
        &self.data.model
    }

    /// Get total cost.
    pub fn total_cost(&self) -> f64 {
        self.data.total_cost
    }

    /// Get effective cost (total cost minus cache discount).
    pub fn effective_cost(&self) -> f64 {
        self.data.effective_cost()
    }

    /// Check if the generation was successful.
    pub fn is_successful(&self) -> bool {
        self.data.is_successful()
    }

    /// Check if the generation was streamed.
    pub fn was_streamed(&self) -> bool {
        self.data.was_streamed()
    }

    /// Get total tokens used.
    pub fn total_tokens(&self) -> Option<i64> {
        self.data.total_tokens()
    }

    /// Get cost per token.
    pub fn cost_per_token(&self) -> Option<f64> {
        self.data.cost_per_token()
    }

    /// Get latency in seconds.
    pub fn latency_seconds(&self) -> Option<f64> {
        self.data.latency_seconds()
    }

    /// Check if web search was used.
    pub fn used_web_search(&self) -> bool {
        self.data.used_web_search()
    }

    /// Check if media was included.
    pub fn included_media(&self) -> bool {
        self.data.included_media()
    }

    /// Check if reasoning was used.
    pub fn used_reasoning(&self) -> bool {
        self.data.used_reasoning()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_generation_data() -> GenerationData {
        GenerationData {
            id: GenerationId::new("gen-123456"),
            upstream_id: Some("upstream-789".to_string()),
            total_cost: 0.025,
            cache_discount: Some(0.005),
            upstream_inference_cost: Some(0.020),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            model: "openai/gpt-4".to_string(),
            app_id: Some(12345),
            streamed: StreamingStatus::Complete,
            cancelled: CancellationStatus::NotCancelled,
            provider_name: Some("OpenAI".to_string()),
            latency: Some(1500),
            moderation_latency: Some(100),
            generation_time: Some(1200),
            finish_reason: Some("stop".to_string()),
            native_finish_reason: Some("stop".to_string()),
            tokens_prompt: Some(50),
            tokens_completion: Some(100),
            native_tokens_prompt: Some(50),
            native_tokens_completion: Some(100),
            native_tokens_reasoning: Some(25),
            num_media_prompt: Some(2),
            num_media_completion: Some(0),
            num_search_results: Some(5),
            origin: "api".to_string(),
            usage: 0.025,
            is_byok: false,
        }
    }

    #[test]
    fn test_generation_data_total_tokens() {
        let data = create_test_generation_data();
        assert_eq!(data.total_tokens(), Some(150)); // 50 + 100
    }

    #[test]
    fn test_generation_data_total_native_tokens() {
        let data = create_test_generation_data();
        assert_eq!(data.total_native_tokens(), Some(175)); // 50 + 100 + 25
    }

    #[test]
    fn test_generation_data_success_checks() {
        let data = create_test_generation_data();
        assert!(data.is_successful());
        assert!(data.was_streamed());
        assert!(!data.was_cancelled());
    }

    #[test]
    fn test_generation_data_cost_calculations() {
        let data = create_test_generation_data();
        assert_eq!(data.effective_cost(), 0.020); // 0.025 - 0.005
        assert_eq!(data.cost_per_token(), Some(0.025 / 150.0));
    }

    #[test]
    fn test_generation_data_time_conversions() {
        let data = create_test_generation_data();
        assert_eq!(data.latency_seconds(), Some(1.5));
        assert_eq!(data.generation_time_seconds(), Some(1.2));
    }

    #[test]
    fn test_generation_data_feature_checks() {
        let data = create_test_generation_data();
        assert!(data.used_web_search());
        assert!(data.included_media());
        assert!(data.used_reasoning());
    }

    #[test]
    fn test_generation_response_convenience_methods() {
        let data = create_test_generation_data();
        let response = GenerationResponse { data };

        assert_eq!(response.id(), "gen-123456");
        assert_eq!(response.model(), "openai/gpt-4");
        assert_eq!(response.total_cost(), 0.025);
        assert_eq!(response.effective_cost(), 0.020);
        assert!(response.is_successful());
        assert!(response.was_streamed());
        assert_eq!(response.total_tokens(), Some(150));
        assert!(response.used_web_search());
        assert!(response.included_media());
        assert!(response.used_reasoning());
    }

    #[test]
    fn test_generation_data_edge_cases() {
        // Test with minimal data
        let minimal_data = GenerationData {
            id: GenerationId::new("gen-minimal"),
            upstream_id: None,
            total_cost: 0.01,
            cache_discount: None,
            upstream_inference_cost: None,
            created_at: "2024-01-15T10:30:00Z".to_string(),
            model: "openai/gpt-3.5-turbo".to_string(),
            app_id: None,
            streamed: StreamingStatus::default(),
            cancelled: CancellationStatus::default(),
            provider_name: None,
            latency: None,
            moderation_latency: None,
            generation_time: None,
            finish_reason: None,
            native_finish_reason: None,
            tokens_prompt: None,
            tokens_completion: None,
            native_tokens_prompt: None,
            native_tokens_completion: None,
            native_tokens_reasoning: None,
            num_media_prompt: None,
            num_media_completion: None,
            num_search_results: None,
            origin: "api".to_string(),
            usage: 0.01,
            is_byok: false,
        };

        assert_eq!(minimal_data.total_tokens(), None);
        assert_eq!(minimal_data.total_native_tokens(), None);
        assert!(minimal_data.is_successful());
        assert!(!minimal_data.was_streamed());
        assert!(!minimal_data.was_cancelled());
        assert_eq!(minimal_data.effective_cost(), 0.01);
        assert_eq!(minimal_data.cost_per_token(), None);
        assert!(!minimal_data.used_web_search());
        assert!(!minimal_data.included_media());
        assert!(!minimal_data.used_reasoning());
    }

    #[test]
    fn test_generation_serialization() {
        let data = create_test_generation_data();
        let json = serde_json::to_string(&data).unwrap();
        let parsed: GenerationData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, parsed);
    }

    #[test]
    fn test_generation_response_serialization() {
        let data = create_test_generation_data();
        let response = GenerationResponse { data };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: GenerationResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
    }

    #[test]
    fn test_generation_id_serialization() {
        let generation = GenerationData {
            id: GenerationId::new("gen-12345"),
            upstream_id: Some("upstream-789".to_string()),
            total_cost: 0.025,
            cache_discount: Some(0.005),
            upstream_inference_cost: Some(0.020),
            created_at: "2024-01-15T10:30:00Z".to_string(),
            model: "openai/gpt-4".to_string(),
            app_id: Some(12345),
            streamed: StreamingStatus::Complete,
            cancelled: CancellationStatus::NotCancelled,
            provider_name: Some("OpenAI".to_string()),
            latency: Some(1500),
            moderation_latency: Some(100),
            generation_time: Some(1200),
            finish_reason: Some("stop".to_string()),
            native_finish_reason: Some("stop".to_string()),
            tokens_prompt: Some(50),
            tokens_completion: Some(100),
            native_tokens_prompt: Some(50),
            native_tokens_completion: Some(100),
            native_tokens_reasoning: Some(25),
            num_media_prompt: Some(2),
            num_media_completion: Some(0),
            num_search_results: Some(5),
            origin: "api".to_string(),
            usage: 0.025,
            is_byok: false,
        };

        // Test that GenerationId serializes as a plain string
        let json = serde_json::to_string(&generation).unwrap();
        assert!(json.contains("\"gen-12345\""));

        // Test deserialization roundtrip
        let deserialized: GenerationData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id.as_str(), "gen-12345");
    }

    #[test]
    fn test_generation_id_from_string() {
        let id: GenerationId = "string-id".into();
        assert_eq!(id.as_str(), "string-id");

        let id2: GenerationId = String::from("string-id-2").into();
        assert_eq!(id2.as_str(), "string-id-2");
    }

    #[test]
    fn test_generation_id_display() {
        let id = GenerationId::new("test-display");
        assert_eq!(format!("{}", id), "test-display");
    }

    #[test]
    fn test_generation_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(GenerationId::new("id-1"));
        set.insert(GenerationId::new("id-2"));
        set.insert(GenerationId::new("id-1")); // Duplicate

        assert_eq!(set.len(), 2); // Should only have 2 unique IDs
    }
}
