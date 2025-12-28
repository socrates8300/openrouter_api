//! Types for OpenRouter Credits API responses.

use serde::{Deserialize, Serialize};

/// Credits data returned by the OpenRouter API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreditsData {
    /// Total credits purchased by the user
    pub total_credits: f64,
    /// Total credits used by the user
    pub total_usage: f64,
}

impl CreditsData {
    /// Calculate remaining credits.
    #[must_use]
    pub fn remaining(&self) -> f64 {
        self.total_credits - self.total_usage
    }

    /// Check if user has credits available.
    #[must_use]
    pub fn has_credits(&self) -> bool {
        self.remaining() > 0.0
    }

    /// Get usage percentage (0.0 to 1.0).
    #[must_use]
    pub fn usage_percentage(&self) -> f64 {
        if self.total_credits == 0.0 {
            0.0
        } else {
            self.total_usage / self.total_credits
        }
    }
}

/// Response from the credits endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreditsResponse {
    /// Credits data
    pub data: CreditsData,
}

impl CreditsResponse {
    /// Get reference to credits data.
    pub fn credits(&self) -> &CreditsData {
        &self.data
    }

    /// Get total credits purchased.
    #[must_use]
    pub fn total_credits(&self) -> f64 {
        self.data.total_credits
    }

    /// Get total credits used.
    #[must_use]
    pub fn total_usage(&self) -> f64 {
        self.data.total_usage
    }

    /// Get remaining credits.
    #[must_use]
    pub fn remaining_credits(&self) -> f64 {
        self.data.remaining()
    }

    /// Check if user has credits available.
    #[must_use]
    pub fn has_credits(&self) -> bool {
        self.data.has_credits()
    }

    /// Get usage percentage (0.0 to 1.0).
    #[must_use]
    pub fn usage_percentage(&self) -> f64 {
        self.data.usage_percentage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credits_data_remaining() {
        let data = CreditsData {
            total_credits: 100.0,
            total_usage: 25.0,
        };
        assert_eq!(data.remaining(), 75.0);
    }

    #[test]
    fn test_credits_data_has_credits() {
        let data_with_credits = CreditsData {
            total_credits: 100.0,
            total_usage: 25.0,
        };
        let data_no_credits = CreditsData {
            total_credits: 100.0,
            total_usage: 100.0,
        };
        let data_negative = CreditsData {
            total_credits: 100.0,
            total_usage: 150.0,
        };

        assert!(data_with_credits.has_credits());
        assert!(!data_no_credits.has_credits());
        assert!(!data_negative.has_credits());
    }

    #[test]
    fn test_credits_data_usage_percentage() {
        let data = CreditsData {
            total_credits: 100.0,
            total_usage: 25.0,
        };
        assert_eq!(data.usage_percentage(), 0.25);

        let data_zero_total = CreditsData {
            total_credits: 0.0,
            total_usage: 25.0,
        };
        assert_eq!(data_zero_total.usage_percentage(), 0.0);
    }

    #[test]
    fn test_credits_response_convenience_methods() {
        let response = CreditsResponse {
            data: CreditsData {
                total_credits: 100.0,
                total_usage: 30.0,
            },
        };

        assert_eq!(response.total_credits(), 100.0);
        assert_eq!(response.total_usage(), 30.0);
        assert_eq!(response.remaining_credits(), 70.0);
        assert!(response.has_credits());
        assert_eq!(response.usage_percentage(), 0.3);
    }

    #[test]
    fn test_credits_serialization() {
        let data = CreditsData {
            total_credits: 100.5,
            total_usage: 25.25,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: CreditsData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, parsed);
    }

    #[test]
    fn test_credits_response_serialization() {
        let response = CreditsResponse {
            data: CreditsData {
                total_credits: 100.5,
                total_usage: 25.25,
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: CreditsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response, parsed);
    }
}
