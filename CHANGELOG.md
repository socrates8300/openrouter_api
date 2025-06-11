# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.4] - 2025-01-14

### 🔒 Security Enhancements

- **BREAKING**: Implemented `SecureApiKey` wrapper with automatic memory zeroing using `zeroize` crate
- Added comprehensive response body redaction system to prevent sensitive data leakage in error messages
- Enhanced streaming safety with buffer limits and backpressure handling
- Implemented secure error message creation utility to sanitize error outputs
- Added input validation for API keys with format and length checks

### 🚀 Ergonomic Improvements

- **NEW**: Added convenience constructors for quick client setup:
  - `OpenRouterClient::from_env()` - Load API key from environment variables
  - `OpenRouterClient::from_api_key(key)` - Direct API key setup
  - `OpenRouterClient::from_env_with_config()` - Environment with custom config
  - `OpenRouterClient::quick()` - Minimal development setup
  - `OpenRouterClient::production()` - Production-ready configuration
- Enhanced builder pattern with better error messages and validation
- Improved URL validation with specific format expectations
- Added timeout and retry convenience methods (`with_timeout_secs`, `with_retries`)

### 🧪 Quality Infrastructure

- **NEW**: Comprehensive unit test suite with 80+ tests achieving 100% pass rate
- **NEW**: Quality gates automation script (`scripts/pre_quality.sh`)
- **NEW**: GitHub Actions CI/CD pipeline with multi-platform testing
- **NEW**: Security audit integration with cargo-audit
- **NEW**: Code coverage reporting with cargo-llvm-cov
- **NEW**: Documentation testing and validation

### 📚 Developer Experience

- **NEW**: `CONTRIBUTING.md` with detailed contribution guidelines
- **NEW**: GitHub issue templates for bugs and feature requests
- **NEW**: Pull request template with comprehensive checklists
- **NEW**: Security best practices documentation
- Enhanced README with production-ready examples and security guidance

### 🐛 Bug Fixes

- Fixed test failures in client configuration validation
- Corrected error type expectations in URL validation tests
- Resolved clippy warnings and formatting issues
- Fixed module inception warnings in test modules

### 🔧 Technical Improvements

- Enhanced type-state builder pattern with compile-time safety guarantees
- Improved error handling with detailed context and metadata
- Better memory management for sensitive data
- Streamlined client configuration flow
- Enhanced request/response validation

### 📖 Documentation

- Completely updated README.md with new features and security enhancements
- Added comprehensive API documentation with examples
- Improved inline code documentation
- Added security considerations and best practices
- Enhanced usage examples for all major features

---

## [0.1.3] - 2024-12-XX

### Added
- Model Context Protocol (MCP) client implementation
- MCP resource access and tool invocation capabilities
- Enhanced streaming support for chat completions
- Provider preferences and routing configuration

### Changed
- Improved type-state builder pattern implementation
- Enhanced error handling and validation

### Fixed
- Various stability and performance improvements

---

## [0.1.2] - 2024-11-XX

### Added
- Web search endpoint integration
- Text completion API support
- Tool calling capabilities
- Structured output validation

### Changed
- Refined API design for better usability
- Improved streaming implementation

---

## [0.1.1] - 2024-10-XX

### Added
- Basic chat completion functionality
- Initial streaming support
- Core type definitions

### Fixed
- Initial bug fixes and stability improvements

---

## [0.1.0] - 2024-09-XX

### Added
- Initial release
- Basic OpenRouter API client implementation
- Type-safe request/response handling
- Async/await support with reqwest

---

## Security Advisories

### Known Issues
- `instant` crate (v0.1.13) is unmaintained but used indirectly through dependency chain
  - **Impact**: Low - Used only in development/test dependencies
  - **Mitigation**: No direct security impact on production usage
  - **Status**: Monitoring for dependency updates

---

## Breaking Changes

### v0.1.4

1. **SecureApiKey Introduction**
   ```rust
   // OLD (v0.1.3 and earlier)
   let client = OpenRouterClient::new()
       .with_api_key("sk-key")?;
   
   // NEW (v0.1.4+) - Still supported, automatically wrapped
   let client = OpenRouterClient::new()
       .with_api_key("sk-key")?;
   
   // NEW - Explicit SecureApiKey usage
   let secure_key = SecureApiKey::new("sk-key")?;
   let client = OpenRouterClient::new()
       .with_secure_api_key(secure_key)?;
   ```

2. **Enhanced Error Types**
   - Error messages are now automatically redacted for security
   - More detailed error context and metadata available
   - Some error message formats may have changed

3. **Validation Changes**
   - Stricter API key validation (minimum length, format checks)
   - Enhanced input validation for all request parameters
   - Some previously accepted invalid inputs may now return errors

---

## Migration Guide

### From v0.1.3 to v0.1.4

1. **No code changes required** for most users - the API remains backward compatible
2. **Recommended changes** for enhanced security:
   ```rust
   // Use new convenience constructors
   let client = OpenRouterClient::from_env()?;
   
   // Or for production
   let client = OpenRouterClient::production(
       "sk-key",
       "My App",
       "https://myapp.com"
   )?;
   ```
3. **Test updates** may be needed if you're testing error messages
4. **Environment variables**: Set `OPENROUTER_API_KEY` or `OR_API_KEY`

---

## Contributors

- Development Team - Core implementation and security enhancements
- Community Contributors - Bug reports and feature suggestions

---

## Acknowledgments

- [OpenRouter](https://openrouter.ai/) for providing the API platform
- [Model Context Protocol](https://modelcontextprotocol.io/) team for the MCP specification
- Rust community for excellent crates: `reqwest`, `tokio`, `serde`, `zeroize`, `thiserror`