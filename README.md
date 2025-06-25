# OpenRouter API Client Library

A production-ready Rust client for the OpenRouter API with comprehensive security, ergonomic design, and extensive testing. The library uses a type‑state builder pattern for compile-time configuration validation, ensuring robust and secure API interactions.

## Features

### 🏗️ **Architecture & Safety**
- **Type‑State Builder Pattern:** Compile-time configuration validation ensures all required settings are provided before making requests
- **Secure Memory Management:** API keys are automatically zeroed on drop using the `zeroize` crate for enhanced security
- **Comprehensive Error Handling:** Centralized error management with safe error message redaction to prevent sensitive data leakage
- **Modular Organization:** Clean separation of concerns across modules for models, API endpoints, types, and utilities

### 🚀 **Ergonomic API Design**
- **Convenient Constructors:** Quick setup with `from_api_key()`, `from_env()`, `quick()`, and `production()` methods
- **Flexible Configuration:** Fluent builder pattern with timeout, retry, and header configuration
- **Environment Integration:** Automatic API key loading from `OPENROUTER_API_KEY` or `OR_API_KEY` environment variables

### 🔒 **Security & Reliability**
- **Memory Safety:** Secure API key handling with automatic memory zeroing
- **Response Redaction:** Automatic sanitization of error messages to prevent sensitive data exposure
- **Streaming Safety:** Buffer limits and backpressure handling for streaming responses
- **Input Validation:** Comprehensive validation of requests and parameters

### 🌐 **OpenRouter API Support**
- **Chat Completions:** Full support for OpenRouter's chat completion API with streaming
- **Text Completions:** Traditional text completion endpoint with customizable parameters
- **Tool Calling:** Define and invoke function tools with proper validation
- **Structured Outputs:** JSON Schema validation for structured response formats
- **Web Search:** Type-safe web search API integration
- **Provider Preferences:** Configure model routing, fallbacks, and provider selection

### 📡 **Model Context Protocol (MCP)**
- **MCP Client:** Full JSON-RPC client implementation for the [Model Context Protocol](https://modelcontextprotocol.io/)
- **Resource Access:** Retrieve resources from MCP servers
- **Tool Invocation:** Execute tools provided by MCP servers
- **Context Integration:** Seamless context sharing between applications and LLMs

### 🧪 **Quality & Testing**
- **100% Test Coverage:** Comprehensive unit and integration test suite
- **CI/CD Pipeline:** Automated quality gates with formatting, linting, security audits, and documentation checks
- **Production Ready:** Extensive error handling, retry logic, and timeout management

## Getting Started

### Installation

Add the following to your project's `Cargo.toml`:

```bash
cargo add openrouter_api
```

Ensure that you have Rust installed (tested with Rust v1.83.0) and that you're using Cargo for building and testing.

### Quick Start Examples

#### Simple Chat Completion

```rust
use openrouter_api::{OpenRouterClient, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};

#[tokio::main]
async fn main() -> Result<()> {
    // Quick setup from environment variable (OPENROUTER_API_KEY)
    let client = OpenRouterClient::from_env()?;
    
    // Or directly from API key
    // let client = OpenRouterClient::from_api_key("sk-or-v1-...")?;

    let request = ChatCompletionRequest {
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
    };

    let response = client.chat()?.chat_completion(request).await?;
    
    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }
    Ok(())
}
```

#### Production Configuration

```rust
use openrouter_api::{OpenRouterClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Production-ready client with optimized settings
    let client = OpenRouterClient::production(
        "sk-or-v1-...",           // API key
        "My Production App",       // App name
        "https://myapp.com"       // App URL
    )?;
    
    // Client is now configured with:
    // - 60 second timeout
    // - 5 retries with exponential backoff
    // - Proper headers for app identification
    
    // Use the client...
    Ok(())
}
```

#### Custom Configuration

```rust
use openrouter_api::{OpenRouterClient, Result};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Full control over client configuration
    let client = OpenRouterClient::new()
        .skip_url_configuration()  // Use default OpenRouter URL
        .with_timeout_secs(120)    // 2-minute timeout
        .with_retries(3, 500)      // 3 retries, 500ms initial backoff
        .with_http_referer("https://myapp.com")
        .with_site_title("My Application")
        .with_api_key("sk-or-v1-...")?;
    
    // Ready to use
    Ok(())
}
```

#### Provider Preferences Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::models::provider_preferences::{DataCollection, ProviderPreferences, ProviderSort};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment variables
    let api_key = utils::load_api_key_from_env()?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;
    
    // Create provider preferences
    let preferences = ProviderPreferences::new()
        .with_order(vec!["OpenAI".to_string(), "Anthropic".to_string()])
        .with_allow_fallbacks(true)
        .with_data_collection(DataCollection::Deny)
        .with_sort(ProviderSort::Throughput);
    
    // Create a request builder with provider preferences
    let request_builder = client.chat_request_builder(vec![
        Message {
            role: "user".to_string(),
            content: "Hello with provider preferences!".to_string(),
            name: None,
            tool_calls: None,
        },
    ]);
    
    // Add provider preferences and build the payload
    let payload = request_builder
        .with_provider_preferences(preferences)?
        .build();
    
    // The payload now includes provider preferences!
    println!("Request payload: {}", serde_json::to_string_pretty(&payload)?);
    
    Ok(())
}
```

#### Model Context Protocol (MCP) Client Example

```rust
use openrouter_api::{MCPClient, Result};
use openrouter_api::mcp_types::{
    ClientCapabilities, GetResourceParams, ToolCallParams,
    MCP_PROTOCOL_VERSION
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new MCP client
    let client = MCPClient::new("https://mcp-server.example.com/mcp")?;
    
    // Initialize the client with client capabilities
    let server_capabilities = client.initialize(ClientCapabilities {
        protocolVersion: MCP_PROTOCOL_VERSION.to_string(),
        supportsSampling: Some(true),
    }).await?;
    
    println!("Connected to MCP server with capabilities: {:?}", server_capabilities);
    
    // Get a resource from the MCP server
    let resource = client.get_resource(GetResourceParams {
        id: "document-123".to_string(),
        parameters: None,
    }).await?;
    
    println!("Retrieved resource: {}", resource.content);
    
    // Call a tool on the MCP server
    let result = client.tool_call(ToolCallParams {
        id: "search-tool".to_string(),
        parameters: serde_json::json!({
            "query": "Rust programming"
        }),
    }).await?;
    
    println!("Tool call result: {:?}", result.result);
    
    Ok(())
}
```

#### Model Context Protocol (MCP) Client Example

```rust
use openrouter_api::{MCPClient, Result};
use openrouter_api::mcp_types::{
    ClientCapabilities, GetResourceParams, ToolCallParams,
    MCP_PROTOCOL_VERSION
};

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new MCP client
    let client = MCPClient::new("https://mcp-server.example.com/mcp")?;
    
    // Initialize the client with client capabilities
    let server_capabilities = client.initialize(ClientCapabilities {
        protocolVersion: MCP_PROTOCOL_VERSION.to_string(),
        supportsSampling: Some(true),
    }).await?;
    
    println!("Connected to MCP server with capabilities: {:?}", server_capabilities);
    
    // Get a resource from the MCP server
    let resource = client.get_resource(GetResourceParams {
        id: "document-123".to_string(),
        parameters: None,
    }).await?;
    
    println!("Retrieved resource: {}", resource.content);
    
    // Call a tool on the MCP server
    let result = client.tool_call(ToolCallParams {
        id: "search-tool".to_string(),
        parameters: serde_json::json!({
            "query": "Rust programming"
        }),
    }).await?;
    
    println!("Tool call result: {:?}", result.result);
    
    Ok(())
}
```

#### Text Completion Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::completion::CompletionRequest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a text completion request
    let request = CompletionRequest {
        model: "openai/gpt-3.5-turbo-instruct".to_string(),
        prompt: "Once upon a time".to_string(),
        // Additional generation parameters
        extra_params: json!({
            "temperature": 0.8,
            "max_tokens": 50
        }),
    };

    // Invoke the text completion endpoint
    let completions_api = client.completions()?;
    let response = completions_api.text_completion(request).await?;

    // Print out the generated text
    if let Some(choice) = response.choices.first() {
        println!("Text Completion: {}", choice.text);
    }
    Ok(())
}
```

#### Streaming Chat Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::chat::{ChatCompletionRequest, Message};
use futures::StreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;

    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Create a chat completion request with streaming enabled
    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Tell me a story.".to_string(),
            name: None,
            tool_calls: None,
        }],
        stream: Some(true),
        response_format: None,
        tools: None,
        provider: None,
        models: None,
        transforms: None,
    };

    // Invoke the streaming chat completion endpoint
    let chat_api = client.chat()?;
    let mut stream = chat_api.chat_completion_stream(request);

    // Process the stream
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                if let Some(choice) = c.choices.first() {
                    print!("{}", choice.message.content);
                    std::io::stdout().flush().unwrap();
                }
            },
            Err(e) => eprintln!("Error during streaming: {}", e),
        }
    }
    println!();
    Ok(())
}
```

## Model Context Protocol (MCP) Client

The library includes a client implementation for the [Model Context Protocol](https://modelcontextprotocol.io/), which is an open protocol that standardizes how applications provide context to LLMs.

Key features of the MCP client include:

- **JSON-RPC Communication:** Implements the JSON-RPC 2.0 protocol for MCP
- **Resource Access:** Retrieve resources from MCP servers
- **Tool Invocation:** Call tools provided by MCP servers
- **Prompt Execution:** Execute prompts on MCP servers
- **Server Capabilities:** Discover and leverage server capabilities
- **Proper Authentication:** Handle initialization and authentication flows

```rust
// Create an MCP client connected to a server
let client = MCPClient::new("https://mcp-server.example.com/mcp")?;

// Initialize with client capabilities
let server_capabilities = client.initialize(ClientCapabilities {
    protocolVersion: "2025-03-26".to_string(),
    supportsSampling: Some(true),
}).await?;

// Access resources from the server
let resource = client.get_resource(GetResourceParams {
    id: "some-resource-id".to_string(),
    parameters: None,
}).await?;
```

See the [Model Context Protocol specification](https://spec.modelcontextprotocol.io/specification/2025-03-26/) for more details.

## Implementation Status

This is a production-ready library with comprehensive functionality:

### ✅ **Core Features (Completed)**
- **Client Framework:** Type‑state builder pattern with compile‑time validation
- **Security:** Secure API key handling with memory zeroing and error redaction
- **Chat Completions:** Full OpenRouter chat API support with streaming
- **Text Completions:** Traditional text completion endpoint
- **Web Search:** Integrated web search capabilities
- **Tool Calling:** Function calling with validation
- **Structured Outputs:** JSON Schema validation
- **Provider Preferences:** Model routing and fallback configuration
- **Model Context Protocol:** Complete MCP client implementation

### ✅ **Quality Infrastructure (Completed)**
- **100% Test Coverage:** 80+ comprehensive unit and integration tests
- **Security Auditing:** Automated security vulnerability scanning
- **CI/CD Pipeline:** GitHub Actions with quality gates
- **Documentation:** Complete API documentation with examples
- **Developer Experience:** Contributing guidelines, issue templates, PR templates

### ✅ **Ergonomic Improvements (Completed)**
- **Convenience Constructors:** `from_env()`, `from_api_key()`, `production()`, `quick()`
- **Flexible Configuration:** Timeout, retry, and header management
- **Error Handling:** Comprehensive error types with context
- **Memory Safety:** Automatic sensitive data cleanup

### 🔄 **Future Enhancements**
- **Models Listing:** Endpoint to list available models
- **Credits API:** Account credit and usage tracking
- **Performance Optimizations:** Connection pooling and caching
- **Extended MCP Features:** Additional MCP protocol capabilities

## Contributing

Contributions are welcome! Please open an issue or submit a pull request with your ideas or fixes. Follow the code style guidelines and ensure that all tests pass.

## License

Distributed under either the MIT license or the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

# OpenRouter API Rust Crate Documentation

_**Version:** 0.1.5 • **License:** MIT / Apache‑2.0_

The `openrouter_api` crate is a comprehensive client for interacting with the [OpenRouter API](https://openrouter.ai/docs) and [Model Context Protocol](https://modelcontextprotocol.io/) servers. It provides strongly‑typed endpoints for chat completions, text completions, web search, and MCP connections. The crate is built using asynchronous Rust and leverages advanced patterns for safe and flexible API usage.

---

## Table of Contents

- [Core Concepts](#core-concepts)
- [Installation](#installation)
- [Architecture & Module Overview](#architecture--module-overview)
- [Client Setup & Type‑State Pattern](#client-setup--type-state-pattern)
- [API Endpoints](#api-endpoints)
  - [Chat Completions](#chat-completions)
  - [Text Completions](#text-completions)
  - [Web Search](#web-search)
  - [Tool Calling & Structured Output](#tool-calling--structured-output)
  - [Model Context Protocol](#model-context-protocol)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Additional Resources](#additional-resources)

---

## Core Concepts

- **Type‑State Client Configuration:**
  The client is built using a type‑state pattern to ensure that required parameters are set before making any API calls.

- **Provider Preferences:**
  Strongly-typed configuration for model routing, fallbacks, and provider selection.

- **Asynchronous Streaming:**
  Support for streaming responses via asynchronous streams.

- **Model Context Protocol:**
  Client implementation for connecting to MCP servers to access resources, tools, and prompts.

- **Error Handling & Validation:**
  Comprehensive error handling with detailed context and validation utilities.

---

## Architecture & Module Overview

The crate is organized into several modules:

- **`client`:** Type-state client implementation with builder pattern
- **`api`:** API endpoint implementations (chat, completions, web search, etc.)
- **`models`:** Domain models for structured outputs, provider preferences, tools
- **`types`:** Type definitions for requests and responses
- **`mcp`:** Model Context Protocol client implementation
- **`error`:** Centralized error handling
- **`utils`:** Utility functions and helpers

---

## Client Setup & Type‑State Pattern

```rust
// Quick setup (recommended for most use cases)
let client = OpenRouterClient::from_env()?;

// Production setup with optimized settings
let client = OpenRouterClient::production(
    "sk-or-v1-...",
    "My App", 
    "https://myapp.com"
)?;

// Full control with type-state pattern
let client = OpenRouterClient::new()
    .with_base_url("https://openrouter.ai/api/v1/")?
    .with_timeout(Duration::from_secs(30))
    .with_http_referer("https://your-app.com/")
    .with_api_key(std::env::var("OPENROUTER_API_KEY")?)?;
```

## API Endpoints

### Chat Completions

```rust
// Basic chat completion
let response = client.chat()?.chat_completion(
    ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "Explain quantum computing".to_string(),
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
).await?;
```

### Tool Calling

```rust
// Define a function tool
let weather_tool = Tool::Function { 
    function: FunctionDescription {
        name: "get_weather".to_string(),
        description: Some("Get weather information for a location".to_string()),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City and state"
                }
            },
            "required": ["location"]
        }),
    }
};

// Make a request with tool calling enabled
let response = client.chat()?.chat_completion(
    ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "What's the weather in Boston?".to_string(),
            name: None,
            tool_calls: None,
        }],
        tools: Some(vec![weather_tool]),
        // other fields...
        stream: None,
        response_format: None,
        provider: None,
        models: None,
        transforms: None,
    }
).await?;
```

### Model Context Protocol

```rust
// Create an MCP client
let mcp_client = MCPClient::new("https://mcp-server.example.com/mcp")?;

// Initialize with client capabilities
let server_capabilities = mcp_client.initialize(ClientCapabilities {
    protocolVersion: MCP_PROTOCOL_VERSION.to_string(),
    supportsSampling: Some(true),
}).await?;

// Access a resource from the MCP server
let resource = mcp_client.get_resource(GetResourceParams {
    id: "document-123".to_string(),
    parameters: None,
}).await?;
```

## Error Handling

```rust
match client.chat()?.chat_completion(request).await {
    Ok(response) => {
        println!("Success: {}", response.choices[0].message.content);
    },
    Err(e) => match e {
        Error::ApiError { code, message, .. } => {
            eprintln!("API Error ({}): {}", code, message);
        },
        Error::HttpError(ref err) if err.is_timeout() => {
            eprintln!("Request timed out!");
        },
        Error::ConfigError(msg) => {
            eprintln!("Configuration error: {}", msg);
        },
        _ => eprintln!("Other error: {:?}", e),
    }
}
```

## Best Practices

1. **Use the Type‑State Pattern:**
   Let the compiler ensure your client is properly configured.

2. **Set Appropriate Timeouts & Headers:**
   Configure reasonable timeouts and identify your application.

3. **Handle Errors Appropriately:**
   Implement proper error handling for each error type.

4. **Use Provider Preferences:**
   Configure provider routing for optimal model selection.

5. **Secure Your API Keys:**
   Store keys in environment variables or secure storage.

## Additional Resources

- [OpenRouter API Documentation](https://openrouter.ai/docs)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/specification/2025-03-26/)

---
