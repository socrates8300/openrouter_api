use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A strongly‑typed representation of a JSON Schema definition.
/// This captures common validation properties.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JsonSchemaDefinition {
    /// JSON Schema type (typically "object").
    #[serde(rename = "type")]
    pub schema_type: String,
    /// A map of property names to their definitions.
    pub properties: Map<String, Value>,
    /// List of required property names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
    /// Indicates whether additional properties are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_properties: Option<bool>,
}

/// JSON Schema configuration for requesting structured outputs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JsonSchemaConfig {
    /// Name for schema, used to identify output type.
    pub name: String,
    /// If true, model response must strictly adhere to schema.
    pub strict: bool,
    /// The JSON Schema definition.
    pub schema: JsonSchemaDefinition,
}
