//! Security utilities for redacting sensitive information

use regex::Regex;
use std::sync::LazyLock;

// Pre-compiled regexes for performance
static API_KEY_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(sk-[A-Za-z0-9]{32,}|or-[A-Za-z0-9]{32,})").unwrap());

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(['"]?(?:token|bearer|auth)['"]?\s*[:=]\s*['"]?)([A-Za-z0-9\-_.]{20,})(['"]?)"#,
    )
    .unwrap()
});

static CREDIT_CARD_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b").unwrap());

/// Redacts sensitive information from response bodies and error messages
pub fn redact_sensitive_content(content: &str) -> String {
    let mut redacted = content.to_string();

    // Redact API keys
    redacted = API_KEY_REGEX
        .replace_all(&redacted, "***REDACTED***")
        .to_string();

    // Redact email addresses
    redacted = EMAIL_REGEX
        .replace_all(&redacted, "***EMAIL***")
        .to_string();

    // Redact tokens
    redacted = TOKEN_REGEX
        .replace_all(&redacted, "${1}***TOKEN***${3}")
        .to_string();

    // Redact potential credit card numbers
    redacted = CREDIT_CARD_REGEX
        .replace_all(&redacted, "****-****-****-****")
        .to_string();

    // Truncate if too long to prevent log overflow
    if redacted.len() > 1000 {
        format!(
            "{}...[truncated {} chars]",
            &redacted[..500],
            redacted.len() - 500
        )
    } else {
        redacted
    }
}

/// Redacts sensitive fields from JSON-like content
pub fn redact_json_fields(content: &str) -> String {
    let mut redacted = content.to_string();

    // Common sensitive field names
    let sensitive_fields = [
        "api_key",
        "apiKey",
        "token",
        "bearer",
        "password",
        "secret",
        "auth",
        "authorization",
        "credential",
        "key",
        "private_key",
        "access_token",
        "refresh_token",
        "session_id",
        "cookie",
    ];

    for field in &sensitive_fields {
        // Match JSON field patterns: "field": "value" or 'field': 'value'
        let pattern = format!(r#"(['\"]{field}['\"])\s*:\s*(['\"])[^'\"]*(['\"])"#);
        if let Ok(regex) = Regex::new(&pattern) {
            redacted = regex
                .replace_all(&redacted, "${1}: ${2}***REDACTED***${3}")
                .to_string();
        }
    }

    redacted
}

/// Creates a safe error message for logging while preserving debugging info
pub fn create_safe_error_message(original_error: &str, context: &str) -> String {
    let redacted = redact_sensitive_content(original_error);
    let json_safe = redact_json_fields(&redacted);

    format!("{context}: {json_safe}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_redaction() {
        let content = "Error with API key sk-1234567890abcdef1234567890abcdef in request";
        let redacted = redact_sensitive_content(content);
        assert!(redacted.contains("***REDACTED***"));
        assert!(!redacted.contains("sk-1234567890abcdef1234567890abcdef"));
    }

    #[test]
    fn test_email_redaction() {
        let content = "User email test@example.com caused error";
        let redacted = redact_sensitive_content(content);
        assert!(redacted.contains("***EMAIL***"));
        assert!(!redacted.contains("test@example.com"));
    }

    #[test]
    fn test_json_field_redaction() {
        let content = r#"{"api_key": "secret123", "user": "john"}"#;
        let redacted = redact_json_fields(content);
        assert!(redacted.contains("***REDACTED***"));
        assert!(!redacted.contains("secret123"));
        assert!(redacted.contains("john")); // Non-sensitive field preserved
    }

    #[test]
    fn test_long_content_truncation() {
        let long_content = "a".repeat(2000);
        let redacted = redact_sensitive_content(&long_content);
        assert!(redacted.len() < long_content.len());
        assert!(redacted.contains("[truncated"));
    }
}
