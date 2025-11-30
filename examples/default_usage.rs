//! Example demonstrating the new Default implementations for ChatCompletionRequest and Message
//!
//! This example shows how the Default trait reduces boilerplate when creating
//! chat completion requests and messages.

use openrouter_api::types::chat::{ChatCompletionRequest, Message, MessageContent};

fn main() {
    // Before Default implementation - verbose:
    let _verbose_request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::Text("Hello, world!".to_string()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }],
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

    // After Default implementation - concise:
    let concise_request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: MessageContent::Text("Hello, world!".to_string()),
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("Concise request created successfully!");
    println!("Model: {}", concise_request.model);
    println!("Messages: {}", concise_request.messages.len());
    println!("First message role: {}", concise_request.messages[0].role);

    // You can also create a default message and then modify it:
    let default_message = Message {
        role: "system".to_string(),
        content: MessageContent::Text("You are a helpful assistant.".to_string()),
        ..Default::default()
    };

    println!("\nDefault message modified:");
    println!("Role: {}", default_message.role);
    println!("Content: {:?}", default_message.content);
    println!("Name: {:?}", default_message.name);
    println!("Tool calls: {:?}", default_message.tool_calls);
}
