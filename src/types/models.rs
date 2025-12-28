use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ids::{ModelId, Price};

/// A model capability, such as "completion" or "chat".
/// This is used for filtering in `ModelsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Nested structure for architecture details within ModelInfo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDetails {
    pub modality: String,
    pub input_modalities: Vec<String>,
    pub output_modalities: Vec<String>,
    pub tokenizer: String,
    pub instruct_type: Option<String>,
}

/// Nested structure for pricing information within ModelInfo.
/// Prices are strongly-typed Price values for type safety and validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub prompt: Price,
    pub completion: Price,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Price>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<Price>,
    // These fields appear to be consistently present in API response (e.g., as "0")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search: Option<Price>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_reasoning: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_read: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_write: Option<Price>,
}

impl PricingInfo {
    /// Gets the prompt price as f64
    pub fn prompt_price(&self) -> f64 {
        self.prompt.as_f64()
    }

    /// Gets the completion price as f64
    pub fn completion_price(&self) -> f64 {
        self.completion.as_f64()
    }
}

/// Nested structure for top provider details within ModelInfo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopProviderInfo {
    pub context_length: Option<u32>, // This context_length is specific to the top_provider
    pub max_completion_tokens: Option<u32>,
    pub is_moderated: bool,
}

/// Information about a specific model, updated to match the API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: ModelId,
    pub name: String,
    pub description: Option<String>,
    pub context_length: u32, // Top-level context_length for the model entry
    pub created: i64,        // Unix timestamp

    pub canonical_slug: Option<String>,
    pub hugging_face_id: Option<String>,

    pub architecture: ArchitectureDetails,
    pub pricing: PricingInfo,
    pub top_provider: TopProviderInfo,

    pub per_request_limits: Option<Value>, // Can be null, structure can vary
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_parameters: Option<Vec<String>>, // Can be null or a list
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
    pub data: Vec<ModelInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_deserialize_model_info() {
        let json_data = r#"
        {
            "id": "moonshotai/kimi-dev-72b:free",
            "canonical_slug": "moonshotai/kimi-dev-72b",
            "hugging_face_id": "moonshotai/Kimi-Dev-72B",
            "name": "Kimi Dev 72b (free)",
            "created": 1750115909,
            "description": "Kimi-Dev-72B is an open-source large language model fine-tuned for software engineering and issue resolution tasks. Based on Qwen2.5-72B, it is optimized using large-scale reinforcement learning that applies code patches in real repositories and validates them via full test suite execution—rewarding only correct, robust completions. The model achieves 60.4% on SWE-bench Verified, setting a new benchmark among open-source models for software bug fixing and code reasoning.",
            "context_length": 131072,
            "architecture": {
                "modality": "text->text",
                "input_modalities": ["text"],
                "output_modalities": ["text"],
                "tokenizer": "Other",
                "instruct_type": null
            },
            "pricing": {
                "prompt": "0",
                "completion": "0",
                "request": "0",
                "image": "0",
                "web_search": "0",
                "internal_reasoning": "0"
            },
            "top_provider": {
                "context_length": 131072,
                "max_completion_tokens": null,
                "is_moderated": false
            },
            "per_request_limits": null,
            "supported_parameters": ["max_tokens", "temperature", "top_p"]
        }
        "#;

        let model_info: Result<ModelInfo, _> = serde_json::from_str(json_data);
        assert!(
            model_info.is_ok(),
            "Failed to deserialize ModelInfo: {:?}",
            model_info.err()
        );

        let model_info = model_info.unwrap();

        assert_eq!(model_info.id, "moonshotai/kimi-dev-72b:free".into());
        assert_eq!(model_info.name, "Kimi Dev 72b (free)");
        assert!(model_info.description.is_some()); // Check it's present
        assert_eq!(model_info.context_length, 131072);
        assert_eq!(model_info.architecture.modality, "text->text");
        assert_eq!(model_info.architecture.input_modalities, vec!["text"]);
        assert_eq!(model_info.pricing.prompt.as_f64(), 0.0);
        assert_eq!(
            model_info.pricing.request.as_ref().map(|p| p.as_f64()),
            Some(0.0)
        );
        assert_eq!(
            model_info.pricing.image.as_ref().map(|p| p.as_f64()),
            Some(0.0)
        );
        assert_eq!(
            model_info.pricing.web_search.as_ref().map(|p| p.as_f64()),
            Some(0.0)
        );
        assert_eq!(
            model_info
                .pricing
                .internal_reasoning
                .as_ref()
                .map(|p| p.as_f64()),
            Some(0.0)
        );
        assert!(!model_info.top_provider.is_moderated);
        assert!(model_info.per_request_limits.is_none());
        assert_eq!(
            model_info.supported_parameters,
            Some(vec![
                "max_tokens".to_string(),
                "temperature".to_string(),
                "top_p".to_string()
            ])
        );
        assert_eq!(
            model_info.canonical_slug.as_deref(),
            Some("moonshotai/kimi-dev-72b")
        );
        assert_eq!(
            model_info.hugging_face_id.as_deref(),
            Some("moonshotai/Kimi-Dev-72B")
        );
    }

    #[test]
    fn test_deserialize_models_response() {
        let json_data = r#"
        {
            "data": [
                {
                    "id": "openai/gpt-4o",
                    "name": "OpenAI: GPT-4o",
                    "description": "GPT-4o is OpenAI's most advanced model.",
                    "context_length": 128000,
                    "created": 1677652288,
                    "canonical_slug": "openai/gpt-4o-2024-05-13",
                    "hugging_face_id": "",
                    "architecture": {
                        "modality": "text+image->text",
                        "input_modalities": ["text", "image"],
                        "output_modalities": ["text"],
                        "tokenizer": "OpenAI",
                        "instruct_type": "openai"
                    },
                    "pricing": {
                        "prompt": "0.000005",
                        "completion": "0.000015",
                        "request": "0",
                        "image": "0",
                        "web_search": "0",
                        "internal_reasoning": "0"
                    },
                    "top_provider": {
                        "context_length": 128000,
                        "max_completion_tokens": 4096,
                        "is_moderated": true
                    },
                    "per_request_limits": null,
                    "supported_parameters": ["tools", "tool_choice", "max_tokens"]
                }
            ]
        }
        "#;
        let models_response: Result<ModelsResponse, _> = serde_json::from_str(json_data);
        assert!(
            models_response.is_ok(),
            "Failed to deserialize ModelsResponse: {:?}",
            models_response.err()
        );
        let models_response = models_response.unwrap();
        assert_eq!(models_response.data.len(), 1);
        assert_eq!(models_response.data[0].id, "openai/gpt-4o".into());
    }

    #[test]
    fn test_deserialize_all_models_from_api() {
        // Construct the path to the test data file relative to the crate root
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let data_path = Path::new(manifest_dir)
            .join("tests")
            .join("data")
            .join("models_api_response.json");

        // Read the JSON data from the file
        let json_data = fs::read_to_string(data_path)
            .expect("Should have been able to read the file models_api_response.json");

        let models_response: Result<ModelsResponse, _> = serde_json::from_str(&json_data);
        assert!(
            models_response.is_ok(),
            "Failed to deserialize ModelsResponse from API data: {:?}",
            models_response.err()
        );

        let models_response = models_response.unwrap();
        assert!(
            !models_response.data.is_empty(),
            "Model data should not be empty"
        );

        // Check a few fields of the first model to ensure correct parsing
        if let Some(first_model) = models_response.data.first() {
            // Allow for flexible checks as the first model might change
            assert!(
                !first_model.id.is_empty(),
                "First model ID should not be empty"
            );
            assert!(
                !first_model.name.is_empty(),
                "First model name should not be empty"
            );
            // Other assertions can be added here if specific fields are guaranteed
            // For example, check that Option fields can be deserialized
            let _ = first_model.description.as_deref();
            let _ = first_model.canonical_slug.as_deref();
            let _ = first_model.hugging_face_id.as_deref();
        } else {
            panic!("No models found in deserialized data");
        }
    }

    #[test]
    fn test_pricing_info_validation() {
        // Test valid pricing
        let valid_pricing = PricingInfo {
            prompt: Price::new(0.001).unwrap(),
            completion: Price::new(0.002).unwrap(),
            request: Some(Price::new(0.003).unwrap()),
            image: Some(Price::new(0.01).unwrap()),
            web_search: Some(Price::new(0.0).unwrap()),
            internal_reasoning: Some(Price::new(0.005).unwrap()),
            input_cache_read: Some(Price::new(0.0005).unwrap()),
            input_cache_write: Some(Price::new(0.001).unwrap()),
        };

        assert_eq!(valid_pricing.prompt_price(), 0.001);
        assert_eq!(valid_pricing.completion_price(), 0.002);

        // Test negative pricing (should parse but might be invalid business logic)
        // Test positive pricing (negative prices are rejected by Price::new)
        let positive_pricing = PricingInfo {
            prompt: Price::new(0.001).unwrap(),
            completion: Price::new(0.002).unwrap(),
            request: Some(Price::new(0.003).unwrap()),
            image: Some(Price::new(0.01).unwrap()),
            web_search: Some(Price::new(0.0).unwrap()),
            internal_reasoning: Some(Price::new(0.005).unwrap()),
            input_cache_read: Some(Price::new(0.001).unwrap()),
            input_cache_write: Some(Price::new(0.002).unwrap()),
        };

        assert_eq!(positive_pricing.prompt_price(), 0.001);
        assert_eq!(positive_pricing.completion_price(), 0.002);

        // Test zero pricing
        let zero_pricing = PricingInfo {
            prompt: Price::new(0.0).unwrap(),
            completion: Price::new(0.0).unwrap(),
            request: Some(Price::new(0.0).unwrap()),
            image: Some(Price::new(0.0).unwrap()),
            web_search: Some(Price::new(0.0).unwrap()),
            internal_reasoning: Some(Price::new(0.0).unwrap()),
            input_cache_read: Some(Price::new(0.0).unwrap()),
            input_cache_write: Some(Price::new(0.0).unwrap()),
        };

        assert_eq!(zero_pricing.prompt_price(), 0.0);
        assert_eq!(zero_pricing.completion_price(), 0.0);
    }

    #[test]
    fn test_model_capability_deserialization() {
        // Test all known capabilities
        let capabilities = vec![
            ("chat", ModelCapability::Chat),
            ("completion", ModelCapability::Completion),
            ("embedding", ModelCapability::Embedding),
            ("tool", ModelCapability::Tool),
            ("instruction", ModelCapability::Instruction),
            ("multimodal", ModelCapability::Multimodal),
            ("vision", ModelCapability::Vision),
        ];

        for (json_str, expected) in capabilities {
            let json = format!("\"{}\"", json_str);
            let capability: ModelCapability = serde_json::from_str(&json).unwrap();
            assert_eq!(capability, expected);
        }

        // Test unknown capability (should deserialize as Other)
        let unknown: ModelCapability = serde_json::from_str("\"unknown_capability\"").unwrap();
        assert!(matches!(unknown, ModelCapability::Other));
    }

    #[test]
    fn test_model_info_edge_cases() {
        // Test model with minimal required fields
        let json_minimal = r#"
        {
            "id": "test/minimal",
            "name": "Minimal Model",
            "context_length": 1000,
            "created": 1234567890,
            "architecture": {
                "modality": "text->text",
                "input_modalities": ["text"],
                "output_modalities": ["text"],
                "tokenizer": "Test"
            },
            "pricing": {
                "prompt": "0.001",
                "completion": "0.002"
            },
            "top_provider": {
                "context_length": 1000,
                "max_completion_tokens": null,
                "is_moderated": false
            }
        }
        "#;

        let model_info: Result<ModelInfo, _> = serde_json::from_str(json_minimal);
        assert!(model_info.is_ok());

        let model = model_info.unwrap();
        assert_eq!(model.id, "test/minimal".into());
        assert_eq!(model.name, "Minimal Model");
        assert!(model.description.is_none());
        assert!(model.canonical_slug.is_none());
        assert!(model.hugging_face_id.is_none());
        assert!(model.per_request_limits.is_none());
        assert!(model.supported_parameters.is_none());

        // Test model with all optional fields
        let json_full = r#"
        {
            "id": "test/full",
            "name": "Full Model",
            "description": "A complete model description",
            "context_length": 100000,
            "created": 1234567890,
            "canonical_slug": "test/full",
            "hugging_face_id": "test-org/full-model",
            "architecture": {
                "modality": "text+image->text",
                "input_modalities": ["text", "image"],
                "output_modalities": ["text"],
                "tokenizer": "TestTokenizer",
                "instruct_type": "test"
            },
            "pricing": {
                "prompt": "0.001",
                "completion": "0.002",
                "request": "0.0001",
                "image": "0.01",
                "web_search": "0",
                "internal_reasoning": "0.005",
                "input_cache_read": "0.0005",
                "input_cache_write": "0.001"
            },
            "top_provider": {
                "context_length": 100000,
                "max_completion_tokens": 4096,
                "is_moderated": true
            },
            "per_request_limits": {
                "max_tokens": 1000,
                "max_images": 10
            },
            "supported_parameters": ["temperature", "top_p", "max_tokens", "tools"]
        }
        "#;

        let model_info: Result<ModelInfo, _> = serde_json::from_str(json_full);
        assert!(model_info.is_ok());

        let model = model_info.unwrap();
        assert!(model.description.is_some());
        assert!(model.canonical_slug.is_some());
        assert!(model.hugging_face_id.is_some());
        assert!(model.per_request_limits.is_some());
        assert!(model.supported_parameters.is_some());
        // Pricing is validated at construction time via Price::new
        assert!(model.pricing.prompt_price() >= 0.0);
    }
}
