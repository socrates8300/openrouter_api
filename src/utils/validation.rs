//! Validation utilities for request and response objects.

use crate::error::{Error, Result};
use crate::models::tool::Tool;
use crate::types::chat::{ChatCompletionRequest, Message};
use std::collections::HashSet;

/// Maximum allowed tokens in a chat completion request
const MAX_TOKENS: u32 = 32_000;

/// Validates a chat completion request for common errors.
pub fn validate_chat_request(request: &ChatCompletionRequest) -> Result<()> {
    // Validate model is not empty
    if request.model.trim().is_empty() {
        return Err(Error::ConfigError("Model ID cannot be empty".into()));
    }

    // Validate messages are present
    if request.messages.is_empty() {
        return Err(Error::ConfigError("Messages array cannot be empty".into()));
    }

    // Validate message roles
    for (i, msg) in request.messages.iter().enumerate() {
        validate_message(msg, i)?;
    }

    // Validate tools if present
    if let Some(tools) = &request.tools {
        validate_tools(tools)?;
    }

    Ok(())
}

/// Validates a single message for errors.
fn validate_message(message: &Message, index: usize) -> Result<()> {
    // Role validation
    match message.role.as_str() {
        "user" | "assistant" | "system" | "tool" => {}
        _ => return Err(Error::ConfigError(format!(
            "Invalid role at message[{}]: '{}'. Must be 'user', 'assistant', 'system', or 'tool'",
            index, message.role
        ))),
    }

    // Content validation
    if message.content.trim().is_empty() && message.tool_calls.is_none() {
        return Err(Error::ConfigError(format!(
            "Message at index {} must have either non-empty content or tool_calls",
            index
        )));
    }

    // Tool calls validation for assistant messages
    if let Some(tool_calls) = &message.tool_calls {
        if message.role != "assistant" {
            return Err(Error::ConfigError(format!(
                "Message at index {} has tool_calls but role is '{}', not 'assistant'",
                index, message.role
            )));
        }

        // Validate each tool call
        for (tc_idx, tc) in tool_calls.iter().enumerate() {
            if tc.id.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "Tool call {} at message {} has empty id",
                    tc_idx, index
                )));
            }

            if tc.kind != "function" {
                return Err(Error::ConfigError(format!(
                    "Tool call {} at message {} has invalid type: '{}'. Must be 'function'",
                    tc_idx, index, tc.kind
                )));
            }

            if tc.function_call.name.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "Function name in tool call {} at message {} cannot be empty",
                    tc_idx, index
                )));
            }
        }
    }

    Ok(())
}

/// Validates tools in a request.
fn validate_tools(tools: &[Tool]) -> Result<()> {
    if tools.is_empty() {
        return Ok(());
    }

    // Check for duplicate function names
    let mut function_names = HashSet::new();

    for (i, tool) in tools.iter().enumerate() {
        match tool {
            Tool::Function { function } => {
                if function.name.trim().is_empty() {
                    return Err(Error::ConfigError(format!(
                        "Function name in tool[{}] cannot be empty",
                        i
                    )));
                }

                if !function_names.insert(&function.name) {
                    return Err(Error::ConfigError(format!(
                        "Duplicate function name '{}' in tools",
                        function.name
                    )));
                }

                // Validate parameters schema
                if !function.parameters.is_object() {
                    return Err(Error::ConfigError(format!(
                        "Parameters for function '{}' must be a JSON object",
                        function.name
                    )));
                }
            }
        }
    }

    Ok(())
}

/// Estimates token count for a message (rough approximation).
pub fn estimate_message_tokens(message: &Message) -> u32 {
    // Very rough approximation: 1 token per 4 characters
    let content_tokens = message.content.len() as u32 / 4;

    // Add tokens for role
    let role_tokens = 3; // Typically "user", "assistant" or "system" is 1-3 tokens

    // Add tokens for tool calls if present
    let tool_call_tokens = if let Some(tool_calls) = &message.tool_calls {
        tool_calls
            .iter()
            .map(|tc| {
                // Function name + arguments
                let name_tokens = tc.function_call.name.len() as u32 / 4;
                let args_tokens = tc.function_call.arguments.len() as u32 / 4;
                name_tokens + args_tokens + 10 // Additional overhead
            })
            .sum()
    } else {
        0
    };

    role_tokens + content_tokens + tool_call_tokens
}

/// Estimates total token count for a request (rough approximation).
pub fn estimate_request_tokens(request: &ChatCompletionRequest) -> u32 {
    // Sum tokens from all messages
    let message_tokens: u32 = request.messages.iter().map(estimate_message_tokens).sum();

    // Add overhead for request structure
    let overhead_tokens = 10;

    // Add tokens for tools if present
    let tool_tokens = if let Some(tools) = &request.tools {
        tools
            .iter()
            .map(|tool| {
                match tool {
                    Tool::Function { function } => {
                        // Function name + description + parameters
                        let name_tokens = function.name.len() as u32 / 4;
                        let desc_tokens = function
                            .description
                            .as_ref()
                            .map(|d| d.len() as u32 / 4)
                            .unwrap_or(0);
                        let params_tokens = serde_json::to_string(&function.parameters)
                            .map(|s| s.len() as u32 / 4)
                            .unwrap_or(0);
                        name_tokens + desc_tokens + params_tokens + 10
                    }
                }
            })
            .sum()
    } else {
        0
    };

    message_tokens + overhead_tokens + tool_tokens
}

/// Checks if a request might exceed token limits.
pub fn check_token_limits(request: &ChatCompletionRequest) -> Result<()> {
    let estimated_tokens = estimate_request_tokens(request);

    if estimated_tokens > MAX_TOKENS {
        return Err(Error::ContextLengthExceeded {
            model: request.model.clone(),
            message: format!(
                "Estimated token count ({}) exceeds maximum context length ({})",
                estimated_tokens, MAX_TOKENS
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
#[path = "validation_tests.rs"]
mod validation_tests;
