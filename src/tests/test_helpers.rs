//! Shared test helper utilities to reduce copy-pasted test setup code.

use crate::client::{ApiConfig, ClientConfig, RetryConfig, SecureApiKey};
use std::time::Duration;
use url::Url;

/// Default test API key that passes validation (sk- prefix, 20+ chars).
pub const TEST_API_KEY: &str = "sk-test123456789012345678901234567890";

/// Default test base URL.
pub const TEST_BASE_URL: &str = "https://openrouter.ai/api/v1/";

/// Creates a standard `ClientConfig` for testing with default values.
pub fn test_client_config() -> ClientConfig {
    ClientConfig {
        api_key: Some(SecureApiKey::new(TEST_API_KEY).unwrap()),
        base_url: Url::parse(TEST_BASE_URL).unwrap(),
        timeout: Duration::from_secs(30),
        http_referer: None,
        site_title: None,
        user_id: None,
        retry_config: RetryConfig::default(),
        max_response_bytes: 10 * 1024 * 1024,
    }
}

/// Creates a standard `ClientConfig` with a custom API key.
#[allow(dead_code)]
pub fn test_client_config_with_key(api_key: &str) -> ClientConfig {
    ClientConfig {
        api_key: Some(SecureApiKey::new(api_key).unwrap()),
        base_url: Url::parse(TEST_BASE_URL).unwrap(),
        timeout: Duration::from_secs(30),
        http_referer: None,
        site_title: None,
        user_id: None,
        retry_config: RetryConfig::default(),
        max_response_bytes: 10 * 1024 * 1024,
    }
}

/// Creates a standard `ApiConfig` for testing by converting from `test_client_config()`.
#[allow(dead_code)]
pub fn test_api_config() -> ApiConfig {
    test_client_config().to_api_config().unwrap()
}
