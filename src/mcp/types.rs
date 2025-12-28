//! Type definitions for the Model Context Protocol.
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The base protocol version
pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

/// Base JSON-RPC request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
}

/// Base JSON-RPC response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Initialize parameters sent by the client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Client capabilities
    pub capabilities: ClientCapabilities,
}

/// Server capabilities once initialized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Protocol version supported by the server
    pub protocol_version: String,
    /// Whether server supports sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_sampling: Option<bool>,
}

/// Client capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Protocol version supported by the client
    pub protocol_version: String,
    /// Whether client supports sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_sampling: Option<bool>,
}

/// Resource response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    /// The resource content
    pub contents: Vec<ResourceContent>,
}

/// Resource content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceContent {
    /// URI of the resource
    pub uri: String,
    /// MIME type of the content
    pub mime_type: Option<String>,
    /// Text content
    pub text: Option<String>,
    /// Blob content
    pub blob: Option<ResourceBlob>,
    /// Optional metadata about the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Blob content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBlob {
    /// MIME type of the blob
    pub mime_type: String,
    /// Base64 encoded data
    pub data: String,
}

/// Get resource parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetResourceParams {
    /// Resource identifier to fetch
    pub id: String,
    /// Optional parameters for the resource request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    /// Tool identifier to call
    pub id: String,
    /// Parameters for the tool call
    pub parameters: serde_json::Value,
}

/// Tool call response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// The result of the tool call
    pub result: serde_json::Value,
}

/// Prompt execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePromptParams {
    /// Prompt identifier to execute
    pub name: String,
    /// Optional arguments for the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// Prompt execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePromptResponse {
    /// The result of the prompt execution
    pub result: serde_json::Value,
}

/// Sampling request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingParams {
    /// Task description for the sampling
    pub task: String,
    /// Optional system prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Optional parameters for the sampling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
    /// Include the context in the response?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_context: Option<bool>,
    /// Maximum number of tokens to sample
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    /// Sampling temperature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Top-p sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
}

/// Sampling response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    /// The result of the sampling
    pub result: String,
    /// Stop reason
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// Message object for sampling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingMessage {
    /// Role of the message author
    pub role: String,
    /// Content of the message
    pub content: String,
    /// Name of the message author
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Configuration for MCP client security and performance limits
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Timeout for individual requests
    pub request_timeout: Duration,
    /// Maximum response size in bytes
    pub max_response_size: usize,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            max_response_size: 10 * 1024 * 1024, // 10MB
            max_request_size: 1024 * 1024,       // 1MB
            max_concurrent_requests: 10,
        }
    }
}
