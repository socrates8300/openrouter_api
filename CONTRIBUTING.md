# Contributing to OpenRouter API

Thank you for your interest in contributing to the OpenRouter API Rust client library! This document provides guidelines and instructions for contributing.

## Quick Start

1. **Fork and clone** the repository
2. **Install Rust** (latest stable version recommended)
3. **Run quality gates**: `./scripts/pre_quality.sh`
4. **Make your changes** following our guidelines
5. **Submit a pull request**

## Development Workflow

### Prerequisites

- Rust 1.70.0 or later
- `cargo` package manager
- Git

### Setting Up Development Environment

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/openrouter_api.git
cd openrouter_api

# Install dependencies and verify setup
cargo build
cargo test

# Run quality gates
./scripts/pre_quality.sh
```

### Quality Gates

**IMPORTANT**: All changes must pass our quality gates before submission:

```bash
# Run all quality checks
./scripts/pre_quality.sh

# Individual checks
cargo fmt              # Code formatting
cargo clippy           # Linting
cargo test             # Unit tests
cargo audit            # Security audit
cargo doc              # Documentation
```

### Code Standards

#### Code Style
- Follow standard Rust formatting (`cargo fmt`)
- Adhere to Clippy recommendations (`cargo clippy`)
- Write clear, self-documenting code
- Use meaningful variable and function names

#### Documentation
- Document all public APIs with rustdoc comments
- Include examples in documentation where helpful
- Update README.md for significant changes

#### Testing
- Write unit tests for all new functionality
- Maintain 100% test pass rate
- Add integration tests for complex features
- Test edge cases and error conditions

#### Security
- Never expose API keys or sensitive data in logs
- Use `SecureApiKey` for handling API credentials
- Sanitize error messages to prevent data leakage
- Follow secure coding practices

## Types of Contributions

### Bug Fixes
1. **Search existing issues** to avoid duplicates
2. **Create an issue** describing the bug if none exists
3. **Write a test** that reproduces the bug
4. **Fix the bug** and ensure the test passes
5. **Update documentation** if needed

### New Features
1. **Discuss the feature** in an issue first
2. **Design the API** following our patterns
3. **Implement with tests** and documentation
4. **Update examples** if applicable

### Documentation Improvements
- Fix typos, grammar, or unclear explanations
- Add missing documentation
- Improve code examples
- Update outdated information

## Code Architecture

### Key Patterns

#### Type-State Builder Pattern
The client uses compile-time state validation:
```rust
let client = OpenRouterClient::new()
    .with_base_url("https://openrouter.ai/api/v1/")?
    .with_api_key(api_key)?;  // Now in Ready state
```

#### Error Handling
Use the centralized `Error` enum and `Result<T>` type:
```rust
pub fn my_function() -> Result<MyType> {
    // ...
    Ok(result)
}
```

#### Security
Always use security utilities for sensitive data:
```rust
use crate::utils::security::create_safe_error_message;

// Good
let safe_msg = create_safe_error_message(&raw_error, "Context");

// Bad - never expose raw response bodies
return Err(Error::ApiError { message: raw_response, .. });
```

### Module Organization
- `client.rs`: Core client with type-state pattern
- `api/`: Endpoint-specific implementations
- `models/`: Domain models and data structures
- `types/`: Request/response type definitions
- `utils/`: Shared utilities (auth, validation, security)
- `error.rs`: Centralized error handling

## Testing Guidelines

### Writing Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_specific_behavior() {
        // Arrange
        let input = create_test_input();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_value);
    }
}
```

### Test Categories
- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test complete workflows
- **Security tests**: Verify data protection and validation
- **Error tests**: Ensure proper error handling

## Submitting Changes

### Pull Request Process

1. **Create a feature branch** from `main`
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our guidelines

3. **Run quality gates** and ensure all tests pass
   ```bash
   ./scripts/pre_quality.sh
   ```

4. **Commit with clear messages**
   ```bash
   git commit -m "Add feature: clear description of what was added"
   ```

5. **Push and create PR**
   ```bash
   git push origin feature/your-feature-name
   ```

### Pull Request Requirements

✅ **Required for all PRs:**
- [ ] All quality gates pass (`./scripts/pre_quality.sh`)
- [ ] Tests written for new functionality
- [ ] Documentation updated for public API changes
- [ ] No breaking changes without discussion
- [ ] Clear, descriptive commit messages

✅ **PR Description should include:**
- What changes were made and why
- Any breaking changes
- Testing approach
- Documentation updates

### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Security review** for sensitive changes
4. **Documentation review** for public API changes

## Common Pitfalls

### Security Issues
❌ **Don't do this:**
```rust
// Exposing sensitive data
println!("Error: {}", response_body);
return Err(Error::ApiError { message: api_response, .. });
```

✅ **Do this instead:**
```rust
// Safe error handling
let safe_msg = create_safe_error_message(&response_body, "API call failed");
return Err(Error::ApiError { message: safe_msg, .. });
```

### Error Handling
❌ **Don't do this:**
```rust
// Swallowing errors
let result = risky_operation().unwrap_or_default();
```

✅ **Do this instead:**
```rust
// Proper error propagation
let result = risky_operation()
    .map_err(|e| Error::ConfigError(format!("Operation failed: {}", e)))?;
```

### Testing
❌ **Don't do this:**
```rust
// Brittle or incomplete tests
#[test]
fn test_something() {
    assert!(true); // Not testing anything useful
}
```

✅ **Do this instead:**
```rust
// Comprehensive, meaningful tests
#[test]
fn test_api_key_validation_rejects_short_keys() {
    let result = SecureApiKey::new("short");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too short"));
}
```

## Getting Help

- **Documentation**: Check README.md and rustdoc comments
- **Issues**: Search existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Security**: Report security issues privately via email

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project.