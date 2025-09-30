//! Structured output API module for handling JSON schema-based responses

use crate::client::ClientConfig;
use crate::error::{Error, Result};
use crate::models::structured::JsonSchemaConfig;
use crate::types::chat::{ChatCompletionRequest, ChatCompletionResponse, Message};
use crate::utils::security::create_safe_error_message;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;

/// API endpoint for structured output generation.
pub struct StructuredApi {
    client: Client,
    config: ClientConfig,
}

impl StructuredApi {
    /// Creates a new StructuredApi with the given reqwest client and configuration.
    pub fn new(client: Client, config: &ClientConfig) -> Self {
        Self {
            client,
            config: config.clone(),
        }
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
            stream: Some(false),
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
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

        // Send the request
        let response = self
            .client
            .post(url)
            .headers(self.config.build_headers()?)
            .json(&body)
            .send()
            .await?;

        // Get the response status and body
        let status = response.status();
        let body = response.text().await?;

        // Check if the HTTP response is successful.
        if !status.is_success() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(&body, "Structured API request failed"),
                metadata: None,
            });
        }

        // Deserialize the JSON response
        let chat_response: ChatCompletionResponse =
            serde_json::from_str(&body).map_err(|e| Error::ApiError {
                code: status.as_u16(),
                message: create_safe_error_message(
                    &format!("Failed to decode JSON: {e}. Body was: {body}"),
                    "Structured JSON parsing error",
                ),
                metadata: None,
            })?;

        // Extract the content from the response
        if chat_response.choices.is_empty() {
            return Err(Error::ApiError {
                code: status.as_u16(),
                message: "No choices returned in response".into(),
                metadata: None,
            });
        }

        let content = &chat_response.choices[0].message.content;

        // Parse the content as JSON
        let json_result: Value = serde_json::from_str(content).map_err(|e| {
            Error::SchemaValidationError(format!("Failed to parse response as JSON: {e}"))
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
        // Check if schema is an object
        if !schema.is_object() {
            return Err(Error::SchemaValidationError(
                "Schema must be an object".into(),
            ));
        }

        let schema_obj = schema.as_object().unwrap();

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
