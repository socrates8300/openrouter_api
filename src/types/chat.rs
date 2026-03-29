use crate::models::tool::{ToolCall, ToolCallChunk};
use crate::types::ids::ToolCallId;
use crate::types::status::StreamingStatus;
use serde::de::Error as DeError;
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

/// Content type for multimodal message parts.
///
/// This enum represents the different types of content that can be included in a message,
/// such as text, images, audio, or files. Using an enum with serde tagging makes
/// invalid content types unrepresentable at compile time.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// Plain text content.
    Text,
    /// Image content with URL or base64 data.
    ImageUrl,
    /// Audio content with URL.
    AudioUrl,
    /// File content (e.g., PDF) with URL.
    FileUrl,
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContentType::Text => write!(f, "text"),
            ContentType::ImageUrl => write!(f, "image_url"),
            ContentType::AudioUrl => write!(f, "audio_url"),
            ContentType::FileUrl => write!(f, "file_url"),
        }
    }
}

/// Defines the role of a chat message (user, assistant, or system).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool,
}

impl std::fmt::Display for ChatRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatRole::User => write!(f, "user"),
            ChatRole::Assistant => write!(f, "assistant"),
            ChatRole::System => write!(f, "system"),
            ChatRole::Tool => write!(f, "tool"),
        }
    }
}

/// Stop sequence for chat completion - can be a string or array of strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StopSequence {
    Single(String),
    Multiple(Vec<String>),
}

/// Prediction configuration for latency optimization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PredictionConfig {
    #[serde(rename = "type")]
    pub prediction_type: String, // "content"
    pub content: String,
}

/// Verbosity level for model responses.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VerbosityLevel {
    Low,
    Medium,
    High,
}

/// Route strategy for model routing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RouteStrategy {
    Fallback,
}

/// Image detail level for vision models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Auto,
    Low,
    High,
}

/// Text content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub content_type: ContentType,
    pub text: String,
}

/// Image URL content for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageUrl {
    pub url: String, // URL or base64 encoded image data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}

/// Image content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub content_type: ContentType,
    pub image_url: ImageUrl,
}

/// Audio URL content for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioUrl {
    pub url: String,
}

/// Audio content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioContent {
    #[serde(rename = "type")]
    pub content_type: ContentType,
    pub audio_url: AudioUrl,
}

/// File URL content for multimodal messages (e.g. PDFs).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileUrl {
    pub url: String,
}

/// File content part for multimodal messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileContent {
    #[serde(rename = "type")]
    pub content_type: ContentType,
    pub file_url: FileUrl,
}

/// Content parts for multimodal messages (user role only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContent),
    Audio(AudioContent),
    File(FileContent),
}

/// Enhanced message content supporting both string and multimodal content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

/// Represents a chat message with a role and content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: ChatRole,
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<ToolCallId>,
    // Optionally include tool_calls when the assistant message contains a tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Reasoning content from thinking models (o1, o3, DeepSeek R1, Claude extended thinking).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Structured reasoning details returned by some reasoning-capable models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Vec<ReasoningDetail>>,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            role: ChatRole::User,
            content: MessageContent::Text("".to_string()),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            reasoning: None,
            reasoning_details: None,
        }
    }
}

impl Message {
    /// Create a simple text message (backward compatible).
    pub fn text(role: ChatRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
            reasoning_details: None,
        }
    }

    /// Create a message with a name.
    pub fn text_with_name(
        role: ChatRole,
        content: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            role,
            content: MessageContent::Text(content.into()),
            name: Some(name.into()),
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
            reasoning_details: None,
        }
    }

    /// Create a multimodal message with content parts.
    pub fn multimodal(role: ChatRole, parts: Vec<ContentPart>) -> Self {
        Self {
            role,
            content: MessageContent::Parts(parts),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
            reasoning_details: None,
        }
    }

    /// Create a tool message.
    pub fn tool(content: impl Into<String>, tool_call_id: impl Into<ToolCallId>) -> Self {
        Self {
            role: ChatRole::Tool,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            reasoning: None,
            reasoning_details: None,
        }
    }

    /// Create an assistant message with tool calls.
    pub fn assistant_with_tools(
        content: Option<impl Into<String>>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: ChatRole::Assistant,
            content: content
                .map(|c| MessageContent::Text(c.into()))
                .unwrap_or(MessageContent::Text("".to_string())),
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            reasoning: None,
            reasoning_details: None,
        }
    }
}

/// Debug configuration for request inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// When true, the transformed upstream request body is echoed as the first streaming chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo_upstream_body: Option<bool>,
}

/// Plugin configuration for enabling OpenRouter server-side features.
#[derive(Debug, Clone, PartialEq)]
pub struct Plugin {
    /// Plugin identifier (e.g., "web", "file-parser", "response-healing").
    pub id: String,
    /// Optional explicit enable/disable flag for the plugin.
    pub enabled: Option<bool>,
    /// Optional plugin-specific configuration.
    ///
    /// When this is a JSON object, its keys are flattened into the plugin object
    /// to match the OpenRouter request schema. Non-object values fall back to a
    /// legacy nested `config` field for backwards compatibility.
    pub config: Option<serde_json::Value>,
}

impl Serialize for Plugin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("id", &self.id)?;

        if let Some(enabled) = self.enabled {
            map.serialize_entry("enabled", &enabled)?;
        }

        if let Some(config) = &self.config {
            match config {
                serde_json::Value::Object(object) => {
                    for (key, value) in object {
                        map.serialize_entry(key, value)?;
                    }
                }
                other => {
                    map.serialize_entry("config", other)?;
                }
            }
        }

        map.end()
    }
}

impl<'de> Deserialize<'de> for Plugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut raw = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;

        let id = match raw.remove("id") {
            Some(serde_json::Value::String(id)) => id,
            Some(other) => {
                return Err(D::Error::custom(format!(
                    "plugin id must be a string, got {other}"
                )))
            }
            None => return Err(D::Error::missing_field("id")),
        };

        let enabled = match raw.remove("enabled") {
            Some(value) => {
                serde_json::from_value::<Option<bool>>(value).map_err(D::Error::custom)?
            }
            None => None,
        };

        let legacy_config = raw.remove("config");
        let flattened_config = if raw.is_empty() {
            None
        } else {
            Some(serde_json::Value::Object(raw))
        };

        let config = match (legacy_config, flattened_config) {
            (None, None) => None,
            (Some(config), None) => Some(config),
            (None, Some(config)) => Some(config),
            (
                Some(serde_json::Value::Object(mut legacy)),
                Some(serde_json::Value::Object(flattened)),
            ) => {
                legacy.extend(flattened);
                Some(serde_json::Value::Object(legacy))
            }
            (Some(config), Some(serde_json::Value::Object(mut flattened))) => {
                flattened.insert("config".to_string(), config);
                Some(serde_json::Value::Object(flattened))
            }
            (Some(config), Some(other)) => {
                return Err(D::Error::custom(format!(
                    "unexpected plugin config payload combination: config={config}, extra={other}"
                )))
            }
        };

        Ok(Self {
            id,
            enabled,
            config,
        })
    }
}

impl Plugin {
    /// Create a plugin by id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            enabled: None,
            config: None,
        }
    }

    /// Enable the Response Healing plugin (auto-fixes malformed JSON responses, Dec 2025).
    pub fn response_healing() -> Self {
        Self::new("response-healing")
    }

    /// Enable the web search plugin.
    pub fn web_search() -> Self {
        Self::new("web")
    }

    /// Enable the file parser plugin (PDF, etc.).
    pub fn file_parser() -> Self {
        Self::new("file-parser")
    }

    /// Enable the context compression plugin.
    pub fn context_compression() -> Self {
        Self::new("context-compression")
    }

    /// Explicitly enable or disable the plugin.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Attach plugin-specific configuration.
    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = Some(config);
        self
    }
}

/// Effort level for reasoning models (o1, o3, Claude extended thinking, DeepSeek R1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    XHigh,
    High,
    Medium,
    Low,
    Minimal,
    None,
}

/// Summary verbosity for reasoning models that expose summarized thinking output.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
}

/// Reasoning configuration for models that support extended thinking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReasoningConfig {
    /// Constrains effort on reasoning-capable models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Caps the reasoning token budget on models that support explicit budgets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Enables or disables reasoning where the provider exposes an explicit toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Requests summarized reasoning details when the provider supports them.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ReasoningSummary>,
}

impl ReasoningConfig {
    /// Create a reasoning config that constrains model effort.
    pub fn with_effort(effort: ReasoningEffort) -> Self {
        Self {
            effort: Some(effort),
            ..Self::default()
        }
    }

    /// Create a reasoning config that caps the reasoning token budget.
    pub fn with_max_tokens(max_tokens: u32) -> Self {
        Self {
            max_tokens: Some(max_tokens),
            ..Self::default()
        }
    }

    /// Attach reasoning summary verbosity preferences.
    pub fn with_summary(mut self, summary: ReasoningSummary) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Explicitly enable or disable reasoning.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
}

/// Structured reasoning detail item returned by some reasoning-capable models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ReasoningDetail {
    #[serde(rename = "reasoning.summary")]
    Summary {
        summary: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
    #[serde(rename = "reasoning.encrypted")]
    Encrypted {
        data: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
    #[serde(rename = "reasoning.text")]
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        index: Option<u32>,
    },
}

/// Chat completion request matching the OpenRouter API schema.
#[derive(Debug, Serialize, Clone, Default)]
pub struct ChatCompletionRequest {
    /// The model ID to use.
    pub model: String,
    /// The list of messages.
    pub messages: Vec<Message>,
    /// Whether the response should be streamed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<StreamingStatus>,
    /// (Optional) Response format for structured outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<crate::api::request::ResponseFormatConfig>,
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
    /// (Optional) Debug configuration for request inspection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<DebugConfig>,
    /// (Optional) Plugins to enable (e.g., "web", "file-parser", "response-healing").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<Vec<Plugin>>,
    /// (Optional) Reasoning configuration for models supporting extended thinking
    /// (o1, o3, Claude extended thinking, DeepSeek R1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningConfig>,
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

/// Server-side tool usage counts (e.g., web search requests).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerToolUse {
    /// Number of web search requests made by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u32>,
    /// Number of server-side fetch/open-page requests made by the server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_fetch_requests: Option<u32>,
}

/// Usage data returned from the API.
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    /// Cost of the request in credits (USD float).
    pub cost: Option<f64>,
    /// Whether the request used a user-provided API key (Bring Your Own Key).
    pub is_byok: Option<bool>,
    /// Server-side tool usage counts.
    pub server_tool_use: Option<ServerToolUse>,
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    pub completion_tokens_details: Option<CompletionTokensDetails>,
}

/// Details about prompt token usage.
#[derive(Debug, Deserialize)]
pub struct PromptTokensDetails {
    pub cached_tokens: Option<u32>,
    pub audio_tokens: Option<u32>,
    pub text_tokens: Option<u32>,
    pub image_tokens: Option<u32>,
    /// Tokens written to cache when the provider exposes explicit cache-write accounting.
    pub cache_write_tokens: Option<u32>,
    /// Video input tokens when supported by the upstream provider.
    pub video_tokens: Option<u32>,
}

/// Details about completion token usage.
#[derive(Debug, Deserialize)]
pub struct CompletionTokensDetails {
    pub reasoning_tokens: Option<u32>,
    pub audio_tokens: Option<u32>,
    pub accepted_prediction_tokens: Option<u32>,
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
    pub tool_calls: Option<Vec<ToolCallChunk>>,
    /// Reasoning content delta from thinking models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Structured reasoning details delta from thinking models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Vec<ReasoningDetail>>,
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
