//! Unit tests for client functionality

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::{ClientConfig, OpenRouterClient, RetryConfig, SecureApiKey, Unconfigured};
    use crate::error::Error;
    use std::time::Duration;

    #[test]
    fn test_secure_api_key_creation_valid() {
        let key = "sk-1234567890abcdef1234567890abcdef123456789";
        let secure_key = SecureApiKey::new(key).unwrap();
        assert_eq!(secure_key.as_str(), key);
    }

    #[test]
    fn test_secure_api_key_creation_with_or_prefix() {
        let key = "or-1234567890abcdef1234567890abcdef123456789";
        let secure_key = SecureApiKey::new(key).unwrap();
        assert_eq!(secure_key.as_str(), key);
    }

    #[test]
    fn test_secure_api_key_too_short() {
        let key = "sk-short";
        let result = SecureApiKey::new(key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[test]
    fn test_secure_api_key_invalid_prefix() {
        let key = "invalid-1234567890abcdef1234567890abcdef123456789";
        let result = SecureApiKey::new(key);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[test]
    fn test_secure_api_key_empty() {
        let result = SecureApiKey::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[test]
    fn test_secure_api_key_whitespace_only() {
        let result = SecureApiKey::new("   ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[test]
    fn test_secure_api_key_debug_redaction() {
        let key = "sk-1234567890abcdef1234567890abcdef123456789";
        let secure_key = SecureApiKey::new(key).unwrap();
        let debug_str = format!("{secure_key:?}");
        assert!(!debug_str.contains("1234567890abcdef"));
        assert!(debug_str.contains("[REDACTED]"));
    }

    #[test]
    fn test_secure_api_key_bearer_header() {
        let key = "sk-1234567890abcdef1234567890abcdef123456789";
        let secure_key = SecureApiKey::new(key).unwrap();
        let bearer = secure_key.to_bearer_header();
        assert_eq!(bearer, format!("Bearer {key}"));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 500);
        assert_eq!(config.max_backoff_ms, 10000);
        assert_eq!(config.retry_on_status_codes, vec![429, 500, 502, 503, 504]);
    }

    #[test]
    fn test_client_default_creation() {
        let client = OpenRouterClient::<Unconfigured>::default();
        assert!(client.config.api_key.is_none());
        assert_eq!(
            client.config.base_url.as_str(),
            "https://openrouter.ai/api/v1/"
        );
        assert_eq!(client.config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_client_new_creation() {
        let client = OpenRouterClient::<Unconfigured>::new();
        assert!(client.config.api_key.is_none());
        assert_eq!(
            client.config.base_url.as_str(),
            "https://openrouter.ai/api/v1/"
        );
        assert!(client.http_client.is_none());
    }

    #[test]
    fn test_client_with_base_url_valid() {
        let client = OpenRouterClient::<Unconfigured>::new();
        let result = client.with_base_url("https://api.example.com/v2/");
        assert!(result.is_ok());
        let client = result.unwrap();
        assert_eq!(
            client.config.base_url.as_str(),
            "https://api.example.com/v2/"
        );
    }

    #[test]
    fn test_client_with_base_url_invalid() {
        let client = OpenRouterClient::<Unconfigured>::new();
        let result = client.with_base_url("not-a-valid-url");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[test]
    fn test_client_configuration_chain() {
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://api.example.com/")
            .unwrap()
            .with_timeout(Duration::from_secs(60))
            .with_http_referer("https://myapp.com")
            .with_site_title("My App")
            .with_user_id("user123");

        assert_eq!(client.config.base_url.as_str(), "https://api.example.com/");
        assert_eq!(client.config.timeout, Duration::from_secs(60));
        assert_eq!(
            client.config.http_referer,
            Some("https://myapp.com".to_string())
        );
        assert_eq!(client.config.site_title, Some("My App".to_string()));
        assert_eq!(client.config.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_client_with_retry_config() {
        let retry_config = RetryConfig {
            max_retries: 5,
            initial_backoff_ms: 1000,
            max_backoff_ms: 30000,
            retry_on_status_codes: vec![429, 500],
            total_timeout: Duration::from_secs(120),
            max_retry_interval: Duration::from_secs(60),
        };

        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://api.example.com/")
            .unwrap()
            .with_retry_config(retry_config.clone());

        assert_eq!(client.config.retry_config.max_retries, 5);
        assert_eq!(client.config.retry_config.initial_backoff_ms, 1000);
        assert_eq!(client.config.retry_config.max_backoff_ms, 30000);
        assert_eq!(
            client.config.retry_config.retry_on_status_codes,
            vec![429, 500]
        );
    }

    #[test]
    fn test_client_with_api_key_valid() {
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://api.example.com/")
            .unwrap();

        let result = client.with_api_key("sk-1234567890abcdef1234567890abcdef123456789");
        assert!(result.is_ok());
    }

    #[test]
    fn test_client_with_api_key_invalid() {
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://api.example.com/")
            .unwrap();

        let result = client.with_api_key("invalid-key");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[tokio::test]
    async fn test_client_config_build_headers_without_api_key() {
        let config = ClientConfig {
            api_key: None,
            base_url: "https://api.example.com/".parse().unwrap(),
            http_referer: None,
            site_title: None,
            user_id: None,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let headers = config.build_headers().unwrap();
        assert!(headers.get("authorization").is_none());
        assert!(headers.get("content-type").is_some());
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
    }

    #[tokio::test]
    async fn test_client_config_build_headers_with_api_key() {
        let api_key = SecureApiKey::new("sk-1234567890abcdef1234567890abcdef123456789").unwrap();
        let config = ClientConfig {
            api_key: Some(api_key),
            base_url: "https://api.example.com/".parse().unwrap(),
            http_referer: None,
            site_title: None,
            user_id: None,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let headers = config.build_headers().unwrap();
        assert!(headers.get("authorization").is_some());
        let auth_header = headers.get("authorization").unwrap().to_str().unwrap();
        assert!(auth_header.starts_with("Bearer sk-"));
    }

    #[tokio::test]
    async fn test_client_config_build_headers_with_all_options() {
        let api_key = SecureApiKey::new("sk-1234567890abcdef1234567890abcdef123456789").unwrap();
        let config = ClientConfig {
            api_key: Some(api_key),
            base_url: "https://api.example.com/".parse().unwrap(),
            http_referer: Some("https://myapp.com".to_string()),
            site_title: Some("My App".to_string()),
            user_id: Some("user123".to_string()),
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let headers = config.build_headers().unwrap();
        assert!(headers.get("authorization").is_some());
        assert!(headers.get("content-type").is_some());
        assert!(headers.get("referer").is_some());
        assert!(headers.get("x-title").is_some());
        assert!(headers.get("x-user-id").is_some());

        assert_eq!(headers.get("referer").unwrap(), "https://myapp.com");
        assert_eq!(headers.get("x-title").unwrap(), "My App");
        assert_eq!(headers.get("x-user-id").unwrap(), "user123");
    }

    #[test]
    fn test_client_config_build_headers_invalid_header_values() {
        let api_key = SecureApiKey::new("sk-1234567890abcdef1234567890abcdef123456789").unwrap();
        let config = ClientConfig {
            api_key: Some(api_key),
            base_url: "https://api.example.com/".parse().unwrap(),
            http_referer: Some("invalid\nheader\nvalue".to_string()),
            site_title: None,
            user_id: None,
            timeout: Duration::from_secs(30),
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let result = config.build_headers();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::ConfigError(_)));
    }

    #[test]
    fn test_client_max_response_bytes_config() {
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_max_response_bytes(1024);
        
        assert_eq!(client.config.max_response_bytes, 1024);
    }
}
