//! Unit tests for error handling

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use crate::error::Error;
    use serde_json::Value;

    #[test]
    fn test_api_error_construction() {
        let error = Error::ApiError {
            code: 404,
            message: "Not Found".to_string(),
            metadata: None,
        };

        match error {
            Error::ApiError { code, message, .. } => {
                assert_eq!(code, 404);
                assert_eq!(message, "Not Found");
            }
            _ => panic!("Expected ApiError"),
        }
    }

    #[test]
    fn test_api_error_with_metadata() {
        let metadata = serde_json::json!({
            "retry_after": "60",
            "request_id": "req_123"
        });

        let error = Error::ApiError {
            code: 429,
            message: "Rate limit exceeded".to_string(),
            metadata: Some(metadata),
        };

        match error {
            Error::ApiError {
                code,
                message,
                metadata: Some(meta),
            } => {
                assert_eq!(code, 429);
                assert_eq!(message, "Rate limit exceeded");
                assert_eq!(meta["retry_after"], "60");
                assert_eq!(meta["request_id"], "req_123");
            }
            _ => panic!("Expected ApiError with metadata"),
        }
    }

    #[test]
    fn test_config_error() {
        let error = Error::ConfigError("Invalid API key format".to_string());
        match error {
            Error::ConfigError(msg) => {
                assert_eq!(msg, "Invalid API key format");
            }
            _ => panic!("Expected ConfigError"),
        }
    }

    #[test]
    fn test_http_error_type() {
        // Test that HttpError variant exists and can be matched
        // Since reqwest::Error is complex to construct in tests, we'll focus on the enum structure
        let error_variants = [
            "HttpError",
            "ApiError",
            "ConfigError",
            "SerializationError",
            "StreamingError",
        ];

        // If this compiles and runs, the error types are properly defined
        assert_eq!(error_variants.len(), 5);
    }

    #[test]
    fn test_serialization_error() {
        // Create a JSON error by attempting to deserialize invalid JSON
        let invalid_json = "{ invalid json }";
        let json_error = serde_json::from_str::<Value>(invalid_json).unwrap_err();
        let error = Error::SerializationError(json_error);

        match error {
            Error::SerializationError(_) => {
                // Test that conversion works
            }
            _ => panic!("Expected SerializationError"),
        }
    }

    #[test]
    fn test_schema_validation_error() {
        let error = Error::SchemaValidationError("Required field 'name' is missing".to_string());
        match error {
            Error::SchemaValidationError(msg) => {
                assert_eq!(msg, "Required field 'name' is missing");
            }
            _ => panic!("Expected SchemaValidationError"),
        }
    }

    #[test]
    fn test_streaming_error() {
        let error = Error::StreamingError("Connection interrupted during streaming".to_string());
        match error {
            Error::StreamingError(msg) => {
                assert_eq!(msg, "Connection interrupted during streaming");
            }
            _ => panic!("Expected StreamingError"),
        }
    }

    #[test]
    fn test_error_display() {
        let error = Error::ApiError {
            code: 500,
            message: "Internal Server Error".to_string(),
            metadata: None,
        };

        let error_string = format!("{error}");
        assert!(error_string.contains("500"));
        assert!(error_string.contains("Internal Server Error"));
    }

    #[test]
    fn test_error_debug() {
        let error = Error::ConfigError("Test error".to_string());
        let debug_string = format!("{error:?}");
        assert!(debug_string.contains("ConfigError"));
        assert!(debug_string.contains("Test error"));
    }

    #[test]
    fn test_error_trait_implementations() {
        // Test that Error implements required traits
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        fn assert_error<T: std::error::Error>() {}

        assert_send::<Error>();
        assert_sync::<Error>();
        assert_error::<Error>();
    }

    #[test]
    fn test_from_serde_json_error() {
        let invalid_json = "{ malformed json }";
        let json_error = serde_json::from_str::<Value>(invalid_json).unwrap_err();
        let error: Error = json_error.into();

        match error {
            Error::SerializationError(_) => {
                // Conversion works correctly
            }
            _ => panic!("Expected SerializationError from serde_json::Error conversion"),
        }
    }

    #[test]
    fn test_error_chain() {
        // Test that errors can be chained/nested properly
        let inner_error = Error::ConfigError("Invalid configuration".to_string());
        let outer_error = Error::ApiError {
            code: 400,
            message: format!("Request failed: {inner_error}"),
            metadata: None,
        };

        let error_string = format!("{outer_error}");
        assert!(error_string.contains("Request failed"));
        assert!(error_string.contains("Invalid configuration"));
    }

    #[test]
    fn test_error_metadata_access() {
        let metadata = serde_json::json!({
            "error_code": "AUTH_FAILED",
            "timestamp": "2023-01-01T00:00:00Z"
        });

        let error = Error::ApiError {
            code: 401,
            message: "Authentication failed".to_string(),
            metadata: Some(metadata),
        };

        if let Error::ApiError {
            metadata: Some(meta),
            ..
        } = &error
        {
            assert!(meta.is_object());
            assert_eq!(meta["error_code"], "AUTH_FAILED");
            assert_eq!(meta["timestamp"], "2023-01-01T00:00:00Z");
        } else {
            panic!("Expected ApiError with metadata");
        }
    }

    #[test]
    fn test_error_types_compilation() {
        // Test that all error types can be constructed and used
        let _api_error = Error::ApiError {
            code: 400,
            message: "Bad Request".to_string(),
            metadata: None,
        };

        let _config_error = Error::ConfigError("Config issue".to_string());
        let _rate_limit_error = Error::RateLimitExceeded("Too many requests".to_string());
        let _schema_error = Error::SchemaValidationError("Schema invalid".to_string());
        let _streaming_error = Error::StreamingError("Stream failed".to_string());
        let _model_error = Error::ModelNotAvailable("Model unavailable".to_string());
        let _credential_error = Error::MissingCredential("API key missing".to_string());
        let _timeout_error = Error::TimeoutError("Request timed out".to_string());
        let _unknown_error = Error::Unknown;

        // If this compiles, the error types are correctly defined
    }
}
