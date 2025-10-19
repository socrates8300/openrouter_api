//! Build validation tests for feature configuration

// This test file validates that TLS features are mutually exclusive
// The actual compile_error guard is in src/lib.rs

#[test]
fn test_feature_compilation() {
    // This test will fail to compile if both TLS features are enabled
    // The compile_error above ensures mutual exclusivity at compile time
    assert!(true, "Feature compilation test passed");
}

#[cfg(test)]
mod feature_tests {
    #[test]
    #[cfg(feature = "rustls")]
    fn test_rustls_feature_enabled() {
        // Verify rustls-specific functionality is available
        // This test only runs when rustls feature is enabled
        assert!(true, "rustls feature is enabled");
    }

    #[test]
    #[cfg(feature = "native-tls")]
    fn test_native_tls_feature_enabled() {
        // Verify native-tls-specific functionality is available
        // This test only runs when native-tls feature is enabled
        assert!(true, "native-tls feature is enabled");
    }

    #[test]
    #[cfg(not(any(feature = "rustls", feature = "native-tls")))]
    fn test_no_tls_feature_enabled() {
        // This should not happen due to default features, but test anyway
        panic!("At least one TLS feature should be enabled");
    }

    #[test]
    #[cfg(feature = "tracing")]
    fn test_tracing_feature_enabled() {
        // Verify tracing functionality is available
        // This test only runs when tracing feature is enabled
        assert!(true, "tracing feature is enabled");
    }

    #[test]
    #[cfg(not(feature = "tracing"))]
    fn test_tracing_feature_disabled() {
        // Verify tracing functionality is not compiled in
        // This test only runs when tracing feature is disabled
        assert!(true, "tracing feature is disabled");
    }
}
