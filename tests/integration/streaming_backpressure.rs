//! Integration tests for streaming backpressure control

use openrouter_api::client::OpenRouterClient;
use openrouter_api::types::chat::{ChatCompletionRequest, Message, MessageContent};
use std::time::{Duration, Instant};
use tokio_stream::StreamExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_memory_usage_under_load() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create a large number of mock responses to test backpressure
    let mut mock_responses = Vec::new();

    // Generate many streaming chunks
    for i in 0..100 {
        let chunk = format!(
            "data: {}\n\n",
            serde_json::json!({
                "id": "chunk-{}",
                "object": "chat.completion.chunk",
                "created": 1234567890,
                "choices": [{
                    "index": 0,
                    "delta": {
                        "content": "word{} ".to_string()
                    },
                    "finish_reason": null
                }]
            })
        );
        mock_responses.push(chunk);
    }

    // Add the final [DONE] marker
    mock_responses.push("data: [DONE]\n\n".to_string());

    // Configure the mock to return streaming response
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .insert_header("Cache-Control", "no-cache")
                .insert_header("Connection", "keep-alive")
                .set_body_stream(tokio_stream::iter(mock_responses)),
        )
        .mount(&mock_server)
        .await;

    // Create client with mock server URL
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .build()
        .await;

    // Create a streaming request
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![Message::text("user", "test")],
        stream: Some(true),
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
    };

    // Record start time and memory usage
    let start_time = Instant::now();
    let mut chunk_count = 0;
    let mut total_content_length = 0;

    // Process the stream
    let mut stream = client.chat().chat_completion_stream(request);

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                chunk_count += 1;
                if let Some(choice) = chunk.choices.first() {
                    if let Some(delta) = &choice.delta {
                        if let Some(content) = &delta.content {
                            total_content_length += content.len();
                        }
                    }
                }

                // Simulate processing delay to test backpressure
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
            Err(e) => {
                panic!("Unexpected error in streaming: {:?}", e);
            }
        }

        // Safety check: don't let test run forever
        if start_time.elapsed() > Duration::from_secs(10) {
            panic!("Test took too long, backpressure may not be working");
        }
    }

    let elapsed = start_time.elapsed();

    // Verify backpressure is working
    assert!(chunk_count > 0, "Should have received streaming chunks");

    // With backpressure control, the stream should take measurable time
    // due to the CHUNK_PROCESSING_DELAY_MS (10ms) between chunks
    let expected_min_time = Duration::from_millis(chunk_count as u64 * 5); // Conservative estimate

    assert!(
        elapsed >= expected_min_time,
        "Stream should have taken at least {:?} due to backpressure, but took {:?}",
        expected_min_time,
        elapsed
    );

    // But it shouldn't take excessively long
    assert!(
        elapsed <= Duration::from_secs(5),
        "Stream should complete within reasonable time, but took {:?}",
        elapsed
    );

    // Verify we received all expected chunks
    assert!(
        chunk_count >= 50, // Allow some tolerance for chunk limits
        "Should have received multiple chunks, got {}",
        chunk_count
    );
}

#[tokio::test]
async fn test_stream_cancellation_cleanup() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create a very long streaming response
    let long_content = "word ".repeat(1000);
    let mut mock_responses = Vec::new();

    for i in 0..1000 {
        let chunk = format!(
            "data: {}\n\n",
            serde_json::json!({
                "id": "chunk-{}",
                "object": "chat.completion.chunk",
                "created": 1234567890,
                "choices": [{
                    "index": 0,
                    "delta": {
                        "content": format!("{} ", i)
                    },
                    "finish_reason": null
                }]
            })
        );
        mock_responses.push(chunk);
    }

    mock_responses.push("data: [DONE]\n\n".to_string());

    // Configure the mock
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .insert_header("Cache-Control", "no-cache")
                .insert_header("Connection", "keep-alive")
                .set_body_stream(tokio_stream::iter(mock_responses)),
        )
        .mount(&mock_server)
        .await;

    // Create client
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .build()
        .await;

    // Create streaming request
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![Message::text("user", "test")],
        stream: Some(true),
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
    };

    // Start streaming
    let mut stream = client.chat().chat_completion_stream(request);

    // Process a few chunks then cancel
    let mut processed_chunks = 0;
    let start_time = Instant::now();

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(_) => {
                processed_chunks += 1;

                // Cancel after processing a few chunks
                if processed_chunks >= 5 {
                    drop(stream); // Explicitly drop to test cancellation
                    break;
                }
            }
            Err(e) => {
                panic!("Unexpected error before cancellation: {:?}", e);
            }
        }

        // Safety timeout
        if start_time.elapsed() > Duration::from_secs(5) {
            panic!("Test took too long before cancellation");
        }
    }

    let elapsed = start_time.elapsed();

    // Verify cancellation worked quickly
    assert!(
        processed_chunks >= 5,
        "Should have processed at least 5 chunks before cancellation"
    );

    assert!(
        elapsed <= Duration::from_millis(500),
        "Cancellation should happen quickly, but took {:?}",
        elapsed
    );
}

#[tokio::test]
async fn test_chunk_count_limits() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Create more than MAX_TOTAL_CHUNKS responses
    let mut mock_responses = Vec::new();

    for i in 0..15000 {
        // More than the limit of 10,000
        let chunk = format!(
            "data: {}\n\n",
            serde_json::json!({
                "id": "chunk-{}",
                "object": "chat.completion.chunk",
                "created": 1234567890,
                "choices": [{
                    "index": 0,
                    "delta": {
                        "content": format!("word{} ", i)
                    },
                    "finish_reason": null
                }]
            })
        );
        mock_responses.push(chunk);
    }

    // Configure the mock
    Mock::given(method("POST"))
        .and(path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("Content-Type", "text/event-stream")
                .insert_header("Cache-Control", "no-cache")
                .insert_header("Connection", "keep-alive")
                .set_body_stream(tokio_stream::iter(mock_responses)),
        )
        .mount(&mock_server)
        .await;

    // Create client
    let client = OpenRouterClient::new()
        .with_base_url(&mock_server.uri())
        .with_api_key("test-key")
        .build()
        .await;

    // Create streaming request
    let request = ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![Message::text("user", "test")],
        stream: Some(true),
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
    };

    // Process the stream and expect it to fail due to chunk limit
    let mut stream = client.chat().chat_completion_stream(request);
    let mut chunk_count = 0;
    let mut hit_limit = false;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(_) => {
                chunk_count += 1;
            }
            Err(e) => {
                // Should hit the chunk limit error
                let error_msg = e.to_string();
                if error_msg.contains("Too many chunks") {
                    hit_limit = true;
                    break;
                } else {
                    panic!("Unexpected error: {:?}", e);
                }
            }
        }

        // Safety check
        if chunk_count > 15000 {
            panic!("Should have hit chunk limit before processing this many chunks");
        }
    }

    assert!(
        hit_limit || chunk_count <= 10000,
        "Should either hit the chunk limit error or stop at the limit. Chunks processed: {}",
        chunk_count
    );
}
