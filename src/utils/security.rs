//! Security utilities for redacting sensitive information

use regex::Regex;
use std::sync::LazyLock;

// Pre-compiled regexes for performance
static API_KEY_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(sk-[A-Za-z0-9_-]{8,}|or-[A-Za-z0-9_-]{8,}|sk-or-v1-[A-Za-z0-9_-]{8,})")
        .unwrap()
});

static EMAIL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap());

static TOKEN_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(['"]?(?:token|bearer|auth)['"]?\s*[:=]\s*['"]?|bearer\s+)([A-Za-z0-9\-_.]{8,})(['"]?)"#,
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

/// Creates a safe error message that redacts sensitive information
pub fn create_safe_error_message(error_content: &str, fallback_message: &str) -> String {
    if error_content.is_empty() {
        fallback_message.to_string()
    } else {
        let redacted = redact_sensitive_content(error_content);
        // If the redacted content is significantly different, it likely contained sensitive info
        if redacted.len() < error_content.len() * 80 / 100 {
            // Content was redacted, use a generic message
            format!(
                "{} [redacted {} chars]",
                fallback_message,
                error_content.len() - redacted.len()
            )
        } else {
            // Content appears safe, use the redacted version
            redacted
        }
    }
}

/// Creates a safe error message for logging while preserving debugging info
pub fn create_safe_error_message_for_logging(
    error_content: &str,
    fallback_message: &str,
) -> String {
    create_safe_error_message(error_content, fallback_message)
}

#[cfg(test)]
pub mod security_tests {
    use super::*;

    #[test]
    pub fn test_create_safe_error_message_with_api_key() {
        let error_msg =
            "Invalid API key: sk-or-v1-abc123def456ghi789jkl012mno345pqr678stu901vwx234yz";
        let safe_msg = create_safe_error_message(error_msg, "Authentication error");

        assert!(safe_msg.contains("redacted"));
        assert!(!safe_msg.contains("sk-or-v1"));
    }

    #[test]
    pub fn test_create_safe_error_message_without_secrets() {
        let error_msg = "Request timeout after 30 seconds";
        let safe_msg = create_safe_error_message(error_msg, "Network error");

        assert_eq!(safe_msg, "Request timeout after 30 seconds");
    }

    #[test]
    pub fn test_create_safe_error_message_empty() {
        let safe_msg = create_safe_error_message("", "Generic error");
        assert_eq!(safe_msg, "Generic error");
    }

    #[test]
    pub fn test_redact_sensitive_content_with_multiple_secrets() {
        let content = r#"{
            "error": "Invalid API key: sk-or-v1-abc123def456",
            "user_email": "user@example.com",
            "details": "Token: Bearer xyz789token123"
        }"#;

        let redacted = redact_sensitive_content(content);

        assert!(redacted.contains("***REDACTED***"));
        assert!(redacted.contains("***EMAIL***"));
        assert!(redacted.contains("***TOKEN***"));
        assert!(!redacted.contains("sk-or-v1-abc123def456"));
        assert!(!redacted.contains("user@example.com"));
        assert!(!redacted.contains("xyz789token123"));
    }

    #[test]
    pub fn test_redact_json_fields_with_sensitive_keys() {
        let content = r#"{
            "api_key": "sk-or-v1-secret123",
            "model": "gpt-4",
            "password": "secret123",
            "data": {"token": "abc123"}
        }"#;

        let redacted = redact_json_fields(content);

        assert!(redacted.contains("\"api_key\": \"***REDACTED***\""));
        assert!(redacted.contains("\"password\": \"***REDACTED***\""));
        assert!(redacted.contains("\"token\": \"***REDACTED***\""));
        assert!(redacted.contains("\"model\": \"gpt-4\"")); // Non-sensitive field preserved
    }

    #[test]
    pub fn test_redact_credit_card_numbers() {
        let content = "Payment failed for card 4111-1111-1111-1111 and 4242424242424242";
        let redacted = redact_sensitive_content(content);

        assert!(redacted.contains("****-****-****-****"));
        assert!(!redacted.contains("4111-1111-1111-1111"));
        assert!(!redacted.contains("4242424242424242"));
    }

    #[test]
    pub fn test_long_content_truncation() {
        let long_content = "A".repeat(2000);
        let redacted = redact_sensitive_content(&long_content);

        assert!(redacted.len() <= 1000);
        assert!(redacted.contains("...[truncated"));
    }
}
