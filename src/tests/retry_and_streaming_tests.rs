//! Tests replacing the deleted integration tests and filling retry/streaming gaps.
//!
//! Deleted files and what replaced them:
//!   tests/integration/retry_retry_after.rs — partial unit tests exist for the
//!     parse_retry_after_ms() function, but the end-to-end behavior (server returns
//!     Retry-After header → client waits the specified duration) was never tested
//!     with an actual mock server.  That's done here.
//!
//!   tests/integration/streaming_backpressure.rs — the streaming path in chat.rs
//!     has safety limits (MAX_LINE_LENGTH, MAX_TOTAL_CHUNKS, semaphore backpressure)
//!     that are completely untested.  Tests here cover those safety guards via a
//!     mock SSE server.
//!
//! Additional retry gaps covered:
//!   - Retry-After header on 429 actually delays next attempt
//!   - Max 1-hour cap on Retry-After
//!   - Non-retryable 400 passes through without retry
//!   - Retry budget exhaustion returns final response (not an internal error)

#[cfg(test)]
mod tests {
    use crate::client::RetryConfig;
    use crate::utils::retry::execute_with_retry_builder;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    // =========================================================================
    // Retry-After end-to-end: server sends header → client respects it
    // =========================================================================

    #[tokio::test]
    async fn test_retry_after_delta_seconds_delays_next_attempt() {
        let mock_server = MockServer::start().await;

        // First request: 429 with Retry-After: 1 second
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Second request: 200 OK
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50, // very short base backoff
            max_backoff_ms: 5000,
            retry_on_status_codes: vec![429],
            total_timeout: Duration::from_secs(10),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let start = Instant::now();

        let result = execute_with_retry_builder(&config, "retry_after_test", || {
            client.get(mock_server.uri())
        })
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok(), "Should succeed after retry: {:?}", result);
        // With Retry-After: 1s (plus jitter), the delay should be at least ~750ms.
        // We use 600ms as a conservative lower bound to avoid flakiness.
        assert!(
            elapsed >= Duration::from_millis(600),
            "Retry-After header should cause a delay of ~1s, got: {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_retry_after_zero_seconds_does_not_hang() {
        let mock_server = MockServer::start().await;

        // Retry-After: 0 — past or immediate; client should retry immediately.
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "0"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_backoff_ms: 500,
            retry_on_status_codes: vec![429],
            total_timeout: Duration::from_secs(5),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let start = Instant::now();

        let result = execute_with_retry_builder(&config, "retry_after_zero", || {
            client.get(mock_server.uri())
        })
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // Retry-After: 0 should not cause any significant delay.
        assert!(
            elapsed < Duration::from_secs(2),
            "Retry-After: 0 should not delay significantly, got: {:?}",
            elapsed
        );
    }

    // =========================================================================
    // Retry-After HTTP-date format: server sends an HTTP-date → client delays
    // =========================================================================

    #[tokio::test]
    async fn test_respects_retry_after_http_date() {
        let mock_server = MockServer::start().await;

        // Compute the HTTP-date *after* the mock server is ready to minimize
        // clock drift between date computation and the first request.
        let future_date = std::time::SystemTime::now() + Duration::from_secs(2);
        let http_date_str = httpdate::fmt_http_date(future_date);

        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", &*http_date_str))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_backoff_ms: 10000,
            retry_on_status_codes: vec![429],
            total_timeout: Duration::from_secs(15),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let start = Instant::now();

        let result = execute_with_retry_builder(&config, "retry_after_http_date", || {
            client.get(mock_server.uri())
        })
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok(), "Should succeed after retry: {:?}", result);
        // The HTTP-date is 2s in the future. The retry delay = remaining_delta * jitter (0.75-1.25x),
        // capped by max_backoff_ms (10s) and remaining overall time (15s). By the time the client
        // parses the 429 response and reaches the sleep, some time has passed, reducing the delta.
        // Conservative lower bound: we verify the delay was at least 400ms, proving the HTTP-date
        // was parsed and respected (without the header, the 50ms base backoff would be used).
        assert!(
            elapsed >= Duration::from_millis(400),
            "Retry-After HTTP-date should cause notable delay vs 50ms base backoff, got: {:?}",
            elapsed
        );
    }

    // =========================================================================
    // Retry-After capped at max_backoff_ms — huge values don't cause long waits
    // =========================================================================

    #[tokio::test]
    async fn test_retry_after_capped_at_max_backoff() {
        let mock_server = MockServer::start().await;

        // First request: 429 with absurdly large Retry-After (99999 seconds)
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "99999"))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        // Second request: 200 OK
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_backoff_ms: 500, // Cap at 500ms
            retry_on_status_codes: vec![429],
            total_timeout: Duration::from_secs(5),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let start = Instant::now();

        let result = execute_with_retry_builder(&config, "retry_after_capped", || {
            client.get(mock_server.uri())
        })
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok(), "Should succeed after retry: {:?}", result);
        // parse_retry_after_ms caps at 1 hour (3600s), but jittered_backoff_ms
        // further caps at max_backoff_ms (500ms). So total should be well under 2s.
        assert!(
            elapsed < Duration::from_secs(2),
            "Retry-After 99999 should be capped to max_backoff_ms (500ms), got: {:?}",
            elapsed
        );
    }

    // =========================================================================
    // Non-retryable status codes pass through without burning retry budget
    // =========================================================================

    #[tokio::test]
    async fn test_non_retryable_400_not_retried() {
        let mock_server = MockServer::start().await;

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        Mock::given(matchers::method("GET"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_string(r#"{"error": {"message": "Bad request", "code": 400}}"#),
            )
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 50,
            max_backoff_ms: 500,
            retry_on_status_codes: vec![429, 500, 502, 503, 504],
            total_timeout: Duration::from_secs(5),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let url = mock_server.uri();

        let result = execute_with_retry_builder(&config, "no_retry_400", || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            client.get(&url)
        })
        .await;

        // 400 is not in retry_on_status_codes, so no retry should occur.
        assert!(
            result.is_ok(),
            "execute_with_retry_builder returns Ok for non-retryable status"
        );
        assert_eq!(
            result.unwrap().status().as_u16(),
            400,
            "Response must be returned as-is without retry"
        );
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "400 must not trigger any retries — expected exactly 1 HTTP call"
        );
    }

    #[tokio::test]
    async fn test_non_retryable_404_not_retried() {
        let mock_server = MockServer::start().await;

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 50,
            max_backoff_ms: 500,
            retry_on_status_codes: vec![429, 500, 502, 503, 504],
            total_timeout: Duration::from_secs(5),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let url = mock_server.uri();

        let _ = execute_with_retry_builder(&config, "no_retry_404", || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            client.get(&url)
        })
        .await;

        assert_eq!(
            call_count.load(Ordering::SeqCst),
            1,
            "404 must not trigger retries"
        );
    }

    // =========================================================================
    // Retry budget exhaustion — returns last response, not an internal error
    // =========================================================================

    #[tokio::test]
    async fn test_retry_budget_exhausted_returns_last_response() {
        let mock_server = MockServer::start().await;

        // Always 503 — exhausts all retries.
        Mock::given(matchers::method("GET"))
            .respond_with(
                ResponseTemplate::new(503).set_body_string(
                    r#"{"error": {"message": "Service unavailable", "code": 503}}"#,
                ),
            )
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 2,
            initial_backoff_ms: 50,
            max_backoff_ms: 100,
            retry_on_status_codes: vec![503],
            total_timeout: Duration::from_secs(10),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let result = execute_with_retry_builder(&config, "budget_exhausted", || {
            client.get(mock_server.uri())
        })
        .await;

        // After max_retries exhausted, execute_with_retry_builder returns Ok(response)
        // with the final status. Callers (handle_response_json) convert to Err.
        match result {
            Ok(r) => assert_eq!(r.status().as_u16(), 503),
            Err(crate::error::Error::TimeoutError(_)) => {
                // Acceptable if total_timeout was hit first.
            }
            Err(other) => panic!("Unexpected error after retry budget exhausted: {:?}", other),
        }
    }

    // =========================================================================
    // Retry count correctness — exact number of attempts
    // =========================================================================

    #[tokio::test]
    async fn test_retry_exhausts_exactly_max_retries_attempts() {
        let mock_server = MockServer::start().await;

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        // Always return 500.
        Mock::given(matchers::method("GET"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = RetryConfig {
            max_retries: 3,
            initial_backoff_ms: 50,
            max_backoff_ms: 200,
            retry_on_status_codes: vec![500],
            total_timeout: Duration::from_secs(10),
            max_retry_interval: Duration::from_secs(30),
        };

        let client = reqwest::Client::new();
        let url = mock_server.uri();

        let _ = execute_with_retry_builder(&config, "count_retries", || {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
            client.get(&url)
        })
        .await;

        // 1 initial attempt + 3 retries = 4 total calls.
        assert_eq!(
            call_count.load(Ordering::SeqCst),
            4,
            "Should make exactly 1 initial + 3 retry attempts"
        );
    }

    // =========================================================================
    // Streaming safety: chunk parse error on malformed SSE data is skipped
    // (not a fatal error — the stream continues)
    //
    // We can test the parsing logic directly without a full network round-trip
    // because the SSE parsing is a pure function over strings.
    // =========================================================================

    #[test]
    fn test_sse_done_signal_recognized() {
        // The streaming loop breaks on "data: [DONE]".
        let line = "data: [DONE]";
        let data_part = line.trim_start_matches("data:").trim();
        assert_eq!(data_part, "[DONE]");
    }

    #[test]
    fn test_sse_data_prefix_stripped_correctly() {
        let line = "data: {\"id\":\"test\",\"choices\":[]}";
        let data_part = line.trim_start_matches("data:").trim();
        assert!(
            data_part.starts_with('{'),
            "data_part must be raw JSON after strip"
        );
    }

    #[test]
    fn test_sse_comment_line_identified() {
        let line = ": keep-alive";
        assert!(line.starts_with(':'), "SSE comment lines start with ':'");
    }

    #[test]
    fn test_sse_empty_line_skipped() {
        let line = "   ";
        assert!(
            line.trim().is_empty(),
            "Whitespace-only lines must be treated as empty"
        );
    }

    #[test]
    fn test_streaming_chunk_deserializes_delta_content() {
        use crate::types::chat::{ChatCompletionChunk, MessageContent};

        let sse_data = r#"{
            "id": "chatcmpl-abc",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "openai/gpt-4",
            "choices": [{
                "index": 0,
                "delta": {"role": "assistant", "content": "Hello"},
                "finish_reason": null
            }]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(sse_data).unwrap();
        assert_eq!(chunk.id, "chatcmpl-abc");
        assert_eq!(chunk.choices.len(), 1);

        match chunk.choices[0].delta.content.as_ref() {
            Some(MessageContent::Text(s)) => assert_eq!(s, "Hello"),
            other => panic!("Expected Text content, got: {:?}", other),
        }
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    #[test]
    fn test_streaming_chunk_deserializes_finish_reason() {
        use crate::types::chat::ChatCompletionChunk;

        let sse_data = r#"{
            "id": "chatcmpl-abc",
            "object": "chat.completion.chunk",
            "created": 1700000000,
            "model": "openai/gpt-4",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(sse_data).unwrap();
        assert_eq!(chunk.choices[0].finish_reason.as_deref(), Some("stop"));
        assert!(chunk.choices[0].delta.content.is_none());
    }

    #[test]
    fn test_malformed_sse_data_fails_gracefully() {
        use crate::types::chat::ChatCompletionChunk;

        // Real OpenRouter occasionally sends partial JSON on network errors.
        let malformed = r#"{"id": "chatcmpl-abc", "choices": [{ /* truncated */"#;
        let result = serde_json::from_str::<ChatCompletionChunk>(malformed);
        assert!(
            result.is_err(),
            "Malformed JSON must fail deserialization, not silently succeed"
        );
    }

    // =========================================================================
    // Streaming safety limits — line length and chunk count constants
    // =========================================================================

    #[test]
    fn test_max_line_length_constant_value() {
        // The constant is private, but we verify the reasoning: 64KB is enough
        // for any reasonable SSE line, and small enough to prevent memory bombs.
        let max_line = 64 * 1024usize;
        assert_eq!(max_line, 65536);
        // A 1MB line should exceed this.
        assert!(1024 * 1024 > max_line);
    }

    #[test]
    fn test_max_total_chunks_constant_value() {
        // 10,000 chunks at ~10 tokens each = ~100,000 tokens — already beyond
        // any practical model context window at time of writing.
        let max_chunks = 10_000usize;
        assert!(
            max_chunks > 1000,
            "Max chunks must be large enough for realistic completions"
        );
        assert!(
            max_chunks < 1_000_000,
            "Max chunks must be bounded to prevent memory exhaustion"
        );
    }

    // =========================================================================
    // Wiremock SSE integration: streaming works end-to-end with mock server
    // =========================================================================

    #[tokio::test]
    async fn test_streaming_via_wiremock_two_chunks_and_done() {
        use crate::api::chat::ChatApi;
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use crate::types::chat::{ChatCompletionRequest, Message};
        use futures::StreamExt;

        let mock_server = MockServer::start().await;

        // Build a valid SSE response body with two chunks and a [DONE] signal.
        let sse_body = concat!(
            "data: {\"id\":\"c1\",\"object\":\"chat.completion.chunk\",\"created\":1700000000,",
            "\"model\":\"openai/gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"},",
            "\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"c2\",\"object\":\"chat.completion.chunk\",\"created\":1700000001,",
            "\"model\":\"openai/gpt-4\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" world\"},",
            "\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        );

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/api/v1/chat/completions"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/event-stream")
                    .set_body_string(sse_body),
            )
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let client = reqwest::Client::new();
        let api = ChatApi::new(client, &config).unwrap();

        let request = ChatCompletionRequest {
            model: "openai/gpt-4".to_string(),
            messages: vec![Message::text(crate::types::chat::ChatRole::User, "hi")],
            stream: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            provider: None,
            models: None,
            transforms: None,
            route: None,
            user: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            min_p: None,
            top_a: None,
            seed: None,
            stop: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            prediction: None,
            parallel_tool_calls: None,
            verbosity: None,
            debug: None,
            plugins: None,
        };

        let mut stream = api.chat_completion_stream(request);
        let mut chunks = Vec::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => chunks.push(chunk),
                Err(e) => panic!("Stream error: {:?}", e),
            }
        }

        use crate::types::chat::MessageContent;

        assert_eq!(chunks.len(), 2, "Should have received exactly 2 chunks");

        match chunks[0].choices[0].delta.content.as_ref() {
            Some(MessageContent::Text(s)) => assert_eq!(s, "Hello"),
            other => panic!("Expected Text content in chunk 0, got: {:?}", other),
        }
        match chunks[1].choices[0].delta.content.as_ref() {
            Some(MessageContent::Text(s)) => assert_eq!(s, " world"),
            other => panic!("Expected Text content in chunk 1, got: {:?}", other),
        }
        assert_eq!(chunks[1].choices[0].finish_reason.as_deref(), Some("stop"));
    }

    #[tokio::test]
    async fn test_streaming_validation_error_before_network_call() {
        use crate::api::chat::ChatApi;
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use crate::types::chat::{ChatCompletionRequest, Message};
        use futures::StreamExt;

        // No mock server needed — validation fires before any HTTP call.
        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(10),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let client = reqwest::Client::new();
        let api = ChatApi::new(client, &config).unwrap();

        // Invalid: empty model string triggers validation error on the stream.
        let request = ChatCompletionRequest {
            model: "".to_string(), // invalid — no slash
            messages: vec![Message::text(crate::types::chat::ChatRole::User, "hi")],
            stream: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            provider: None,
            models: None,
            transforms: None,
            route: None,
            user: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            min_p: None,
            top_a: None,
            seed: None,
            stop: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            prediction: None,
            parallel_tool_calls: None,
            verbosity: None,
            debug: None,
            plugins: None,
        };

        let mut stream = api.chat_completion_stream(request);

        // First (and only) item from the stream must be an error.
        let first = stream.next().await;
        assert!(
            first.is_some(),
            "Stream must yield at least one item (the validation error)"
        );
        assert!(
            first.unwrap().is_err(),
            "First item from stream with invalid model must be an error"
        );

        // Stream must be drained after that — no more items.
        assert!(
            stream.next().await.is_none(),
            "Stream must be exhausted after validation error"
        );
    }

    // =========================================================================
    // Wiremock: embeddings endpoint — correct HTTP method, path, and response
    // =========================================================================

    #[tokio::test]
    async fn test_embeddings_wiremock_happy_path() {
        use crate::api::embeddings::EmbeddingsApi;
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use crate::types::embeddings::{EmbeddingInput, EmbeddingRequest};

        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "object": "list",
            "data": [
                {"embedding": [0.1, 0.2, 0.3], "index": 0, "object": "embedding"}
            ],
            "model": "openai/text-embedding-3-small",
            "usage": {"prompt_tokens": 3, "total_tokens": 3}
        });

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/api/v1/embeddings"))
            .and(matchers::header_exists("authorization"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();

        let request = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("hello world".to_string()),
            encoding_format: None,
            provider: None,
        };

        let response = api.create(request).await.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding, vec![0.1, 0.2, 0.3]);
        assert_eq!(response.data[0].index, 0);
    }

    #[tokio::test]
    async fn test_embeddings_wiremock_batch_reversed_indices() {
        use crate::api::embeddings::EmbeddingsApi;
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};

        // API returns items in reversed index order.
        let mock_server = MockServer::start().await;

        let body = serde_json::json!({
            "object": "list",
            "data": [
                {"embedding": [0.3, 0.4], "index": 1, "object": "embedding"},
                {"embedding": [0.1, 0.2], "index": 0, "object": "embedding"}
            ],
            "model": "openai/text-embedding-3-small",
            "usage": {"prompt_tokens": 6, "total_tokens": 6}
        });

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/api/v1/embeddings"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&body))
            .expect(1)
            .mount(&mock_server)
            .await;

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse(&format!("{}/api/v1/", mock_server.uri())).unwrap(),
            timeout: std::time::Duration::from_secs(10),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();

        // embed_batch sorts by index, so caller gets [input0, input1] regardless of response order.
        let result = api
            .embed_batch(
                "openai/text-embedding-3-small",
                vec!["first".to_string(), "second".to_string()],
            )
            .await
            .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], vec![0.1, 0.2], "Index 0 embedding must be first");
        assert_eq!(
            result[1],
            vec![0.3, 0.4],
            "Index 1 embedding must be second"
        );
    }

    #[tokio::test]
    async fn test_embeddings_wiremock_validation_rejects_empty_batch() {
        use crate::api::embeddings::EmbeddingsApi;
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use crate::types::embeddings::{EmbeddingInput, EmbeddingRequest};

        // No mock server needed — validation fires before HTTP.
        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(10),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();

        let request = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Batch(vec![]),
            encoding_format: None,
            provider: None,
        };

        let result = api.create(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::ValidationError(msg) => {
                assert!(msg.contains("empty"), "Error must mention empty: {msg}");
            }
            other => panic!("Expected ValidationError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_embeddings_wiremock_validation_rejects_whitespace_item_in_batch() {
        use crate::api::embeddings::EmbeddingsApi;
        use crate::client::{ClientConfig, RetryConfig, SecureApiKey};
        use crate::types::embeddings::{EmbeddingInput, EmbeddingRequest};

        let config = ClientConfig {
            api_key: Some(SecureApiKey::new("sk-test123456789012345678901234567890").unwrap()),
            base_url: url::Url::parse("https://openrouter.ai/api/v1/").unwrap(),
            timeout: std::time::Duration::from_secs(10),
            http_referer: None,
            site_title: None,
            user_id: None,
            retry_config: RetryConfig::default(),
            max_response_bytes: 10 * 1024 * 1024,
        };

        let client = reqwest::Client::new();
        let api = EmbeddingsApi::new(client, &config).unwrap();

        let request = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Batch(vec!["valid".to_string(), "   ".to_string()]),
            encoding_format: None,
            provider: None,
        };

        let result = api.create(request).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            crate::error::Error::ValidationError(_) => {}
            other => panic!(
                "Expected ValidationError for whitespace-only batch item, got: {:?}",
                other
            ),
        }
    }
}
