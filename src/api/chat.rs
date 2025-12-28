use crate::error::{Error, Result};
use crate::models::tool::ToolType;
use crate::types::chat::{
    ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse, ChatRole, Message, MessageContent,
};
use crate::utils::{
    retry::execute_with_retry_builder, retry::handle_response_json,
    retry::operations::CHAT_COMPLETION, security::create_safe_error_message, validation,
};
use async_stream::try_stream;
use futures::stream::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::Client;
use serde_json;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader;

// Streaming safety limits to prevent memory exhaustion
const MAX_LINE_LENGTH: usize = 64 * 1024; // 64KB per line
const MAX_TOTAL_CHUNKS: usize = 10_000; // Maximum chunks per stream
const MAX_CONCURRENT_CHUNKS: usize = 10; // Maximum chunks processing concurrently
const CHUNK_PROCESSING_DELAY_MS: u64 = 10; // Delay between chunks for backpressure

/// API endpoint for chat completions.
pub struct ChatApi {
    pub client: Client,
    pub config: crate::client::ApiConfig,
}

impl ChatApi {
    /// Creates a new ChatApi with the given reqwest client and configuration.
    #[must_use = "returns an API client that should be used for chat operations"]
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Sends a chat completion request and returns a complete ChatCompletionResponse.
    #[must_use = "returns the chat completion response that should be processed"]
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        // Validate the request
        validation::validate_chat_request(&request)?;
        validation::check_token_limits(&request)?;

        // Build the complete URL for the chat completions endpoint.
        let url = self
            .config
            .base_url
            .join("chat/completions")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL: {e}"),
                metadata: None,
            })?;

        // Execute request with retry logic
        let response =
            execute_with_retry_builder(&self.config.retry_config, CHAT_COMPLETION, || {
                self.client
                    .post(url.clone())
                    .headers((*self.config.headers).clone())
                    .json(&request)
            })
            .await?;

        // Handle response with consistent error parsing
        let chat_response: ChatCompletionResponse =
            handle_response_json::<ChatCompletionResponse>(response, CHAT_COMPLETION).await?;

        // Validate any tool calls in the response
        for choice in &chat_response.choices {
            if let Some(tool_calls) = &choice.message.tool_calls {
                for tc in tool_calls {
                    if tc.kind != ToolType::Function {
                        return Err(Error::SchemaValidationError(format!(
                            "Invalid tool call kind: {}. Expected 'function'",
                            tc.kind
                        )));
                    }
                }
            }
        }

        Ok(chat_response)
    }

    /// Returns a stream for a chat completion request.
    /// Each yielded item is a ChatCompletionChunk.
    #[must_use = "returns a stream that should be consumed to receive completion chunks"]
    pub fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send + '_>> {
        let client = self.client.clone();
        let headers = Arc::clone(&self.config.headers);

        // Validate the request before streaming
        if let Err(e) = validation::validate_chat_request(&request) {
            return Box::pin(futures::stream::once(async { Err(e) }));
        }

        if let Err(e) = validation::check_token_limits(&request) {
            return Box::pin(futures::stream::once(async { Err(e) }));
        }

        // Set up backpressure control
        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_CHUNKS));
        let chunk_count = Arc::new(AtomicUsize::new(0));

        // Build the URL for the chat completions endpoint.
        let url = match self.config.base_url.join("chat/completions") {
            Ok(url) => url,
            Err(e) => {
                return Box::pin(futures::stream::once(async move {
                    Err(Error::ApiError {
                        code: 400,
                        message: format!("Invalid URL: {e}"),
                        metadata: None,
                    })
                }));
            }
        };

        // Serialize the request with streaming enabled.
        let mut req_body = match serde_json::to_value(&request) {
            Ok(body) => body,
            Err(e) => {
                return Box::pin(futures::stream::once(async move {
                    Err(Error::ApiError {
                        code: 500,
                        message: format!("Request serialization error: {e}"),
                        metadata: None,
                    })
                }));
            }
        };
        req_body["stream"] = serde_json::Value::Bool(true);

        // ApiConfig already contains headers

        let stream = try_stream! {
            // Issue the POST request
            let response = client
                .post(url)
                .headers((*headers).clone())
                .json(&req_body)
                .send()
                .await
                .map_err(|e| {
                    Error::ApiError {
                        code: 500,
                        message: format!("Request failed: {e}"),
                        metadata: None,
                    }
                })?;

            let response = response.error_for_status().map_err(|e| {
                Error::ApiError {
                    code: e.status().map(|s| s.as_u16()).unwrap_or(500),
                    message: e.to_string(),
                    metadata: None,
                }
            })?;

            // Process the bytes stream as an asynchronous line stream.
            let byte_stream = response.bytes_stream().map_err(std::io::Error::other);
            let stream_reader = StreamReader::new(byte_stream);
            let mut lines = FramedRead::new(stream_reader, LinesCodec::new());

            while let Some(line_result) = lines.next().await {
                let line = line_result.map_err(|e| Error::StreamingError(format!("Failed to read stream line: {e}")))?;

                // Safety check: Line length limit
                if line.len() > MAX_LINE_LENGTH {
                    Err(Error::StreamingError(format!(
                        "Line too long: {} bytes (max: {})",
                        line.len(),
                        MAX_LINE_LENGTH
                    )))?;
                }

                // Safety check: Chunk count limit
                let current_chunk = chunk_count.fetch_add(1, Ordering::Relaxed) + 1;
                if current_chunk > MAX_TOTAL_CHUNKS {
                      Err(Error::StreamingError(format!(
                        "Too many chunks: {current_chunk} (max: {MAX_TOTAL_CHUNKS})"
                    )))?;
                }

                // Apply backpressure - acquire semaphore permit
                let _permit = semaphore.acquire().await
                    .map_err(|_| Error::StreamingError("Failed to acquire backpressure permit".to_string()))?;

                // Add small delay for backpressure control
                tokio::time::sleep(Duration::from_millis(CHUNK_PROCESSING_DELAY_MS)).await;

                if line.trim().is_empty() {
                    continue;
                }

                if line.starts_with("data:") {
                    let data_part = line.trim_start_matches("data:").trim();
                    if data_part == "[DONE]" {
                        break;
                    }

                    match serde_json::from_str::<ChatCompletionChunk>(data_part) {
                        Ok(chunk) => {
                            // Permit is automatically released when _permit goes out of scope
                            yield chunk;
                        },
                        Err(e) => {
                            let error_msg = create_safe_error_message(
                                &format!("Failed to parse streaming chunk: {e}. Data: {data_part}"),
                                "Streaming chunk parse error"
                            );

                            // Use tracing if available, otherwise fall back to eprintln
                            #[cfg(feature = "tracing")]
                            tracing::error!("Streaming parse error: {}", error_msg);

                            #[cfg(not(feature = "tracing"))]
                            eprintln!("Streaming parse error: {}", error_msg);

                            continue;
                        }
                    }
                } else if line.starts_with(":") {
                    // Ignore SSE comment lines.
                    continue;
                } else {
                    // Try to parse as a regular JSON message (non-SSE format)
                    match serde_json::from_str::<ChatCompletionChunk>(&line) {
                        Ok(chunk) => {
                            // Permit is automatically released when _permit goes out of scope
                            yield chunk;
                        },
                        Err(_) => continue,
                    }
                }
            }
        };

        Box::pin(stream)
    }

    /// Simple function to complete a chat with a single user message
    pub async fn simple_completion(&self, model: &str, user_message: &str) -> Result<String> {
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages: vec![Message::text(ChatRole::User, user_message)],
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
            plugins: None,
        };

        let response = self.chat_completion(request).await?;

        match &response.choices[0].message.content {
            MessageContent::Text(content) => Ok(content.clone()),
            MessageContent::Parts(_) => Err(Error::ConfigError(
                "Unexpected multimodal content in simple completion response".into(),
            )),
        }
    }
}
