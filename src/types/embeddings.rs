//! Types for OpenRouter Embeddings API.

use serde::{Deserialize, Serialize};

/// Input for an embedding request — single string, batch of strings, or multimodal content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// A single string to embed.
    Single(String),
    /// A batch of strings to embed.
    Batch(Vec<String>),
}

/// Encoding format for the embedding output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    Float,
    Base64,
}

/// Request body for `POST /api/v1/embeddings`.
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingRequest {
    /// The model to use for embeddings.
    pub model: String,
    /// Input text(s) to embed.
    pub input: EmbeddingInput,
    /// Encoding format for the response (default: float).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EncodingFormat>,
    /// Provider preferences for routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<crate::models::provider_preferences::ProviderPreferences>,
}

/// A single embedding result.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingData {
    /// The embedding vector.
    pub embedding: Vec<f64>,
    /// Index of this embedding in the input batch.
    pub index: usize,
    /// Object type — always "embedding".
    #[serde(default)]
    pub object: String,
}

/// Usage information for an embedding request.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingUsage {
    /// Number of tokens in the input.
    pub prompt_tokens: u32,
    /// Total tokens used (same as prompt_tokens for embeddings).
    pub total_tokens: u32,
}

/// Response from `POST /api/v1/embeddings`.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type — "list".
    pub object: String,
    /// The embedding results.
    pub data: Vec<EmbeddingData>,
    /// The model used.
    pub model: String,
    /// Token usage for the request.
    pub usage: Option<EmbeddingUsage>,
}

impl EmbeddingResponse {
    /// Get the first embedding vector (convenience for single-input requests).
    pub fn first_embedding(&self) -> Option<&Vec<f64>> {
        self.data.first().map(|d| &d.embedding)
    }

    /// Get all embedding vectors.
    pub fn embeddings(&self) -> Vec<&Vec<f64>> {
        self.data.iter().map(|d| &d.embedding).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_request_single_serialization() {
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Single("Hello world".to_string()),
            encoding_format: None,
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["model"], "openai/text-embedding-3-small");
        assert_eq!(json["input"], "Hello world");
    }

    #[test]
    fn test_embedding_request_batch_serialization() {
        let req = EmbeddingRequest {
            model: "openai/text-embedding-3-small".to_string(),
            input: EmbeddingInput::Batch(vec!["Hello".to_string(), "World".to_string()]),
            encoding_format: Some(EncodingFormat::Float),
            provider: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["input"].as_array().unwrap().len(), 2);
        assert_eq!(json["encoding_format"], "float");
    }

    #[test]
    fn test_embedding_response_deserialization() {
        let json = r#"{
            "object": "list",
            "data": [
                {
                    "embedding": [0.1, 0.2, 0.3, 0.4],
                    "index": 0,
                    "object": "embedding"
                }
            ],
            "model": "openai/text-embedding-3-small",
            "usage": {
                "prompt_tokens": 5,
                "total_tokens": 5
            }
        }"#;

        let response: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.object, "list");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding, vec![0.1, 0.2, 0.3, 0.4]);
        assert_eq!(response.data[0].index, 0);
        assert_eq!(response.model, "openai/text-embedding-3-small");
        assert_eq!(response.usage.as_ref().unwrap().prompt_tokens, 5);
    }

    #[test]
    fn test_embedding_response_batch() {
        let json = r#"{
            "object": "list",
            "data": [
                {"embedding": [0.1, 0.2], "index": 0, "object": "embedding"},
                {"embedding": [0.3, 0.4], "index": 1, "object": "embedding"}
            ],
            "model": "openai/text-embedding-3-small",
            "usage": {"prompt_tokens": 10, "total_tokens": 10}
        }"#;

        let response: EmbeddingResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.first_embedding().unwrap(), &vec![0.1, 0.2]);
        assert_eq!(response.embeddings().len(), 2);
    }

    #[test]
    fn test_embedding_input_single_deserialization() {
        let json = r#""Hello world""#;
        let input: EmbeddingInput = serde_json::from_str(json).unwrap();
        match input {
            EmbeddingInput::Single(s) => assert_eq!(s, "Hello world"),
            _ => panic!("Expected Single variant"),
        }
    }

    #[test]
    fn test_embedding_input_batch_deserialization() {
        let json = r#"["Hello", "World"]"#;
        let input: EmbeddingInput = serde_json::from_str(json).unwrap();
        match input {
            EmbeddingInput::Batch(v) => assert_eq!(v, vec!["Hello", "World"]),
            _ => panic!("Expected Batch variant"),
        }
    }
}
