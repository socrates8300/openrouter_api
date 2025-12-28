//! Main validation module tests

#[cfg(test)]
mod validation_tests {
    use crate::types::chat::{ChatCompletionRequest, Message};
    use crate::types::completion::CompletionRequest;
    use crate::types::web_search::WebSearchRequest;
    use crate::utils::validation::{
        check_prompt_token_limits, check_token_limits, estimate_query_complexity,
        validate_chat_request, validate_completion_request, validate_date_format,
        validate_model_id, validate_non_empty_string, validate_numeric_range,
        validate_sampling_parameters, validate_string_length, validate_web_search_request,
    };

    #[test]
    fn test_validation_module_exports() {
        // Test that commonly used functions are available
        let _validate_fn = validate_chat_request as fn(_) -> _;
        let _check_fn = check_token_limits as fn(_) -> _;
        let _completion_fn = validate_completion_request as fn(_) -> _;
        let _search_fn = validate_web_search_request as fn(_) -> _;
    }

    #[test]
    fn test_common_validation_utilities() {
        // Test string validation
        assert!(validate_non_empty_string("hello", "test").is_ok());
        assert!(validate_non_empty_string("", "test").is_err());
        assert!(validate_non_empty_string("   ", "test").is_err());

        // Test string length validation
        assert!(validate_string_length("hello", "test", 1, 10).is_ok());
        assert!(validate_string_length("hello", "test", 6, 10).is_err());

        // Test numeric validation
        assert!(validate_numeric_range(5, "test", 1, 10).is_ok());
        assert!(validate_numeric_range(0, "test", 1, 10).is_err());

        // Test model validation
        assert!(validate_model_id("openai/gpt-4").is_ok());
        assert!(validate_model_id("invalid-model").is_err());

        // Test date validation
        assert!(validate_date_format("2024-01-15", "test").is_ok());
        assert!(validate_date_format("2024-13-15", "test").is_err());
    }

    #[test]
    fn test_chat_validation_integration() {
        let request = ChatCompletionRequest {
            model: "openai/gpt-4".to_string(),
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
        };

        assert!(validate_chat_request(&request).is_ok());
        assert!(check_token_limits(&request).is_ok());
    }

    #[test]
    fn test_completion_validation_integration() {
        let request = CompletionRequest {
            model: "openai/gpt-4".to_string(),
            prompt: "Once upon a time,".to_string(),
            extra_params: serde_json::json!({"temperature": 0.7}),
        };

        assert!(validate_completion_request(&request).is_ok());
        assert!(check_prompt_token_limits(&request.prompt, &request.model).is_ok());
    }

    #[test]
    fn test_web_search_validation_integration() {
        let request = WebSearchRequest {
            query: "rust programming language".to_string(),
            num_results: Some(10),
        };

        assert!(validate_web_search_request(&request).is_ok());
        assert!(estimate_query_complexity(&request.query) >= 1);
    }

    #[test]
    fn test_validation_error_messages() {
        // Test that error messages are descriptive
        let result = validate_model_id("");
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("empty") || error_msg.contains("required"));

        let result = validate_non_empty_string("", "test_field");
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("test_field"));
    }

    #[test]
    fn test_validation_performance() {
        // Test that validation doesn't add significant overhead
        let start = std::time::Instant::now();

        for _ in 0..1000 {
            let request = ChatCompletionRequest {
                model: "openai/gpt-4".to_string(),
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
            };
            let _ = validate_chat_request(&request);
        }

        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000, "Validation should be fast"); // Less than 1 second for 1000 validations
    }

    #[test]
    fn test_validation_edge_cases() {
        // Test edge cases that might cause issues

        // Very long strings
        let long_string = "a".repeat(1000);
        assert!(validate_string_length(&long_string, "test", 1, 1000).is_ok());
        assert!(validate_string_length(&long_string, "test", 1, 999).is_err());

        // Numeric boundaries
        assert!(validate_numeric_range(i64::MIN, "test", i64::MIN, i64::MAX).is_ok());
        assert!(validate_numeric_range(i64::MAX, "test", i64::MIN, i64::MAX).is_ok());

        // Unicode content
        let unicode_content = "🦀 Rust 编程语言";
        assert!(validate_non_empty_string(unicode_content, "test").is_ok());
    }

    #[test]
    fn test_validation_consistency() {
        // Test that similar validations behave consistently across endpoints

        let model = "openai/gpt-4";

        // All endpoints should validate model ID the same way
        assert!(validate_model_id(model).is_ok());

        // Test completion validation instead since ChatCompletionRequest doesn't have Default
        let completion_request = CompletionRequest {
            model: model.to_string(),
            prompt: "Hello".to_string(),
            extra_params: serde_json::json!({}),
        };
        assert!(validate_completion_request(&completion_request).is_ok());

        let invalid_completion_request = CompletionRequest {
            model: "invalid".to_string(),
            prompt: "Hello".to_string(),
            extra_params: serde_json::json!({}),
        };
        assert!(validate_completion_request(&invalid_completion_request).is_err());
    }

    #[test]
    fn test_sampling_parameters_validation() {
        // Test that sampling parameter validation is consistent

        let valid_params = (Some(0.7), Some(0.9), Some(40), Some(0.5), Some(0.3));
        assert!(validate_sampling_parameters(
            valid_params.0,
            valid_params.1,
            valid_params.2,
            valid_params.3,
            valid_params.4
        )
        .is_ok());

        // Test invalid temperature
        assert!(validate_sampling_parameters(Some(3.0), None, None, None, None).is_err());

        // Test invalid top_p
        assert!(validate_sampling_parameters(None, Some(0.0), None, None, None).is_err());

        // Test invalid top_k
        assert!(validate_sampling_parameters(None, None, Some(0), None, None).is_ok()); // 0 is allowed (disabled)
        assert!(validate_sampling_parameters(None, None, Some(0), None, None).is_ok());
        // 0 is allowed
    }
}
