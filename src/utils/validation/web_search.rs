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
    fn test_validate_web_search_request_tab_newline_query() {
        let mut request = create_valid_web_search_request();
        request.query = "\t\n\r".to_string();
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_minimum_length() {
        let mut request = create_valid_web_search_request();
        request.query = "a".to_string();
        assert!(validate_web_search_request(&request).is_ok());
    }

    #[test]
    fn test_validate_web_search_request_maximum_length() {
        let mut request = create_valid_web_search_request();
        // Create a 1000 character query that isn't repetitive
        let base = "This is a test query with varied content to avoid repetition detection ";
        let query = base.repeat(20); // This will be more than 1000 chars
        request.query = query[..1000].to_string();
        assert_eq!(request.query.len(), 1000);
        assert!(validate_web_search_request(&request).is_ok());
    }

    #[test]
    fn test_validate_web_search_request_too_long() {
        let mut request = create_valid_web_search_request();
        request.query = "a".repeat(1001);
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_no_alphanumeric() {
        let test_cases = ["!@#$%^&*()", "-----", "     ", "!@# $%^", "....", "???"];

        for query in test_cases {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_err());
        }
    }

    #[test]
    fn test_validate_web_search_request_with_alphanumeric() {
        let test_cases = [
            "test123",
            "hello world",
            "rust programming",
            "a!b@c#",
            "123 456",
        ];

        for query in test_cases {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_ok());
        }
    }

    #[test]
    fn test_validate_web_search_request_javascript_injection() {
        let injection_attempts = [
            "test javascript:alert('xss')",
            "javascript:void(0)",
            "JAVASCRIPT:alert(1)",
            "test javascript:document.cookie",
        ];

        for query in injection_attempts {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_err());
        }
    }

    #[test]
    fn test_validate_web_search_request_data_uri() {
        let data_uri_attempts = [
            "test data:text/html,<script>alert(1)</script>",
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==",
            "DATA:application/json,{\"malicious\":true}",
        ];

        for query in data_uri_attempts {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_err());
        }
    }

    #[test]
    fn test_validate_web_search_request_script_tags() {
        let script_attempts = [
            "test <script>alert('xss')</script>",
            "<script>document.location='http://evil.com'</script>",
            "test <SCRIPT>alert(1)</SCRIPT>",
            "<script src='evil.js'></script>",
        ];

        for query in script_attempts {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_err());
        }
    }

    #[test]
    fn test_validate_web_search_request_event_handlers() {
        let event_attempts = [
            "test onclick='alert(1)'",
            "onerror='malicious()'",
            "onload='document.cookie'",
            "ONCLICK='xss()'",
            "test onmouseover='dangerous()'",
        ];

        for query in event_attempts {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_err());
        }
    }

    #[test]
    fn test_validate_web_search_request_eval_attempts() {
        let eval_attempts = [
            "eval('alert(1)')",
            "test eval('malicious code')",
            " EVAL('dangerous') ",
            "window.eval('xss')",
        ];

        for query in eval_attempts {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_err());
        }
    }

    #[test]
    fn test_validate_web_search_request_valid_results() {
        let test_cases = [1, 5, 10, 25, 50, 100];

        for num_results in test_cases {
            let mut request = create_valid_web_search_request();
            request.num_results = Some(num_results);
            assert!(validate_web_search_request(&request).is_ok());
        }
    }

    #[test]
    fn test_validate_web_search_request_too_many_results() {
        let mut request = create_valid_web_search_request();
        request.num_results = Some(101);
        assert!(validate_web_search_request(&request).is_err());

        request.num_results = Some(1000);
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_zero_results() {
        let mut request = create_valid_web_search_request();
        request.num_results = Some(0);
        assert!(validate_web_search_request(&request).is_err());
    }

    #[test]
    fn test_validate_web_search_request_no_results() {
        let mut request = create_valid_web_search_request();
        request.num_results = None;
        assert!(validate_web_search_request(&request).is_ok());
    }

    #[test]
    fn test_has_excessive_repetition() {
        let test_cases = [
            ("normal query", false),
            ("hello world", false),
            ("aaaaaaaaaaaaaaaaaaa", true),
            ("bbbbbbbbbbbbbbbbbbbb", true),
            ("aa aa aa aa", false), // separated by spaces
            ("hello!!!!!", false),  // not the same character
            ("??????????????", true),
            ("$$$$$$$$$$$$$$$$$$", true),
            ("test", false),
            ("a", false), // too short to trigger
        ];

        for (query, expected) in test_cases {
            assert_eq!(
                has_excessive_repetition(query),
                expected,
                "Query '{}' should have excessive repetition: {}",
                query,
                expected
            );
        }
    }

    #[test]
    fn test_looks_like_url_injection() {
        let test_cases = [
            ("normal search query", false),
            ("rust programming tutorial", false),
            ("http://example.com", false),
            ("https://example.com test", false),
            (
                "http://example.com http://malicious.com http://spam.com",
                true,
            ),
            (
                "https://site1.com https://site2.com https://site3.com",
                true,
            ),
            ("http://test.com https://test2.com", false), // exactly 2 is ok
            ("://not-a-url", false),
            ("test http://example.com", false),
        ];

        for (query, expected) in test_cases {
            assert_eq!(
                looks_like_url_injection(query),
                expected,
                "Query '{}' should look like URL injection: {}",
                query,
                expected
            );
        }
    }

    #[test]
    fn test_estimate_query_complexity() {
        let test_cases = [
            ("simple", 1),
            ("rust programming", 1),
            ("longer query with more words than average length that exceeds fifty characters", 2),
            ("\"exact phrase\" search", 2),
            ("site:example.com search", 2),
            ("filetype:pdf documents", 2),
            ("AND:this OR:that NOT:other", 3),
            ("\"phrase\" site:example.com AND:search", 5),
            ("AND:complex OR:query site:example.com filetype:pdf \"exact phrase\"", 7),
            ("AND:very OR:complex AND:query OR:with AND:multiple OR:operators site:test.com filetype:pdf \"phrase\" \"another phrase\"", 7),
        ];

        for (query, expected) in test_cases {
            let actual = estimate_query_complexity(query);
            assert_eq!(
                actual, expected,
                "Query '{}' complexity should be {}, got {}",
                query, expected, actual
            );
        }
    }

    #[test]
    fn test_validate_results_for_complexity() {
        // Simple queries can request many results
        let simple_request = WebSearchRequest {
            query: "simple search".to_string(),
            num_results: Some(100),
        };
        assert!(validate_results_for_complexity(&simple_request).is_ok());

        // Moderate complexity queries have lower limits
        let moderate_request = WebSearchRequest {
            query: "\"exact phrase\" search".to_string(),
            num_results: Some(50),
        };
        assert!(validate_results_for_complexity(&moderate_request).is_ok());

        // Complex queries have even lower limits
        let complex_request = WebSearchRequest {
            query: "AND:complex OR:query site:example.com".to_string(),
            num_results: Some(25),
        };
        assert!(validate_results_for_complexity(&complex_request).is_ok());

        // Very complex queries have strict limits
        let very_complex_request = WebSearchRequest {
            query: "AND:very OR:complex AND:query OR:with AND:multiple site:test.com filetype:pdf \"phrase\""
                .to_string(),
            num_results: Some(15),
        };
        assert!(validate_results_for_complexity(&very_complex_request).is_err());

        // Complex query requesting too many results should fail
        let too_many_results_request = WebSearchRequest {
            query: "AND:complex OR:query site:example.com filetype:pdf \"exact phrase\""
                .to_string(),
            num_results: Some(50),
        };
        assert!(validate_results_for_complexity(&too_many_results_request).is_err());
    }

    #[test]
    fn test_validate_and_suggest_query_improvement() {
        // Very short query should get suggestions
        let suggestions = validate_and_suggest_query_improvement("rust").unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("more specific")));

        // Single word query should get suggestions
        let suggestions = validate_and_suggest_query_improvement("hello").unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("keywords")));

        // Question-like query should get suggestions
        let suggestions = validate_and_suggest_query_improvement("what is rust").unwrap();
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("question")));

        // Good query should not get suggestions
        let suggestions = validate_and_suggest_query_improvement(
            "rust programming tutorial guide advanced comprehensive",
        )
        .unwrap();
        assert_eq!(suggestions.len(), 0);

        // Query with special characters but still valid
        let suggestions =
            validate_and_suggest_query_improvement("rust+programming+tutorial").unwrap();
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_validate_web_search_request_unicode() {
        let unicode_queries = [
            "программирование на rust", // Russian
            "rust编程语言",             // Chinese
            "rustプログラミング",       // Japanese
            "rust programmación",       // Spanish with accent
            "rust 🦀 programming",      // Emoji
        ];

        for query in unicode_queries {
            let mut request = create_valid_web_search_request();
            request.query = query.to_string();
            assert!(validate_web_search_request(&request).is_ok());
        }
    }

    #[test]
    fn test_validate_web_search_request_edge_cases() {
        // Query with mixed content
        let mut request = create_valid_web_search_request();
        request.query = "rust programming 🦀 tutorial 2024".to_string();
        assert!(validate_web_search_request(&request).is_ok());

        // Query with numbers and letters
        let mut request = create_valid_web_search_request();
        request.query = "rust 1.75 programming tutorial".to_string();
        assert!(validate_web_search_request(&request).is_ok());

        // Query with allowed special characters
        let mut request = create_valid_web_search_request();
        request.query = "rust + programming - tutorial".to_string();
        assert!(validate_web_search_request(&request).is_ok());
    }

    #[test]
    fn test_validate_web_search_request_comprehensive() {
        let mut request = create_valid_web_search_request();
        request.query = "rust programming language tutorial for beginners 2024".to_string();
        request.num_results = Some(25);

        assert!(validate_web_search_request(&request).is_ok());

        // Also test the complexity validation
        assert!(validate_results_for_complexity(&request).is_ok());

        // Test suggestions
        let suggestions = validate_and_suggest_query_improvement(&request.query).unwrap();
        assert_eq!(suggestions.len(), 0); // Good query, no suggestions
    }
}
