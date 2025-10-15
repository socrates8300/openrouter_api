/// Integration tests for the OpenRouter client.
#[cfg(test)]
mod tests {
    use crate::client::{OpenRouterClient, RetryConfig, Unconfigured};
    #[allow(unused_imports)]
    use crate::models::chat::{ChatMessage, ChatRole};
    #[allow(unused_imports)]
    use crate::models::provider_preferences::{
        DataCollection, ProviderPreferences, ProviderSort, Quantization,
    };
    #[allow(unused_imports)]
    use crate::models::structured::{JsonSchemaConfig, JsonSchemaDefinition};
    #[allow(unused_imports)]
    use crate::models::tool::{FunctionCall, FunctionDescription, Tool, ToolCall};
    use crate::types::chat::{
        ChatCompletionRequest, ChatCompletionResponse, Message, MessageContent,
    };
    use serde_json::{json, Value};
    use url::Url;

    // Helper function to deserialize a ChatCompletionResponse from JSON.
    fn deserialize_chat_response(json_str: &str) -> ChatCompletionResponse {
        serde_json::from_str::<ChatCompletionResponse>(json_str).expect("Valid JSON")
    }

    #[tokio::test]
    async fn test_basic_chat_completion() -> Result<(), Box<dyn std::error::Error>> {
        // Use a dummy API key for testing since we're not making real API calls
        let api_key = "sk-1234567890abcdef1234567890abcdef";

        // Build the client: Unconfigured -> NoAuth -> Ready.
        let _client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_http_referer("https://github.com/your_org/your_repo")
            .with_site_title("OpenRouter Rust SDK Tests")
            .with_api_key(api_key);

        // Create a basic chat completion request.
        let _request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "What is a phantom type in Rust?")],
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
        };

        // For this integration test we are simulating a response.
        let simulated_response_json = r#"
        {
            "id": "gen-123",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "A phantom type is a type parameter that is not used in any fields.",
                    "tool_calls": null
                },
                "finish_reason": "stop",
                "native_finish_reason": "stop"
            }],
            "created": 1234567890,
            "model": "openai/gpt-4o",
            "object": "chat.completion",
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 15,
                "total_tokens": 25
            }
        }
        "#;
        let response = deserialize_chat_response(simulated_response_json);
        assert!(!response.choices.is_empty());
        assert_eq!(response.choices[0].message.role, "assistant");

        Ok(())
    }

    #[tokio::test]
    async fn test_valid_tool_call_response() -> Result<(), Box<dyn std::error::Error>> {
        // Simulate a valid ChatCompletionResponse with a proper tool call.
        let simulated_response_json = r#"
        {
            "id": "gen-valid-tool",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Calling tool for weather.",
                    "tool_calls": [{
                        "id": "call-001",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"Boston\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls",
                "native_finish_reason": "tool_calls"
            }],
            "created": 1234567890,
            "model": "openai/gpt-4o",
            "object": "chat.completion"
        }
        "#;
        let response = deserialize_chat_response(simulated_response_json);

        // Create a dummy client in Ready state to call our validation helper.
        let client = OpenRouterClient::<crate::client::Ready> {
            config: crate::client::ClientConfig {
                api_key: Some(
                    crate::client::SecureApiKey::new("sk-1234567890abcdef1234567890abcdef")
                        .unwrap(),
                ),
                base_url: Url::parse("https://dummy/").unwrap(),
                http_referer: None,
                site_title: None,
                user_id: None, // Add this field
                timeout: std::time::Duration::from_secs(30),
                retry_config: RetryConfig::default(), // Add this field
            },
            http_client: None,
            _state: std::marker::PhantomData,
            router_config: None, // Add this field
        };

        // Validate the tool calls – should return Ok.
        client.validate_tool_calls(&response)?;

        Ok(())
    }

    #[tokio::test]
    async fn test_text_completion_response_deserialization(
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Simulated response JSON from the text completion endpoint.
        let simulated_response_json = r#"
        {
            "id": "comp-123",
            "choices": [
                {
                    "text": "Once upon a time, in a land far, far away...",
                    "index": 0,
                    "finish_reason": "stop"
                }
            ]
        }
        "#;

        // Deserialize the response.
        let response = serde_json::from_str::<crate::types::completion::CompletionResponse>(
            simulated_response_json,
        )?;

        // Verify that the deserialization worked correctly.
        assert!(!response.choices.is_empty());
        assert_eq!(response.choices[0].finish_reason.as_deref(), Some("stop"));
        assert!(response.choices[0].text.contains("Once upon a time"));

        Ok(())
    }

    #[tokio::test]
    async fn test_invalid_tool_call_response() -> Result<(), Box<dyn std::error::Error>> {
        // Simulate an invalid ChatCompletionResponse where the tool call kind is not "function".
        let simulated_response_json = r#"
        {
            "id": "gen-invalid-tool",
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Invalid tool call.",
                    "tool_calls": [{
                        "id": "call-002",
                        "type": "invalid",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"location\": \"Boston\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls",
                "native_finish_reason": "tool_calls"
            }],
            "created": 1234567890,
            "model": "openai/gpt-4o",
            "object": "chat.completion"
        }
        "#;
        let response = deserialize_chat_response(simulated_response_json);

        // Create a dummy client to perform validation.
        let client = OpenRouterClient::<crate::client::Ready> {
            config: crate::client::ClientConfig {
                api_key: Some(
                    crate::client::SecureApiKey::new("sk-1234567890abcdef1234567890abcdef")
                        .unwrap(),
                ),
                base_url: Url::parse("https://dummy/").unwrap(),
                http_referer: None,
                site_title: None,
                user_id: None, // Add this field
                timeout: std::time::Duration::from_secs(30),
                retry_config: RetryConfig::default(), // Add this field
            },
            http_client: None,
            _state: std::marker::PhantomData,
            router_config: None, // Add this field
        };

        // Validate the tool calls – should return a SchemaValidationError.
        let validation_result = client.validate_tool_calls(&response);
        assert!(validation_result.is_err());
        if let Err(err) = validation_result {
            match err {
                crate::error::Error::SchemaValidationError(msg) => {
                    assert!(msg.contains("Invalid tool call kind"));
                }
                _ => panic!("Expected a SchemaValidationError"),
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_provider_preferences_serialization() -> Result<(), Box<dyn std::error::Error>> {
        // Build a provider preferences configuration.
        let preferences = crate::models::provider_preferences::ProviderPreferences {
            order: Some(vec!["OpenAI".to_string(), "Anthropic".to_string()]),
            allow_fallbacks: Some(false),
            require_parameters: Some(true),
            data_collection: Some(crate::models::provider_preferences::DataCollection::Deny),
            ignore: Some(vec!["Azure".to_string()]),
            quantizations: Some(vec![
                crate::models::provider_preferences::Quantization::Fp8,
                crate::models::provider_preferences::Quantization::Int8,
            ]),
            sort: Some(crate::models::provider_preferences::ProviderSort::Throughput),
        };

        // Start with an empty extra parameters object.
        let extra_params = json!({});

        // Use the request builder to attach the provider preferences.
        let builder =
            crate::api::request::RequestBuilder::new("openai/gpt-4o", vec![], extra_params)
                .with_provider_preferences(preferences)
                .expect("Provider preferences should be valid");

        // Serialize the complete payload.
        let payload = builder.build();
        let payload_json = serde_json::to_string_pretty(&payload)?;
        println!("Payload with provider preferences:\n{payload_json}");

        // Check that the serialized JSON contains the "provider" key with the expected configuration.
        let payload_value: Value = serde_json::from_str(&payload_json)?;
        let provider_config = payload_value.get("provider").expect("provider key missing");
        assert_eq!(provider_config.get("allowFallbacks").unwrap(), false);
        assert_eq!(provider_config.get("sort").unwrap(), "throughput");

        Ok(())
    }

    #[tokio::test]
    async fn test_web_search_response_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        // Simulated web search response JSON.
        let simulated_response_json = r#"
        {
            "query": "rust programming",
            "results": [
                {
                    "title": "The Rust Programming Language",
                    "url": "https://www.rust-lang.org",
                    "snippet": "Learn Rust programming."
                },
                {
                    "title": "Rust by Example",
                    "url": "https://doc.rust-lang.org/rust-by-example/",
                    "snippet": "A collection of runnable examples."
                }
            ],
            "total_results": 2
        }
        "#;
        let response: crate::types::web_search::WebSearchResponse =
            serde_json::from_str(simulated_response_json)?;
        assert_eq!(response.query, "rust programming");
        assert_eq!(response.total_results, 2);
        assert_eq!(response.results.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_chat_completion_with_provider_preferences(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::models::provider_preferences::{
            DataCollection, ProviderPreferences, ProviderSort,
        };
        use crate::types::chat::{ChatCompletionRequest, Message};

        // Create provider preferences
        let preferences = ProviderPreferences {
            order: Some(vec!["OpenAI".to_string(), "Anthropic".to_string()]),
            allow_fallbacks: Some(true),
            require_parameters: Some(false),
            data_collection: Some(DataCollection::Deny),
            ignore: Some(vec!["Azure".to_string()]),
            quantizations: None,
            sort: Some(ProviderSort::Throughput),
        };

        // Create a chat completion request with provider preferences
        let request = ChatCompletionRequest {
            model: "openai/gpt-4o".to_string(),
            messages: vec![Message::text("user", "Hello with provider preferences!")],
            stream: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            provider: Some(preferences),
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
        };

        // Serialize to JSON to verify the structure
        let json = serde_json::to_string_pretty(&request)?;
        println!("Chat request with provider preferences: {json}");

        // Verify that the provider field is serialized as an object, not a string
        let parsed: serde_json::Value = serde_json::from_str(&json)?;
        let provider_field = parsed.get("provider").expect("Provider field should exist");

        // Ensure it's an object, not a string
        assert!(
            provider_field.is_object(),
            "Provider field should be an object"
        );

        // Verify specific fields
        let order = provider_field
            .get("order")
            .expect("Order field should exist");
        assert!(order.is_array(), "Order should be an array");

        let allow_fallbacks = provider_field
            .get("allowFallbacks")
            .expect("allowFallbacks field should exist");
        assert!(
            allow_fallbacks.is_boolean(),
            "allowFallbacks should be boolean"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_streaming_chunk_deserialization() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::chat::ChatCompletionChunk;

        // Test typical streaming response
        let streaming_chunk_json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "openai/gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello"
                },
                "finish_reason": null
            }]
        }"#;

        let chunk: ChatCompletionChunk = serde_json::from_str(streaming_chunk_json)?;
        assert_eq!(chunk.id, "chatcmpl-123");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(
            chunk.choices[0].delta.content,
            Some(MessageContent::Text("Hello".to_string()))
        );
        assert!(chunk.usage.is_none());

        // Test final chunk with usage
        let final_chunk_json = r#"{
            "id": "chatcmpl-123",
            "object": "chat.completion.chunk",
            "created": 1677652288,
            "model": "openai/gpt-4o",
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        }"#;

        let final_chunk: ChatCompletionChunk = serde_json::from_str(final_chunk_json)?;
        assert_eq!(
            final_chunk.choices[0].finish_reason,
            Some("stop".to_string())
        );
        assert!(final_chunk.usage.is_some());
        let usage = final_chunk.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 10);
        assert_eq!(usage.completion_tokens, 5);
        assert_eq!(usage.total_tokens, 15);

        Ok(())
    }

    #[tokio::test]
    async fn test_credits_api_integration() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::credits::{CreditsData, CreditsResponse};

        // Test credits response deserialization
        let credits_json = r#"{
            "data": {
                "total_credits": 150.75,
                "total_usage": 45.25
            }
        }"#;

        let credits_response: CreditsResponse = serde_json::from_str(credits_json)?;
        assert_eq!(credits_response.total_credits(), 150.75);
        assert_eq!(credits_response.total_usage(), 45.25);
        assert_eq!(credits_response.remaining_credits(), 105.50);
        assert!(credits_response.has_credits());
        assert!((credits_response.usage_percentage() - 0.300).abs() < 0.001);

        // Test credits data methods
        let credits_data = CreditsData {
            total_credits: 200.0,
            total_usage: 50.0,
        };
        assert_eq!(credits_data.remaining(), 150.0);
        assert!(credits_data.has_credits());
        assert_eq!(credits_data.usage_percentage(), 0.25);

        // Test edge cases
        let zero_credits = CreditsData {
            total_credits: 0.0,
            total_usage: 0.0,
        };
        assert_eq!(zero_credits.remaining(), 0.0);
        assert!(!zero_credits.has_credits());
        assert_eq!(zero_credits.usage_percentage(), 0.0);

        let over_usage = CreditsData {
            total_credits: 100.0,
            total_usage: 120.0,
        };
        assert_eq!(over_usage.remaining(), -20.0);
        assert!(!over_usage.has_credits());
        assert_eq!(over_usage.usage_percentage(), 1.2);

        Ok(())
    }

    #[tokio::test]
    async fn test_credits_api_client_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test that the credits API can be created from the client
        let api_key = "sk-1234567890abcdef1234567890abcdef";

        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_http_referer("https://github.com/your_org/your_repo")
            .with_site_title("OpenRouter Rust SDK Tests")
            .with_api_key(api_key)?;

        // Test that we can create a credits API instance
        let credits_api = client.credits()?;
        assert!(credits_api
            .config
            .base_url
            .as_str()
            .contains("openrouter.ai"));

        Ok(())
    }

    #[tokio::test]
    async fn test_credits_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::credits::{CreditsData, CreditsResponse};

        let original = CreditsResponse {
            data: CreditsData {
                total_credits: 99.99,
                total_usage: 33.33,
            },
        };

        // Serialize to JSON
        let json_str = serde_json::to_string(&original)?;

        // Deserialize back
        let deserialized: CreditsResponse = serde_json::from_str(&json_str)?;

        // Verify they're equal
        assert_eq!(original, deserialized);
        assert_eq!(deserialized.total_credits(), 99.99);
        assert_eq!(deserialized.total_usage(), 33.33);
        assert_eq!(deserialized.remaining_credits(), 66.66);

        Ok(())
    }

    #[tokio::test]
    async fn test_generation_api_integration() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::generation::{GenerationData, GenerationResponse};

        // Test generation response deserialization
        let generation_json = r#"{
            "data": {
                "id": "gen-123456789",
                "upstream_id": "upstream-abc123",
                "total_cost": 0.025,
                "cache_discount": 0.005,
                "upstream_inference_cost": 0.020,
                "created_at": "2024-01-15T10:30:00Z",
                "model": "openai/gpt-4",
                "app_id": 12345,
                "streamed": true,
                "cancelled": false,
                "provider_name": "OpenAI",
                "latency": 1500,
                "moderation_latency": 100,
                "generation_time": 1200,
                "finish_reason": "stop",
                "native_finish_reason": "stop",
                "tokens_prompt": 50,
                "tokens_completion": 100,
                "native_tokens_prompt": 50,
                "native_tokens_completion": 100,
                "native_tokens_reasoning": 25,
                "num_media_prompt": 2,
                "num_media_completion": 0,
                "num_search_results": 5,
                "origin": "api",
                "usage": 0.025,
                "is_byok": false
            }
        }"#;

        let generation_response: GenerationResponse = serde_json::from_str(generation_json)?;
        assert_eq!(generation_response.id(), "gen-123456789");
        assert_eq!(generation_response.model(), "openai/gpt-4");
        assert_eq!(generation_response.total_cost(), 0.025);
        assert_eq!(generation_response.effective_cost(), 0.020);
        assert!(generation_response.is_successful());
        assert!(generation_response.was_streamed());
        assert_eq!(generation_response.total_tokens(), Some(150));
        assert!(generation_response.used_web_search());
        assert!(generation_response.included_media());
        assert!(generation_response.used_reasoning());

        // Test generation data methods
        let generation_data = GenerationData {
            id: "gen-test".to_string(),
            upstream_id: None,
            total_cost: 0.01,
            cache_discount: None,
            upstream_inference_cost: None,
            created_at: "2024-01-15T10:30:00Z".to_string(),
            model: "openai/gpt-3.5-turbo".to_string(),
            app_id: None,
            streamed: Some(false),
            cancelled: Some(false),
            provider_name: Some("OpenAI".to_string()),
            latency: Some(800),
            moderation_latency: None,
            generation_time: Some(600),
            finish_reason: Some("stop".to_string()),
            native_finish_reason: Some("stop".to_string()),
            tokens_prompt: Some(20),
            tokens_completion: Some(30),
            native_tokens_prompt: Some(20),
            native_tokens_completion: Some(30),
            native_tokens_reasoning: None,
            num_media_prompt: None,
            num_media_completion: None,
            num_search_results: None,
            origin: "api".to_string(),
            usage: 0.01,
            is_byok: false,
        };

        assert_eq!(generation_data.total_tokens(), Some(50));
        assert_eq!(generation_data.total_native_tokens(), Some(50));
        assert!(generation_data.is_successful());
        assert!(!generation_data.was_streamed());
        assert_eq!(generation_data.effective_cost(), 0.01);
        assert_eq!(generation_data.cost_per_token(), Some(0.01 / 50.0));
        assert_eq!(generation_data.latency_seconds(), Some(0.8));
        assert_eq!(generation_data.generation_time_seconds(), Some(0.6));
        assert!(!generation_data.used_web_search());
        assert!(!generation_data.included_media());
        assert!(!generation_data.used_reasoning());

        // Test edge cases
        let minimal_generation = GenerationData {
            id: "gen-minimal".to_string(),
            upstream_id: None,
            total_cost: 0.005,
            cache_discount: None,
            upstream_inference_cost: None,
            created_at: "2024-01-15T10:30:00Z".to_string(),
            model: "openai/gpt-3.5-turbo".to_string(),
            app_id: None,
            streamed: None,
            cancelled: None,
            provider_name: None,
            latency: None,
            moderation_latency: None,
            generation_time: None,
            finish_reason: None,
            native_finish_reason: None,
            tokens_prompt: None,
            tokens_completion: None,
            native_tokens_prompt: None,
            native_tokens_completion: None,
            native_tokens_reasoning: None,
            num_media_prompt: None,
            num_media_completion: None,
            num_search_results: None,
            origin: "api".to_string(),
            usage: 0.005,
            is_byok: false,
        };

        assert_eq!(minimal_generation.total_tokens(), None);
        assert_eq!(minimal_generation.total_native_tokens(), None);
        assert!(minimal_generation.is_successful());
        assert!(!minimal_generation.was_streamed());
        assert_eq!(minimal_generation.effective_cost(), 0.005);
        assert_eq!(minimal_generation.cost_per_token(), None);
        assert!(!minimal_generation.used_web_search());
        assert!(!minimal_generation.included_media());
        assert!(!minimal_generation.used_reasoning());

        Ok(())
    }

    #[tokio::test]
    async fn test_generation_api_client_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test that the generation API can be created from the client
        let api_key = "sk-1234567890abcdef1234567890abcdef";

        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_http_referer("https://github.com/your_org/your_repo")
            .with_site_title("OpenRouter Rust SDK Tests")
            .with_api_key(api_key)?;

        // Test that we can create a generation API instance
        let generation_api = client.generation()?;
        assert!(generation_api
            .config
            .base_url
            .as_str()
            .contains("openrouter.ai"));

        Ok(())
    }

    #[tokio::test]
    async fn test_generation_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::generation::{GenerationData, GenerationResponse};

        let original = GenerationResponse {
            data: GenerationData {
                id: "gen-roundtrip".to_string(),
                upstream_id: Some("upstream-456".to_string()),
                total_cost: 0.015,
                cache_discount: Some(0.002),
                upstream_inference_cost: Some(0.013),
                created_at: "2024-01-15T11:00:00Z".to_string(),
                model: "anthropic/claude-3-opus".to_string(),
                app_id: Some(67890),
                streamed: Some(true),
                cancelled: Some(false),
                provider_name: Some("Anthropic".to_string()),
                latency: Some(2000),
                moderation_latency: Some(150),
                generation_time: Some(1800),
                finish_reason: Some("stop".to_string()),
                native_finish_reason: Some("end_turn".to_string()),
                tokens_prompt: Some(100),
                tokens_completion: Some(200),
                native_tokens_prompt: Some(100),
                native_tokens_completion: Some(200),
                native_tokens_reasoning: Some(50),
                num_media_prompt: Some(1),
                num_media_completion: Some(0),
                num_search_results: Some(3),
                origin: "api".to_string(),
                usage: 0.015,
                is_byok: false,
            },
        };

        // Serialize to JSON
        let json_str = serde_json::to_string(&original)?;

        // Deserialize back
        let deserialized: GenerationResponse = serde_json::from_str(&json_str)?;

        // Verify they're equal
        assert_eq!(original, deserialized);
        assert_eq!(deserialized.id(), "gen-roundtrip");
        assert_eq!(deserialized.model(), "anthropic/claude-3-opus");
        assert_eq!(deserialized.total_cost(), 0.015);
        assert_eq!(deserialized.effective_cost(), 0.013);
        assert_eq!(deserialized.total_tokens(), Some(300));
        assert!(deserialized.used_web_search());
        assert!(deserialized.included_media());
        assert!(deserialized.used_reasoning());

        Ok(())
    }

    #[tokio::test]
    async fn test_generation_cost_calculations() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::generation::{GenerationData, GenerationResponse};

        // Test cost calculations with various scenarios
        let scenarios = vec![
            // (total_cost, cache_discount, expected_effective)
            (0.100, Some(0.020), 0.080),
            (0.050, None, 0.050),
            (0.075, Some(0.075), 0.000), // Full discount
            (0.200, Some(0.025), 0.175),
        ];

        for (total_cost, cache_discount, expected_effective) in scenarios {
            let data = GenerationData {
                id: "gen-cost-test".to_string(),
                upstream_id: None,
                total_cost,
                cache_discount,
                upstream_inference_cost: None,
                created_at: "2024-01-15T10:30:00Z".to_string(),
                model: "openai/gpt-4".to_string(),
                app_id: None,
                streamed: None,
                cancelled: None,
                provider_name: None,
                latency: None,
                moderation_latency: None,
                generation_time: None,
                finish_reason: None,
                native_finish_reason: None,
                tokens_prompt: Some(100),
                tokens_completion: Some(200),
                native_tokens_prompt: Some(100),
                native_tokens_completion: Some(200),
                native_tokens_reasoning: None,
                num_media_prompt: None,
                num_media_completion: None,
                num_search_results: None,
                origin: "api".to_string(),
                usage: total_cost,
                is_byok: false,
            };

            assert!((data.effective_cost() - expected_effective).abs() < f64::EPSILON);
            assert!((data.cost_per_token().unwrap() - (total_cost / 300.0)).abs() < f64::EPSILON);

            let response = GenerationResponse { data };
            assert!((response.effective_cost() - expected_effective).abs() < f64::EPSILON);
            assert!(
                (response.cost_per_token().unwrap() - (total_cost / 300.0)).abs() < f64::EPSILON
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_analytics_api_integration() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::analytics::ActivityResponse;

        // Test activity response deserialization
        let activity_json = r#"{
            "data": [
                {
                    "date": "2024-01-01",
                    "model": "openai/gpt-4",
                    "model_permaslug": "openai-gpt-4",
                    "endpoint_id": "openai-gpt-4-turbo",
                    "provider_name": "OpenAI",
                    "usage": 1.23,
                    "byok_usage_inference": 0.23,
                    "requests": 10.0,
                    "prompt_tokens": 1000.0,
                    "completion_tokens": 500.0,
                    "reasoning_tokens": 100.0
                },
                {
                    "date": "2024-01-01",
                    "model": "anthropic/claude-3-opus",
                    "model_permaslug": "anthropic-claude-3-opus",
                    "endpoint_id": "anthropic-claude-3-opus-20240229",
                    "provider_name": "Anthropic",
                    "usage": 2.56,
                    "byok_usage_inference": 0.0,
                    "requests": 15.0,
                    "prompt_tokens": 1500.0,
                    "completion_tokens": 750.0,
                    "reasoning_tokens": 0.0
                },
                {
                    "date": "2024-01-02",
                    "model": "openai/gpt-4",
                    "model_permaslug": "openai-gpt-4",
                    "endpoint_id": "openai-gpt-4-turbo",
                    "provider_name": "OpenAI",
                    "usage": 0.89,
                    "byok_usage_inference": 0.0,
                    "requests": 8.0,
                    "prompt_tokens": 800.0,
                    "completion_tokens": 400.0,
                    "reasoning_tokens": 0.0
                }
            ]
        }"#;

        let activity_response: ActivityResponse = serde_json::from_str(activity_json)?;
        assert_eq!(activity_response.data.len(), 3);

        // Test activity data methods
        let first_activity = &activity_response.data[0];
        assert_eq!(first_activity.date, "2024-01-01");
        assert_eq!(first_activity.model, "openai/gpt-4");
        assert_eq!(first_activity.provider_name, "OpenAI");
        assert_eq!(first_activity.usage, 1.23);
        assert_eq!(first_activity.requests, 10.0);
        assert_eq!(first_activity.total_tokens(), 1600.0);
        assert_eq!(first_activity.cost_per_request(), 0.123);
        assert!(first_activity.has_reasoning());
        assert!(first_activity.uses_byok());
        assert!((first_activity.byok_percentage() - 18.7).abs() < 0.1);

        // Test activity response methods
        assert_eq!(activity_response.for_date("2024-01-01").len(), 2);
        assert_eq!(activity_response.for_date("2024-01-02").len(), 1);
        assert_eq!(activity_response.for_model("openai/gpt-4").len(), 2);
        assert_eq!(activity_response.for_provider("OpenAI").len(), 2);

        let unique_dates = activity_response.unique_dates();
        assert_eq!(unique_dates.len(), 2);
        assert!(unique_dates.contains(&"2024-01-01".to_string()));
        assert!(unique_dates.contains(&"2024-01-02".to_string()));

        let unique_models = activity_response.unique_models();
        assert_eq!(unique_models.len(), 2);
        assert!(unique_models.contains(&"openai/gpt-4".to_string()));
        assert!(unique_models.contains(&"anthropic/claude-3-opus".to_string()));

        assert_eq!(activity_response.total_usage(), 4.68);
        assert_eq!(activity_response.total_requests(), 33.0);
        assert_eq!(activity_response.total_prompt_tokens(), 3300.0);
        assert_eq!(activity_response.total_completion_tokens(), 1650.0);
        assert_eq!(activity_response.total_reasoning_tokens(), 100.0);
        assert_eq!(activity_response.total_tokens(), 5050.0);

        // Test sorting methods
        let sorted_by_date = activity_response.sorted_by_date_desc();
        assert_eq!(sorted_by_date[0].date, "2024-01-02");
        assert_eq!(sorted_by_date[1].date, "2024-01-01");

        let sorted_by_usage = activity_response.sorted_by_usage_desc();
        assert_eq!(sorted_by_usage[0].usage, 2.56);
        assert_eq!(sorted_by_usage[1].usage, 1.23);

        let sorted_by_requests = activity_response.sorted_by_requests_desc();
        assert_eq!(sorted_by_requests[0].requests, 15.0);
        assert_eq!(sorted_by_requests[1].requests, 10.0);

        Ok(())
    }

    #[tokio::test]
    async fn test_analytics_request_builder() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::analytics::ActivityRequest;
        use chrono::{DateTime, NaiveDate, Utc};

        // Test basic request
        let request = ActivityRequest::new();
        assert!(request.date.is_none());

        // Test with date string
        let request = ActivityRequest::new().date("2024-01-01");
        assert_eq!(request.date, Some("2024-01-01".to_string()));

        // Test with NaiveDate
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let request = ActivityRequest::new().date_from_naive_date(date);
        assert_eq!(request.date, Some("2024-01-01".to_string()));

        // Test with DateTime<Utc>
        let datetime = DateTime::parse_from_rfc3339("2024-01-01T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let request = ActivityRequest::new().date_from_datetime(datetime);
        assert_eq!(request.date, Some("2024-01-01".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_analytics_api_client_integration() -> Result<(), Box<dyn std::error::Error>> {
        // Test that the analytics API can be created from the client
        let api_key = "sk-1234567890abcdef1234567890abcdef";
        let client = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://openrouter.ai/api/v1/")?
            .with_api_key(api_key)?;

        // Test that we can create an analytics API instance
        let analytics_api = client.analytics()?;
        assert!(analytics_api.validate_date_format("2024-01-01").is_ok());
        assert!(analytics_api.validate_date_format("invalid-date").is_err());

        // Test date retention validation
        let today = chrono::Utc::now().date_naive();
        let today_str = today.format("%Y-%m-%d").to_string();
        assert!(analytics_api.is_date_within_retention(&today_str)); // Today should be within retention
        assert!(!analytics_api.is_date_within_retention("invalid-date"));

        Ok(())
    }

    #[tokio::test]
    async fn test_analytics_serialization_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::analytics::{ActivityData, ActivityResponse};

        let activity_data = ActivityData {
            date: "2024-01-01".to_string(),
            model: "openai/gpt-4".to_string(),
            model_permaslug: "openai-gpt-4".to_string(),
            endpoint_id: "openai-gpt-4-turbo".to_string(),
            provider_name: "OpenAI".to_string(),
            usage: 1.23,
            byok_usage_inference: 0.23,
            requests: 10.0,
            prompt_tokens: 1000.0,
            completion_tokens: 500.0,
            reasoning_tokens: 100.0,
        };

        // Test serialization
        let json = serde_json::to_string(&activity_data)?;
        let deserialized: ActivityData = serde_json::from_str(&json)?;

        // Verify roundtrip
        assert_eq!(activity_data.date, deserialized.date);
        assert_eq!(activity_data.model, deserialized.model);
        assert_eq!(activity_data.usage, deserialized.usage);
        assert_eq!(activity_data.total_tokens(), deserialized.total_tokens());

        // Test with full response
        let response = ActivityResponse {
            data: vec![activity_data.clone()],
        };

        let response_json = serde_json::to_string(&response)?;
        let deserialized_response: ActivityResponse = serde_json::from_str(&response_json)?;

        assert_eq!(deserialized_response.data.len(), 1);
        assert_eq!(deserialized_response.data[0].date, activity_data.date);
        assert_eq!(deserialized_response.total_usage(), activity_data.usage);

        Ok(())
    }

    #[tokio::test]
    async fn test_analytics_cost_calculations() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::analytics::{ActivityData, ActivityResponse};

        // Test various cost calculation scenarios
        let test_cases = vec![
            // (usage, requests, prompt_tokens, completion_tokens, reasoning_tokens, expected_cost_per_request, expected_cost_per_million_tokens)
            (1.0, 10.0, 1000.0, 500.0, 0.0, 0.1, 666.67),
            (2.5, 25.0, 2000.0, 1000.0, 500.0, 0.1, 714.29),
            (0.5, 5.0, 500.0, 250.0, 0.0, 0.1, 666.67),
        ];

        for (
            usage,
            requests,
            prompt_tokens,
            completion_tokens,
            reasoning_tokens,
            expected_cost_per_request,
            expected_cost_per_million_tokens,
        ) in test_cases
        {
            let data = ActivityData {
                date: "2024-01-01".to_string(),
                model: "test-model".to_string(),
                model_permaslug: "test-model".to_string(),
                endpoint_id: "test-endpoint".to_string(),
                provider_name: "test-provider".to_string(),
                usage,
                byok_usage_inference: 0.0,
                requests,
                prompt_tokens,
                completion_tokens,
                reasoning_tokens,
            };

            assert!((data.cost_per_request() - expected_cost_per_request).abs() < 0.001);
            assert!(
                (data.cost_per_million_tokens() - expected_cost_per_million_tokens).abs() < 0.01
            );

            let response = ActivityResponse { data: vec![data] };
            assert!((response.total_usage() - usage).abs() < 0.001);
            assert!((response.total_requests() - requests).abs() < 0.001);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_analytics_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
        use crate::types::analytics::{ActivityData, ActivityResponse};

        // Test zero requests edge case
        let zero_requests_data = ActivityData {
            date: "2024-01-01".to_string(),
            model: "test-model".to_string(),
            model_permaslug: "test-model".to_string(),
            endpoint_id: "test-endpoint".to_string(),
            provider_name: "test-provider".to_string(),
            usage: 1.23,
            byok_usage_inference: 0.0,
            requests: 0.0, // Zero requests
            prompt_tokens: 1000.0,
            completion_tokens: 500.0,
            reasoning_tokens: 0.0,
        };

        assert_eq!(zero_requests_data.cost_per_request(), 0.0);

        // Test zero tokens edge case
        let zero_tokens_data = ActivityData {
            date: "2024-01-01".to_string(),
            model: "test-model".to_string(),
            model_permaslug: "test-model".to_string(),
            endpoint_id: "test-endpoint".to_string(),
            provider_name: "test-provider".to_string(),
            usage: 1.23,
            byok_usage_inference: 0.0,
            requests: 10.0,
            prompt_tokens: 0.0, // Zero tokens
            completion_tokens: 0.0,
            reasoning_tokens: 0.0,
        };

        assert_eq!(zero_tokens_data.cost_per_million_tokens(), 0.0);

        // Test empty response
        let empty_response = ActivityResponse { data: vec![] };
        assert_eq!(empty_response.total_usage(), 0.0);
        assert_eq!(empty_response.total_requests(), 0.0);
        assert_eq!(empty_response.total_tokens(), 0.0);
        assert_eq!(empty_response.unique_dates().len(), 0);
        assert_eq!(empty_response.unique_models().len(), 0);
        assert_eq!(empty_response.unique_providers().len(), 0);

        // Test BYOK percentage calculations
        let byok_data = ActivityData {
            date: "2024-01-01".to_string(),
            model: "test-model".to_string(),
            model_permaslug: "test-model".to_string(),
            endpoint_id: "test-endpoint".to_string(),
            provider_name: "test-provider".to_string(),
            usage: 0.0, // Zero total usage
            byok_usage_inference: 0.0,
            requests: 10.0,
            prompt_tokens: 1000.0,
            completion_tokens: 500.0,
            reasoning_tokens: 0.0,
        };

        assert_eq!(byok_data.byok_percentage(), 0.0);

        Ok(())
    }
}
