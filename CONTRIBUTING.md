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

## Type Design Guidelines

This library uses a strongly-typed system to prevent bugs and improve code safety. All new types must follow these principles.

### Core Principles

1. **Prefer Newtypes over Primitives**: Wrap strings and numbers to prevent mixing incompatible values
2. **Enum over String**: Use enums for fixed sets of values (roles, types, states)
3. **Transparent Serialization**: Serialize as plain values for API compatibility
4. **Custom Deserialization**: Accept both formats when APIs vary (strings vs numbers)
5. **Comprehensive Traits**: All newtypes should implement standard traits

### Newtype Pattern

Use newtypes for entity identifiers and validated primitives:

```rust
/// Strongly-typed identifier for entities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<String> for EntityId { /* ... */ }
impl From<&str> for EntityId { /* ... */ }
impl AsRef<str> for EntityId { /* ... */ }
impl fmt::Display for EntityId { /* ... */ }
impl Into<String> for EntityId { /* ... */ }
```

**Required Traits:**
- `Debug`: For logging and debugging
- `Clone`: For passing values around
- `PartialEq`, `Eq`: For comparisons and HashMap keys
- `Hash`: For HashSet and HashMap usage
- `Serialize`, `Deserialize`: For JSON API compatibility
- `Display`: For formatting in logs and errors
- `AsRef<str>` or `AsRef<T>`: For conversion helpers
- `Into<String>` or `Into<T>`: For explicit conversions

**Required Attributes:**
- `#[serde(transparent)]`: Serialize as plain values, not wrapped objects

### Enum over String

Use enums instead of string validation:

```rust
// ✅ GOOD - compile-time validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
    System,
    Tool,
}

impl fmt::Display for ChatRole {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChatRole::User => write!(f, "user"),
            ChatRole::Assistant => write!(f, "assistant"),
            ChatRole::System => write!(f, "system"),
            ChatRole::Tool => write!(f, "tool"),
        }
    }
}

// ❌ BAD - runtime validation, easy to make typos
pub fn parse_role(role: &str) -> Result<String> {
    match role {
        "user" | "assistant" | "system" | "tool" => Ok(role.to_string()),
        _ => Err("Invalid role".into()),
    }
}
```

**When to use enums:**
- Fixed, finite sets of values (roles, types, states)
- Values used in match statements
- Values that need serialization control (rename_all)

### Validated Newtypes

For values that need validation at creation:

```rust
/// Price with validation for non-negative values.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct Price(f64);

impl<'de> Deserialize<'de> for Price {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de>
    {
        struct PriceVisitor;

        impl<'de> serde::de::Visitor<'de> for PriceVisitor {
            type Value = Price;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a number or string representing a price")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                if value.is_finite() {
                    Ok(Price(value))
                } else {
                    Err(serde::de::Error::custom("price must be finite"))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
                value.parse::<f64>()
                    .map_err(|e| serde::de::Error::custom(e))
                    .and_then(|v| {
                        if v.is_finite() {
                            Ok(Price(v))
                        } else {
                            Err(serde::de::Error::custom("price must be finite"))
                        }
                    })
            }
        }

        deserializer.deserialize_any(PriceVisitor)
    }
}

impl Price {
    pub fn new(value: impl Into<f64>) -> Option<Self> {
        let v = value.into();
        if v.is_finite() {
            Some(Self(v))
        } else {
            None
        }
    }

    pub fn is_valid_business_logic(&self) -> bool {
        self.0 >= 0.0
    }
}
```

**When to use validated newtypes:**
- Numeric values with business logic constraints
- Values that need validation at API boundary
- Values where API may return special indicators (negative, null)

### API Compatibility

The OpenRouter API may return data in multiple formats. Handle both:

```rust
// API returns prices as strings: "0.001", "0", "-1" (special indicator)
// But we want to use as numbers internally
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct Price(f64);

impl<'de> Deserialize<'de> for Price {
    // Custom deserializer handles both:
    // - Numbers: 0.001
    // - Strings: "0.001", "-1"
    // - Nulls: treat as default value
}
```

### Type Safety Examples

**Prevent mixing different IDs:**
```rust
// ✅ GOOD - compile-time safety
let model_id: ModelId = "openai/gpt-4".into();
let gen_id: GenerationId = "gen-123".into();

// These won't compile:
// let x: ModelId = gen_id;  // ❌ Type mismatch!
// let y: GenerationId = model_id;  // ❌ Type mismatch!
```

**Prevent invalid states:**
```rust
// ✅ GOOD - enum makes invalid states unrepresentable
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StreamingStatus {
    NotStarted,
    InProgress,
    Complete,
    Cancelled,
}

// ❌ BAD - many invalid combinations possible
pub struct StreamingState {
    is_streaming: bool,
    is_complete: bool,
    is_cancelled: bool,
}
```

### Testing Requirements

All newtypes must have comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_newtype_creation() {
        let id = MyId::new("test-id");
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_newtype_serialization() {
        let id = MyId::new("test-id");
        let json = serde_json::to_value(&id).unwrap();
        assert_eq!(json, "test-id"); // Transparent, not {"id": "test-id"}
    }

    #[test]
    fn test_newtype_roundtrip() {
        let original = MyId::new("test-id");
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MyId = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_newtype_traits() {
        let id = MyId::new("test-id");

        // Clone
        let _ = id.clone();

        // PartialEq
        assert_eq!(id, MyId::new("test-id"));

        // Hash
        let mut set = std::collections::HashSet::new();
        set.insert(id);

        // Display
        format!("{}", id);

        // AsRef
        let s: &str = id.as_ref();
        assert_eq!(s, "test-id");
    }
}
```

### Documentation Requirements

All newtypes must document:
1. Purpose and what it prevents
2. API compatibility notes (e.g., "Accepts negative for API compatibility")
3. Validation rules (what's valid vs invalid)
4. When to use `new()` vs `new_unchecked()`
5. Trait implementations provided

```rust
/// Strongly-typed identifier for AI models.
///
/// Prevents accidental mixing of model IDs with other entity IDs.
/// The OpenRouter API uses string IDs, but this wrapper provides compile-time
/// type safety.
///
/// # Examples
///
/// ```rust
/// let model_id = ModelId::new("openai/gpt-4");
/// ```
///
/// # API Compatibility
///
/// Serializes as a plain string for seamless API integration.
/// 
/// # Validation
///
/// - `new()` accepts any string (no validation)
/// - `as_str()` returns the underlying value
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ModelId(String);
```

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