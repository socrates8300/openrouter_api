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
- **Standardized Error Handling:** Enterprise-grade retry logic with exponential backoff and jitter across all endpoints
- **Memory Safety:** Secure API key handling with automatic memory zeroing
- **Response Redaction:** Automatic sanitization of error messages to prevent sensitive data exposure
- **Bounded Response Reading:** Strict, configurable size limits on response body reading to prevent OOM attacks (default 10MB)
- **Streaming Safety:** Buffer limits and backpressure handling for streaming responses
- **Input Validation:** Comprehensive validation of requests and parameters
- **Automatic Retries:** Configurable retry behavior for network failures and rate limiting
- **Production Reliability:** Enterprise-grade error handling with exponential backoff and jitter

### 🌐 **OpenRouter API Support**
- **Chat Completions:** Full support for OpenRouter's chat completion API with streaming
- **Text Completions:** Traditional text completion endpoint with customizable parameters
- **Tool Calling:** Define and invoke function tools with proper validation
- **Structured Outputs:** JSON Schema validation for structured response formats
- **Web Search:** Type-safe web search API integration
- **Provider Preferences:** Configure model routing, fallbacks, and provider selection
- **Analytics API:** Comprehensive activity data retrieval with filtering and pagination
- **Providers API:** Provider information management with search and filtering
- **Enhanced Models API:** Advanced model discovery with filtering, sorting, and search

### 📡 **Model Context Protocol (MCP)**
- **MCP Client:** Full JSON-RPC client implementation for the [Model Context Protocol](https://modelcontextprotocol.io/)
- **Resource Access:** Retrieve resources from MCP servers
- **Tool Invocation:** Execute tools provided by MCP servers
- **Context Integration:** Seamless context sharing between applications and LLMs
- **Concurrency Control:** Semaphore-based limiting for concurrent requests
- **Secure ID Generation:** UUID v4 usage for request tracking

### 🧪 **Quality & Testing**
- **100% Test Coverage:** 162 comprehensive unit and integration tests
- **CI/CD Pipeline:** Automated quality gates with formatting, linting, security audits, and documentation checks
- **Production Ready:** Extensive error handling, standardized retry logic, and timeout management

## Getting Started

### Installation

Add the following to your project's `Cargo.toml`:

```bash
cargo add openrouter_api

# With optional tracing support for better error logging
cargo add openrouter_api --features tracing
```

**Available Features:**
- `rustls` (default): Use rustls for TLS
- `native-tls`: Use system TLS
- `tracing`: Enhanced error logging with tracing support

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
        .with_max_response_bytes(10 * 1024 * 1024) // 10MB limit (default)
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

    // Process the stream - accumulating content and tracking usage
    let mut total_content = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                if let Some(choice) = c.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content);
                        total_content.push_str(content);
                        std::io::stdout().flush().unwrap();
                    }
                }
                
                // Check for usage information in final chunk
                if let Some(usage) = c.usage {
                    println!("\nUsage: {} prompt + {} completion = {} total tokens", 
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
                }
            },
            Err(e) => eprintln!("Error during streaming: {}", e),
        }
    }
    println!();
    Ok(())
}
```

#### Analytics API Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::analytics::{AnalyticsQuery, ActivityType, DateRange};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;
    
    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Get the analytics API
    let analytics_api = client.analytics()?;

    // Example 1: Get all activity data with pagination
    let mut all_activities = Vec::new();
    let mut page = 1;
    
    loop {
        let query = AnalyticsQuery::new()
            .with_page(page)
            .with_per_page(100);
            
        let response = analytics_api.query(query).await?;
        all_activities.extend(response.data);
        
        if response.data.len() < 100 {
            break; // Last page
        }
        page += 1;
    }
    
    println!("Retrieved {} total activities", all_activities.len());

    // Example 2: Filter by specific activity types
    let chat_query = AnalyticsQuery::new()
        .with_activity_type(vec![ActivityType::ChatCompletion])
        .with_per_page(50);
        
    let chat_response = analytics_api.query(chat_query).await?;
    println!("Found {} chat completion activities", chat_response.data.len());

    // Example 3: Get activity within a date range
    let date_range_query = AnalyticsQuery::new()
        .with_date_range(DateRange::Custom {
            start: "2024-01-01".to_string(),
            end: "2024-01-31".to_string(),
        });
        
    let january_response = analytics_api.query(date_range_query).await?;
    println!("January activities: {}", january_response.data.len());

    // Example 4: Get usage statistics
    let usage_stats = analytics_api.usage().await?;
    println!("Total requests: {}", usage_stats.total_requests);
    println!("Total tokens: {}", usage_stats.total_tokens);

    // Example 5: Get daily activity for the last 7 days
    let daily_activity = analytics_api.daily_activity().await?;
    for day in daily_activity {
        println!("{}: {} requests, {} tokens", 
            day.date, day.request_count, day.token_count);
    }

    Ok(())
}
```

#### Providers API Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::providers::{ProvidersQuery, ProviderSort};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;
    
    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Get the providers API
    let providers_api = client.providers()?;

    // Example 1: List all available providers
    let all_providers = providers_api.list().await?;
    println!("Found {} providers", all_providers.len());
    
    for provider in &all_providers {
        println!("{}: {} models", provider.name, provider.model_count);
    }

    // Example 2: Search for specific providers
    let search_query = ProvidersQuery::new()
        .with_search("openai")
        .with_sort(ProviderSort::Name);
        
    let search_results = providers_api.search(search_query).await?;
    println!("Found {} providers matching 'openai'", search_results.len());

    // Example 3: Get provider by name
    if let Some(openai) = providers_api.get_by_name("OpenAI").await? {
        println!("OpenAI provider details:");
        println!("  Models: {}", openai.model_count);
        println!("  Status: {:?}", openai.status);
        
        // Extract domain from provider's first model URL
        if let Some(first_model) = openai.models.first() {
            if let Some(domain) = first_model.extract_domain() {
                println!("  Domain: {}", domain);
            }
        }
    }

    // Example 4: Get providers with specific capabilities
    let capability_query = ProvidersQuery::new()
        .with_capability("chat");
        
    let chat_providers = providers_api.query(capability_query).await?;
    println!("{} providers support chat", chat_providers.len());

    Ok(())
}
```

#### Enhanced Models API Example

```rust
use openrouter_api::{OpenRouterClient, utils, Result};
use openrouter_api::types::models::{ModelsQuery, ModelSort, ModelArchitecture};

#[tokio::main]
async fn main() -> Result<()> {
    // Load API key from environment
    let api_key = utils::load_api_key_from_env()?;
    
    // Build the client
    let client = OpenRouterClient::new()
        .with_base_url("https://openrouter.ai/api/v1/")?
        .with_api_key(api_key)?;

    // Get the models API
    let models_api = client.models()?;

    // Example 1: List all available models
    let all_models = models_api.list().await?;
    println!("Found {} models", all_models.len());

    // Example 2: Search for models with specific capabilities
    let search_query = ModelsQuery::new()
        .with_search("gpt-4")
        .with_capability("chat")
        .with_sort(ModelSort::Name);
        
    let search_results = models_api.search(search_query).await?;
    println!("Found {} GPT-4 models with chat capability", search_results.len());

    // Example 3: Filter by architecture
    let architecture_query = ModelsQuery::new()
        .with_architecture(ModelArchitecture::Transformer);
        
    let transformer_models = models_api.query(architecture_query).await?;
    println!("Found {} transformer models", transformer_models.len());

    // Example 4: Get models by provider
    let openai_models = models_api.get_by_provider("OpenAI").await?;
    println!("OpenAI has {} models", openai_models.len());

    // Example 5: Filter by context length
    let context_query = ModelsQuery::new()
        .with_min_context_length(32000)
        .with_max_context_length(128000);
        
    let high_context_models = models_api.query(context_query).await?;
    println!("Found {} models with 32k-128k context", high_context_models.len());

    // Example 6: Get free models
    let free_models = models_api.get_free_models().await?;
    println!("Found {} free models", free_models.len());

    // Example 7: Get model details
    if let Some(gpt4) = models_api.get_by_id("openai/gpt-4").await? {
        println!("GPT-4 Details:");
        println!("  Name: {}", gpt4.name);
        println!("  Context Length: {}", gpt4.context_length);
        println!("  Pricing: ${}/1M tokens", gpt4.pricing.prompt);
        
        if let Some(description) = gpt4.description {
            println!("  Description: {}", description);
        }
    }

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
- **Analytics API:** Comprehensive activity data retrieval with filtering and pagination
- **Providers API:** Provider information management with search and filtering
- **Enhanced Models API:** Advanced model discovery with filtering, sorting, and search
- **Credits API:** Account credit and usage tracking
- **Generation API:** Generation metadata and cost tracking
- **Model Context Protocol:** Complete MCP client implementation

### ✅ **Quality Infrastructure (Completed)**
- **100% Test Coverage:** 162 comprehensive unit and integration tests
- **Security Auditing:** Automated security vulnerability scanning
- **CI/CD Pipeline:** GitHub Actions with quality gates
- **Documentation:** Complete API documentation with examples
- **Developer Experience:** Contributing guidelines, issue templates, PR templates
- **Error Handling Standardization:** Enterprise-grade retry logic across all endpoints

### ✅ **Ergonomic Improvements (Completed)**
- **Convenience Constructors:** `from_env()`, `from_api_key()`, `production()`, `quick()`
- **Flexible Configuration:** Timeout, retry, and header management
- **Error Handling:** Comprehensive error types with context and automatic retries
- **Memory Safety:** Automatic sensitive data cleanup
- **Advanced Filtering:** Sophisticated query builders for analytics, providers, and models
- **Convenience Methods:** Helper methods for common operations like domain extraction
- **Production Reliability:** Exponential backoff with jitter, rate limit handling, and consistent retry behavior
- **Retry-After Support:** Respects server-provided retry guidance (headers) for rate limiting

### 🔄 **Future Enhancements**
- **Circuit Breaker Pattern:** Prevent cascading failures
- **Retry Budget Management:** Prevent excessive retries in high-throughput scenarios
- **Performance Optimizations:** Connection pooling and caching
- **Extended MCP Features:** Additional MCP protocol capabilities
- **Generation API Enhancements:** Additional generation endpoints and features

## Contributing

Contributions are welcome! Please open an issue or submit a pull request with your ideas or fixes. Follow the code style guidelines and ensure that all tests pass.

## License

Distributed under either the MIT license or the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

# OpenRouter API Rust Crate Documentation

_**Version:** 0.1.6 • **License:** MIT / Apache‑2.0_

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

The library provides enterprise-grade error handling with automatic retries and consistent behavior across all endpoints.

### Standardized Retry Logic

All API endpoints automatically retry failed requests with:
- **Exponential Backoff:** Starting at 500ms, doubling up to 10 seconds maximum
- **Jitter:** ±25% random variation to prevent thundering herd effects
- **Smart Status Codes:** Retries on rate limiting (429) and server errors (500, 502, 503, 504)
- **Configurable Limits:** Customizable maximum retries and backoff settings

```rust
use openrouter_api::{OpenRouterClient, Result};

// Custom retry configuration
let client = OpenRouterClient::from_env()?
    .with_retry_config(RetryConfig {
        max_retries: 5,
        initial_backoff_ms: 1000,
        max_backoff_ms: 30000,
        retry_on_status_codes: vec![429, 500, 502, 503, 504],
    })?;

// Automatic retries happen transparently
let response = client.chat()?.chat_completion(request).await?;
```

### Error Types

```rust
match client.chat()?.chat_completion(request).await {
    Ok(response) => {
        println!("Success: {}", response.choices[0].message.content);
    },
    Err(e) => match e {
        Error::ApiError { code, message, .. } => {
            eprintln!("API Error ({}): {}", code, message);
        },
        Error::RateLimitExceeded(msg) => {
            eprintln!("Rate limit exceeded: {}", msg);
            // Automatic retry with exponential backoff
        },
        Error::HttpError(ref err) if err.is_timeout() => {
            eprintln!("Request timed out!");
            // Automatic retry with exponential backoff
        },
        Error::ConfigError(msg) => {
            eprintln!("Configuration error: {}", msg);
        },
        Error::ContextLengthExceeded { model, message } => {
            eprintln!("Context limit exceeded for {}: {}", model, message);
        },
        _ => eprintln!("Other error: {:?}", e),
    }
}
```

### Retry Behavior

The library automatically handles:
- **Network Timeouts:** Retries with increasing delays
- **Rate Limiting:** Respects HTTP 429 with exponential backoff
- **Server Errors:** Retries 5xx errors to handle temporary failures
- **Connection Issues:** Retries connection failures and DNS errors

All retry attempts are logged with operation context for debugging:

```
Retrying chat_completion request (1/3) after 625 ms (base: 500 ms, jitter: 25.00%) due to status code 429
Retrying chat_completion request (2/3) after 1250 ms (base: 1000 ms, jitter: 25.00%) due to status code 429
```

## Best Practices

1. **Use the Type‑State Pattern:**
   Let the compiler ensure your client is properly configured.

2. **Configure Retry Behavior:**
   Adjust retry settings based on your application's needs:
   ```rust
   let client = OpenRouterClient::from_env()?
       .with_retries(5)?  // More retries for resilience
       .without_retries()?; // Disable for testing
   ```

3. **Set Appropriate Timeouts & Headers:**
   Configure reasonable timeouts and identify your application.

4. **Handle Errors Appropriately:**
   Implement proper error handling for each error type. The automatic retry logic handles most transient failures.

5. **Use Provider Preferences:**
   Configure provider routing for optimal model selection.

6. **Monitor Retry Behavior:**
   Watch retry logs in production to identify patterns and adjust configuration.

7. **Secure Your API Keys:**
   Store keys in environment variables or secure storage.

## Additional Resources

- [OpenRouter API Documentation](https://openrouter.ai/docs)
- [Model Context Protocol Specification](https://modelcontextprotocol.io/specification/2025-03-26/)

---
