# OpenRouter API Library - Update Plan

## Executive Summary

This document outlines a structured plan to enhance the OpenRouter API Rust client library from its current ~65% API coverage to near-complete implementation. The plan prioritizes critical missing functionality while maintaining the library's high-quality standards and architectural patterns.

## Current Status

### ✅ Implemented Features (~65% coverage)
- **Chat Completions API** (`/api/v1/chat/completions`) - Full implementation with streaming support
- **Text Completions API** (`/api/v1/completions`) - Basic implementation
- **Models API** (`/api/v1/models`) - With filtering capabilities
- **Web Search API** (`/api/v1/web/search`) - Implemented
- **Structured Outputs** - Via chat completions with JSON schema validation
- **MCP (Model Context Protocol)** - Full client implementation
- **Provider Preferences** - Advanced routing and fallback configuration

### ❌ Critical Gaps Identified
- Account management APIs (Credits, Analytics)
- Advanced chat parameters and sampling options
- Multimodal input support
- API key management
- Provider listing functionality

## Implementation Roadmap

### Phase 1: Critical Foundation (Week 1-2)
**Priority: HIGH - Essential for production use**

#### 1.1 Enhanced Chat Parameters
**Target:** `src/types/chat.rs`, `src/api/chat.rs`

Add missing sampling parameters to `ChatCompletionRequest`:
```rust
pub struct ChatCompletionRequest {
    // ... existing fields ...
    
    // Sampling parameters
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub repetition_penalty: Option<f32>,
    pub min_p: Option<f32>,
    pub top_a: Option<f32>,
    pub seed: Option<u64>,
    pub stop: Option<StopSequence>,
    
    // Advanced parameters
    pub logit_bias: Option<HashMap<u32, f32>>,
    pub top_logprobs: Option<u8>,
    pub logprobs: Option<bool>,
    pub prediction: Option<PredictionConfig>,
    pub parallel_tool_calls: Option<bool>,
    pub verbosity: Option<VerbosityLevel>,
    
    // OpenRouter-specific
    pub transforms: Option<Vec<String>>,
    pub models: Option<Vec<String>>,
    pub route: Option<RouteStrategy>,
    pub user: Option<String>,
}
```

#### 1.2 Credits API Implementation
**Target:** `src/api/credits.rs`, `src/types/credits.rs`

```rust
// New API module
pub struct CreditsApi {
    client: Client,
    config: ClientConfig,
}

impl CreditsApi {
    pub async fn get_balance(&self) -> Result<CreditsResponse> {
        // GET /api/v1/credits
    }
}
```

#### 1.3 Generation API Implementation  
**Target:** `src/api/generation.rs`, `src/types/generation.rs`

```rust
pub struct GenerationApi {
    client: Client,
    config: ClientConfig,
}

impl GenerationApi {
    pub async fn get_generation(&self, id: &str) -> Result<GenerationResponse> {
        // GET /api/v1/generation?id={id}
    }
}
```

### Phase 2: Account Management (Week 3-4)
**Priority: HIGH - Important for enterprise users**

#### 2.1 Analytics API
**Target:** `src/api/analytics.rs`, `src/types/analytics.rs`

```rust
pub struct AnalyticsApi {
    client: Client,
    config: ClientConfig,
}

impl AnalyticsApi {
    pub async fn get_activity(&self, request: ActivityRequest) -> Result<ActivityResponse> {
        // GET /api/v1/analytics/activity
    }
}
```

#### 2.2 Providers API
**Target:** `src/api/providers.rs`, `src/types/providers.rs`

```rust
pub struct ProvidersApi {
    client: Client,
    config: ClientConfig,
}

impl ProvidersApi {
    pub async fn list_providers(&self) -> Result<ProvidersResponse> {
        // GET /api/v1/providers
    }
}
```

#### 2.3 OpenRouter-Specific Features
- **Prompt Transforms**: Implement transform types and validation
- **Model Routing**: Enhanced routing strategies and fallback logic
- **User Tracking**: End-user identification for analytics

### Phase 3: Multimodal Support (Week 5-6)
**Priority: MEDIUM - Important for modern AI applications**

#### 3.1 Image Input Support
**Target:** `src/types/chat.rs`, `src/models/multimodal.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentPart {
    Text(TextContent),
    Image(ImageContent),
    // Future: Audio, PDF
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    #[serde(rename = "type")]
    pub content_type: String, // "image_url"
    pub image_url: ImageUrl,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    pub url: String, // URL or base64
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<ImageDetail>,
}
```

#### 3.2 Audio and PDF Support
- Extend `ContentPart` enum for audio and PDF inputs
- Add validation for supported formats
- Implement multipart request handling

### Phase 4: Advanced Features (Week 7-8)
**Priority: LOW - Nice-to-have for specialized use cases**

#### 4.1 API Key Management
**Target:** `src/api/keys.rs`

```rust
pub struct ApiKeysApi {
    client: Client,
    config: ClientConfig,
}

impl ApiKeysApi {
    pub async fn list_keys(&self) -> Result<Vec<ApiKey>>;
    pub async fn create_key(&self, request: CreateKeyRequest) -> Result<ApiKey>;
    pub async fn get_key(&self, id: &str) -> Result<ApiKey>;
    pub async fn update_key(&self, id: &str, request: UpdateKeyRequest) -> Result<ApiKey>;
    pub async fn delete_key(&self, id: &str) -> Result<()>;
}
```

#### 4.2 Authentication API
- OAuth PKCE flow implementation
- Authorization code exchange

#### 4.3 Responses API Alpha
- OpenAI-compatible stateless API
- Reasoning and enhanced tool calling support

## Implementation Guidelines

### Code Quality Standards
1. **Follow existing patterns**: Use the established error handling, client configuration, and API structure
2. **Comprehensive testing**: Unit tests for all new functionality, integration tests for API endpoints
3. **Documentation**: Full rustdoc coverage with examples for all public APIs
4. **Security**: Continue using `SecureApiKey` and proper input validation

### Breaking Changes Management
1. **Version compatibility**: Maintain backward compatibility where possible
2. **Feature flags**: Use Rust features for optional functionality
3. **Deprecation warnings**: Mark old APIs as deprecated before removal

### Testing Strategy
1. **Unit tests**: 95%+ coverage for all new modules
2. **Integration tests**: Mock HTTP responses using `wiremock`
3. **Property-based testing**: Use `proptest` for complex validation logic
4. **Performance tests**: Benchmark streaming and large request handling

## Success Metrics

### Coverage Goals
- **API Coverage**: 95%+ of documented OpenRouter APIs
- **Parameter Coverage**: 100% of chat completion parameters
- **Feature Coverage**: All major OpenRouter-specific features

### Quality Metrics
- **Test Coverage**: Maintain 90%+ overall coverage
- **Documentation**: 100% public API documentation
- **Performance**: No regression in streaming or request latency

### Adoption Metrics
- **Breaking changes**: Minimize to <2 per major version
- **API stability**: All new APIs follow semantic versioning
- **Developer experience**: Improved ergonomics and error messages

## Risk Assessment

### Technical Risks
- **API changes**: OpenRouter may evolve APIs during implementation
- **Complex parameters**: Some parameters have complex validation requirements
- **Multimodal handling**: Large file uploads may require special handling

### Mitigation Strategies
- **Flexible design**: Use `serde_json::Value` for complex, evolving parameters
- **Comprehensive validation**: Implement robust input validation
- **Incremental rollout**: Phase features to minimize risk

## Timeline Summary

| Phase | Duration | Priority | Deliverables |
|-------|----------|----------|--------------|
| Phase 1 | 2 weeks | Critical | Enhanced chat params, Credits & Generation APIs |
| Phase 2 | 2 weeks | High | Analytics, Providers, OpenRouter features |
| Phase 3 | 2 weeks | Medium | Multimodal support (images, audio, PDF) |
| Phase 4 | 2 weeks | Low | API key management, Authentication, Responses API |

**Total Estimated Time: 8 weeks**

## Next Steps

1. **Immediate (Week 1)**: Begin Phase 1.1 with enhanced chat parameters
2. **Review**: Architectural review of proposed changes
3. **Resource allocation**: Assign developers to each phase
4. **Milestone planning**: Set specific dates for each phase completion

This plan provides a clear path to significantly enhancing the OpenRouter API library while maintaining its high quality and developer-friendly design.