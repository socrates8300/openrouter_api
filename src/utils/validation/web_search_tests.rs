//! Unit tests for web search validation utilities

#[cfg(test)]
mod tests {
    use crate::types::web_search::WebSearchRequest;
    use crate::utils::validation::web_search::{
        has_excessive_repetition, looks_like_url_injection,
    };
    use crate::utils::validation::{
        estimate_query_complexity, validate_and_suggest_query_improvement,
        validate_results_for_complexity, validate_web_search_request,
    };

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
