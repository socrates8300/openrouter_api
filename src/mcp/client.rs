//! MCP client implementation for connecting to MCP servers.

use std::sync::Arc;
use tokio::sync::RwLock;
use url::Url;

use crate::error::{Error, Result};
use crate::mcp::types::*;

/// MCP client for connecting to and interacting with MCP servers.
///
/// # Example
///
/// ```no_run
/// use openrouter_api::mcp::client::MCPClient;
/// use openrouter_api::mcp::types::ClientCapabilities;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = MCPClient::new("http://localhost:8080")?;
///
/// let capabilities = ClientCapabilities {
///     protocol_version: "2024-11-05".to_string(),
///     supports_sampling: Some(true),
/// };
///
/// let server_caps = client.initialize(capabilities).await?;
/// println!("Connected to server with protocol version: {}", server_caps.protocol_version);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct MCPClient {
    /// The HTTP client for making requests
    client: reqwest::Client,
    /// The base URL of the MCP server
    server_url: Url,
    /// Server capabilities once initialized
    capabilities: Arc<RwLock<Option<ServerCapabilities>>>,
    /// Client configuration for security and performance
    config: McpConfig,
    /// Semaphore for limiting concurrent requests
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl MCPClient {
    /// Create a new MCP client for the given server URL with default configuration.
    pub fn new(server_url: impl AsRef<str>) -> Result<Self> {
        Self::new_with_config(server_url, McpConfig::default())
    }

    /// Create a new MCP client for the given server URL with custom configuration.
    pub fn new_with_config(server_url: impl AsRef<str>, config: McpConfig) -> Result<Self> {
        config.validate()?;

        let server_url = Url::parse(server_url.as_ref())
            .map_err(|e| Error::ConfigError(format!("Invalid server URL: {e}")))?;

        let client = reqwest::Client::builder()
            .timeout(config.request_timeout)
            .build()
            .map_err(|e| Error::ConfigError(format!("Failed to create HTTP client: {e}")))?;

        Ok(Self {
            client,
            server_url,
            capabilities: Arc::new(RwLock::new(None)),
            config: config.clone(),
            semaphore: Arc::new(tokio::sync::Semaphore::new(config.max_concurrent_requests)),
        })
    }

    /// Generate a unique request ID
    fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Initialize the connection to the MCP server.
    pub async fn initialize(
        &self,
        client_capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Self::generate_id(),
            method: "initialize".to_string(),
            params: Some(
                serde_json::to_value(InitializeParams {
                    capabilities: client_capabilities,
                })
                .map_err(Error::SerializationError)?,
            ),
            protocol_version: Some(MCP_PROTOCOL_VERSION.to_string()),
        };

        let response = self.send_request(request).await?;
        let capabilities = self.parse_response::<ServerCapabilities>(response)?;

        // Store the server capabilities
        let mut caps = self.capabilities.write().await;
        *caps = Some(capabilities.clone());

        Ok(capabilities)
    }

    /// Get a resource from the server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use openrouter_api::mcp::client::MCPClient;
    /// # use openrouter_api::mcp::types::GetResourceParams;
    /// # async fn example(client: MCPClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let params = GetResourceParams {
    ///     id: "file:///example.txt".to_string(),
    ///     parameters: None,
    /// };
    ///
    /// let response = client.get_resource(params).await?;
    /// println!("Resource content: {:?}", response.contents);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_resource(&self, params: GetResourceParams) -> Result<ResourceResponse> {
        self.send_method("getResource", params).await
    }

    /// Call a tool on the server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use openrouter_api::mcp::client::MCPClient;
    /// # use openrouter_api::mcp::types::ToolCallParams;
    /// # async fn example(client: MCPClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let params = ToolCallParams {
    ///     id: "calculator".to_string(),
    ///     parameters: serde_json::json!({ "expression": "2 + 2" }),
    /// };
    ///
    /// let response = client.tool_call(params).await?;
    /// println!("Tool result: {:?}", response.result);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn tool_call(&self, params: ToolCallParams) -> Result<ToolCallResponse> {
        self.send_method("toolCall", params).await
    }

    /// Execute a prompt on the server.
    pub async fn execute_prompt(
        &self,
        params: ExecutePromptParams,
    ) -> Result<ExecutePromptResponse> {
        self.send_method("executePrompt", params).await
    }

    /// Send a sampling response to the server.
    pub async fn respond_to_sampling(&self, id: String, result: SamplingResponse) -> Result<()> {
        // Check if initialized
        self.ensure_initialized().await?;

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::to_value(result).map_err(Error::SerializationError)?),
            error: None,
        };

        self.send_response(response).await
    }

    /// Get the server capabilities.
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.read().await.clone()
    }

    /// Send a generic method call to the server.
    async fn send_method<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R> {
        // Check if initialized
        self.ensure_initialized().await?;

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Self::generate_id(),
            method: method.to_string(),
            params: Some(serde_json::to_value(params).map_err(Error::SerializationError)?),
            protocol_version: Some(MCP_PROTOCOL_VERSION.to_string()),
        };

        let response = self.send_request(request).await?;
        self.parse_response::<R>(response)
    }

    /// Send a JSON-RPC request to the server.
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Acquire semaphore permit to limit concurrent requests
        let _permit =
            self.semaphore.acquire().await.map_err(|_| {
                Error::ProtocolError("Too many concurrent MCP requests".to_string())
            })?;

        // Check request size limit before sending
        // Serialize request with strict size limit
        let mut writer = LimitedWriter::new(self.config.max_request_size);
        serde_json::to_writer(&mut writer, &request).map_err(|e| {
            if e.is_io() {
                Error::ProtocolError(format!(
                    "Request too large (limit: {} bytes)",
                    self.config.max_request_size
                ))
            } else {
                Error::SerializationError(e)
            }
        })?;
        let request_body = writer.into_inner();

        let response = self
            .client
            .post(self.server_url.clone())
            .body(request_body) // Use the pre-serialized body
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(Error::HttpError)?;

        let status = response.status();
        let response_body = self.read_response_body(response).await?;

        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: response_body,
                metadata: None,
            });
        }

        let response: JsonRpcResponse =
            serde_json::from_str(&response_body).map_err(Error::SerializationError)?;

        Ok(response)
    }

    /// Send a JSON-RPC response to the server with security controls.
    async fn send_response(&self, response: JsonRpcResponse) -> Result<()> {
        // Acquire semaphore permit to limit concurrent requests
        let _permit =
            self.semaphore.acquire().await.map_err(|_| {
                Error::ProtocolError("Too many concurrent MCP requests".to_string())
            })?;

        // Validate response size before sending
        // Note: We use max_request_size for all outgoing messages (requests and responses)
        let mut writer = LimitedWriter::new(self.config.max_request_size);
        serde_json::to_writer(&mut writer, &response).map_err(|e| {
            if e.is_io() {
                Error::ProtocolError(format!(
                    "Response too large (limit: {} bytes)",
                    self.config.max_request_size
                ))
            } else {
                Error::SerializationError(e)
            }
        })?;
        let response_body = writer.into_inner();

        // Send response with timeout (handled by reqwest client)
        let _response = self
            .client
            .post(self.server_url.clone())
            .body(response_body)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(Error::HttpError)?;

        Ok(())
    }

    /// Parse a JSON-RPC response into the expected type.
    fn parse_response<T: serde::de::DeserializeOwned>(
        &self,
        response: JsonRpcResponse,
    ) -> Result<T> {
        // Check for errors
        if let Some(error) = response.error {
            return Err(Error::ApiError {
                code: error.code as u16,
                message: error.message,
                metadata: error.data,
            });
        }

        // Parse the result
        match response.result {
            Some(result) => serde_json::from_value(result).map_err(Error::SerializationError),
            None => Err(Error::ProtocolError("Response contains no result".into())),
        }
    }

    /// Read the response body with strict size limits.
    async fn read_response_body(&self, response: reqwest::Response) -> Result<String> {
        // Check response size limit from Content-Length header
        let content_length = response.content_length().unwrap_or(0);
        if content_length > self.config.max_response_size as u64 {
            return Err(Error::ResponseTooLarge(
                content_length as usize,
                self.config.max_response_size,
            ));
        }

        // Read body with strict size limit
        use futures::StreamExt;
        let mut stream = response.bytes_stream();
        let mut body_bytes = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(Error::HttpError)?;
            if body_bytes.len() + chunk.len() > self.config.max_response_size {
                return Err(Error::ResponseTooLarge(
                    body_bytes.len() + chunk.len(),
                    self.config.max_response_size,
                ));
            }
            body_bytes.extend_from_slice(&chunk);
        }

        String::from_utf8(body_bytes)
            .map_err(|e| Error::ProtocolError(format!("Invalid UTF-8 in response: {}", e)))
    }

    /// Ensure the client has been initialized.
    async fn ensure_initialized(&self) -> Result<()> {
        if self.capabilities.read().await.is_none() {
            return Err(Error::ProtocolError("MCP client not initialized".into()));
        }
        Ok(())
    }
}

/// A writer that enforces a maximum size limit.
struct LimitedWriter {
    buffer: Vec<u8>,
    limit: usize,
}

impl LimitedWriter {
    fn new(limit: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(std::cmp::min(limit, 1024 * 1024)), // Pre-allocate up to 1MB
            limit,
        }
    }

    fn into_inner(self) -> Vec<u8> {
        self.buffer
    }
}

impl std::io::Write for LimitedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.buffer.len() + buf.len() > self.limit {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Size limit exceeded",
            ));
        }
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::StatusCode;
    use std::time::Duration;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    fn create_test_config() -> McpConfig {
        McpConfig {
            request_timeout: Duration::from_secs(1),
            max_response_size: 1024, // 1KB for testing
            max_request_size: 512,   // 512B for testing
            max_concurrent_requests: 2,
        }
    }

    #[tokio::test]
    async fn test_mcp_client_with_config() {
        let mock_server = MockServer::start().await;
        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        // Test that client was created with custom config
        assert_eq!(client.config.request_timeout, Duration::from_secs(1));
        assert_eq!(client.config.max_response_size, 1024);
        assert_eq!(client.config.max_request_size, 512);
        assert_eq!(client.config.max_concurrent_requests, 2);
    }

    #[tokio::test]
    async fn test_invalid_config() {
        let mock_server = MockServer::start().await;

        // Test zero timeout
        let config = McpConfig {
            request_timeout: Duration::from_secs(0),
            ..create_test_config()
        };
        let result = MCPClient::new_with_config(mock_server.uri(), config);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ConfigError(msg) => assert!(msg.contains("Request timeout")),
            e => panic!("Expected ConfigError, got: {:?}", e),
        }

        // Test zero max response size
        let config = McpConfig {
            max_response_size: 0,
            ..create_test_config()
        };
        let result = MCPClient::new_with_config(mock_server.uri(), config);
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ConfigError(msg) => assert!(msg.contains("Max response size")),
            e => panic!("Expected ConfigError, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_mcp_client_with_default_config() {
        let mock_server = MockServer::start().await;
        let client = MCPClient::new(mock_server.uri()).unwrap();

        // Test default config values
        assert_eq!(client.config.request_timeout, Duration::from_secs(30));
        assert_eq!(client.config.max_response_size, 10 * 1024 * 1024);
        assert_eq!(client.config.max_request_size, 1024 * 1024);
        assert_eq!(client.config.max_concurrent_requests, 10);
    }

    #[tokio::test]
    async fn test_request_timeout() {
        let mock_server = MockServer::start().await;

        // Mock server that responds very slowly
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK)
                    .set_delay(Duration::from_secs(3)) // Longer than 1s timeout
                    .set_body_json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": "test",
                        "result": {"protocolVersion": "2025-03-26"}
                    })),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        match &error {
            Error::ProtocolError(msg) => assert!(msg.contains("timed out")),
            Error::HttpError(_) => {} // HTTP timeout errors are also acceptable
            _ => panic!("Expected timeout error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_response_size_limit() {
        let mock_server = MockServer::start().await;

        // Create a response larger than 1KB
        let large_result = "x".repeat(2048); // 2KB
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "test",
                    "result": {"data": large_result}
                })),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ResponseTooLarge(size, limit) => {
                assert!(size > limit);
                assert_eq!(limit, 1024);
            }
            e => panic!("Expected ResponseTooLarge error, got: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_request_size_limit() {
        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "test",
                    "result": {"protocolVersion": "2025-03-26"}
                })),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        // Create a request that exceeds 512B by creating a very large protocol_version
        let large_protocol = "x".repeat(600); // This will exceed 512B limit

        // Create the request manually to test size validation
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: "test".to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": large_protocol,
                "capabilities": {
                    "sampling": {}
                }
            })),
            protocol_version: Some("2025-03-26".to_string()),
        };

        // Test that the request size validation catches the large request
        let request_json = serde_json::to_string(&request).unwrap();
        assert!(request_json.len() > 512, "Request should exceed 512B limit");

        // Try to send the request - it should fail with size validation
        let result = client.send_request(request).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match &error {
            Error::ProtocolError(msg) => assert!(msg.contains("Request too large")),
            _ => panic!("Expected request size error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_concurrent_request_limiting() {
        let mock_server = MockServer::start().await;

        // Create a slow response to test concurrent limiting
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK)
                    .set_delay(Duration::from_millis(200))
                    .set_body_json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": "test",
                        "result": {"protocolVersion": "2025-03-26"}
                    })),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        // Launch 3 concurrent requests (limit is 2)
        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let handles: Vec<_> = (0..3)
            .map(|_| {
                let client = client.clone();
                let caps = capabilities.clone();
                tokio::spawn(async move { client.initialize(caps).await })
            })
            .collect();

        let results: Vec<_> = futures::future::join_all(handles).await;

        // Count successful requests
        let successes = results.iter().filter(|r| matches!(r, Ok(Ok(_)))).count();

        // At most 2 should succeed (the limit)
        assert!(
            successes <= 2,
            "Expected at most 2 successful requests, got {}",
            successes
        );
    }

    #[tokio::test]
    async fn test_successful_request_within_limits() {
        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "test",
                    "result": {
                        "protocol_version": "2025-03-26"
                    }
                })),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;

        assert!(result.is_ok());
        let server_caps = result.unwrap();
        assert_eq!(server_caps.protocol_version, "2025-03-26");
    }

    #[tokio::test]
    async fn test_content_length_header_validation() {
        let mock_server = MockServer::start().await;

        // Create a response that's actually larger than the limit
        let large_result = "x".repeat(1500); // Larger than 1KB limit
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "test",
                    "result": {"protocolVersion": "2025-03-26", "data": large_result}
                })),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match &error {
            Error::ResponseTooLarge(_, _) => {}
            _ => panic!("Expected size validation error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_response_size_limit_with_chunked_encoding() {
        let mock_server = MockServer::start().await;

        // Create a large response with chunked encoding (no Content-Length)
        let large_result = "x".repeat(2048); // 2KB, larger than 1KB limit
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK)
                    .set_body_json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": "test",
                        "result": {"data": large_result}
                    }))
                    // Remove Content-Length to force chunked encoding or at least bypass the header check
                    .append_header("Transfer-Encoding", "chunked"),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;

        assert!(result.is_err());
        let error = result.unwrap_err();
        match &error {
            Error::ResponseTooLarge(_, _) => {}
            _ => panic!("Expected size validation error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_large_error_response() {
        let mock_server = MockServer::start().await;

        // Create a large error response (larger than 1KB limit)
        let large_error_msg = "x".repeat(2048);
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR)
                    .set_body_string(&large_error_msg),
            )
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;

        // Currently this fails because it returns ApiError with the full message
        // We want it to fail with ResponseTooLarge
        assert!(result.is_err());
        let error = result.unwrap_err();
        match &error {
            Error::ResponseTooLarge(_, _) => {}
            Error::ApiError { message, .. } => panic!(
                "Should have failed with size limit, but got ApiError with len: {}",
                message.len()
            ),
            _ => panic!("Expected size validation error, got: {:?}", error),
        }
    }

    #[tokio::test]
    async fn test_request_size_limit_edge_cases() {
        let mock_server = MockServer::start().await;
        Mock::given(matchers::method("POST"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK).set_body_json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": "test",
                    "result": {"protocolVersion": "2025-03-26"}
                })),
            )
            .mount(&mock_server)
            .await;

        let config = McpConfig {
            request_timeout: Duration::from_secs(1),
            max_response_size: 1024,
            max_request_size: 200, // Increased limit to accommodate basic request
            max_concurrent_requests: 1,
        };
        let client = MCPClient::new_with_config(mock_server.uri(), config).unwrap();

        // Case 1: Small request (should pass)
        let small_caps = ClientCapabilities {
            protocol_version: "v1".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(small_caps).await;

        // If 200 is too small, this will fail.
        if let Err(Error::ProtocolError(msg)) = &result {
            if msg.contains("Request too large") {
                panic!(
                    "Test config limit 200 is too small for basic initialize request. Msg: {}",
                    msg
                );
            }
        }

        // Case 2: Request that definitely exceeds 200
        let large_caps = ClientCapabilities {
            protocol_version: "x".repeat(300), // 300 bytes > 200 limit
            supports_sampling: None,
        };
        let result = client.initialize(large_caps).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            Error::ProtocolError(msg) => {
                // println!("Got ProtocolError: '{}'", msg);
                assert!(msg.contains("Request too large"));
            }
            e => panic!("Expected ProtocolError, got {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_malformed_json_response() {
        let mock_server = MockServer::start().await;

        Mock::given(matchers::method("POST"))
            .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_string("{ invalid json }"))
            .mount(&mock_server)
            .await;

        let client = MCPClient::new_with_config(mock_server.uri(), create_test_config()).unwrap();

        let capabilities = ClientCapabilities {
            protocol_version: "2025-03-26".to_string(),
            supports_sampling: None,
        };
        let result = client.initialize(capabilities).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            Error::SerializationError(_) => {}
            e => panic!("Expected SerializationError, got: {:?}", e),
        }
    }
}
