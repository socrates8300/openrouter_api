use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::chat::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse};
use crate::utils::{security::create_safe_error_message, validation};
use async_stream::try_stream;
use futures::stream::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::Client;
use serde_json;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader;

// Streaming safety limits to prevent memory exhaustion
const MAX_LINE_LENGTH: usize = 64 * 1024; // 64KB per line
const MAX_TOTAL_CHUNKS: usize = 10_000; // Maximum chunks per stream

pub struct ChatApi {
    pub client: Client,
    pub config: ClientConfig,
}

impl ChatApi {
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
    }

    /// Sends a chat completion request and returns a complete ChatCompletionResponse.
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
                message: format!("Invalid URL: {}", e),
                metadata: None,
            })?;

        // Initialize retry counter and backoff duration
        let mut retry_count = 0;
        let mut backoff_ms = self.config.retry_config.initial_backoff_ms;

        let response = loop {
            // Issue the POST request with appropriate headers and JSON body.
            let response = self
                .client
                .post(url.clone())
                .headers(self.config.build_headers()?)
                .json(&request)
                .send()
                .await?;

            let status = response.status();

            // Check if we should retry based on status code
            if self
                .config
                .retry_config
                .retry_on_status_codes
                .contains(&status.as_u16())
                && retry_count < self.config.retry_config.max_retries
            {
                // Increment retry counter and exponential backoff
                retry_count += 1;

                // Log retry attempt
                eprintln!(
                    "Retrying request ({}/{}) after {} ms due to status code {}",
                    retry_count,
                    self.config.retry_config.max_retries,
                    backoff_ms,
                    status.as_u16()
                );

                // Wait before retrying
                sleep(Duration::from_millis(backoff_ms)).await;

                // Calculate next backoff with exponential increase
                backoff_ms = std::cmp::min(backoff_ms * 2, self.config.retry_config.max_backoff_ms);

                continue;
            }

            break response;
        };

        // Capture the HTTP status.
        let status = response.status();

        // Retrieve the response body.
        let body = response.text().await?;

        // Check if the HTTP response is successful.
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: body.clone(),
                metadata: None,
            });
        }

        if body.trim().is_empty() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: "Empty response body".into(),
                metadata: None,
            });
        }

        // Deserialize the JSON response into ChatCompletionResponse.
        let chat_response =
            serde_json::from_str::<ChatCompletionResponse>(&body).map_err(|e| Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(
                    &format!("Failed to decode JSON: {}. Body was: {}", e, body),
                    "Chat completion JSON parsing error",
                ),
                metadata: None,
            })?;

        // Validate any tool calls in the response
        for choice in &chat_response.choices {
            if let Some(tool_calls) = &choice.message.tool_calls {
                for tc in tool_calls {
                    if tc.kind != "function" {
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
    pub fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>> {
        let client = self.client.clone();
        let config = self.config.clone();

        // Validate the request before streaming
        if let Err(e) = validation::validate_chat_request(&request) {
            return Box::pin(futures::stream::once(async { Err(e) }));
        }

        if let Err(e) = validation::check_token_limits(&request) {
            return Box::pin(futures::stream::once(async { Err(e) }));
        }

        let stream = try_stream! {
            // Build the URL for the chat completions endpoint.
            let url = config.base_url.join("chat/completions").map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL: {}", e),
                metadata: None,
            })?;

            // Serialize the request with streaming enabled.
            let mut req_body = serde_json::to_value(&request).map_err(|e| Error::ApiError {
                code: 500,
                message: format!("Request serialization error: {}", e),
                metadata: None,
            })?;
            req_body["stream"] = serde_json::Value::Bool(true);

            // Issue the POST request with error-for-status checking.
            let response = client
                .post(url)
                .headers(config.build_headers()?)
                .json(&req_body)
                .send()
                .await?
                .error_for_status()
                .map_err(|e| {
                    Error::ApiError {
                        code: e.status().map(|s| s.as_u16()).unwrap_or(500),
                        message: e.to_string(),
                        metadata: None,
                    }
                })?;

            // Process the bytes stream as an asynchronous line stream.
            let byte_stream = response.bytes_stream().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
            let stream_reader = StreamReader::new(byte_stream);
            let mut lines = FramedRead::new(stream_reader, LinesCodec::new());

            // Track chunk count for safety
            let mut chunk_count = 0usize;

            while let Some(line_result) = lines.next().await {
                let line = line_result.map_err(|e| Error::StreamingError(format!("Failed to read stream line: {}", e)))?;

                // Safety check: Line length limit
                if line.len() > MAX_LINE_LENGTH {
                    Err(Error::StreamingError(format!(
                        "Line too long: {} bytes (max: {})",
                        line.len(),
                        MAX_LINE_LENGTH
                    )))?;
                }

                // Safety check: Chunk count limit
                chunk_count += 1;
                if chunk_count > MAX_TOTAL_CHUNKS {
                    Err(Error::StreamingError(format!(
                        "Too many chunks: {} (max: {})",
                        chunk_count,
                        MAX_TOTAL_CHUNKS
                    )))?;
                }

                if line.trim().is_empty() {
                    continue;
                }

                if line.starts_with("data:") {
                    let data_part = line.trim_start_matches("data:").trim();
                    if data_part == "[DONE]" {
                        break;
                    }

                    match serde_json::from_str::<ChatCompletionChunk>(data_part) {
                        Ok(chunk) => yield chunk,
                        Err(e) => {
                            let error_msg = create_safe_error_message(
                                &format!("Failed to parse streaming chunk: {}. Data: {}", e, data_part),
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
                        Ok(chunk) => yield chunk,
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
            messages: vec![crate::types::chat::Message {
                role: "user".to_string(),
                content: user_message.to_string(),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }],
            stream: None,
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
        };

        let response = self.chat_completion(request).await?;

        if response.choices.is_empty() {
            return Err(Error::ApiError {
                code: 500,
                message: "No choices returned in response".into(),
                metadata: None,
            });
        }

        Ok(response.choices[0].message.content.clone())
    }
}
