//! Structured output API module for handling JSON schema-based responses

use crate::error::{Error, Result};
use crate::models::structured::{JsonSchemaConfig, JsonSchemaDefinition};
use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse, Message, MessageContent};
use crate::types::status::StreamingStatus;
use crate::utils::{
    retry::execute_with_retry_builder, retry::handle_response_json,
    retry::operations::STRUCTURED_GENERATE,
};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// API endpoint for structured output generation.
pub struct StructuredApi {
    client: Client,
    config: crate::client::ApiConfig,
}

impl StructuredApi {
    /// Creates a new StructuredApi with the given reqwest client and configuration.
    #[must_use = "returns an API client that should be used for API calls"]
    pub fn new(client: Client, config: &crate::client::ClientConfig) -> Result<Self> {
        Ok(Self {
            client,
            config: config.to_api_config()?,
        })
    }

    /// Generates a structured output that conforms to the provided JSON schema.
    /// Returns the parsed response deserialized into the specified type T.
    pub async fn generate<T>(
        &self,
        model: &str,
        messages: Vec<Message>,
        schema_config: JsonSchemaConfig,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        // Build the request with structured output configuration
        let request = ChatCompletionRequest {
            model: model.to_string(),
            messages,
            stream: Some(StreamingStatus::NotStarted),
            response_format: Some(crate::api::request::ResponseFormatConfig {
                format_type: "json_schema".to_string(),
                json_schema: JsonSchemaConfig {
                    name: "structured_output".to_string(),
                    strict: false,
                    schema: JsonSchemaDefinition {
                        schema_type: "object".to_string(),
                        properties: serde_json::Map::new(),
                        required: None,
                        additional_properties: None,
                    },
                },
            }),
            tools: None,
            tool_choice: None,
            provider: None,
            models: None,
            transforms: None,
            route: None,
            user: None,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            min_p: None,
            top_a: None,
            seed: None,
            stop: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            prediction: None,
            parallel_tool_calls: None,
            verbosity: None,
            plugins: None,
        };

        // Build the complete URL for the chat completions endpoint.
        let url = self
            .config
            .base_url
            .join("chat/completions")
            .map_err(|e| Error::ApiError {
                code: 400,
                message: format!("Invalid URL: {e}"),
                metadata: None,
            })?;

        // Build the request body with the structured output schema
        let mut body = serde_json::to_value(&request).map_err(Error::SerializationError)?;
        body["response_format"] = serde_json::json!({
            "type": "json_schema",
            "json_schema": {
                "schema": schema_config.schema,
                "name": schema_config.name,
                "strict": schema_config.strict
            },
        });

        // Execute request with retry logic
        let response =
            execute_with_retry_builder(&self.config.retry_config, STRUCTURED_GENERATE, || {
                self.client
                    .post(url.clone())
                    .headers((*self.config.headers).clone())
                    .json(&body)
            })
            .await?;

        // Handle response with consistent error parsing
        let chat_response: ChatCompletionResponse =
            handle_response_json::<ChatCompletionResponse>(response, STRUCTURED_GENERATE).await?;

        // Extract the content from the response
        if chat_response.choices.is_empty() {
            return Err(Error::ApiError {
                code: 200,
                message: "No choices returned in response".into(),
                metadata: None,
            });
        }

        let content_str = match &chat_response.choices[0].message.content {
            MessageContent::Text(content) => content,
            MessageContent::Parts(_) => {
                return Err(Error::ApiError {
                    code: 200,
                    message: "Unexpected multimodal content in structured response".into(),
                    metadata: None,
                });
            }
        };

        // Parse the content as JSON
        let json_result: Value = serde_json::from_str(content_str).map_err(|e| {
            Error::SchemaValidationError(format!("Failed to parse response as JSON: {}", e))
        })?;

        // Basic validation of required fields if strict mode is enabled
        if schema_config.strict {
            // Convert schema_config.schema to a Value before validation
            let schema_value =
                serde_json::to_value(&schema_config.schema).map_err(Error::SerializationError)?;

            self.basic_schema_validation(&schema_value, &json_result)?;
        }

        // Deserialize the result into the target type
        serde_json::from_value::<T>(json_result).map_err(|e| {
            Error::SchemaValidationError(format!(
                "Failed to deserialize response into target type: {e}"
            ))
        })
    }

    /// Simple schema validation for required fields and top-level type checking
    fn basic_schema_validation(&self, schema: &Value, data: &Value) -> Result<()> {
        // Check if schema is an object and extract it in one operation
        let schema_obj = match schema.as_object() {
            Some(obj) => obj,
            None => {
                return Err(Error::SchemaValidationError(
                    "Schema must be an object".into(),
                ));
            }
        };

        // Check type
        if let Some(type_val) = schema_obj.get("type") {
            if let Some(type_str) = type_val.as_str() {
                match type_str {
                    "object" => {
                        if !data.is_object() {
                            return Err(Error::SchemaValidationError(
                                "Expected an object but received a different type".into(),
                            ));
                        }
                    }
                    "array" => {
                        if !data.is_array() {
                            return Err(Error::SchemaValidationError(
                                "Expected an array but received a different type".into(),
                            ));
                        }
                    }
                    "string" => {
                        if !data.is_string() {
                            return Err(Error::SchemaValidationError(
                                "Expected a string but received a different type".into(),
                            ));
                        }
                    }
                    "number" | "integer" => {
                        if !data.is_number() {
                            return Err(Error::SchemaValidationError(
                                "Expected a number but received a different type".into(),
                            ));
                        }
                    }
                    "boolean" => {
                        if !data.is_boolean() {
                            return Err(Error::SchemaValidationError(
                                "Expected a boolean but received a different type".into(),
                            ));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check required fields
        if let Some(required) = schema_obj.get("required") {
            if let Some(required_arr) = required.as_array() {
                let data_obj = match data.as_object() {
                    Some(obj) => obj,
                    None => return Ok(()), // Skip if not an object
                };

                for field in required_arr {
                    if let Some(field_str) = field.as_str() {
                        if !data_obj.contains_key(field_str) {
                            return Err(Error::SchemaValidationError(format!(
                                "Required field '{field_str}' is missing"
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use serde_json::json;

    #[test]
    fn test_basic_schema_validation_non_object_schema() {
        let schema = json!("not an object");
        let data = json!({"key": "value"});
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert_eq!(msg, "Schema must be an object");
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_object_type_mismatch() {
        let schema = json!({"type": "object"});
        let data = json!("not an object");
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Expected an object"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_array_type_mismatch() {
        let schema = json!({"type": "array"});
        let data = json!("not an array");
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Expected an array"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_string_type_mismatch() {
        let schema = json!({"type": "string"});
        let data = json!(123);
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Expected a string"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_number_type_mismatch() {
        let schema = json!({"type": "number"});
        let data = json!("not a number");
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Expected a number"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_boolean_type_mismatch() {
        let schema = json!({"type": "boolean"});
        let data = json!("not a boolean");
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Expected a boolean"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_missing_required_field() {
        let schema = json!({
            "type": "object",
            "required": ["title", "author"]
        });
        let data = json!({
            "title": "Test"
        });
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err());
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Required field 'author' is missing"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_basic_schema_validation_valid_object() {
        let schema = json!({
            "type": "object",
            "required": ["title", "author"]
        });
        let data = json!({
            "title": "Test",
            "author": "Author"
        });
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_schema_validation_valid_array() {
        let schema = json!({"type": "array"});
        let data = json!([1, 2, 3]);
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_schema_validation_unknown_type_skips() {
        let schema = json!({"type": "unknown_type"});
        let data = json!("any value");
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        // Unknown types should be skipped (not validated)
        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_schema_validation_no_type_field_skips() {
        let schema = json!({"required": ["field1"]});
        let data = json!({"field1": "value"});
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        // No type field means no type validation, but required fields still checked
        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_basic_schema_validation_non_object_data_skips_required() {
        let schema = json!({
            "type": "object",
            "required": ["field1"]
        });
        let data = json!("not an object");
        let api = StructuredApi::new(
            reqwest::Client::new(),
            &crate::client::ClientConfig::default(),
        ).unwrap();

        // Non-object data should skip required field check
        let result = api.basic_schema_validation(&schema, &data);
        assert!(result.is_err()); // But should fail type check
        match result {
            Err(Error::SchemaValidationError(msg)) => {
                assert!(msg.contains("Expected an object"));
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }
}
