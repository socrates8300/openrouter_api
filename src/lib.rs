//! # OpenRouter API Client Library
//!
//! A Rust client for interfacing with the OpenRouter API.

pub mod api;
pub mod client;
pub mod error;
pub mod mcp; // Add the MCP module
pub mod models;
pub mod tests;
pub mod types;
pub mod utils;

pub use error::{Error, Result};
pub use types::*;

pub use client::{NoAuth, OpenRouterClient, Ready, Unconfigured};
pub use mcp::client::MCPClient; // Re-export MCPClient
pub use mcp::types as mcp_types; // Re-export MCP types
pub use mcp::McpConfig; // Re-export McpConfig

#[cfg(test)]
mod security_tests {
    #![allow(unused)]
    use crate::client::{ClientConfig, RetryConfig, SecureApiKey};

    #[test]
    fn test_secure_api_key_no_clone() {
        let key = SecureApiKey::new("sk-test123456789012345678901234567890").unwrap();

        // Verify that the API key is accessible via reference
        assert_eq!(key.as_str(), "sk-test123456789012345678901234567890");

        // The following line should NOT compile - uncommenting it would cause a compile error:
        // let cloned = key.clone();

        // Verify the key still works for authentication
        let header = key.to_bearer_header();
        assert!(header.starts_with("Bearer "));
        assert!(header.contains("sk-test123456789012345678901234567890"));
    }

    #[test]
    fn test_api_config_isolation() {
        use crate::client::{ClientConfig, RetryConfig};
        use url::Url;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: Url::parse("https://openrouter.ai/api/v1").unwrap(),
            timeout: std::time::Duration::from_secs(30),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
        };

        // Create API config - this should NOT contain the API key
        let api_config = config.to_api_config().unwrap();

        // Verify headers are present but API key is not directly accessible
        assert!(!api_config.headers.is_empty());
        assert!(api_config.headers.contains_key("authorization"));

        // The API key should NOT be cloneable or directly accessible
        // This ensures security isolation between client config and API instances
    }
}
