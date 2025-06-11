use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Model Coverage Profile (MCP) for ensuring consistent model availability
/// while optimizing for quality, cost, and reliability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCoverageProfile {
    /// Primary model to use for requests
    pub primary: String,

    /// Ordered list of fallback models if primary is unavailable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallbacks: Option<Vec<String>>,

    /// Whether to enable automatic fallbacks based on provider availability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_fallback: Option<bool>,

    /// Maximum latency threshold in milliseconds before trying fallbacks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_threshold_ms: Option<u32>,

    /// Whether to fail fast if primary model is unavailable
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_fast: Option<bool>,

    /// Provider-specific options for this profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<HashMap<String, serde_json::Value>>,
}

/// Named profiles for common model coverage scenarios
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PredefinedModelCoverageProfile {
    /// Optimizes for lowest latency across available models
    LowestLatency,

    /// Optimizes for lowest cost across available models
    LowestCost,

    /// Optimizes for highest quality across available models
    HighestQuality,

    /// Custom configuration using specified models
    Custom(ModelCoverageProfile),
}

/// Router configuration for model selection and fallback behavior
#[derive(Debug, Serialize)]
pub struct RouterConfig {
    /// The model coverage profile to use
    pub profile: PredefinedModelCoverageProfile,

    /// Additional provider preferences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_preferences: Option<crate::types::provider::ProviderPreferences>,
}

/// Predefined model groups for common tasks
pub struct ModelGroups;

impl ModelGroups {
    /// Models optimized for instruction following and general tasks
    pub fn general() -> ModelCoverageProfile {
        ModelCoverageProfile {
            primary: "openai/gpt-4o".to_string(),
            fallbacks: Some(vec![
                "anthropic/claude-3-opus-20240229".to_string(),
                "anthropic/claude-3-sonnet-20240229".to_string(),
                "google/gemini-1.5-pro".to_string(),
            ]),
            auto_fallback: Some(true),
            latency_threshold_ms: Some(10000),
            fail_fast: Some(false),
            provider_options: None,
        }
    }

    /// Models optimized for code generation and understanding
    pub fn code() -> ModelCoverageProfile {
        ModelCoverageProfile {
            primary: "anthropic/claude-3-opus-20240229".to_string(),
            fallbacks: Some(vec![
                "openai/gpt-4o".to_string(),
                "google/gemini-1.5-pro".to_string(),
            ]),
            auto_fallback: Some(true),
            latency_threshold_ms: Some(8000),
            fail_fast: Some(false),
            provider_options: None,
        }
    }

    /// Models optimized for long context windows
    pub fn long_context() -> ModelCoverageProfile {
        ModelCoverageProfile {
            primary: "anthropic/claude-3-opus-20240229".to_string(),
            fallbacks: Some(vec![
                "google/gemini-1.5-pro".to_string(),
                "openai/gpt-4-turbo".to_string(),
            ]),
            auto_fallback: Some(true),
            latency_threshold_ms: Some(15000),
            fail_fast: Some(false),
            provider_options: None,
        }
    }
}
