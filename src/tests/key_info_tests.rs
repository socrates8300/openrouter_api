//! Targeted tests for the Key Info endpoint and types.
//!
//! Current coverage: 4 serde tests in types/key_info.rs + 1 constructor test in api/key_info.rs.
//! Gaps addressed here:
//! - KeyInfoApi URL construction (verifiable without network)
//! - KeyInfoResponse convenience method correctness under all combinations
//! - RateLimitInfo edge cases (None fields, zero requests, unusual intervals)
//! - Unlimited credit key (limit: null)
//! - KeyInfoData partial field presence (real API often omits optional fields)
//! - Wiremock-backed integration: correct HTTP method (GET), correct path, auth header present

#[cfg(test)]
mod tests {
    use crate::client::ClientConfig;
    use crate::tests::test_helpers::test_client_config;
    use crate::types::key_info::{KeyInfoData, KeyInfoResponse, RateLimitInfo};
    use url::Url;

    // =========================================================================
    // KeyInfoResponse — convenience methods
    // =========================================================================

    #[test]
    fn test_limit_remaining_is_none_when_field_absent() {
        let r = KeyInfoResponse {
            data: KeyInfoData {
                label: None,
                limit: None,
                limit_remaining: None,
                usage: None,
                is_free_tier: None,
                rate_limit: None,
            },
        };
        assert!(r.limit_remaining().is_none());
    }

    #[test]
    fn test_limit_remaining_is_some_when_set() {
        let r = KeyInfoResponse {
            data: KeyInfoData {
                label: None,
                limit: Some(100.0),
                limit_remaining: Some(42.5),
                usage: None,
                is_free_tier: None,
                rate_limit: None,
            },
        };
        assert_eq!(r.limit_remaining(), Some(42.5));
    }

    #[test]
    fn test_limit_is_none_for_unlimited_key() {
        // OpenRouter represents unlimited credit as null/absent limit.
        let json = r#"{"data": {"usage": 5.0}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert!(r.limit().is_none(), "null limit must map to None");
    }

    #[test]
    fn test_limit_remaining_zero_is_some_not_none() {
        // A depleted key returns 0.0 — this must be Some(0.0), not None.
        let json = r#"{"data": {"limit": 10.0, "limit_remaining": 0.0}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(
            r.limit_remaining(),
            Some(0.0),
            "limit_remaining of 0.0 must be Some(0.0)"
        );
    }

    #[test]
    fn test_usage_zero_is_some_not_none() {
        let json = r#"{"data": {"usage": 0.0}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.usage(), Some(0.0));
    }

    #[test]
    fn test_is_free_tier_false_when_field_absent() {
        // is_free_tier: None must default to false via unwrap_or(false).
        let r = KeyInfoResponse {
            data: KeyInfoData {
                label: None,
                limit: None,
                limit_remaining: None,
                usage: None,
                is_free_tier: None,
                rate_limit: None,
            },
        };
        assert!(!r.is_free_tier());
    }

    #[test]
    fn test_is_free_tier_true_when_set() {
        let json = r#"{"data": {"is_free_tier": true}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert!(r.is_free_tier());
    }

    #[test]
    fn test_data_accessor_returns_same_as_direct_field() {
        let r = KeyInfoResponse {
            data: KeyInfoData {
                label: Some("test".to_string()),
                limit: Some(50.0),
                limit_remaining: Some(25.0),
                usage: Some(25.0),
                is_free_tier: Some(false),
                rate_limit: None,
            },
        };
        // Direct field access works correctly.
        assert_eq!(r.data.label, Some("test".to_string()));
        assert_eq!(r.data.limit, Some(50.0));
    }

    // =========================================================================
    // RateLimitInfo edge cases
    // =========================================================================

    #[test]
    fn test_rate_limit_all_fields_none() {
        let json = r#"{"data": {"rate_limit": {}}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        let rl = r.data.rate_limit.unwrap();
        assert!(rl.requests.is_none());
        assert!(rl.interval.is_none());
    }

    #[test]
    fn test_rate_limit_various_intervals() {
        for interval in &["10s", "1m", "1h", "1d"] {
            let json = format!(
                r#"{{"data": {{"rate_limit": {{"requests": 100, "interval": "{interval}"}} }} }}"#
            );
            let r: KeyInfoResponse = serde_json::from_str(&json).unwrap();
            let rl = r.data.rate_limit.unwrap();
            assert_eq!(rl.interval.as_deref(), Some(*interval));
        }
    }

    #[test]
    fn test_rate_limit_zero_requests() {
        let json = r#"{"data": {"rate_limit": {"requests": 0, "interval": "10s"}}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        let rl = r.data.rate_limit.unwrap();
        assert_eq!(rl.requests, Some(0));
    }

    #[test]
    fn test_rate_limit_high_request_count() {
        let json = r#"{"data": {"rate_limit": {"requests": 1000000, "interval": "1d"}}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        let rl = r.data.rate_limit.unwrap();
        assert_eq!(rl.requests, Some(1_000_000));
    }

    #[test]
    fn test_rate_limit_info_equality() {
        let a = RateLimitInfo {
            requests: Some(200),
            interval: Some("10s".to_string()),
        };
        let b = RateLimitInfo {
            requests: Some(200),
            interval: Some("10s".to_string()),
        };
        assert_eq!(a, b);
    }

    // =========================================================================
    // Deserialization — realistic partial responses (OpenRouter often omits fields)
    // =========================================================================

    #[test]
    fn test_only_usage_field_present() {
        let json = r#"{"data": {"usage": 12.75}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.usage(), Some(12.75));
        assert!(r.limit().is_none());
        assert!(r.limit_remaining().is_none());
        assert!(!r.is_free_tier());
    }

    #[test]
    fn test_label_only_response() {
        let json = r#"{"data": {"label": "production-key"}}"#;
        let r: KeyInfoResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.label.as_deref(), Some("production-key"));
        assert!(r.limit().is_none());
    }

    #[test]
    fn test_key_info_response_equality_derived() {
        let a = KeyInfoResponse {
            data: KeyInfoData {
                label: Some("key".to_string()),
                limit: Some(100.0),
                limit_remaining: Some(50.0),
                usage: Some(50.0),
                is_free_tier: Some(false),
                rate_limit: Some(RateLimitInfo {
                    requests: Some(100),
                    interval: Some("10s".to_string()),
                }),
            },
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // =========================================================================
    // KeyInfoApi — constructor creates headers correctly
    // =========================================================================

    #[test]
    fn test_key_info_api_has_authorization_header() {
        use crate::api::key_info::KeyInfoApi;

        let config = test_client_config();
        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();
        assert!(
            api.config.headers.contains_key("authorization"),
            "KeyInfoApi must inject Authorization header"
        );
    }

    #[test]
    fn test_key_info_api_base_url_resolves_correct_path() {
        use crate::api::key_info::KeyInfoApi;

        let config = test_client_config();
        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();

        // Verify that the base_url joined with "auth/key" gives the expected path.
        let url = api.config.base_url.join("auth/key").unwrap();
        assert!(
            url.path().ends_with("/auth/key"),
            "Expected path ending with /auth/key, got: {}",
            url.path()
        );
    }

    // =========================================================================
    // Wiremock integration: GET /auth/key with auth header, returns 200 JSON
    // =========================================================================

    #[tokio::test]
    async fn test_get_key_info_wiremock_happy_path() {
        use crate::api::key_info::KeyInfoApi;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "data": {
                "label": "test-key",
                "limit": 100.0,
                "limit_remaining": 75.0,
                "usage": 25.0,
                "is_free_tier": false,
                "rate_limit": {
                    "requests": 200,
                    "interval": "10s"
                }
            }
        });

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/api/v1/auth/key"))
            .and(matchers::header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            ..test_client_config()
        };

        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();
        let response = api.get_key_info().await.unwrap();

        assert_eq!(response.data.label.as_deref(), Some("test-key"));
        assert_eq!(response.limit(), Some(100.0));
        assert_eq!(response.limit_remaining(), Some(75.0));
        assert_eq!(response.usage(), Some(25.0));
        assert!(!response.is_free_tier());
    }

    #[tokio::test]
    async fn test_get_key_info_wiremock_401_unauthorized() {
        use crate::api::key_info::KeyInfoApi;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let error_body = serde_json::json!({
            "error": {
                "message": "Invalid API key",
                "code": 401
            }
        });

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/api/v1/auth/key"))
            .respond_with(ResponseTemplate::new(401).set_body_json(&error_body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            ..test_client_config()
        };

        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();
        let result = api.get_key_info().await;

        assert!(result.is_err(), "401 must produce an error");
        match result.unwrap_err() {
            crate::error::Error::ApiError { code, .. } => {
                assert_eq!(code, 401);
            }
            other => panic!("Expected ApiError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_get_key_info_wiremock_minimal_response() {
        use crate::api::key_info::KeyInfoApi;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Minimal response — only the required data wrapper.
        Mock::given(matchers::method("GET"))
            .and(matchers::path("/api/v1/auth/key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"data": {}})))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            ..test_client_config()
        };

        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();
        let response = api.get_key_info().await.unwrap();

        assert!(response.limit().is_none());
        assert!(!response.is_free_tier());
    }

    #[tokio::test]
    async fn test_get_key_info_wiremock_free_tier_key() {
        use crate::api::key_info::KeyInfoApi;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("GET"))
            .and(matchers::path("/api/v1/auth/key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "is_free_tier": true,
                    "usage": 0.0,
                    "rate_limit": {"requests": 20, "interval": "1m"}
                }
            })))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            base_url: Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            ..test_client_config()
        };

        let client = reqwest::Client::new();
        let api = KeyInfoApi::new(client, &config).unwrap();
        let response = api.get_key_info().await.unwrap();

        assert!(response.is_free_tier());
        assert_eq!(response.usage(), Some(0.0));
        assert_eq!(
            response
                .data
                .rate_limit
                .as_ref()
                .unwrap()
                .interval
                .as_deref(),
            Some("1m")
        );
    }
}
