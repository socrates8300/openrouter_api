# OpenRouter API Client Library

A production-ready Rust client for the [OpenRouter API](https://openrouter.ai/docs) with comprehensive security, ergonomic design, and extensive testing. The library uses a type-state builder pattern for compile-time configuration validation, ensuring robust and secure API interactions.

## Features

### Architecture & Safety
- **Type-State Builder Pattern:** Compile-time configuration validation ensures all required settings are provided before making requests
- **Secure Memory Management:** API keys are automatically zeroed on drop using the `zeroize` crate
- **Comprehensive Error Handling:** Centralized error management with safe error message redaction to prevent sensitive data leakage
- **Modular Organization:** Clean separation of concerns across modules for models, API endpoints, types, and utilities

### Ergonomic API Design
- **Convenient Constructors:** Quick setup with `from_api_key()`, `from_env()`, and `production()` methods
- **Flexible Configuration:** Fluent builder pattern with timeout, retry, and header configuration
- **Environment Integration:** Automatic API key loading from `OPENROUTER_API_KEY` or `OR_API_KEY` environment variables

### Security & Reliability
- **Bounded Response Reading:** Configurable size limits on response body reading to prevent OOM attacks (default 10 MB)
- **Streaming Safety:** Buffer limits and backpressure handling for streaming responses
- **Input Validation:** Comprehensive validation of requests and parameters
- **Automatic Retries:** Configurable retry behavior with exponential backoff, jitter, and Retry-After header support
- **Error Redaction:** Automatic sanitization of error messages to prevent sensitive data exposure

### OpenRouter API Support
- **Chat Completions:** Full support with streaming, tool calling, and structured outputs
- **Text Completions:** Traditional text completion endpoint
- **Embeddings:** Vector embedding generation for text inputs
- **Web Search:** Type-safe web search API integration
- **Analytics:** Activity data retrieval with filtering, sorting, and pagination
- **Providers:** Provider information and discovery
- **Models:** Model listing and discovery
- **Credits:** Account credit and usage tracking
- **Key Info:** API key metadata, rate limits, and usage
- **Generation:** Generation metadata and cost tracking
- **Provider Preferences:** Model routing, fallbacks, and provider selection
- **Multimodal Support:** Audio and file (PDF) input support
- **Policy Controls:** Allow/deny lists and Zero Data Retention (ZDR)

### Model Context Protocol (MCP)
- **MCP Client:** Full JSON-RPC client implementation for the [Model Context Protocol](https://modelcontextprotocol.io/)
- **Resource Access:** Retrieve resources from MCP servers
- **Tool Invocation:** Execute tools provided by MCP servers
- **Secure ID Generation:** UUID v4 usage for request tracking

### Quality & Testing
- **471 Tests:** Comprehensive unit and integration test suite
- **CI/CD Pipeline:** GitHub Actions with quality gates (formatting, linting, security audits, documentation)
- **Production Ready:** Enterprise-grade error handling, retry logic, and timeout management

## Getting Started

### Installation

Add the following to your project's `Cargo.toml`:

```bash
cargo add openrouter_api

# With optional tracing support
cargo add openrouter_api --features tracing
```

**Feature Flags:**

| Feature | Description | Default |
|---|---|---|
| `tls-rustls` | Use rustls for TLS | Yes |
| `tls-native-tls` | Use system native TLS | No |
| `tracing` | Enhanced logging via the `tracing` crate | No |
| `allow-http` | Allow non-HTTPS base URLs (testing only) | No |

Requires Rust 1.83.0 or later.

### Quick Start

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::types::chat::{ChatCompletionRequest, ChatRole, Message};

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    // From environment variable (OPENROUTER_API_KEY or OR_API_KEY)
    let client = OpenRouterClient::from_env()?;

    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message::text(ChatRole::User, "Hello, world!")],
        ..Default::default()
    };

    let response = client.chat()?.chat_completion(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("Response: {}", choice.message.content);
    }
    Ok(())
}
```

### Client Constructors

```rust
use openrouter_api::OpenRouterClient;

// From environment variable
let client = OpenRouterClient::from_env()?;

// From API key directly
let client = OpenRouterClient::from_api_key("sk-or-v1-...")?;

// Production-ready with optimized settings (60s timeout, 5 retries, app headers)
let client = OpenRouterClient::production(
    "sk-or-v1-...",
    "My Production App",
    "https://myapp.com",
)?;

// Full builder control
let client = OpenRouterClient::new()
    .skip_url_configuration()
    .with_timeout_secs(120)
    .with_retries(3, 500)
    .with_http_referer("https://myapp.com")
    .with_site_title("My Application")
    .with_max_response_bytes(10 * 1024 * 1024)
    .with_api_key("sk-or-v1-...")?;
```

## Examples

### Streaming Chat

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::types::chat::{ChatCompletionRequest, ChatRole, Message};
use futures::StreamExt;
use std::io::Write;

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message::text(ChatRole::User, "Tell me a story.")],
        stream: Some(true.into()),
        ..Default::default()
    };

    let mut stream = client.chat()?.chat_completion_stream(request);

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(c) => {
                if let Some(choice) = c.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content);
                        std::io::stdout().flush().unwrap();
                    }
                }
                if let Some(usage) = c.usage {
                    println!("\nUsage: {} prompt + {} completion = {} total tokens",
                        usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}
```

### Routing Shortcuts & Web Search Plugin

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::client::{ROUTING_NITRO, ROUTING_ONLINE};
use openrouter_api::types::chat::{ChatCompletionRequest, ChatRole, Message, Plugin};

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let model = OpenRouterClient::model_with_shortcut("openai/gpt-4o", ROUTING_NITRO);

    let request = ChatCompletionRequest {
        model,
        messages: vec![Message::text(ChatRole::User, "Search for latest Rust news")],
        plugins: Some(vec![Plugin { id: "web".to_string(), config: None }]),
        ..Default::default()
    };

    let response = client.chat()?.chat_completion(request).await?;
    Ok(())
}
```

### Tool Calling

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::models::tool::{Tool, FunctionDescription};
use openrouter_api::types::chat::{ChatCompletionRequest, ChatRole, Message};
use serde_json::json;

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let weather_tool = Tool::Function {
        function: FunctionDescription {
            name: "get_weather".to_string(),
            description: Some("Get weather for a location".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": { "type": "string", "description": "City and state" }
                },
                "required": ["location"]
            }),
        },
    };

    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message::text(ChatRole::User, "What's the weather in Boston?")],
        tools: Some(vec![weather_tool]),
        ..Default::default()
    };

    let response = client.chat()?.chat_completion(request).await?;
    Ok(())
}
```

### Multimodal Chat (Audio & PDF)

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::types::chat::*;

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message::multimodal(ChatRole::User, vec![
            ContentPart::Text(TextContent {
                content_type: ContentType::Text,
                text: "Analyze this audio and document".to_string(),
            }),
            ContentPart::Audio(AudioContent {
                content_type: ContentType::AudioUrl,
                audio_url: AudioUrl {
                    url: "https://example.com/audio.mp3".to_string(),
                },
            }),
            ContentPart::File(FileContent {
                content_type: ContentType::FileUrl,
                file_url: FileUrl {
                    url: "https://example.com/document.pdf".to_string(),
                },
            }),
        ])],
        ..Default::default()
    };

    let response = client.chat()?.chat_completion(request).await?;
    Ok(())
}
```

### Provider Preferences

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::models::provider_preferences::{DataCollection, ProviderPreferences, ProviderSort};
use openrouter_api::types::chat::{ChatCompletionRequest, ChatRole, Message};

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let preferences = ProviderPreferences::new()
        .with_order(vec!["OpenAI".to_string(), "Anthropic".to_string()])
        .with_allow_fallbacks(true)
        .with_data_collection(DataCollection::Deny)
        .with_allow(vec!["OpenAI".to_string(), "Anthropic".to_string()])
        .with_sort(ProviderSort::Throughput);

    let request = ChatCompletionRequest {
        model: "openai/gpt-4o".to_string(),
        messages: vec![Message::text(ChatRole::User, "Hello with provider preferences!")],
        provider: Some(preferences),
        ..Default::default()
    };

    let response = client.chat()?.chat_completion(request).await?;
    Ok(())
}
```

### Text Completion

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::types::completion::CompletionRequest;
use serde_json::json;

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let request = CompletionRequest {
        model: "openai/gpt-3.5-turbo-instruct".to_string(),
        prompt: "Once upon a time".to_string(),
        extra_params: json!({
            "temperature": 0.8,
            "max_tokens": 50
        }),
    };

    let response = client.completions()?.text_completion(request).await?;

    if let Some(choice) = response.choices.first() {
        println!("Completion: {}", choice.text);
    }
    Ok(())
}
```

### Embeddings API

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::types::embeddings::{EmbeddingRequest, EmbeddingInput};

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let request = EmbeddingRequest {
        model: "openai/text-embedding-3-small".to_string(),
        input: EmbeddingInput::Single("Hello world".to_string()),
        encoding_format: None,
        provider: None,
    };

    let response = client.embeddings()?.create_embedding(request).await?;

    if let Some(embedding) = response.first_embedding() {
        println!("Embedding dimensions: {}", embedding.len());
    }
    Ok(())
}
```

### Key Info API

```rust
use openrouter_api::OpenRouterClient;

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let key_info = client.key_info()?.get_key_info().await?;
    println!("Usage: {:?}", key_info.usage());
    println!("Remaining: {:?}", key_info.limit_remaining());
    println!("Free tier: {}", key_info.is_free_tier());
    Ok(())
}
```

### Analytics API

```rust
use openrouter_api::OpenRouterClient;
use openrouter_api::types::analytics::{ActivityRequest, SortField, SortOrder};

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = OpenRouterClient::from_env()?;

    let request = ActivityRequest::new()
        .with_start_date("2024-01-01")
        .with_end_date("2024-01-31")
        .with_sort(SortField::CreatedAt)
        .with_order(SortOrder::Descending)
        .with_limit(100);

    let response = client.analytics()?.get_activity(request).await?;
    println!("Retrieved {} activities", response.data.len());

    // Convenience methods
    let recent = client.analytics()?.get_recent_activity(7).await?;
    println!("Last 7 days: {} activities", recent.data.len());

    Ok(())
}
```

### Model Context Protocol (MCP)

```rust
use openrouter_api::{MCPClient, mcp_types};

#[tokio::main]
async fn main() -> openrouter_api::Result<()> {
    let client = MCPClient::new("https://mcp-server.example.com/mcp")?;

    // Initialize with client capabilities
    let server_capabilities = client.initialize(mcp_types::ClientCapabilities {
        protocolVersion: mcp_types::MCP_PROTOCOL_VERSION.to_string(),
        supportsSampling: Some(true),
    }).await?;

    println!("Connected: {:?}", server_capabilities);
    Ok(())
}
```

## Error Handling

The library provides structured error types with automatic retries for transient failures.

### Error Types

```rust
use openrouter_api::error::Error;

match client.chat()?.chat_completion(request).await {
    Ok(response) => {
        println!("Success: {}", response.choices[0].message.content);
    }
    Err(e) => match e {
        Error::ApiError { code, message, .. } => {
            eprintln!("API Error ({}): {}", code, message);
        }
        Error::RateLimitExceeded(msg) => {
            eprintln!("Rate limited: {}", msg);
        }
        Error::ContextLengthExceeded { model, message } => {
            eprintln!("Context limit for {}: {}", model, message);
        }
        Error::ValidationError(msg) => {
            eprintln!("Validation: {}", msg);
        }
        Error::TimeoutError(msg) => {
            eprintln!("Timeout: {}", msg);
        }
        Error::ResponseTooLarge(size, limit) => {
            eprintln!("Response too large: {} bytes (limit: {})", size, limit);
        }
        _ => eprintln!("Error: {:?}", e),
    },
}
```

### Retry Configuration

All API endpoints automatically retry with exponential backoff and jitter. Default behavior retries on 429 (rate limit) and 5xx (server errors).

```rust
use openrouter_api::client::RetryConfig;
use std::time::Duration;

let config = RetryConfig {
    max_retries: 5,
    initial_backoff_ms: 1000,
    max_backoff_ms: 30000,
    retry_on_status_codes: vec![429, 500, 502, 503, 504],
    total_timeout: Duration::from_secs(120),
    max_retry_interval: Duration::from_secs(30),
};
```

Retry attempts are logged with context:

```
Retrying chat_completion request (1/3) after 625 ms (base: 500 ms, jitter: 25.00%) due to status code 429
```

## API Reference

| Endpoint | Access | Key Methods |
|---|---|---|
| Chat Completions | `client.chat()?` | `chat_completion`, `chat_completion_stream` |
| Text Completions | `client.completions()?` | `text_completion` |
| Embeddings | `client.embeddings()?` | `create_embedding` |
| Models | `client.models()?` | `list_models` |
| Providers | `client.providers()?` | `get_providers`, `get_provider_by_slug` |
| Analytics | `client.analytics()?` | `get_activity`, `get_activity_by_date_range`, `get_recent_activity`, `get_activity_by_model`, `get_activity_by_provider` |
| Credits | `client.credits()?` | `get_credits` |
| Key Info | `client.key_info()?` | `get_key_info` |
| Generation | `client.generation()?` | `get_generation` |
| Structured | `client.structured()?` | Structured output generation |
| Web Search | `client.web_search()?` | Type-safe web search |

## Architecture

The crate is organized into several modules:

- **`client`** -- Type-state client with builder pattern (`Unconfigured` -> `NoAuth` -> `Ready`)
- **`api`** -- API endpoint implementations
- **`models`** -- Domain models (tools, provider preferences, structured output schemas)
- **`types`** -- Request/response type definitions
- **`mcp`** -- Model Context Protocol client
- **`error`** -- Centralized error types
- **`utils`** -- Retry, validation, security, and authentication helpers

## Contributing

Contributions are welcome! Please open an issue or submit a pull request with your ideas or fixes. Follow the code style guidelines and ensure that all tests pass.

## License

Distributed under either the MIT license or the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.
