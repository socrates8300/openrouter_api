//! Unit tests for completion validation utilities

#[cfg(test)]
mod tests {
    use crate::types::completion::CompletionRequest;
    use crate::utils::validation::completion::estimate_prompt_tokens;
    use crate::utils::validation::{check_prompt_token_limits, validate_completion_request};
    use serde_json::json;

    fn create_valid_completion_request() -> CompletionRequest {
        CompletionRequest {
            model: "openai/gpt-4".to_string(),
            prompt: "Once upon a time,".to_string(),
            extra_params: serde_json::json!({}),
        }
    }

    #[test]
    fn test_validate_completion_request_valid() {
        let request = create_valid_completion_request();
        assert!(validate_completion_request(&request).is_ok());
    }

    #[test]
    fn test_validate_completion_request_empty_model() {
        let mut request = create_valid_completion_request();
        request.model = "".to_string();
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_completion_request_invalid_model_format() {
        let mut request = create_valid_completion_request();
        request.model = "invalid-model-name".to_string();
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_completion_request_empty_prompt() {
        let mut request = create_valid_completion_request();
        request.prompt = "".to_string();
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_completion_request_whitespace_prompt() {
        let mut request = create_valid_completion_request();
        request.prompt = "   ".to_string();
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_completion_request_prompt_too_long() {
        let mut request = create_valid_completion_request();
        request.prompt = "a".repeat(1_000_001);
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_completion_request_valid_extra_params() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({
            "temperature": 0.7,
            "max_tokens": 100,
            "top_p": 0.9,
            "frequency_penalty": 0.5,
            "presence_penalty": 0.3
        });
        assert!(validate_completion_request(&request).is_ok());
    }

    #[test]
    fn test_validate_completion_request_temperature_bounds() {
        let test_cases = [
            (-0.1, false), // Too low
            (0.0, true),   // Minimum valid
            (1.0, true),   // Valid
            (2.0, true),   // Maximum valid
            (2.1, false),  // Too high
        ];

        for (temp, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({"temperature": temp});

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Temperature {} should be valid", temp);
            } else {
                assert!(result.is_err(), "Temperature {} should be invalid", temp);
            }
        }
    }

    #[test]
    fn test_validate_completion_request_top_p_bounds() {
        let test_cases = [
            (0.0, false), // Too low (exclusive)
            (0.1, true),  // Valid
            (1.0, true),  // Maximum valid (inclusive)
            (1.1, false), // Too high
        ];

        for (top_p, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({"top_p": top_p});

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Top P {} should be valid", top_p);
            } else {
                assert!(result.is_err(), "Top P {} should be invalid", top_p);
            }
        }
    }

    #[test]
    fn test_validate_completion_request_max_tokens_bounds() {
        let test_cases = [
            (0, true),     // 0 means unlimited
            (1, true),     // Minimum valid
            (8192, true),  // Maximum valid
            (8193, false), // Too high
        ];

        for (max_tokens, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({"max_tokens": max_tokens});

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Max tokens {} should be valid", max_tokens);
            } else {
                assert!(
                    result.is_err(),
                    "Max tokens {} should be invalid",
                    max_tokens
                );
            }
        }
    }

    #[test]
    fn test_validate_completion_request_penalty_bounds() {
        let test_cases = [
            (-2.0, true), // Minimum valid
            (-1.0, true), // Valid
            (0.0, true),  // Valid
            (1.0, true),  // Valid
            (2.0, true),  // Maximum valid
            (2.1, false), // Too high
        ];

        for (penalty, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({
                "frequency_penalty": penalty,
                "presence_penalty": penalty
            });

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Penalty {} should be valid", penalty);
            } else {
                assert!(result.is_err(), "Penalty {} should be invalid", penalty);
            }
        }
    }

    #[test]
    fn test_validate_stop_sequence_string() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({"stop": "END"});
        assert!(validate_completion_request(&request).is_ok());
    }

    #[test]
    fn test_validate_stop_sequence_array() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({"stop": ["END", "STOP", "FINISHED"]});
        assert!(validate_completion_request(&request).is_ok());
    }

    #[test]
    fn test_validate_stop_sequence_too_many() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({"stop": ["A", "B", "C", "D", "E"]}); // 5 items, max is 4
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_stop_sequence_empty() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({"stop": ""});
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_logit_bias_valid() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({
            "logit_bias": {
                "1000": -10.0,
                "2000": 5.0,
                "3000": 0.0
            }
        });
        assert!(validate_completion_request(&request).is_ok());
    }

    #[test]
    fn test_validate_logit_bias_invalid_range() {
        let test_cases = [
            (-100.1, false), // Too low
            (-100.0, true),  // Minimum valid
            (0.0, true),     // Valid
            (100.0, true),   // Maximum valid
            (100.1, false),  // Too high
        ];

        for (bias, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({
                "logit_bias": {
                    "1000": bias
                }
            });

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Bias {} should be valid", bias);
            } else {
                assert!(result.is_err(), "Bias {} should be invalid", bias);
            }
        }
    }

    #[test]
    fn test_validate_logit_bias_invalid_token() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({
            "logit_bias": {
                "invalid_token": 5.0
            }
        });
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_echo_parameter() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({"echo": true});
        assert!(validate_completion_request(&request).is_ok());

        request.extra_params = json!({"echo": false});
        assert!(validate_completion_request(&request).is_ok());

        request.extra_params = json!({"echo": "invalid"});
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_suffix_parameter() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({"suffix": "completed"});
        assert!(validate_completion_request(&request).is_ok());

        request.extra_params = json!({"suffix": ""});
        assert!(validate_completion_request(&request).is_ok());

        request.extra_params = json!({"suffix": null});
        assert!(validate_completion_request(&request).is_ok());

        request.extra_params = json!({"suffix": 123});
        assert!(validate_completion_request(&request).is_err());
    }

    #[test]
    fn test_validate_best_of_parameter() {
        let test_cases = [
            (0, false),  // Too low
            (1, true),   // Minimum valid
            (10, true),  // Valid
            (20, true),  // Maximum valid
            (21, false), // Too high
        ];

        for (best_of, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({"best_of": best_of});

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Best of {} should be valid", best_of);
            } else {
                assert!(result.is_err(), "Best of {} should be invalid", best_of);
            }
        }
    }

    #[test]
    fn test_validate_logprobs_parameter() {
        let test_cases = [
            (0, true),  // Valid
            (1, true),  // Valid
            (5, true),  // Maximum valid
            (6, false), // Too high
        ];

        for (logprobs, should_pass) in test_cases {
            let mut request = create_valid_completion_request();
            request.extra_params = json!({"logprobs": logprobs});

            let result = validate_completion_request(&request);
            if should_pass {
                assert!(result.is_ok(), "Logprobs {} should be valid", logprobs);
            } else {
                assert!(result.is_err(), "Logprobs {} should be invalid", logprobs);
            }
        }
    }

    #[test]
    fn test_estimate_prompt_tokens() {
        let test_cases = [
            ("Hello", 2),
            ("Hello, world!", 4),
            ("This is a longer sentence with more words.", 9),
            ("", 0),
        ];

        for (prompt, _expected_approx) in test_cases {
            let tokens = estimate_prompt_tokens(prompt);
            if !prompt.is_empty() {
                assert!(
                    tokens > 0,
                    "Should estimate some tokens for non-empty prompt"
                );
            }
            assert!(
                tokens <= prompt.len() as u32,
                "Should be less than or equal to character count"
            );

            // Rough approximation check
            let expected = (prompt.len() as f32 / 4.0).ceil() as u32;
            assert_eq!(tokens, expected, "Should match expected calculation");
        }
    }

    #[test]
    fn test_check_prompt_token_limits() {
        let short_prompt = "Hello, world!";
        assert!(check_prompt_token_limits(short_prompt, "openai/gpt-4").is_ok());

        let medium_prompt = "word ".repeat(1000);
        assert!(check_prompt_token_limits(&medium_prompt, "openai/gpt-4").is_ok());

        let long_prompt = "word ".repeat(200_000); // ~250,000 tokens - exceeds limit
        assert!(check_prompt_token_limits(&long_prompt, "openai/gpt-4").is_err());
    }

    #[test]
    fn test_validate_completion_request_complex_params() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({
            "temperature": 0.8,
            "max_tokens": 150,
            "top_p": 0.95,
            "frequency_penalty": 0.1,
            "presence_penalty": 0.1,
            "stop": ["END", "STOP"],
            "logit_bias": {
                "100": -5.0,
                "200": 3.0
            },
            "echo": false,
            "suffix": null,
            "best_of": 1,
            "logprobs": 2
        });

        assert!(validate_completion_request(&request).is_ok());
    }

    #[test]
    fn test_validate_completion_request_mixed_valid_invalid() {
        let mut request = create_valid_completion_request();
        request.extra_params = json!({
            "temperature": 0.8,     // valid
            "max_tokens": 25000,    // invalid - too high
            "top_p": 0.95,          // valid
            "frequency_penalty": 0.1 // valid
        });

        assert!(validate_completion_request(&request).is_err());
    }
}
