//! Unit tests for authentication utilities

#[cfg(test)]
mod tests {
    use crate::utils::auth::{
        load_api_key_from_env, load_secure_api_key_from_env, validate_api_key,
    };
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_load_api_key_from_env_success() {
        // Ensure clean state first
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("OR_API_KEY");

        // Set up a test environment variable
        env::set_var(
            "OPENROUTER_API_KEY",
            "sk-test1234567890abcdef1234567890abcdef",
        );

        let result = load_api_key_from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-test1234567890abcdef1234567890abcdef");

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_api_key_from_env_missing() {
        // Ensure the environment variable is not set
        env::remove_var("OPENROUTER_API_KEY");

        let result = load_api_key_from_env();
        assert!(result.is_err());

        if let Err(error) = result {
            // Check that it's a ConfigError about missing environment variable
            let error_msg = format!("{error}");
            assert!(error_msg.contains("OPENROUTER_API_KEY"));
            assert!(error_msg.contains("environment"));
        }
    }

    #[test]
    #[serial]
    fn test_load_api_key_from_env_empty() {
        // Set environment variable to empty string
        env::set_var("OPENROUTER_API_KEY", "");

        let result = load_api_key_from_env();
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{error}");
            assert!(error_msg.contains("not found") || error_msg.contains("MissingCredential"));
        }

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_api_key_from_env_whitespace() {
        // Set environment variable to whitespace only
        env::set_var("OPENROUTER_API_KEY", "   \n\t   ");

        let result = load_api_key_from_env();
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{error}");
            assert!(error_msg.contains("not found") || error_msg.contains("MissingCredential"));
        }

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_api_key_returns_raw_value() {
        // Set environment variable with surrounding whitespace
        env::set_var(
            "OPENROUTER_API_KEY",
            "  sk-test1234567890abcdef1234567890abcdef  ",
        );

        let result = load_api_key_from_env();
        assert!(result.is_ok());
        // The function returns the raw value, trimming is done in validation
        assert_eq!(
            result.unwrap(),
            "  sk-test1234567890abcdef1234567890abcdef  "
        );

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_api_key_with_newlines() {
        // Set environment variable with newlines (which can happen in some CI environments)
        env::set_var(
            "OPENROUTER_API_KEY",
            "sk-test1234567890abcdef1234567890abcdef\n",
        );

        let result = load_api_key_from_env();
        assert!(result.is_ok());
        // The function returns the raw value, trimming is done in validation
        assert_eq!(result.unwrap(), "sk-test1234567890abcdef1234567890abcdef\n");

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_api_key_preserves_internal_content() {
        // Test that internal content is preserved (no trimming within the key)
        let test_key = "sk-test123_with-special.chars890abcdef";

        // Ensure clean state first
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("OR_API_KEY");

        env::set_var("OPENROUTER_API_KEY", test_key);

        let result = load_api_key_from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), test_key);

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[tokio::test]
    #[serial]
    async fn test_api_key_integration_with_client() {
        // Integration test: load API key and use it with client
        use crate::client::{OpenRouterClient, Unconfigured};

        // Ensure clean state first
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("OR_API_KEY");

        env::set_var(
            "OPENROUTER_API_KEY",
            "sk-test1234567890abcdef1234567890abcdef123456789",
        );

        let api_key = load_api_key_from_env().unwrap();
        let client_result = OpenRouterClient::<Unconfigured>::new()
            .with_base_url("https://api.example.com/")
            .unwrap()
            .with_api_key(api_key);

        assert!(client_result.is_ok());

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    fn test_auth_functions_thread_safe() {
        // Test that auth functions can be called safely from multiple threads
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;
        use std::thread;

        // Test validate_api_key function concurrency
        let success_count = Arc::new(AtomicU32::new(0));
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let success_count = Arc::clone(&success_count);
                thread::spawn(move || {
                    let test_key = format!("sk-thread{i}1234567890abcdef1234567890abcdef");
                    if validate_api_key(&test_key).is_ok() {
                        success_count.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // All 10 threads should succeed with validate_api_key
        assert_eq!(success_count.load(Ordering::SeqCst), 10);
    }

    #[test]
    fn test_validate_api_key_valid() {
        let result = validate_api_key("sk-1234567890abcdef");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_key_empty() {
        let result = validate_api_key("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_key_whitespace() {
        let result = validate_api_key("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_key_too_short() {
        let result = validate_api_key("short");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_key_with_whitespace() {
        let result = validate_api_key("  sk-1234567890abcdef  ");
        assert!(result.is_ok()); // Should trim and validate
    }

    #[test]
    #[serial]
    fn test_load_secure_api_key_from_env_success() {
        env::set_var(
            "OPENROUTER_API_KEY",
            "sk-1234567890abcdef1234567890abcdef123456789",
        );

        let result = load_secure_api_key_from_env();
        assert!(result.is_ok());

        let secure_key = result.unwrap();
        assert_eq!(
            secure_key.as_str(),
            "sk-1234567890abcdef1234567890abcdef123456789"
        );

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_secure_api_key_from_env_invalid() {
        env::set_var("OPENROUTER_API_KEY", "invalid-key");

        let result = load_secure_api_key_from_env();
        assert!(result.is_err());

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
    }

    #[test]
    #[serial]
    fn test_load_secure_api_key_from_env_missing() {
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("OR_API_KEY");

        let result = load_secure_api_key_from_env();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_or_api_key_fallback() {
        // Remove the primary env var and set the fallback
        env::remove_var("OPENROUTER_API_KEY");
        env::set_var("OR_API_KEY", "or-1234567890abcdef1234567890abcdef");

        let result = load_api_key_from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "or-1234567890abcdef1234567890abcdef");

        // Clean up
        env::remove_var("OR_API_KEY");
    }

    #[test]
    #[serial]
    fn test_or_api_key_priority() {
        // Both env vars set, should prefer OPENROUTER_API_KEY
        env::set_var("OPENROUTER_API_KEY", "sk-primary-key-1234567890abcdef");
        env::set_var("OR_API_KEY", "or-fallback-key-1234567890abcdef");

        let result = load_api_key_from_env();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "sk-primary-key-1234567890abcdef");

        // Clean up
        env::remove_var("OPENROUTER_API_KEY");
        env::remove_var("OR_API_KEY");
    }
}
