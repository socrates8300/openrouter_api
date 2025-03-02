use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::types::chat::{ChatCompletionChunk, ChatCompletionRequest, ChatCompletionResponse};
use async_stream::try_stream;
use futures::stream::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::Client;
use serde_json;
use std::pin::Pin;
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader;

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

        // Issue the POST request with appropriate headers and JSON body.
        let response = self
            .client
            .post(url)
            .headers(self.config.build_headers()?)
            .json(&request)
            .send()
            .await?;

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
        serde_json::from_str::<ChatCompletionResponse>(&body).map_err(|e| Error::ApiError {
            code: status.as_u16(),
            message: format!("Failed to decode JSON: {}. Body was: {}", e, body),
            metadata: None,
        })
    }

    /// Returns a stream for a chat completion request.
    /// Each yielded item is a ChatCompletionChunk.
    pub fn chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk>> + Send>> {
        let client = self.client.clone();
        let config = self.config.clone();

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

            while let Some(line_result) = lines.next().await {
                let line = line_result.map_err(|e| Error::ApiError {
                    code: 500,
                    message: format!("LinesCodec error: {}", e),
                    metadata: None,
                })?;

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
                        Err(err) => {
                            tracing::error!("Stream error: {}", err); 
                            continue;
                        },
                    }
                } else if line.starts_with(":") {
                    // Ignore SSE comment lines.
                    continue;
                }
            }
        };

        Box::pin(stream)
    }
}
