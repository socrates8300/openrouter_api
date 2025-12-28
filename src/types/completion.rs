use serde::{Deserialize, Serialize};

/// Represents a text completion request. It minimally contains:
/// - `model`: The model ID to use.
/// - `prompt`: The text prompt to be completed.
///
/// Any extra parameters (e.g., `temperature`, `top_p`, etc.) can also be provided and will be flattened
/// into the resulting JSON.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct CompletionRequest {
    pub model: String,
    pub prompt: String,
    #[serde(flatten)]
    pub extra_params: serde_json::Value,
}

/// Represents a choice returned by the completions endpoint.
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CompletionChoice {
    pub text: String,
    pub index: Option<u32>,
    #[serde(rename = "finish_reason")]
    pub finish_reason: Option<String>,
}

/// Represents the text completion response. It includes:
/// - an optional `id` for the request
/// - a list of choices with the completed text
#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct CompletionResponse {
    pub id: Option<String>,
    pub choices: Vec<CompletionChoice>,
}
