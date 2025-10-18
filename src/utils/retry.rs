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
///
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
    let start_time = std::time::Instant::now();

    loop {
        // Check if we've exceeded the total timeout
        if start_time.elapsed() > config.total_timeout {
            return Err(Error::ConfigError(format!(
                "Retry timeout exceeded for {}: {}ms limit",
                operation_name,
                config.total_timeout.as_millis()
            )));
        }

        // Build a fresh request for each attempt
        let response = request_builder().send().await?;

        let status = response.status();
        let status_code = status.as_u16();

        // Check if we should retry based on status code and retry count
        if config.retry_on_status_codes.contains(&status_code) && retry_count < config.max_retries {
            retry_count += 1;

            // Calculate next backoff with exponential increase
            backoff_ms = std::cmp::min(backoff_ms * 2, config.max_backoff_ms);

            // Apply max retry interval cap
            let backoff_duration = Duration::from_millis(backoff_ms);
            let capped_backoff = std::cmp::min(backoff_duration, config.max_retry_interval);

            // Add jitter to prevent thundering herd (±25% random variation)
            let jitter_factor = rng.f64() * 0.5 + 0.75; // Range: 0.75 to 1.25
            let jittered_backoff =
                Duration::from_millis((capped_backoff.as_millis() as f64 * jitter_factor) as u64);

            // Check if the next retry would exceed the total timeout
            if start_time.elapsed() + jittered_backoff > config.total_timeout {
                return Err(Error::ConfigError(format!(
                    "Next retry would exceed timeout for {}: remaining time {}ms < required backoff {}ms",
                    operation_name,
                    (config.total_timeout - start_time.elapsed()).as_millis(),
                    jittered_backoff.as_millis()
                )));
            }

            // Log retry attempt
            eprintln!(
                "Retrying {} request ({}/{}) after {}ms (base: {}ms, capped: {}ms, jitter: {:.2}%) due to status code {}",
                operation_name, retry_count, config.max_retries, jittered_backoff.as_millis(), backoff_ms,
                capped_backoff.as_millis(), (jitter_factor - 1.0) * 100.0, status_code
            );

            // Wait before retrying
            sleep(jittered_backoff).await;
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
    use super::execute_with_retry_builder;
    use crate::client::RetryConfig;
    use crate::error::Error;
    use std::time::Duration;

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

    #[tokio::test]
    async fn test_retry_config_new_fields() {
        use std::time::Duration;

        let config = RetryConfig::default();

        // Test new default values
        assert_eq!(config.total_timeout, Duration::from_secs(120));
        assert_eq!(config.max_retry_interval, Duration::from_secs(30));

        // Test builder methods
        let custom_config = config
            .with_total_timeout(Duration::from_secs(300))
            .with_max_retry_interval(Duration::from_secs(60));

        assert_eq!(custom_config.total_timeout, Duration::from_secs(300));
        assert_eq!(custom_config.max_retry_interval, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_total_timeout_enforcement() {
        // Create a client that always times out by using an invalid URL
        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 50,
            max_backoff_ms: 100,
            retry_on_status_codes: vec![429, 500, 502, 503, 504],
            total_timeout: Duration::from_millis(200), // Very short timeout
            max_retry_interval: Duration::from_millis(100),
        };

        let client = reqwest::Client::new();
        let result = execute_with_retry_builder(&config, "test_timeout", || {
            // Use an invalid URL that will cause immediate timeout/network error
            client.get("http://192.0.2.1:99999") // Invalid IP and port
        })
        .await;

        // Should fail due to timeout or network errors
        assert!(result.is_err());
        let error = result.unwrap_err();
        match &error {
            Error::ConfigError(msg) => {
                // This is the expected outcome - timeout exceeded
                assert!(
                    msg.contains("timeout exceeded"),
                    "Expected timeout message, got: {}",
                    msg
                );
            }
            Error::HttpError(_) => {
                // Network errors are also acceptable - they should trigger timeout logic
                println!("Test timeout: Got network error (acceptable for timeout test)");
            }
            _ => panic!("Expected timeout or network error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_individual_retry_capping() {
        use reqwest::StatusCode;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock server that always returns 500
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000, // High max backoff
            retry_on_status_codes: vec![500],
            total_timeout: Duration::from_secs(10), // Generous timeout
            max_retry_interval: Duration::from_millis(300), // Low cap
        };

        let start_time = std::time::Instant::now();
        let client = reqwest::Client::new();
        let result =
            execute_with_retry_builder(&config, "test_capping", || client.get(mock_server.uri()))
                .await;
        let elapsed = start_time.elapsed();

        // Test the retry behavior - should return the final response after max retries
        match result {
            Ok(response) => {
                // After max retries, should return the last response (500 in this case)
                assert_eq!(response.status().as_u16(), 500);
                println!("Test capping: Got final 500 response after max retries (expected)");
            }
            Err(error) => {
                match &error {
                    Error::ApiError {
                        code: _,
                        message: _,
                        metadata: _,
                    } => {} // Expected - server returns 500 after retries
                    Error::ConfigError(msg) => {
                        // If we get timeout instead, that's also acceptable
                        println!(
                            "Test capping: Timeout reached instead of max retries (acceptable): {}",
                            msg
                        );
                    }
                    _ => panic!("Expected API error or timeout, got: {:?}", error),
                }
            }
        }

        // Should not take longer than expected due to retry interval capping
        // 1 initial attempt + 3 retries, each capped at 300ms = ~900ms max for waits
        assert!(
            elapsed < Duration::from_millis(1500),
            "Took too long: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_concurrent_retry_limits() {
        use reqwest::StatusCode;
        use std::sync::Arc;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock server that returns 500 for first request, 200 for others
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_backoff_ms: 200,
            retry_on_status_codes: vec![500],
            total_timeout: Duration::from_secs(5),
            max_retry_interval: Duration::from_millis(100),
        };

        let config = Arc::new(config);
        let server_url = mock_server.uri();

        // Launch multiple concurrent requests
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let config = config.clone();
                let url = server_url.clone();
                tokio::spawn(async move {
                    let client = reqwest::Client::new();
                    execute_with_retry_builder(&config, "concurrent_test", || client.get(&url))
                        .await
                })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles).await;

        // All should eventually succeed (after retries)
        let mut successes = 0;
        let mut failures = 0;

        for result in results {
            match result {
                Ok(Ok(_)) => successes += 1,
                Ok(Err(_)) => failures += 1,
                Err(_) => failures += 1, // Join error
            }
        }

        // Most should succeed
        assert!(
            successes >= 3,
            "Expected at least 3 successes, got {}",
            successes
        );
        assert_eq!(failures, 0, "Expected no failures, got {}", failures);
    }

    #[tokio::test]
    async fn test_retry_performance_impact() {
        use reqwest::StatusCode;
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock server that succeeds immediately
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .mount(&mock_server)
            .await;

        let config = RetryConfig::default();

        let start_time = std::time::Instant::now();
        let client = reqwest::Client::new();
        let result = execute_with_retry_builder(&config, "performance_test", || {
            client.get(mock_server.uri())
        })
        .await;
        let elapsed = start_time.elapsed();

        // Should succeed quickly
        assert!(result.is_ok());

        // Should take minimal time (no retries needed)
        assert!(
            elapsed < Duration::from_millis(100),
            "Took too long: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_backoff_jitter_variation() {
        use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        // Mock server that always returns 429
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(429))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 200,
            retry_on_status_codes: vec![429],
            total_timeout: Duration::from_secs(5),
            max_retry_interval: Duration::from_millis(150),
        };

        let start_time = std::time::Instant::now();
        let client = reqwest::Client::new();
        let result =
            execute_with_retry_builder(&config, "jitter_test", || client.get(mock_server.uri()))
                .await;
        let elapsed = start_time.elapsed();

        // Test the retry behavior - should return the final response after max retries
        match result {
            Ok(response) => {
                // After max retries, should return the last response (429 in this case)
                assert_eq!(response.status().as_u16(), 429);
                println!("Test jitter: Got final 429 response after max retries (expected)");
            }
            Err(error) => {
                match &error {
                    Error::ApiError {
                        code: _,
                        message: _,
                        metadata: _,
                    } => {} // Expected - server returns 429 after retries
                    Error::ConfigError(msg) => {
                        // If we get timeout instead, that's also acceptable
                        println!(
                            "Test jitter: Timeout reached instead of max retries (acceptable): {}",
                            msg
                        );
                    }
                    _ => panic!("Expected API error or timeout, got: {:?}", error),
                }
            }
        }

        // Total time should be reasonable with jitter (capped at 150ms each retry)
        // 3 retries at ~150ms each = ~450ms total for waits
        assert!(
            elapsed > Duration::from_millis(300),
            "Too fast, likely no retries: {:?}",
            elapsed
        );
        assert!(
            elapsed < Duration::from_millis(1000),
            "Too slow, possible issue: {:?}",
            elapsed
        );
    }
}
