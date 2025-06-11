/*!
   # Request Builder Module

   This module provides a unified request builder for non‑interactive endpoints.
   It supports:

   - **Structured Outputs:** Clients can enable structured outputs by supplying a JSON Schema configuration,
     allowing the model response to be validated against a predefined schema.

   - **Tool Calling:** Clients can include a list of tools (callable functions) in the request payload.
     This enables the model to suggest or invoke external functions via the API.

   - **Provider Preferences:** Clients can attach routing options using our first‑class provider preferences.
*/

use crate::models::structured::JsonSchemaConfig;
use crate::models::tool::Tool;
use crate::types::chat::Message;
use serde::Serialize;
use serde_json::Value;

/// Representation for enabling structured outputs in the request payload.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFormatConfig {
    /// Fixed string indicating the type of response format.
    #[serde(rename = "type")]
    pub format_type: String,
    /// The JSON Schema configuration used to validate the model's response.
    pub json_schema: JsonSchemaConfig,
}

/// Payload sent to the API. It is generic over extra parameters.
///
/// This payload includes:
/// - The model to use.
/// - The messages to send.
/// - Optionally, structured output configuration.
/// - Optionally, tool calling instructions.
/// - Any extra parameters merged using flattening.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPayload<T: Serialize> {
    /// The model ID to use.
    pub model: String,
    /// The list of chat messages.
    pub messages: Vec<Message>,
    /// Optional structured output configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormatConfig>,
    /// Optional tool calling instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Additional parameters merged into the request payload.
    #[serde(flatten)]
    pub extra_params: T,
}

/// A unified request builder for non‑interactive endpoints.
///
/// This builder supports configuration for:
/// - **Structured Outputs:** By invoking [with_structured_output], clients can enable a specific JSON Schema for responses.
/// - **Tool Calling:** By invoking [with_tools], clients can supply a list of callable tools.
/// - **Provider Preferences:** By invoking [with_provider_preferences], clients can configure provider routing.
pub struct RequestBuilder<T: Serialize> {
    model: String,
    messages: Vec<Message>,
    extra_params: T,
    structured_output: Option<ResponseFormatConfig>,
    /// Optional list of tools for tool calling.
    tools: Option<Vec<Tool>>,
    /// Whether to perform JSON Schema validation on the response.
    pub validate_structured: bool,
    /// If true, fallback to an unstructured response on validation failure.
    pub fallback_on_failure: bool,
}

impl<T: Serialize> RequestBuilder<T> {
    /// Creates a new request builder.
    ///
    /// # Parameters
    ///
    /// - `model`: The model ID to be used for the request.
    /// - `messages`: A vector of [Message] objects representing the conversation.
    /// - `extra_params`: Any additional parameters to include in the payload.
    pub fn new(model: impl Into<String>, messages: Vec<Message>, extra_params: T) -> Self {
        Self {
            model: model.into(),
            messages,
            extra_params,
            structured_output: None,
            tools: None,
            validate_structured: true,
            fallback_on_failure: false,
        }
    }

    /// Enables structured output support.
    ///
    /// # Parameters
    ///
    /// - `config`: A [JsonSchemaConfig] defining the expected structure of the response.
    /// - `validate`: Whether to perform JSON Schema validation against the response.
    /// - `fallback`: Whether to fallback to an unstructured response if validation fails.
    pub fn with_structured_output(
        mut self,
        config: JsonSchemaConfig,
        validate: bool,
        fallback: bool,
    ) -> Self {
        self.structured_output = Some(ResponseFormatConfig {
            format_type: "json_schema".to_string(),
            json_schema: config,
        });
        self.validate_structured = validate;
        self.fallback_on_failure = fallback;
        self
    }

    /// Enables tool calling by providing a list of tools.
    ///
    /// # Parameters
    ///
    /// - `tools`: A vector of [Tool] objects representing callable functions.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Consumes the builder and returns the complete request payload.
    ///
    /// # Returns
    ///
    /// A [RequestPayload] that includes the model, messages, structured output settings,
    /// tool calling instructions, and extra parameters.
    pub fn build(self) -> RequestPayload<T> {
        RequestPayload {
            model: self.model,
            messages: self.messages,
            response_format: self.structured_output,
            tools: self.tools,
            extra_params: self.extra_params,
        }
    }
}

/// Extension methods when extra parameters are represented as a serde_json::Value.
impl RequestBuilder<Value> {
    /// Adds provider preferences into the request payload.
    ///
    /// This method accepts a strongly‑typed [ProviderPreferences] instance and serializes it
    /// into the JSON payload under the "provider" key. It validates the preferences and returns
    /// an error if validation fails.
    ///
    /// # Returns
    ///
    /// Returns `Ok(Self)` if the preferences are valid, or an `Err` with a descriptive error
    /// if validation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use openrouter_api::api::request::RequestBuilder;
    /// use openrouter_api::models::provider_preferences::{DataCollection, ProviderPreferences, ProviderSort};
    /// use serde_json::json;
    ///
    /// let prefs = ProviderPreferences {
    ///     order: Some(vec!["OpenAI".to_string(), "Anthropic".to_string()]),
    ///     allow_fallbacks: Some(true),
    ///     require_parameters: None,
    ///     data_collection: Some(DataCollection::Allow),
    ///     ignore: None,
    ///     quantizations: None,
    ///     sort: Some(ProviderSort::Throughput),
    /// };
    ///
    /// let builder = RequestBuilder::new("openai/gpt-4o", vec![], json!({}))
    ///     .with_provider_preferences(prefs)
    ///     .expect("Valid provider preferences");
    /// ```
    pub fn with_provider_preferences(
        mut self,
        preferences: crate::models::provider_preferences::ProviderPreferences,
    ) -> Result<Self, crate::error::Error> {
        // Validate the preferences
        preferences.validate()?;

        // Serialize to JSON
        let provider_value =
            serde_json::to_value(preferences).map_err(crate::error::Error::SerializationError)?;

        // Add to the extra params
        if let Value::Object(ref mut map) = self.extra_params {
            map.insert("provider".to_string(), provider_value);
        }

        Ok(self)
    }
}
