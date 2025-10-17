//! Validation utilities for web search requests

use super::common::*;
use crate::error::{Error, Result};
use crate::types::web_search::WebSearchRequest;

/// Maximum query length for web searches
const MAX_QUERY_LENGTH: usize = 1000;

/// Minimum query length for web searches
const MIN_QUERY_LENGTH: usize = 1;

/// Maximum number of results that can be requested
const MAX_RESULTS: u32 = 100;

/// Minimum number of results that can be requested
const MIN_RESULTS: u32 = 1;

/// Validates a web search request for common errors
pub fn validate_web_search_request(request: &WebSearchRequest) -> Result<()> {
    // Validate query
    validate_non_empty_string(&request.query, "query")?;
    validate_string_length(&request.query, "query", MIN_QUERY_LENGTH, MAX_QUERY_LENGTH)?;

    // Validate query content - should contain meaningful characters
    validate_query_content(&request.query)?;

    // Validate number of results if specified
    if let Some(num_results) = request.num_results {
        validate_numeric_range(num_results, "num_results", MIN_RESULTS, MAX_RESULTS)?;
    }

    Ok(())
}

/// Validates that the query contains meaningful content
fn validate_query_content(query: &str) -> Result<()> {
    let trimmed = query.trim();

    // Check if query is just whitespace
    if trimmed.is_empty() {
        return Err(Error::ConfigError(
            "Search query cannot be empty or contain only whitespace".to_string(),
        ));
    }

    // Check if query contains at least some alphanumeric characters
    let has_alphanumeric = trimmed.chars().any(|c| c.is_alphanumeric());
    if !has_alphanumeric {
        return Err(Error::ConfigError(
            "Search query must contain at least some alphanumeric characters".to_string(),
        ));
    }

    // Check for potentially malicious or problematic content
    validate_query_safety(trimmed)?;

    Ok(())
}

/// Validates query content for safety and potential issues
fn validate_query_safety(query: &str) -> Result<()> {
    // Convert to lowercase for case-insensitive checks
    let query_lower = query.to_lowercase();

    // List of potentially problematic patterns
    let problematic_patterns = [
        "javascript:",
        "data:",
        "vbscript:",
        "file:",
        "ftp:",
        "<script",
        "</script",
        "onclick",
        "onerror",
        "onload",
        "onmouseover",
        "eval(",
        "alert(",
        "confirm(",
        "prompt(",
    ];

    // Check for potentially dangerous patterns
    for pattern in &problematic_patterns {
        if query_lower.contains(pattern) {
            return Err(Error::ConfigError(format!(
                "Search query contains potentially unsafe content: {}",
                pattern
            )));
        }
    }

    // Check for excessive repetition (likely spam/bot behavior)
    if has_excessive_repetition(query) {
        return Err(Error::ConfigError(
            "Search query appears to contain excessive repetitive content".to_string(),
        ));
    }

    // Check for URL patterns that might be direct injection attempts
    if looks_like_url_injection(query) {
        return Err(Error::ConfigError(
            "Search query appears to contain URL injection patterns".to_string(),
        ));
    }

    Ok(())
}

/// Checks if a query has excessive character repetition
pub fn has_excessive_repetition(query: &str) -> bool {
    let chars: Vec<char> = query.chars().collect();
    if chars.len() < 10 {
        return false;
    }

    // Check for the same character repeated many times
    let mut consecutive_count = 1;
    let mut max_consecutive = 1;

    for i in 1..chars.len() {
        if chars[i] == chars[i - 1] {
            consecutive_count += 1;
            max_consecutive = max_consecutive.max(consecutive_count);
        } else {
            consecutive_count = 1;
        }
    }

    // Flag if more than 10 consecutive same characters
    max_consecutive > 10
}

/// Checks if a query looks like a URL injection attempt
pub fn looks_like_url_injection(query: &str) -> bool {
    // Look for patterns like multiple http://, weird URL structures
    let http_count = query.matches("http").count();
    let url_count = query.matches("://").count();

    // More than 2 URLs is suspicious for a search query
    http_count > 2 || url_count > 2
}

/// Validates search query format and provides suggestions for improvement
pub fn validate_and_suggest_query_improvement(query: &str) -> Result<Vec<String>> {
    validate_web_search_request(&WebSearchRequest {
        query: query.to_string(),
        num_results: None,
    })?;

    let mut suggestions = Vec::new();

    // Suggest improvements based on query analysis
    if query.len() < 5 {
        suggestions
            .push("Consider using a more specific search query for better results".to_string());
    }

    // Only suggest adding keywords if the query is very short and doesn't have
    // common search operators like + or -
    if query.len() < 15
        && !query.contains('+')
        && !query.contains('-')
        && query.chars().filter(|c| c.is_whitespace()).count() == 0
    {
        suggestions.push("Consider adding more keywords to your search query".to_string());
    }

    if query.to_lowercase().starts_with("what is")
        || query.to_lowercase().starts_with("how do")
        || query.to_lowercase().starts_with("why does")
    {
        suggestions
            .push("Your query looks like a question - consider rephrasing as keywords".to_string());
    }

    Ok(suggestions)
}

/// Estimates the complexity of a search query for rate limiting purposes
pub fn estimate_query_complexity(query: &str) -> u8 {
    let mut complexity = 1u8;

    // Add complexity for longer queries
    if query.len() >= 50 {
        complexity += 1;
    }

    // Add complexity for quotes (exact phrases)
    if query.contains('"') {
        complexity += 1;
    }

    // Add complexity for advanced operators
    if query.contains("AND:") || query.contains("OR:") || query.contains("NOT:") {
        complexity += 2;
    }

    // Add complexity for site-specific searches
    if query.to_lowercase().contains("site:") {
        complexity += 1;
    }

    // Add complexity for file type searches
    if query.to_lowercase().contains("filetype:") {
        complexity += 1;
    }

    complexity.min(8) // Cap at 8 for rate limiting
}

/// Validates that the number of requested results is reasonable for the query complexity
pub fn validate_results_for_complexity(request: &WebSearchRequest) -> Result<()> {
    let complexity = estimate_query_complexity(&request.query);

    if let Some(num_results) = request.num_results {
        // Simple queries can request more results
        let max_allowed = match complexity {
            1..=2 => 100, // Simple queries
            3..=4 => 50,  // Moderate complexity
            5..=6 => 25,  // Complex queries
            _ => 10,      // Very complex queries
        };

        if num_results > max_allowed {
            return Err(Error::ConfigError(format!(
                "Query complexity ({}) limits maximum results to {}. Consider simplifying your query or requesting fewer results.",
                complexity, max_allowed
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_web_search_request() -> WebSearchRequest {
        WebSearchRequest {
            query: "rust programming language".to_string(),
            num_results: Some(10),
        }
    }

    #[test]
    fn test_validate_web_search_request_valid() {
        let request = create_valid_web_search_request();
        assert!(validate_web_search_request(&request).is_ok());
    }

    #[test]
    fn test_validate_web_search_request_empty_query() {
        let mut request = create_valid_web_search_request();
        request.query = "".to_string();
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_whitespace_query() {
        let mut request = create_valid_web_search_request();
        request.query = "   ".to_string();
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_too_long() {
        let mut request = create_valid_web_search_request();
        request.query = "a".repeat(1001);
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_no_alphanumeric() {
        let mut request = create_valid_web_search_request();
        request.query = "!@#$%^&*()".to_string();
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_javascript_injection() {
        let mut request = create_valid_web_search_request();
        request.query = "test javascript:alert('xss')".to_string();
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_script_tags() {
        let mut request = create_valid_web_search_request();
        request.query = "test <script>alert('xss')</script>".to_string();
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_valid_results() {
        let mut request = create_valid_web_search_request();
        request.num_results = Some(50);
        assert!(validate_web_search_request(&request).is_ok());
    }

    #[test]
    fn test_validate_web_search_request_too_many_results() {
        let mut request = create_valid_web_search_request();
        request.num_results = Some(150);
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_zero_results() {
        let mut request = create_valid_web_search_request();
        request.num_results = Some(0);
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_has_excessive_repetition() {
        assert!(!has_excessive_repetition("normal query"));
        assert!(has_excessive_repetition("aaaaaaaaaaaaaaaaaaa"));
        assert!(!has_excessive_repetition("aa aa aa aa"));
    }

    #[test]
    fn test_looks_like_url_injection() {
        assert!(!looks_like_url_injection("normal search query"));
        assert!(looks_like_url_injection(
            "http://example.com http://malicious.com http://spam.com"
        ));
    }

    #[test]
    fn test_estimate_query_complexity() {
        assert_eq!(estimate_query_complexity("simple"), 1);
        assert_eq!(
            estimate_query_complexity(
                "longer query with more words than average length that exceeds fifty characters"
            ),
            2
        );
        assert_eq!(estimate_query_complexity("\"exact phrase\" search"), 2);
        assert_eq!(estimate_query_complexity("site:example.com search"), 2);
        assert_eq!(estimate_query_complexity("AND:this OR:that NOT:other"), 3);
    }

    #[test]
    fn test_validate_results_for_complexity() {
        let simple_request = WebSearchRequest {
            query: "simple search".to_string(),
            num_results: Some(100),
        };
        assert!(validate_results_for_complexity(&simple_request).is_ok());

        let complex_request = WebSearchRequest {
            query: "AND:complex OR:query site:example.com filetype:pdf \"exact phrase\""
                .to_string(),
            num_results: Some(50),
        };
        assert!(validate_results_for_complexity(&complex_request).is_err());
    }

    #[test]
    fn test_validate_and_suggest_query_improvement() {
        let suggestions = validate_and_suggest_query_improvement("rust").unwrap();
        assert!(!suggestions.is_empty());

        let suggestions = validate_and_suggest_query_improvement(
            "rust programming advanced comprehensive guide tutorial",
        )
        .unwrap();
        assert_eq!(suggestions.len(), 0); // Good query, no suggestions
    }
}
