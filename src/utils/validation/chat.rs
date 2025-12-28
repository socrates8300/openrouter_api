//! Validation utilities for chat completion requests

use crate::error::{Error, Result};
use crate::models::tool::Tool;

use crate::types::chat::{ChatCompletionRequest, ContentPart, Message, MessageContent};
use std::collections::HashSet;

/// Maximum allowed tokens in a chat completion request
const MAX_TOKENS: u32 = 1_000_000;

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

    // Validate sampling parameters
    validate_sampling_parameters(request)?;

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

/// Validates sampling parameters for valid ranges.
fn validate_sampling_parameters(request: &ChatCompletionRequest) -> Result<()> {
    // Temperature: [0.0, 2.0]
    if let Some(temp) = request.temperature {
        if !(0.0..=2.0).contains(&temp) {
            return Err(Error::ConfigError(format!(
                "Temperature must be between 0.0 and 2.0, got {}",
                temp
            )));
        }
    }

    // Top P: (0.0, 1.0]
    if let Some(top_p) = request.top_p {
        if top_p <= 0.0 || top_p > 1.0 {
            return Err(Error::ConfigError(format!(
                "Top P must be between 0.0 (exclusive) and 1.0 (inclusive), got {}",
                top_p
            )));
        }
    }

    // Top K: [1, ∞) or 0 (disabled)
    if let Some(top_k) = request.top_k {
        if top_k != 0 && top_k < 1 {
            return Err(Error::ConfigError(format!(
                "Top K must be 0 (disabled) or >= 1, got {}",
                top_k
            )));
        }
    }

    // Frequency Penalty: [-2.0, 2.0]
    if let Some(fp) = request.frequency_penalty {
        if !(-2.0..=2.0).contains(&fp) {
            return Err(Error::ConfigError(format!(
                "Frequency penalty must be between -2.0 and 2.0, got {}",
                fp
            )));
        }
    }

    // Presence Penalty: [-2.0, 2.0]
    if let Some(pp) = request.presence_penalty {
        if !(-2.0..=2.0).contains(&pp) {
            return Err(Error::ConfigError(format!(
                "Presence penalty must be between -2.0 and 2.0, got {}",
                pp
            )));
        }
    }

    // Repetition Penalty: (0.0, 2.0]
    if let Some(rp) = request.repetition_penalty {
        if rp <= 0.0 || rp > 2.0 {
            return Err(Error::ConfigError(format!(
                "Repetition penalty must be between 0.0 (exclusive) and 2.0 (inclusive), got {}",
                rp
            )));
        }
    }

    // Min P: [0.0, 1.0]
    if let Some(min_p) = request.min_p {
        if !(0.0..=1.0).contains(&min_p) {
            return Err(Error::ConfigError(format!(
                "Min P must be between 0.0 and 1.0, got {}",
                min_p
            )));
        }
    }

    // Top A: [0.0, 1.0]
    if let Some(top_a) = request.top_a {
        if !(0.0..=1.0).contains(&top_a) {
            return Err(Error::ConfigError(format!(
                "Top A must be between 0.0 and 1.0, got {}",
                top_a
            )));
        }
    }

    // Top Logprobs: [0, 20]
    if let Some(tlp) = request.top_logprobs {
        if tlp > 20 {
            return Err(Error::ConfigError(format!(
                "Top logprobs must be <= 20, got {}",
                tlp
            )));
        }
    }

    Ok(())
}

/// Validates a single message for errors.
fn validate_message(message: &Message, index: usize) -> Result<()> {
    // Role validation - ChatRole enum ensures only valid values
    match message.role {
        crate::types::chat::ChatRole::User
        | crate::types::chat::ChatRole::Assistant
        | crate::types::chat::ChatRole::System
        | crate::types::chat::ChatRole::Tool => {}
    }

    // Content validation based on role
    validate_message_content(message, index)?;

    // Tool calls validation for assistant messages
    if let Some(tool_calls) = &message.tool_calls {
        if message.role != crate::types::chat::ChatRole::Assistant {
            return Err(Error::ConfigError(format!(
                "Message at index {} has tool_calls but role is '{}', not 'assistant'",
                index, message.role
            )));
        }

        // Validate each tool call
        for (tc_idx, tc) in tool_calls.iter().enumerate() {
            if tc.id.is_empty() {
                return Err(Error::ConfigError(format!(
                    "Tool call {tc_idx} at message {index} has empty id"
                )));
            }

            if tc.kind != crate::models::tool::ToolType::Function {
                return Err(Error::ConfigError(format!(
                    "Tool call {} at message {} has invalid type: '{}'. Must be 'function'",
                    tc_idx, index, tc.kind
                )));
            }

            if tc.function_call.name.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "Function name in tool call {tc_idx} at message {index} cannot be empty"
                )));
            }
        }
    }

    // Tool call ID validation for tool messages
    if message.role == crate::types::chat::ChatRole::Tool
        && (message.tool_call_id.is_none() || message.tool_call_id.as_ref().unwrap().is_empty())
    {
        return Err(Error::ConfigError(format!(
            "Tool message at index {} must have a non-empty tool_call_id",
            index
        )));
    }

    Ok(())
}

/// Validates message content based on role and content type.
fn validate_message_content(message: &Message, index: usize) -> Result<()> {
    match &message.content {
        MessageContent::Text(text) => {
            // For tool messages, content can be empty (some providers allow empty results)
            if message.role != crate::types::chat::ChatRole::Tool
                && text.trim().is_empty()
                && message.tool_calls.is_none()
            {
                return Err(Error::ConfigError(format!(
                    "Message at index {} must have either non-empty content or tool_calls",
                    index
                )));
            }
        }
        MessageContent::Parts(parts) => {
            // Multimodal content is only allowed for user messages
            if message.role != crate::types::chat::ChatRole::User {
                return Err(Error::ConfigError(format!(
                    "Multimodal content (ContentParts) is only allowed for user messages, got role '{}' at index {}",
                    message.role, index
                )));
            }

            // Validate each content part
            if parts.is_empty() {
                return Err(Error::ConfigError(format!(
                    "Content parts array cannot be empty for message at index {}",
                    index
                )));
            }

            for (part_idx, part) in parts.iter().enumerate() {
                validate_content_part(part, index, part_idx)?;
            }
        }
    }

    Ok(())
}

/// Validates a single content part.
fn validate_content_part(part: &ContentPart, msg_index: usize, part_index: usize) -> Result<()> {
    match part {
        ContentPart::Text(text_content) => {
            if text_content.text.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "Text content part {} at message {} cannot be empty",
                    part_index, msg_index
                )));
            }
        }
        ContentPart::Image(image_content) => {
            // Validate image URL
            if image_content.image_url.url.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "Image URL cannot be empty for image part {} at message {}",
                    part_index, msg_index
                )));
            }

            // Basic URL validation - should start with http://, https://, or data:image/
            let url = &image_content.image_url.url;
            if !(url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("data:image/"))
            {
                return Err(Error::ConfigError(format!(
                    "Image URL must be a valid HTTP(S) URL or base64 data URI for image part {} at message {}",
                    part_index, msg_index
                )));
            }
        }
        ContentPart::Audio(audio_content) => {
            if audio_content.audio_url.url.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "Audio URL cannot be empty for audio part {} at message {}",
                    part_index, msg_index
                )));
            }
        }
        ContentPart::File(file_content) => {
            if file_content.file_url.url.trim().is_empty() {
                return Err(Error::ConfigError(format!(
                    "File URL cannot be empty for file part {} at message {}",
                    part_index, msg_index
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
                        "Function name in tool[{i}] cannot be empty"
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
    let content_tokens = match &message.content {
        MessageContent::Text(text) => {
            // Very rough approximation: 1 token per 4 characters
            text.len() as u32 / 4
        }
        MessageContent::Parts(parts) => {
            // Estimate tokens for each part
            parts
                .iter()
                .map(|part| {
                    match part {
                        ContentPart::Text(text_content) => text_content.text.len() as u32 / 4,
                        ContentPart::Image(_) => {
                            // Images typically cost ~85-100 tokens each for vision models
                            85
                        }
                        ContentPart::Audio(_) => 100,
                        ContentPart::File(_) => 100,
                    }
                })
                .sum()
        }
    };

    // Add tokens for role
    let role_tokens = 3; // Typically "user", "assistant", "system" or "tool" is 1-3 tokens

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

    // Add tokens for tool call ID if present
    let tool_call_id_tokens = if let Some(tool_call_id) = &message.tool_call_id {
        tool_call_id.as_str().len() as u32 / 4
    } else {
        0
    };

    role_tokens + content_tokens + tool_call_tokens + tool_call_id_tokens
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
                "Estimated token count ({estimated_tokens}) exceeds maximum context length ({MAX_TOKENS})"
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_chat_request() -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text(
                crate::types::chat::ChatRole::User,
                "Hello, world!",
            )],
            stream: None,
            response_format: None,
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
        }
    }

    #[test]
    fn test_validate_chat_request_valid() {
        let request = create_valid_chat_request();
        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chat_request_empty_model() {
        let mut request = create_valid_chat_request();
        request.model = "".to_string();
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_empty_messages() {
        let mut request = create_valid_chat_request();
        request.messages = vec![];
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_token_limits_moderate_content() {
        let mut request = create_valid_chat_request();
        // Create content that should be well within limits
        let moderate_content = "This is a moderate length message. ".repeat(100);
        request.messages[0].content = MessageContent::Text(moderate_content);
        let result = check_token_limits(&request);
        assert!(result.is_ok());
    }
}
