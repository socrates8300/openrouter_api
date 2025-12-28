/*!
   # Tool Calling Module

   This module defines the data types required for tool calling functionality. It includes types for representing function descriptions, callable tools, tool calls returned in API responses, and tool selection options.

   ## Overview

   - **FunctionDescription:** Describes a callable function with a name, optional description, and a JSON Schema for its parameters.
   - **Tool:** An enum representing available types of tools. Currently, only function‑type tools are supported.
   - **FunctionCall:** Represents the details of a requested tool call including the function name and JSON‑encoded arguments.
   - **ToolCall:** Captures the tool call details as returned by the API, including a unique identifier and the associated function call details.
   - **ToolChoice:** Represents the possible outcomes when the model must select a tool (for example, "none", "auto", or a specific function choice).
   - **FunctionName:** A simple structure to represent a function name for tool selection.
*/

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::ids::ToolCallId;

/// Represents a description for a callable function (tool).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDescription {
    /// The name of the function.
    pub name: String,
    /// An optional description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A JSON Schema object representing the function parameters.
    /// This should be a valid JSON object describing the expected arguments.
    pub parameters: Value,
    pub strict: Option<bool>,
}

/// Encapsulates a tool that the model can call.
///
/// Currently, only function‑type tools are supported.
/// In the future, this enum could be extended for other tool types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Tool {
    /// A function call tool with an associated [FunctionDescription].
    Function {
        #[serde(rename = "function")]
        function: FunctionDescription,
    },
}

/// Represents the type of tool call.
///
/// Currently only function-type tools are supported. This enum makes invalid tool types
/// unrepresentable at compile time, preventing errors from typos or unknown values.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    /// A function call tool.
    Function,
}

impl std::fmt::Display for ToolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolType::Function => write!(f, "function"),
        }
    }
}

/// Represents a specific function call requested by the model.
///
/// The `arguments` field is a JSON‑encoded string that should be parseable into a structured object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    /// The name of function to call.
    pub name: String,
    /// A JSON string representing the arguments for the function call.
    pub arguments: String,
}

/// Represents tool call details returned by the API.
///
/// This structure appears in responses when the model indicates that a tool should be invoked.
/// The `kind` field is a type-safe enum that only allows valid tool types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// A unique identifier for the tool call.
    pub id: ToolCallId,
    /// The type of call. Only `ToolType::Function` is currently supported.
    #[serde(rename = "type")]
    pub kind: ToolType,
    /// The details of the function call, including its function name and arguments.
    #[serde(rename = "function")]
    pub function_call: FunctionCall,
}

/// Represents a chunk of a function call as streamed from the API.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FunctionCallChunk {
    /// The name of the function to call. Appears in the first chunk for a given function call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A JSON string representing the arguments for the function call. Can be streamed in parts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Represents a chunk of a tool call as streamed from the API.
/// Fields are optional to accommodate partial data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallChunk {
    /// The index of the tool call in the list of tool calls.
    pub index: u32,
    /// A unique identifier for the tool call. Appears in the first chunk for a given tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of call. Only `ToolType::Function` is currently supported.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ToolType>,
    /// The details of the function call, including its function name and arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<FunctionCallChunk>,
}

/// Represents a tool selection option when model must choose among available tools.
///
/// This enum covers three cases:
/// - **None:** No tool is selected (represented by a string, e.g. "none").
/// - **Auto:** The model automatically selects a tool (represented as "auto").
/// - **FunctionChoice:** A specific function is selected. The `kind` field uses the type-safe `ToolType` enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tool is selected.
    None(String),
    /// The model automatically selects a tool.
    Auto(String),
    /// A specific function is selected.
    FunctionChoice {
        #[serde(rename = "type")]
        kind: ToolType,
        function: FunctionName,
    },
}

/// A simple struct to represent a function name for tool selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionName {
    /// The name of the function.
    pub name: String,
}
