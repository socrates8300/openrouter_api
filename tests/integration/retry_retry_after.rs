//! Integration tests for Retry-After header handling in retry logic

use openrouter_api::client::{OpenRouterClient, RetryConfig};
use openrouter_api::utils::retry::operations::CHAT_COMPLETION;
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_respects_retry_after_seconds() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Configure the mock to return 429 with Retry-After header
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "2")
                .set_body_json(serde_json::json!({
                    "error": {
                        "message": "Rate limit exceeded",
                        "type": "rate_limit_error"
                    }
                })),
        )
        .up_to_n_times(2) // First two attempts get rate limited
        .mount(&mock_server)
        .await;

    // Configure the mock to succeed on third attempt
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1234567890,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(&mock_server)
        .await;

    // Create retry configuration with reasonable limits
    let retry_config = RetryConfig {
        max_retries: 3,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        retry_on_status_codes: vec![429, 500, 502, 503, 504],
        total_timeout: Duration::from_secs(10),
        max_retry_interval: Duration::from_secs(5),
    };

    // Create client with mock server URL
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .with_retry_config(retry_config)
        .build()
        .await;

    // Record start time
    let start_time = std::time::Instant::now();

    // Make the request - should succeed after retries
    let result = client
        .chat()
        .simple_completion("gpt-3.5-turbo", "Hello")
        .await;

    // Verify request succeeded
    assert!(result.is_ok(), "Request should succeed after retries");

    // Verify timing - should wait at least 2 seconds due to Retry-After header
    let elapsed = start_time.elapsed();
    assert!(
        elapsed >= Duration::from_secs(2),
        "Should have waited at least 2 seconds due to Retry-After header, but waited only {:?}",
        elapsed
    );

    // Should not wait excessively long (more than 4 seconds would indicate exponential backoff was used)
    assert!(
        elapsed < Duration::from_secs(4),
        "Should not wait more than 4 seconds, but waited {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_respects_retry_after_http_date() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Calculate future time for Retry-After header (3 seconds from now)
    let future_time = chrono::Utc::now() + chrono::Duration::seconds(3);
    let retry_after_date = future_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

    // Configure the mock to return 429 with Retry-After header as HTTP date
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", retry_after_date)
                .set_body_json(serde_json::json!({
                    "error": {
                        "message": "Rate limit exceeded",
                        "type": "rate_limit_error"
                    }
                })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Configure the mock to succeed on second attempt
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1234567890,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(&mock_server)
        .await;

    // Create retry configuration
    let retry_config = RetryConfig {
        max_retries: 2,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        retry_on_status_codes: vec![429, 500, 502, 503, 504],
        total_timeout: Duration::from_secs(10),
        max_retry_interval: Duration::from_secs(5),
    };

    // Create client with mock server URL
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .with_retry_config(retry_config)
        .build()
        .await;

    // Record start time
    let start_time = std::time::Instant::now();

    // Make the request
    let result = client
        .chat()
        .simple_completion("gpt-3.5-turbo", "Hello")
        .await;

    // Verify request succeeded
    assert!(result.is_ok(), "Request should succeed after retries");

    // Verify timing - should wait at least 2.5 seconds (allowing for clock skew)
    let elapsed = start_time.elapsed();
    assert!(
        elapsed >= Duration::from_millis(2500),
        "Should have waited at least 2.5 seconds due to Retry-After date, but waited only {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_fallback_to_exponential_backoff_when_no_retry_after() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Configure the mock to return 429 without Retry-After header
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {
                "message": "Rate limit exceeded",
                "type": "rate_limit_error"
            }
        })))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Configure the mock to succeed on second attempt
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1234567890,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(&mock_server)
        .await;

    // Create retry configuration with short initial backoff
    let retry_config = RetryConfig {
        max_retries: 2,
        initial_backoff_ms: 100,
        max_backoff_ms: 5000,
        retry_on_status_codes: vec![429, 500, 502, 503, 504],
        total_timeout: Duration::from_secs(5),
        max_retry_interval: Duration::from_secs(5),
    };

    // Create client with mock server URL
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .with_retry_config(retry_config)
        .build()
        .await;

    // Record start time
    let start_time = std::time::Instant::now();

    // Make the request
    let result = client
        .chat()
        .simple_completion("gpt-3.5-turbo", "Hello")
        .await;

    // Verify request succeeded
    assert!(result.is_ok(), "Request should succeed after retries");

    // Verify timing - should use exponential backoff with jitter (around 100ms +/- 25%)
    let elapsed = start_time.elapsed();
    assert!(
        elapsed >= Duration::from_millis(75) && elapsed <= Duration::from_millis(200),
        "Should have used exponential backoff with jitter, but waited {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_retry_after_capped_at_max_backoff() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Configure the mock to return 429 with very long Retry-After header
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "300") // 5 minutes
                .set_body_json(serde_json::json!({
                    "error": {
                        "message": "Rate limit exceeded",
                        "type": "rate_limit_error"
                    }
                })),
        )
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    // Configure the mock to succeed on second attempt
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-test",
            "object": "chat.completion",
            "created": 1234567890,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(&mock_server)
        .await;

    // Create retry configuration with short max backoff
    let retry_config = RetryConfig {
        max_retries: 2,
        initial_backoff_ms: 100,
        max_backoff_ms: 1000, // 1 second max
        retry_on_status_codes: vec![429, 500, 502, 503, 504],
        total_timeout: Duration::from_secs(10),
        max_retry_interval: Duration::from_secs(2),
    };

    // Create client with mock server URL
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .with_retry_config(retry_config)
        .build()
        .await;

    // Record start time
    let start_time = std::time::Instant::now();

    // Make the request
    let result = client
        .chat()
        .simple_completion("gpt-3.5-turbo", "Hello")
        .await;

    // Verify request succeeded
    assert!(result.is_ok(), "Request should succeed after retries");

    // Verify timing - should be capped at max_backoff (with jitter)
    let elapsed = start_time.elapsed();
    assert!(
        elapsed <= Duration::from_millis(1500), // max_backoff + jitter
        "Retry-After should be capped at max_backoff, but waited {:?}",
        elapsed
    );
}
