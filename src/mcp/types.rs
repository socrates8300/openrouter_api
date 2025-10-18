//! Type definitions for the Model Context Protocol.
#![allow(unused)]
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

/// MCP Server capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Protocol version supported by the server
    pub protocol_version: String,

    /// Resources exposed by the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceCapabilities>,

    /// Tools exposed by the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,

    /// Prompts exposed by the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptCapabilities>,

    /// Whether server requires sampling capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_sampling: Option<bool>,
}

/// Resource capabilities offered by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceCapabilities {
    /// Available resource groups
    pub resource_groups: Vec<ResourceGroup>,
}

/// Resource group definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGroup {
    /// Unique identifier for the resource group
    pub id: String,

    /// Human-readable name for the resource group
    pub name: String,

    /// Optional description of the resource group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Resources available within this group
    pub resources: Vec<Resource>,
}

/// Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Unique identifier for the resource
    pub id: String,

    /// Human-readable name for the resource
    pub name: String,

    /// Optional description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional metadata about the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Tool capabilities offered by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    /// Available tools
    pub tools: Vec<Tool>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Unique identifier for the tool
    pub id: String,

    /// Human-readable name for the tool
    pub name: String,

    /// Optional description of the tool
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter schema for the tool (OpenAPI schema)
    pub parameter_schema: serde_json::Value,

    /// Return value schema for the tool (OpenAPI schema)
    pub return_schema: serde_json::Value,
}

/// Prompt capabilities offered by the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptCapabilities {
    /// Available prompts
    pub prompts: Vec<Prompt>,
}

/// Prompt definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique identifier for the prompt
    pub id: String,

    /// Human-readable name for the prompt
    pub name: String,

    /// Optional description of the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Parameter schema for the prompt (OpenAPI schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_schema: Option<serde_json::Value>,
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

/// Resource response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceResponse {
    /// The resource content
    pub content: String,

    /// MIME type of the content
    pub mime_type: String,

    /// Optional metadata about the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
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

/// Configuration for MCP client behavior
#[derive(Debug, Clone)]
pub struct McpConfig {
    /// Request timeout for individual MCP requests
    pub request_timeout: Duration,
    /// Maximum number of concurrent requests
    pub max_concurrent_requests: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            request_timeout: Duration::from_secs(30),
            max_concurrent_requests: 10,
        }
    }
}

/// Prompt execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePromptParams {
    /// Prompt identifier to execute
    pub id: String,

    /// Optional parameters for the prompt
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Prompt execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePromptResponse {
    /// The result of executing the prompt
    pub result: serde_json::Value,
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
}

/// Sampling response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingResponse {
    /// The result of the sampling
    pub result: String,
}
