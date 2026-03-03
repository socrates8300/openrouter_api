//! Types for OpenRouter Key Info API responses.

use serde::{Deserialize, Serialize};

/// Rate limit information for the API key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RateLimitInfo {
    /// Maximum requests allowed per interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests: Option<u32>,
    /// Rate limit interval (e.g., "10s", "1m").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
}

/// Key info data returned by the OpenRouter API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyInfoData {
    /// Human-readable label for the API key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Credit limit for the key (None means unlimited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<f64>,
    /// Remaining credit limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_remaining: Option<f64>,
    /// Total usage in credits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<f64>,
    /// Whether the key is on the free tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_free_tier: Option<bool>,
    /// Rate limit configuration for the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimitInfo>,
}

/// Response from the key info endpoint (`GET /api/v1/auth/key`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyInfoResponse {
    /// Key info data.
    pub data: KeyInfoData,
}

impl KeyInfoResponse {
    /// Get credit limit (None means unlimited).
    pub fn limit(&self) -> Option<f64> {
        self.data.limit
    }

    /// Get remaining credit limit.
    pub fn limit_remaining(&self) -> Option<f64> {
        self.data.limit_remaining
    }

    /// Get total usage.
    pub fn usage(&self) -> Option<f64> {
        self.data.usage
    }

    /// Check if the key is on the free tier.
    pub fn is_free_tier(&self) -> bool {
        self.data.is_free_tier.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_info_deserialization() {
        let json = r#"{
            "data": {
                "label": "My Key",
                "limit": 100.0,
                "limit_remaining": 75.5,
                "usage": 24.5,
                "is_free_tier": false,
                "rate_limit": {
                    "requests": 200,
                    "interval": "10s"
                }
            }
        }"#;

        let response: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.label, Some("My Key".to_string()));
        assert_eq!(response.limit(), Some(100.0));
        assert_eq!(response.limit_remaining(), Some(75.5));
        assert_eq!(response.usage(), Some(24.5));
        assert!(!response.is_free_tier());
        assert_eq!(
            response.data.rate_limit.as_ref().unwrap().requests,
            Some(200)
        );
    }

    #[test]
    fn test_key_info_minimal_response() {
        let json = r#"{"data": {}}"#;
        let response: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.limit(), None);
        assert!(!response.is_free_tier()); // defaults to false via unwrap_or
    }

    #[test]
    fn test_key_info_free_tier() {
        let json = r#"{"data": {"is_free_tier": true, "usage": 0.0}}"#;
        let response: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert!(response.is_free_tier());
    }

    #[test]
    fn test_key_info_serialization_roundtrip() {
        let response = KeyInfoResponse {
            data: KeyInfoData {
                label: Some("Test".to_string()),
                limit: Some(50.0),
                limit_remaining: Some(25.0),
                usage: Some(25.0),
                is_free_tier: Some(false),
                rate_limit: Some(RateLimitInfo {
                    requests: Some(100),
                    interval: Some("10s".to_string()),
                }),
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: KeyInfoResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
    }
}
