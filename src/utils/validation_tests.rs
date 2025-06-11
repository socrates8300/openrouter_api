//! Unit tests for validation utilities

#[cfg(test)]
mod tests {
    use crate::types::chat::{ChatCompletionRequest, Message};
    use crate::utils::validation::{check_token_limits, validate_chat_request};

    fn create_valid_chat_request() -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "Hello, world!".to_string(),
                name: None,
                tool_calls: None,
            }],
            stream: None,
            response_format: None,
            tools: None,
            provider: None,
            models: None,
            transforms: None,
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
    fn test_validate_chat_request_whitespace_model() {
        let mut request = create_valid_chat_request();
        request.model = "   ".to_string();
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
    fn test_validate_chat_request_invalid_role() {
        let mut request = create_valid_chat_request();
        request.messages[0].role = "invalid_role".to_string();
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_valid_roles() {
        let valid_roles = ["user", "assistant", "system"];

        for role in &valid_roles {
            let mut request = create_valid_chat_request();
            request.messages[0].role = role.to_string();
            let result = validate_chat_request(&request);
            assert!(result.is_ok(), "Role '{}' should be valid", role);
        }
    }

    #[test]
    fn test_validate_chat_request_empty_content() {
        let mut request = create_valid_chat_request();
        request.messages[0].content = "".to_string();
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_whitespace_content() {
        let mut request = create_valid_chat_request();
        request.messages[0].content = "   ".to_string();
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_multiple_messages() {
        let mut request = create_valid_chat_request();
        request.messages.push(Message {
            role: "assistant".to_string(),
            content: "Hello! How can I help you?".to_string(),
            name: None,
            tool_calls: None,
        });
        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chat_request_message_with_name() {
        let mut request = create_valid_chat_request();
        request.messages[0].name = Some("user_123".to_string());
        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_token_limits_within_limit() {
        let request = create_valid_chat_request();
        let result = check_token_limits(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_token_limits_very_long_content() {
        let mut request = create_valid_chat_request();
        // Create a message with approximately 50,000 tokens (rough estimate: 4 chars per token)
        let long_content = "word ".repeat(50_000);
        request.messages[0].content = long_content;
        let result = check_token_limits(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_token_limits_many_messages() {
        let mut request = create_valid_chat_request();
        // Add many messages with large content to definitely exceed token limit
        // Each message has ~400 characters = ~100 tokens, need 320+ messages to exceed 32k limit
        for i in 0..500 {
            request.messages.push(Message {
                role: "user".to_string(),
                content: format!(
                    "This is message number {} with a lot of content to consume many tokens. \
                    This content is intentionally verbose and repetitive to ensure we exceed \
                    the token limit for testing purposes. More text here to increase token count. \
                    Additional padding text to make sure we have enough tokens per message.",
                    i
                ),
                name: None,
                tool_calls: None,
            });
        }
        let result = check_token_limits(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_edge_case_empty_after_trim() {
        // Test with content that becomes empty after trimming
        let mut request = create_valid_chat_request();
        request.messages[0].content = "   \n\t   ".to_string();
        let result = validate_chat_request(&request);
        assert!(result.is_err()); // Should fail validation for empty content
    }

    #[test]
    fn test_check_token_limits_moderate_content() {
        let mut request = create_valid_chat_request();
        // Create content that should be well within limits
        let moderate_content = "This is a moderate length message. ".repeat(100);
        request.messages[0].content = moderate_content;
        let result = check_token_limits(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chat_request_complex_scenario() {
        let request = ChatCompletionRequest {
            model: "anthropic/claude-3-opus".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    tool_calls: None,
                },
                Message {
                    role: "user".to_string(),
                    content: "What is the weather like today?".to_string(),
                    name: Some("user_123".to_string()),
                    tool_calls: None,
                },
                Message {
                    role: "assistant".to_string(),
                    content: "I don't have access to real-time weather data.".to_string(),
                    name: None,
                    tool_calls: None,
                },
            ],
            stream: Some(false),
            response_format: Some("json".to_string()),
            tools: None,
            provider: None,
            models: None,
            transforms: None,
        };

        let validation_result = validate_chat_request(&request);
        assert!(validation_result.is_ok());

        let token_result = check_token_limits(&request);
        assert!(token_result.is_ok());
    }
}
