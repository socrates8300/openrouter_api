use crate::models::tool::ToolCall;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines the role of a chat message (user, assistant, or system).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool,
}

/// Stop sequence for chat completion - can be a string or array of strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StopSequence {
    Single(String),
    Multiple(Vec<String>),
}

/// Prediction configuration for latency optimization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    #[serde(rename = "type")]
    pub prediction_type: String, // "content"
    pub content: String,
}

/// Verbosity level for model responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerbosityLevel {
    Low,
    Medium,
    High,
}

/// Route strategy for model routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RouteStrategy {
    Fallback,
}

/// Image detail level for vision models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// Text content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: String, // "text"
    pub text: String,
}

/// Image URL content for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    pub url: String, // URL or base64 encoded image data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

/// Image content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub content_type: String, // "image_url"
    pub image_url: ImageUrl,
}

/// Content parts for multimodal messages (user role only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContent),
}

/// Enhanced message content supporting both string and multimodal content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// Represents a chat message with a role and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    // Optionally include tool_calls when the assistant message contains a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    // Tool call ID for tool role messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    /// Create a simple text message (backward compatible).
    pub fn text(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a message with a name.
    pub fn text_with_name(
        role: impl Into<String>,
        content: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            role: role.into(),
            content: MessageContent::Text(content.into()),
            name: Some(name.into()),
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a multimodal message with content parts.
    pub fn multimodal(role: impl Into<String>, parts: Vec<ContentPart>) -> Self {
        Self {
            role: role.into(),
            content: MessageContent::Parts(parts),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a tool message.
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<String>) -> Self {
        Self {
            role: "tool".to_string(),
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
        }
    }

    /// Create an assistant message with tool calls.
    pub fn assistant_with_tools(
        content: Option<impl Into<String>>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content
                .map(|c| MessageContent::Text(c.into()))
                .unwrap_or(MessageContent::Text("".to_string())),
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
        }
    }
}

/// Chat completion request matching the OpenRouter API schema.
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    /// The model ID to use.
    pub model: String,
    /// The list of messages.
    pub messages: Vec<Message>,
    /// Whether the response should be streamed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// (Optional) Response format for structured outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>,
    /// (Optional) Tool calling field. Now uses our production‑ready tool types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<crate::models::tool::Tool>>,
    /// (Optional) Tool choice configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    /// (Optional) Provider preferences for routing and fallback configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<crate::models::provider_preferences::ProviderPreferences>,
    /// (Optional) Fallback models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    /// (Optional) Message transforms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
    /// (Optional) Route strategy for model routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<RouteStrategy>,
    /// (Optional) User identifier for tracking and abuse prevention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    // Sampling parameters
    /// (Optional) Maximum number of tokens to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// (Optional) Sampling temperature (0.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// (Optional) Nucleus sampling threshold (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// (Optional) Top-k sampling (1 or above, 0 disables).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// (Optional) Frequency penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// (Optional) Presence penalty (-2.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// (Optional) Repetition penalty (0.0 to 2.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f32>,
    /// (Optional) Minimum probability threshold (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f32>,
    /// (Optional) Top-A threshold (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f32>,
    /// (Optional) Seed for deterministic sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    /// (Optional) Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopSequence>,

    // Advanced parameters
    /// (Optional) Logit bias for token selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<u32, f32>>,
    /// (Optional) Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// (Optional) Number of top log probabilities to return (0-20).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u8>,
    /// (Optional) Prediction configuration for latency optimization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<PredictionConfig>,
    /// (Optional) Whether to enable parallel tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// (Optional) Response verbosity level.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<VerbosityLevel>,
}

/// A choice returned by the chat API.
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: Option<String>,
    #[serde(rename = "native_finish_reason")]
    pub native_finish_reason: Option<String>,
    pub index: Option<u32>,
    pub logprobs: Option<LogProbs>,
}

/// Log probabilities information.
#[derive(Debug, Deserialize)]
pub struct LogProbs {
    pub content: Option<Vec<TokenLogProb>>,
}

/// Token log probability information.
#[derive(Debug, Deserialize)]
pub struct TokenLogProb {
    pub token: String,
    pub logprob: f32,
    pub bytes: Option<Vec<u8>>,
    pub top_logprobs: Option<Vec<TopLogProb>>,
}

/// Top log probability information.
#[derive(Debug, Deserialize)]
pub struct TopLogProb {
    pub token: String,
    pub logprob: f32,
    pub bytes: Option<Vec<u8>>,
}

/// Usage data returned from the API.
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Details about prompt token usage.
#[derive(Debug, Deserialize)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_tokens: Option<u32>,
}

/// Details about completion token usage.
#[derive(Debug, Deserialize)]
pub struct CompletionTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u32>,
}

/// Chat completion response.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: i64,
    pub model: String,
    pub object: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub usage: Option<Usage>,
}

/// A choice returned by the streaming chat API.
/// Different from regular Choice as it contains deltas instead of complete messages.
#[derive(Debug, Deserialize)]
pub struct ChoiceStream {
    pub index: u32,
    pub delta: StreamDelta,
    pub finish_reason: Option<String>,
    #[serde(rename = "native_finish_reason")]
    pub native_finish_reason: Option<String>,
    pub logprobs: Option<LogProbs>,
}

/// Delta content for streaming responses.
#[derive(Debug, Deserialize)]
pub struct StreamDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// A streaming chunk for chat completions.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChoiceStream>,
    /// Usage information is typically provided in the final chunk
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
}
