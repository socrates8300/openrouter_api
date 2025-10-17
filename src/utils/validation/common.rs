//! Common validation utilities used across all API endpoints

use crate::error::{Error, Result};
use std::collections::HashSet;

/// Validates a required string field
pub fn validate_required_string<'a>(
    value: &'a Option<String>,
    field_name: &str,
) -> Result<&'a String> {
    value
        .as_ref()
        .ok_or_else(|| Error::ConfigError(format!("Required field '{}' is missing", field_name)))
}

/// Validates a string field that must not be empty after trimming
pub fn validate_non_empty_string(value: &str, field_name: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::ConfigError(format!(
            "Field '{}' cannot be empty",
            field_name
        )));
    }
    Ok(())
}

/// Validates string length bounds
pub fn validate_string_length(value: &str, field_name: &str, min: usize, max: usize) -> Result<()> {
    if value.len() < min {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be at least {} characters",
            field_name, min
        )));
    }
    if value.len() > max {
        return Err(Error::ConfigError(format!(
            "Field '{}' must not exceed {} characters",
            field_name, max
        )));
    }
    Ok(())
}

/// Validates numeric range for any comparable numeric type
pub fn validate_numeric_range<T>(value: T, field_name: &str, min: T, max: T) -> Result<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be between {} and {}",
            field_name, min, max
        )));
    }
    Ok(())
}

/// Validates that a numeric value is >= minimum
pub fn validate_numeric_min<T>(value: T, field_name: &str, min: T) -> Result<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be at least {}",
            field_name, min
        )));
    }
    Ok(())
}

/// Validates that a numeric value is <= maximum
pub fn validate_numeric_max<T>(value: T, field_name: &str, max: T) -> Result<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value > max {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be at most {}",
            field_name, max
        )));
    }
    Ok(())
}

/// Validates URL format
pub fn validate_url(url: &str, field_name: &str) -> Result<()> {
    url::Url::parse(url)
        .map_err(|_| Error::ConfigError(format!("Field '{}' must be a valid URL", field_name)))?;
    Ok(())
}

/// Validates that a URL uses specific schemes
pub fn validate_url_scheme(url: &str, field_name: &str, allowed_schemes: &[&str]) -> Result<()> {
    let parsed = url::Url::parse(url)
        .map_err(|_| Error::ConfigError(format!("Field '{}' must be a valid URL", field_name)))?;

    if !allowed_schemes.contains(&parsed.scheme()) {
        return Err(Error::ConfigError(format!(
            "Field '{}' must use one of these schemes: {}",
            field_name,
            allowed_schemes.join(", ")
        )));
    }

    Ok(())
}

/// Validates date format (YYYY-MM-DD)
pub fn validate_date_format(date: &str, field_name: &str) -> Result<()> {
    if date.len() != 10 {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be in YYYY-MM-DD format",
            field_name
        )));
    }

    // Basic format validation
    if date.chars().nth(4) != Some('-') || date.chars().nth(7) != Some('-') {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be in YYYY-MM-DD format",
            field_name
        )));
    }

    // Try to parse as date
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").map_err(|_| {
        Error::ConfigError(format!(
            "Field '{}' must be a valid date in YYYY-MM-DD format",
            field_name
        ))
    })?;

    Ok(())
}

/// Validates date range (start_date <= end_date)
pub fn validate_date_range(start_date: &str, end_date: &str) -> Result<()> {
    let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d")
        .map_err(|_| Error::ConfigError("Invalid start_date format. Use YYYY-MM-DD".to_string()))?;

    let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d")
        .map_err(|_| Error::ConfigError("Invalid end_date format. Use YYYY-MM-DD".to_string()))?;

    if start > end {
        return Err(Error::ConfigError(
            "start_date cannot be after end_date".to_string(),
        ));
    }

    Ok(())
}

/// Validates that a field is one of allowed values
pub fn validate_enum_value<T: AsRef<str>>(
    value: T,
    field_name: &str,
    allowed_values: &[&str],
) -> Result<()> {
    let value_str = value.as_ref();
    if !allowed_values.contains(&value_str) {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be one of: {}",
            field_name,
            allowed_values.join(", ")
        )));
    }
    Ok(())
}

/// Validates that a collection is not empty
pub fn validate_non_empty_collection<T>(collection: &[T], field_name: &str) -> Result<()> {
    if collection.is_empty() {
        return Err(Error::ConfigError(format!(
            "Field '{}' cannot be empty",
            field_name
        )));
    }
    Ok(())
}

/// Validates collection size bounds
pub fn validate_collection_size<T>(
    collection: &[T],
    field_name: &str,
    min: usize,
    max: usize,
) -> Result<()> {
    if collection.len() < min {
        return Err(Error::ConfigError(format!(
            "Field '{}' must contain at least {} items",
            field_name, min
        )));
    }
    if collection.len() > max {
        return Err(Error::ConfigError(format!(
            "Field '{}' must contain at most {} items",
            field_name, max
        )));
    }
    Ok(())
}

/// Validates that all items in a collection are unique
pub fn validate_unique_items<T: std::hash::Hash + Eq + std::fmt::Display>(
    items: &[T],
    field_name: &str,
) -> Result<()> {
    let mut seen = HashSet::new();

    for (index, item) in items.iter().enumerate() {
        if !seen.insert(item) {
            return Err(Error::ConfigError(format!(
                "Duplicate item '{}' found in field '{}' at index {}",
                item, field_name, index
            )));
        }
    }

    Ok(())
}

/// Validates JSON object structure
pub fn validate_json_object(value: &serde_json::Value, field_name: &str) -> Result<()> {
    if !value.is_object() {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be a JSON object",
            field_name
        )));
    }
    Ok(())
}

/// Validates that a value is a specific JSON type
pub fn validate_json_type(
    value: &serde_json::Value,
    field_name: &str,
    expected_type: &str,
) -> Result<()> {
    let is_valid = match expected_type {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        "null" => value.is_null(),
        _ => false,
    };

    if !is_valid {
        return Err(Error::ConfigError(format!(
            "Field '{}' must be of type {}",
            field_name, expected_type
        )));
    }

    Ok(())
}

/// Validates optional numeric parameter from JSON object
pub fn validate_optional_numeric_param(
    params: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    min: f64,
    max: f64,
) -> Result<()> {
    if let Some(value) = params.get(key) {
        if let Some(num) = value.as_f64() {
            validate_numeric_range(num, key, min, max)?;
        } else {
            return Err(Error::ConfigError(format!(
                "Parameter '{}' must be a number",
                key
            )));
        }
    }
    Ok(())
}

/// Validates optional integer parameter from JSON object
pub fn validate_optional_integer_param(
    params: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    min: i64,
    max: i64,
) -> Result<()> {
    if let Some(value) = params.get(key) {
        if let Some(num) = value.as_i64() {
            validate_numeric_range(num, key, min, max)?;
        } else {
            return Err(Error::ConfigError(format!(
                "Parameter '{}' must be an integer",
                key
            )));
        }
    }
    Ok(())
}

/// Validates optional string parameter from JSON object
pub fn validate_optional_string_param(
    params: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    min_length: usize,
    max_length: usize,
) -> Result<()> {
    if let Some(value) = params.get(key) {
        if let Some(s) = value.as_str() {
            validate_string_length(s, key, min_length, max_length)?;
        } else {
            return Err(Error::ConfigError(format!(
                "Parameter '{}' must be a string",
                key
            )));
        }
    }
    Ok(())
}

/// Validates sampling parameters commonly used across APIs
pub fn validate_sampling_parameters(
    temperature: Option<f64>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    frequency_penalty: Option<f64>,
    presence_penalty: Option<f64>,
) -> Result<()> {
    // Temperature: [0.0, 2.0]
    if let Some(temp) = temperature {
        validate_numeric_range(temp, "temperature", 0.0, 2.0)?;
    }

    // Top P: (0.0, 1.0]
    if let Some(top_p_val) = top_p {
        if top_p_val <= 0.0 || top_p_val > 1.0 {
            return Err(Error::ConfigError(format!(
                "Top P must be between 0.0 (exclusive) and 1.0 (inclusive), got {}",
                top_p_val
            )));
        }
    }

    // Top K: [1, ∞) or 0 (disabled)
    if let Some(top_k_val) = top_k {
        if top_k_val != 0 && top_k_val < 1 {
            return Err(Error::ConfigError(format!(
                "Top K must be 0 (disabled) or >= 1, got {}",
                top_k_val
            )));
        }
    }

    // Frequency Penalty: [-2.0, 2.0]
    if let Some(fp) = frequency_penalty {
        validate_numeric_range(fp, "frequency_penalty", -2.0, 2.0)?;
    }

    // Presence Penalty: [-2.0, 2.0]
    if let Some(pp) = presence_penalty {
        validate_numeric_range(pp, "presence_penalty", -2.0, 2.0)?;
    }

    Ok(())
}

/// Validates model identifier format
pub fn validate_model_id(model: &str) -> Result<()> {
    validate_non_empty_string(model, "model")?;
    validate_string_length(model, "model", 1, 200)?;

    // Basic format validation - should contain provider/model format
    if !model.contains('/') {
        return Err(Error::ConfigError(
            "Model ID should be in format 'provider/model' (e.g., 'openai/gpt-4')".to_string(),
        ));
    }

    Ok(())
}

/// Validates that a field value matches a regex pattern
pub fn validate_regex_pattern(value: &str, field_name: &str, pattern: &str) -> Result<()> {
    let regex = regex::Regex::new(pattern).map_err(|e| {
        Error::ConfigError(format!(
            "Invalid regex pattern for field '{}': {}",
            field_name, e
        ))
    })?;

    if !regex.is_match(value) {
        return Err(Error::ConfigError(format!(
            "Field '{}' does not match required pattern",
            field_name
        )));
    }

    Ok(())
}

/// Validates that all strings in a collection are non-empty after trimming
pub fn validate_non_empty_strings(strings: &[String], field_name: &str) -> Result<()> {
    for (index, s) in strings.iter().enumerate() {
        if s.trim().is_empty() {
            return Err(Error::ConfigError(format!(
                "String at index {} in field '{}' cannot be empty",
                index, field_name
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty_string() {
        assert!(validate_non_empty_string("hello", "test").is_ok());
        assert!(validate_non_empty_string("  hello  ", "test").is_ok());
        assert!(validate_non_empty_string("", "test").is_err());
        assert!(validate_non_empty_string("   ", "test").is_err());
    }

    #[test]
    fn test_validate_string_length() {
        assert!(validate_string_length("hello", "test", 1, 10).is_ok());
        assert!(validate_string_length("hello", "test", 5, 10).is_ok());
        assert!(validate_string_length("hello", "test", 6, 10).is_err()); // too short
        assert!(validate_string_length("hello world", "test", 1, 5).is_err()); // too long
    }

    #[test]
    fn test_validate_numeric_range() {
        assert!(validate_numeric_range(5, "test", 1, 10).is_ok());
        assert!(validate_numeric_range(1, "test", 1, 10).is_ok());
        assert!(validate_numeric_range(10, "test", 1, 10).is_ok());
        assert!(validate_numeric_range(0, "test", 1, 10).is_err()); // too low
        assert!(validate_numeric_range(11, "test", 1, 10).is_err()); // too high
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com", "test").is_ok());
        assert!(validate_url("http://example.com", "test").is_ok());
        assert!(validate_url("not-a-url", "test").is_err());
    }

    #[test]
    fn test_validate_date_format() {
        assert!(validate_date_format("2024-01-15", "test").is_ok());
        assert!(validate_date_format("2024-13-15", "test").is_err()); // invalid month
        assert!(validate_date_format("2024-01-32", "test").is_err()); // invalid day
        assert!(validate_date_format("24-01-15", "test").is_err()); // wrong format
    }

    #[test]
    fn test_validate_enum_value() {
        let allowed = ["user", "assistant", "system"];
        assert!(validate_enum_value("user", "test", &allowed).is_ok());
        assert!(validate_enum_value("invalid", "test", &allowed).is_err());
    }

    #[test]
    fn test_validate_sampling_parameters() {
        // Valid parameters
        assert!(
            validate_sampling_parameters(Some(0.7), Some(0.9), Some(40), Some(0.5), Some(0.3))
                .is_ok()
        );

        // Invalid temperature
        assert!(validate_sampling_parameters(Some(3.0), None, None, None, None).is_err());

        // Invalid top_p
        assert!(validate_sampling_parameters(None, Some(0.0), None, None, None).is_err());
    }

    #[test]
    fn test_validate_model_id() {
        assert!(validate_model_id("openai/gpt-4").is_ok());
        assert!(validate_model_id("anthropic/claude-3").is_ok());
        assert!(validate_model_id("invalid-model").is_err()); // missing slash
        assert!(validate_model_id("").is_err()); // empty
    }
}
