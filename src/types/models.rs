use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A model capability, such as "completion" or "chat".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelCapability {
    Chat,
    Completion,
    Embedding,
    Tool,
    Instruction,
    Multimodal,
    Vision,
    /// For future compatibility
    #[serde(other)]
    Other,
}

/// Model response formatting capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFormatting {
    Json,
    Markdown,
    Html,
    Xml,
    /// For future compatibility
    #[serde(other)]
    Other,
}

/// Information about a specific model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// The model identifier.
    pub id: String,

    /// The name of the provider for this model.
    pub provider: String,

    /// A human-readable name for the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Brief description of the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The capabilities supported by this model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<ModelCapability>>,

    /// The output formats supported by this model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatting: Option<Vec<ModelFormatting>>,

    /// The maximum context length supported by this model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,

    /// Additional model-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Request to list available models.
#[derive(Debug, Serialize)]
pub struct ModelsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<ModelCapability>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Response containing available models.
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    /// A list of available models.
    pub models: Vec<ModelInfo>,
}
