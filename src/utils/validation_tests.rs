//! Unit tests for validation utilities

#[cfg(test)]
mod tests {
    use crate::types::chat::{
        ChatCompletionRequest, ContentPart, ImageContent, ImageDetail, ImageUrl, Message,
        MessageContent, PredictionConfig, RouteStrategy, StopSequence, TextContent, VerbosityLevel,
    };
    use crate::utils::validation::{check_token_limits, validate_chat_request};
    use std::collections::HashMap;

    fn create_valid_chat_request() -> ChatCompletionRequest {
        ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello, world!")],
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
        }
    }

    #[test]
    fn test_validate_chat_request_valid() {
        let request = create_valid_chat_request();
        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_enhanced_chat_request_with_all_parameters() {
        let mut logit_bias = HashMap::new();
        logit_bias.insert(1000, -10.0);
        logit_bias.insert(2000, 5.0);

        let request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![
                Message::text("system", "You are a helpful assistant."),
                Message::text("user", "Hello!"),
            ],
            stream: Some(false),
            response_format: Some(serde_json::json!({
                "type": "json_object"
            })),
            tools: None,
            tool_choice: Some(serde_json::json!("auto")),
            provider: None,
            models: Some(vec![
                "openai/gpt-4o".to_string(),
                "anthropic/claude-3".to_string(),
            ]),
            transforms: Some(vec!["middle-out".to_string()]),
            route: Some(RouteStrategy::Fallback),
            user: Some("user123".to_string()),

            // Sampling parameters
            max_tokens: Some(1000),
            temperature: Some(0.7),
            top_p: Some(0.9),
            top_k: Some(40),
            frequency_penalty: Some(0.5),
            presence_penalty: Some(0.3),
            repetition_penalty: Some(1.1),
            min_p: Some(0.1),
            top_a: Some(0.2),
            seed: Some(42),
            stop: Some(StopSequence::Single("END".to_string())),

            // Advanced parameters
            logit_bias: Some(logit_bias),
            logprobs: Some(true),
            top_logprobs: Some(5),
            prediction: Some(PredictionConfig {
                prediction_type: "content".to_string(),
                content: "Expected response".to_string(),
            }),
            parallel_tool_calls: Some(true),
            verbosity: Some(VerbosityLevel::Medium),
        };

        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multimodal_message() {
        let message = Message::multimodal(
            "user",
            vec![
                ContentPart::Text(TextContent {
                    content_type: "text".to_string(),
                    text: "What do you see in this image?".to_string(),
                }),
                ContentPart::Image(ImageContent {
                    content_type: "image_url".to_string(),
                    image_url: ImageUrl {
                        url: "https://example.com/image.jpg".to_string(),
                        detail: Some(ImageDetail::High),
                    },
                }),
            ],
        );

        let request = ChatCompletionRequest {
            model: "openai/gpt-4-vision-preview".to_string(),
            messages: vec![message],
            ..create_valid_chat_request()
        };

        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tool_message() {
        let message = Message::tool("The result is 42", "call_123");

        let request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![
                Message::text("user", "What is 6 * 7?"),
                Message::assistant_with_tools(
                    Some("I need to calculate 6 * 7."),
                    vec![/* tool call */],
                ),
                message,
            ],
            ..create_valid_chat_request()
        };

        let result = validate_chat_request(&request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stop_sequence_variants() {
        let request_single = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello")],
            stop: Some(StopSequence::Single("END".to_string())),
            ..create_valid_chat_request()
        };
        assert!(validate_chat_request(&request_single).is_ok());

        let request_multiple = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello")],
            stop: Some(StopSequence::Multiple(vec![
                "END".to_string(),
                "STOP".to_string(),
            ])),
            ..create_valid_chat_request()
        };
        assert!(validate_chat_request(&request_multiple).is_ok());
    }

    #[test]
    fn test_parameter_validation() {
        // Test temperature bounds
        let request_temp_high = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello")],
            temperature: Some(2.5), // Too high
            ..create_valid_chat_request()
        };
        assert!(validate_chat_request(&request_temp_high).is_err());

        let request_temp_low = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello")],
            temperature: Some(-0.1), // Too low
            ..create_valid_chat_request()
        };
        assert!(validate_chat_request(&request_temp_low).is_err());

        // Test top_p bounds
        let request_top_p_high = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello")],
            top_p: Some(1.5), // Too high
            ..create_valid_chat_request()
        };
        assert!(validate_chat_request(&request_top_p_high).is_err());

        let request_top_p_low = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello")],
            top_p: Some(0.0), // Too low
            ..create_valid_chat_request()
        };
        assert!(validate_chat_request(&request_top_p_low).is_err());
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
        request.messages[0].content = MessageContent::Text("".to_string());
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_whitespace_content() {
        let mut request = create_valid_chat_request();
        request.messages[0].content = MessageContent::Text("   ".to_string());
        let result = validate_chat_request(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_multiple_messages() {
        let mut request = create_valid_chat_request();
        request
            .messages
            .push(Message::text("assistant", "Hello! How can I help you?"));
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
        request.messages[0].content = MessageContent::Text(long_content);
        let result = check_token_limits(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_token_limits_many_messages() {
        let mut request = create_valid_chat_request();
        // Add many messages with large content to definitely exceed token limit
        // Each message has ~400 characters = ~100 tokens, need 320+ messages to exceed 32k limit
        for i in 0..500 {
            request.messages.push(Message::text(
                "user",
                format!(
                    "This is message number {} with a lot of content to consume many tokens. \
                    This content is intentionally verbose and repetitive to ensure we exceed \
                    the token limit for testing purposes. More text here to increase token count. \
                    Additional padding text to make sure we have enough tokens per message.",
                    i
                ),
            ));
        }
        let result = check_token_limits(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chat_request_edge_case_empty_after_trim() {
        // Test with content that becomes empty after trimming
        let mut request = create_valid_chat_request();
        request.messages[0].content = MessageContent::Text("   \n\t   ".to_string());
        let result = validate_chat_request(&request);
        assert!(result.is_err()); // Should fail validation for empty content
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

    #[test]
    fn test_validate_chat_request_complex_scenario() {
        let request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![
                Message::text("system", "You are a helpful assistant."),
                Message::text("user", "What is the weather like today?"),
                Message::text(
                    "assistant",
                    "I don't have access to real-time weather data.",
                ),
            ],
            response_format: Some(serde_json::Value::String("json".to_string())),
            ..create_valid_chat_request()
        };

        let validation_result = validate_chat_request(&request);
        assert!(validation_result.is_ok());

        let token_result = check_token_limits(&request);
        assert!(token_result.is_ok());
    }
}
