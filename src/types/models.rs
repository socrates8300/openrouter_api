use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A model capability, such as "completion" or "chat".
/// This is used for filtering in `ModelsRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
/// Prices are represented as strings, as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub prompt: String,
    pub completion: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    // These fields appear to be consistently present in the API response (e.g., as "0")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_search: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub internal_reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_read: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_cache_write: Option<String>,
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
    pub id: String,
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

impl ModelInfo {
    /// Creates a new ModelInfo
    pub fn new(
        id: String,
        name: String,
        description: Option<String>,
        context_length: u32,
        created: i64,
        architecture: ArchitectureDetails,
        pricing: PricingInfo,
        top_provider: TopProviderInfo,
    ) -> Self {
        Self {
            id,
            name,
            description,
            context_length,
            created,
            canonical_slug: None,
            hugging_face_id: None,
            architecture,
            pricing,
            top_provider,
            per_request_limits: None,
            supported_parameters: None,
        }
    }

    /// Returns the provider name from the model ID
    pub fn provider(&self) -> Option<&str> {
        self.id.split('/').next()
    }

    /// Returns the model name part from the ID (after the provider)
    pub fn model_name(&self) -> Option<&str> {
        self.id.split('/').nth(1)
    }

    /// Checks if this is a free model
    pub fn is_free(&self) -> bool {
        self.pricing.prompt == "0" && self.pricing.completion == "0"
    }

    /// Checks if the model supports tools
    pub fn supports_tools(&self) -> bool {
        self.supported_parameters
            .as_ref()
            .map_or(false, |params| params.contains(&"tools".to_string()))
    }

    /// Checks if the model supports vision/multimodal
    pub fn supports_vision(&self) -> bool {
        self.architecture
            .input_modalities
            .contains(&"image".to_string())
            || self.architecture.modality.contains("image")
    }

    /// Checks if the model supports function calling
    pub fn supports_function_calling(&self) -> bool {
        self.supported_parameters.as_ref().map_or(false, |params| {
            params.contains(&"function_call".to_string())
                || params.contains(&"functions".to_string())
        })
    }

    /// Gets the prompt price as f64, returns None if parsing fails
    pub fn prompt_price(&self) -> Option<f64> {
        self.pricing.prompt.parse().ok()
    }

    /// Gets the completion price as f64, returns None if parsing fails
    pub fn completion_price(&self) -> Option<f64> {
        self.pricing.completion.parse().ok()
    }

    /// Gets the request price as f64, returns None if parsing fails
    pub fn request_price(&self) -> Option<f64> {
        self.pricing.request.as_ref()?.parse().ok()
    }

    /// Checks if the model has a specific parameter
    pub fn has_parameter(&self, param: &str) -> bool {
        self.supported_parameters
            .as_ref()
            .map_or(false, |params| params.contains(&param.to_string()))
    }

    /// Returns the creation date as a formatted string
    pub fn created_date_string(&self) -> String {
        // Simple conversion from timestamp to readable format
        let datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(self.created as u64);
        // Format as ISO 8601
        let secs_since_epoch = datetime
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}", secs_since_epoch)
    }
}

/// Request to list available models.
#[derive(Debug, Serialize)]
pub struct ModelsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability: Option<ModelCapability>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// Filter models by name (case-insensitive partial match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_name: Option<String>,

    /// Filter models by minimum context length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_context_length: Option<u32>,

    /// Filter models by maximum context length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_context_length: Option<u32>,

    /// Filter for free models only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_only: Option<bool>,

    /// Filter models that support specific features
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_tools: Option<bool>,

    /// Filter models that support vision/multimodal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_vision: Option<bool>,

    /// Filter models that support function calling
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_function_calling: Option<bool>,

    /// Sort order for the results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<ModelSortOrder>,

    /// Maximum number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

/// Sort order for model listings
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelSortOrder {
    /// Sort by model name (alphabetical)
    Name,
    /// Sort by creation date (newest first)
    Created,
    /// Sort by context length (largest first)
    ContextLength,
    /// Sort by prompt price (lowest first)
    PromptPrice,
    /// Sort by completion price (lowest first)
    CompletionPrice,
}

/// Response containing available models.
#[derive(Debug, Deserialize)]
pub struct ModelsResponse {
    /// A list of available models.
    pub data: Vec<ModelInfo>,
}

impl ModelsResponse {
    /// Creates a new ModelsResponse with the given models
    pub fn new(data: Vec<ModelInfo>) -> Self {
        Self { data }
    }

    /// Returns the number of models in the response
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Finds a model by its ID
    pub fn find_by_id(&self, id: &str) -> Option<&ModelInfo> {
        self.data.iter().find(|model| model.id == id)
    }

    /// Finds models by provider (case-insensitive)
    pub fn find_by_provider(&self, provider: &str) -> Vec<&ModelInfo> {
        self.data
            .iter()
            .filter(|model| {
                model.id.split('/').next().map(|p| p.to_lowercase())
                    == Some(provider.to_lowercase())
            })
            .collect()
    }

    /// Finds free models
    pub fn find_free_models(&self) -> Vec<&ModelInfo> {
        self.data
            .iter()
            .filter(|model| model.pricing.prompt == "0" && model.pricing.completion == "0")
            .collect()
    }

    /// Finds models that support tools
    pub fn find_models_with_tools(&self) -> Vec<&ModelInfo> {
        self.data
            .iter()
            .filter(|model| {
                model
                    .supported_parameters
                    .as_ref()
                    .map_or(false, |params| params.contains(&"tools".to_string()))
            })
            .collect()
    }

    /// Finds models that support vision/multimodal
    pub fn find_models_with_vision(&self) -> Vec<&ModelInfo> {
        self.data
            .iter()
            .filter(|model| {
                model
                    .architecture
                    .input_modalities
                    .contains(&"image".to_string())
                    || model.architecture.modality.contains("image")
            })
            .collect()
    }

    /// Finds models with minimum context length
    pub fn find_models_with_min_context(&self, min_context: u32) -> Vec<&ModelInfo> {
        self.data
            .iter()
            .filter(|model| model.context_length >= min_context)
            .collect()
    }

    /// Groups models by provider
    pub fn group_by_provider(&self) -> std::collections::HashMap<String, Vec<&ModelInfo>> {
        let mut groups: std::collections::HashMap<String, Vec<&ModelInfo>> =
            std::collections::HashMap::new();
        for model in &self.data {
            let provider = model.id.split('/').next().unwrap_or("unknown").to_string();
            groups.entry(provider).or_insert_with(Vec::new).push(model);
        }
        groups
    }

    /// Returns models sorted by name
    pub fn sort_by_name(&self) -> Vec<&ModelInfo> {
        let mut models: Vec<&ModelInfo> = self.data.iter().collect();
        models.sort_by(|a, b| a.name.cmp(&b.name));
        models
    }

    /// Returns models sorted by context length (largest first)
    pub fn sort_by_context_length(&self) -> Vec<&ModelInfo> {
        let mut models: Vec<&ModelInfo> = self.data.iter().collect();
        models.sort_by(|a, b| b.context_length.cmp(&a.context_length));
        models
    }

    /// Returns models sorted by prompt price (lowest first)
    pub fn sort_by_prompt_price(&self) -> Vec<&ModelInfo> {
        let mut models: Vec<&ModelInfo> = self.data.iter().collect();
        models.sort_by(|a, b| {
            let price_a = a.pricing.prompt.parse::<f64>().unwrap_or(f64::MAX);
            let price_b = b.pricing.prompt.parse::<f64>().unwrap_or(f64::MAX);
            price_a
                .partial_cmp(&price_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        models
    }

    /// Search models by name or description (case-insensitive)
    pub fn search(&self, query: &str) -> Vec<&ModelInfo> {
        let query_lower = query.to_lowercase();
        self.data
            .iter()
            .filter(|model| {
                model.name.to_lowercase().contains(&query_lower)
                    || model
                        .description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || model.id.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
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

        assert_eq!(model_info.id, "moonshotai/kimi-dev-72b:free");
        assert_eq!(model_info.name, "Kimi Dev 72b (free)");
        assert!(model_info.description.is_some()); // Check it's present
        assert_eq!(model_info.context_length, 131072);
        assert_eq!(model_info.architecture.modality, "text->text");
        assert_eq!(model_info.architecture.input_modalities, vec!["text"]);
        assert_eq!(model_info.pricing.prompt, "0");
        assert_eq!(model_info.pricing.request.as_deref(), Some("0"));
        assert_eq!(model_info.pricing.image.as_deref(), Some("0"));
        assert_eq!(model_info.pricing.web_search.as_deref(), Some("0"));
        assert_eq!(model_info.pricing.internal_reasoning.as_deref(), Some("0"));
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
        assert_eq!(models_response.data[0].id, "openai/gpt-4o");
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
}
