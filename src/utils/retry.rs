//! Retry utilities for HTTP requests with exponential backoff

use crate::client::RetryConfig;
use crate::error::{Error, Result};
use fastrand::Rng;
use reqwest::{RequestBuilder, Response};
use std::time::Duration;
use tokio::time::sleep;

/// Operation names for retry logging and error context
pub mod operations {
    pub const TEXT_COMPLETION: &str = "text_completion";
    pub const WEB_SEARCH: &str = "web_search";
    pub const LIST_MODELS: &str = "list_models";
    pub const GET_BALANCE: &str = "get_balance";
    pub const GET_ACTIVITY: &str = "get_activity";
    pub const GET_PROVIDERS: &str = "get_providers";
    pub const GET_GENERATION: &str = "get_generation";
    pub const STRUCTURED_GENERATE: &str = "structured_generate";
    pub const CHAT_COMPLETION: &str = "chat_completion";
}

/// Executes an HTTP request with retry logic using a closure for request building
///
/// This version is more flexible as it recreates the request for each retry attempt,
/// avoiding issues with request builder consumption.
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation_name` - Name of the operation for logging purposes

/// ```
pub async fn execute_with_retry_builder<F>(
    config: &RetryConfig,
    operation_name: &str,
    mut request_builder: F,
) -> Result<Response>
where
    F: FnMut() -> RequestBuilder,
{
    let mut retry_count = 0;
    let mut backoff_ms = config.initial_backoff_ms;
    let mut rng = Rng::new();

    loop {
        // Build a fresh request for each attempt
        let response = request_builder().send().await?;

        let status = response.status();
        let status_code = status.as_u16();

        // Check if we should retry based on status code and retry count
        if config.retry_on_status_codes.contains(&status_code) && retry_count < config.max_retries {
            retry_count += 1;

            // Add jitter to prevent thundering herd (±25% random variation)
            let jitter_factor = rng.f64() * 0.5 + 0.75; // Range: 0.75 to 1.25
            let jittered_backoff_ms = (backoff_ms as f64 * jitter_factor) as u64;

            // Log retry attempt
            eprintln!(
                "Retrying {} request ({}/{}) after {} ms (base: {} ms, jitter: {:.2}%) due to status code {}",
                operation_name, retry_count, config.max_retries, jittered_backoff_ms, backoff_ms,
                (jitter_factor - 1.0) * 100.0, status_code
            );

            // Wait before retrying
            sleep(Duration::from_millis(jittered_backoff_ms)).await;

            // Calculate next backoff with exponential increase
            backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);
            continue;
        }

        // Return the response (whether successful or not)
        return Ok(response);
    }
}

/// Handles HTTP response with consistent error parsing
///
/// # Arguments
/// * `response` - The HTTP response to handle
/// * `operation_name` - Name of the operation for error context
///
/// # Returns
/// * `Result<String>` - The response body text or error
pub async fn handle_response_text(response: Response, operation_name: &str) -> Result<String> {
    let status = response.status();
    let status_code = status.as_u16();
    let body = response.text().await?;

    // Check if the HTTP response was successful
    if !status.is_success() {
        return Err(Error::from_response_text(status_code, &body)?);
    }

    // Check for empty response body
    if body.trim().is_empty() {
        return Err(Error::ApiError {
            code: status_code,
            message: format!("Empty response body for {}", operation_name),
            metadata: None,
        });
    }

    Ok(body)
}

/// Handles HTTP response with JSON deserialization
///
/// # Arguments
/// * `response` - The HTTP response to handle
/// * `operation_name` - Name of the operation for error context
///
/// # Returns
/// * `Result<T>` - The deserialized response or error
pub async fn handle_response_json<T: serde::de::DeserializeOwned>(
    response: Response,
    operation_name: &str,
) -> Result<T> {
    let status = response.status();
    let status_code = status.as_u16();
    let body = response.text().await?;

    // Check if the HTTP response was successful
    if !status.is_success() {
        return Err(Error::from_response_text(status_code, &body)?);
    }

    // Check for empty response body
    if body.trim().is_empty() {
        return Err(Error::ApiError {
            code: status_code,
            message: format!("Empty response body for {}", operation_name),
            metadata: None,
        });
    }

    // Deserialize the JSON response
    serde_json::from_str::<T>(&body).map_err(|e| Error::ApiError {
        code: status_code,
        message: crate::utils::security::create_safe_error_message(
            &format!(
                "Failed to decode JSON response for {}: {}. Body was: {}",
                operation_name, e, body
            ),
            &format!("{} JSON parsing error", operation_name),
        ),
        metadata: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::client::RetryConfig;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 500);
        assert_eq!(config.max_backoff_ms, 10000);
        assert!(config.retry_on_status_codes.contains(&429));
        assert!(config.retry_on_status_codes.contains(&500));
    }

    #[test]
    fn test_exponential_backoff_calculation() {
        let config = RetryConfig::default();
        let mut backoff_ms = config.initial_backoff_ms;

        // Test exponential backoff progression
        assert_eq!(backoff_ms, 500);

        backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);
        assert_eq!(backoff_ms, 1000);

        backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);
        assert_eq!(backoff_ms, 2000);

        // Should cap at max_backoff_ms
        for _ in 0..10 {
            backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);
        }
        assert_eq!(backoff_ms, config.max_backoff_ms);
    }

    #[tokio::test]
    async fn test_retry_config_status_codes() {
        let config = RetryConfig::default();

        // Test default retry status codes
        assert!(config.retry_on_status_codes.contains(&429)); // Rate limited
        assert!(config.retry_on_status_codes.contains(&500)); // Internal server error
        assert!(config.retry_on_status_codes.contains(&502)); // Bad gateway
        assert!(config.retry_on_status_codes.contains(&503)); // Service unavailable
        assert!(config.retry_on_status_codes.contains(&504)); // Gateway timeout

        // Test that success codes are not included
        assert!(!config.retry_on_status_codes.contains(&200));
        assert!(!config.retry_on_status_codes.contains(&201));

        // Test that client errors (except 429) are not included
        assert!(!config.retry_on_status_codes.contains(&400));
        assert!(!config.retry_on_status_codes.contains(&401));
        assert!(!config.retry_on_status_codes.contains(&404));
    }
}
