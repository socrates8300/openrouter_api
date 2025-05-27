use crate::models::tool::ToolCall;
use serde::{Deserialize, Serialize};

/// Defines the role of a chat message (user, assistant, or system).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

/// Represents a chat message with a role and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    // Optionally include tool_calls when the assistant message contains a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Chat completion request matching the OpenRouter API schema.
#[derive(Debug, Serialize, Clone)]
pub struct ChatCompletionRequest {
    /// The model ID to use.
    pub model: String,
    /// The list of messages.
    pub messages: Vec<Message>,
    /// Whether the response should be streamed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// (Optional) Stub for response_format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<String>,
    /// (Optional) Tool calling field. Now uses our production‑ready tool types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<crate::models::tool::Tool>>,
    /// (Optional) Provider preferences for routing and fallback configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<crate::models::provider_preferences::ProviderPreferences>,
    /// (Optional) Fallback models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    /// (Optional) Message transforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
}

/// A choice returned by the chat API.
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: Option<String>,
    #[serde(rename = "native_finish_reason")]
    pub native_finish_reason: Option<String>,
}

/// Usage data returned from the API.
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Chat completion response.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    pub usage: Option<Usage>,
}

/// A choice returned by the streaming chat API.
/// Different from regular Choice as it contains deltas instead of complete messages.
#[derive(Debug, Deserialize)]
pub struct ChoiceStream {
    pub index: Option<u32>,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
    #[serde(rename = "native_finish_reason")]
    pub native_finish_reason: Option<String>,
}

/// Delta content for streaming responses.
#[derive(Debug, Deserialize)]
pub struct StreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A streaming chunk for chat completions.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub model: Option<String>,
    pub choices: Vec<ChoiceStream>,
    /// Usage information is typically provided in the final chunk
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}
