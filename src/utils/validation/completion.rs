//! Validation utilities for text completion requests

use super::common::*;
use crate::error::{Error, Result};
use crate::types::completion::CompletionRequest;

/// Maximum allowed prompt length for completions
const MAX_PROMPT_LENGTH: usize = 1_000_000;

/// Validates a completion request for common errors
pub fn validate_completion_request(request: &CompletionRequest) -> Result<()> {
    // Validate model
    validate_model_id(&request.model)?;

    // Validate prompt
    validate_non_empty_string(&request.prompt, "prompt")?;
    validate_string_length(&request.prompt, "prompt", 1, MAX_PROMPT_LENGTH)?;

    // Validate extra parameters if present
    if let serde_json::Value::Object(params) = &request.extra_params {
        validate_extra_params(params)?;
    }

    Ok(())
}

/// Validates extra parameters in completion requests
fn validate_extra_params(params: &serde_json::Map<String, serde_json::Value>) -> Result<()> {
    // Temperature: [0.0, 2.0]
    validate_optional_numeric_param(params, "temperature", 0.0, 2.0)?;

    // Top P: (0.0, 1.0]
    if let Some(value) = params.get("top_p") {
        if let Some(top_p) = value.as_f64() {
            if top_p <= 0.0 || top_p > 1.0 {
                return Err(Error::ConfigError(format!(
                    "Top P must be between 0.0 (exclusive) and 1.0 (inclusive), got {}",
                    top_p
                )));
            }
        } else {
            return Err(Error::ConfigError(
                "Parameter 'top_p' must be a number".to_string(),
            ));
        }
    }

    // Max tokens: [1, 8192] or 0 for unlimited
    if let Some(value) = params.get("max_tokens") {
        if let Some(tokens) = value.as_u64() {
            if tokens != 0 && !(1..=8192).contains(&tokens) {
                return Err(Error::ConfigError(format!(
                    "Max tokens must be 0 (unlimited) or between 1 and 8192, got {}",
                    tokens
                )));
            }
        } else {
            return Err(Error::ConfigError(
                "Parameter 'max_tokens' must be an integer".to_string(),
            ));
        }
    }

    // Frequency Penalty: [-2.0, 2.0]
    validate_optional_numeric_param(params, "frequency_penalty", -2.0, 2.0)?;

    // Presence Penalty: [-2.0, 2.0]
    validate_optional_numeric_param(params, "presence_penalty", -2.0, 2.0)?;

    // Validate stop sequences if present
    if let Some(value) = params.get("stop") {
        validate_stop_sequence(value)?;
    }

    // Validate logit bias if present
    if let Some(value) = params.get("logit_bias") {
        validate_logit_bias(value)?;
    }

    // Validate echo parameter if present
    if let Some(value) = params.get("echo") {
        if !value.is_boolean() {
            return Err(Error::ConfigError(
                "Parameter 'echo' must be a boolean".to_string(),
            ));
        }
    }

    // Validate suffix parameter if present
    if let Some(value) = params.get("suffix") {
        if let Some(suffix) = value.as_str() {
            validate_string_length(suffix, "suffix", 0, 1000)?;
        } else if !value.is_null() {
            return Err(Error::ConfigError(
                "Parameter 'suffix' must be a string or null".to_string(),
            ));
        }
    }

    // Validate best_of parameter if present
    if let Some(value) = params.get("best_of") {
        if let Some(best_of) = value.as_u64() {
            validate_numeric_range(best_of, "best_of", 1, 20)?;
        } else {
            return Err(Error::ConfigError(
                "Parameter 'best_of' must be an integer".to_string(),
            ));
        }
    }

    // Validate logprobs parameter if present
    if let Some(value) = params.get("logprobs") {
        if let Some(logprobs) = value.as_u64() {
            validate_numeric_range(logprobs, "logprobs", 0, 5)?;
        } else {
            return Err(Error::ConfigError(
                "Parameter 'logprobs' must be an integer".to_string(),
            ));
        }
    }

    Ok(())
}

/// Validates stop sequence parameter
fn validate_stop_sequence(value: &serde_json::Value) -> Result<()> {
    match value {
        serde_json::Value::String(stop) => {
            // Single stop sequence
            validate_string_length(stop, "stop", 1, 100)?;
        }
        serde_json::Value::Array(stops) => {
            // Multiple stop sequences
            validate_collection_size(stops, "stop", 1, 4)?;

            for (index, stop_val) in stops.iter().enumerate() {
                if let Some(stop_str) = stop_val.as_str() {
                    validate_string_length(stop_str, &format!("stop[{}]", index), 1, 100)?;
                } else {
                    return Err(Error::ConfigError(format!(
                        "Stop sequence at index {} must be a string",
                        index
                    )));
                }
            }
        }
        _ => {
            return Err(Error::ConfigError(
                "Parameter 'stop' must be a string or array of strings".to_string(),
            ));
        }
    }
    Ok(())
}

/// Validates logit bias parameter
fn validate_logit_bias(value: &serde_json::Value) -> Result<()> {
    if let serde_json::Value::Object(bias_map) = value {
        // Validate each token-bias pair
        for (token_str, bias_val) in bias_map {
            // Validate token is a valid integer string
            if token_str.parse::<i32>().is_err() {
                return Err(Error::ConfigError(format!(
                    "Logit bias token '{}' must be a valid integer",
                    token_str
                )));
            }

            // Validate bias value is a number
            if !bias_val.is_number() {
                return Err(Error::ConfigError(format!(
                    "Logit bias for token '{}' must be a number",
                    token_str
                )));
            }

            // Validate bias range: [-100, 100]
            if let Some(bias) = bias_val.as_f64() {
                if !(-100.0..=100.0).contains(&bias) {
                    return Err(Error::ConfigError(format!(
                        "Logit bias for token '{}' must be between -100 and 100, got {}",
                        token_str, bias
                    )));
                }
            }
        }
    } else {
        return Err(Error::ConfigError(
            "Parameter 'logit_bias' must be a JSON object".to_string(),
        ));
    }
    Ok(())
}

/// Estimates token count for a completion prompt (rough approximation)
pub fn estimate_prompt_tokens(prompt: &str) -> u32 {
    // Very rough approximation: 1 token per 4 characters
    // This is less accurate than for chat since completion prompts can be any format
    (prompt.len() as f32 / 4.0).ceil() as u32
}

/// Checks if a completion prompt might exceed reasonable token limits
pub fn check_prompt_token_limits(prompt: &str, model: &str) -> Result<()> {
    let estimated_tokens = estimate_prompt_tokens(prompt);

    // Use a more conservative limit for completions since context windows vary
    const MAX_COMPLETION_TOKENS: u32 = 200_000;

    if estimated_tokens > MAX_COMPLETION_TOKENS {
        return Err(Error::ContextLengthExceeded {
            model: model.to_string(),
            message: format!(
                "Estimated prompt token count ({}) exceeds maximum recommended limit ({})",
                estimated_tokens, MAX_COMPLETION_TOKENS
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
