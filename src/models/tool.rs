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

/// Represents a description for a callable function (tool).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDescription {
    /// The name of the function.
    pub name: String,
    /// An optional description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// A JSON Schema object representing the function parameters.
    /// This should be a valid JSON object describing the expected arguments.
    pub parameters: Value,
}

/// Encapsulates a tool that the model can call.
///
/// Currently, only function‑type tools are supported.
/// In the future, this enum could be extended for other tool types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Tool {
    /// A function call tool with an associated [FunctionDescription].
    Function {
        #[serde(rename = "function")]
        function: FunctionDescription,
    },
}

/// Represents the specific function call requested by the model.
///
/// The `arguments` field is a JSON‑encoded string that should be parseable into a structured object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    /// The name of the function to call.
    pub name: Option<String>,
    /// A JSON string representing the arguments for the function call.
    pub arguments: String,
}

/// Represents the tool call details returned by the API.
///
/// This structure appears in responses when the model indicates that a tool should be invoked.
/// The `kind` field must be "function".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// A unique identifier for the tool call.
    pub id: Option<String>,
    /// The index of the tool call in the list of tool calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    /// The type of call. When streaming, the first chunk only will contain "function".
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// The details of the function call, including its function name and arguments.
    #[serde(rename = "function")]
    pub function_call: FunctionCall,
}

/// Represents a tool selection option when the model must choose among available tools.
///
/// This enum covers three cases:
/// - **None:** No tool is selected (represented by a string, e.g. "none").
/// - **Auto:** The model automatically selects a tool (represented as "auto").
/// - **FunctionChoice:** A specific function is selected. The `kind` field must be "function".
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
        kind: String,
        function: FunctionName,
    },
}

/// A simple struct to represent a function name for tool selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionName {
    /// The name of the function.
    pub name: String,
}
