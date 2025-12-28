// Integration tests for feature configuration
// This ensures the crate can be compiled with various feature combinations

#[test]
fn test_default_features_compile() {
    // The crate should compile with default features
    // This test will be run by cargo test with default features
    // If it fails, it means there's a compilation issue with default features
    let _ = openrouter_api::OpenRouterClient::<openrouter_api::Unconfigured>::new();
}

#[cfg(feature = "tls-rustls")]
#[test]
fn test_rustls_feature_compiles() {
    // The crate should compile with tls-rustls feature
    let _ = openrouter_api::OpenRouterClient::<openrouter_api::Unconfigured>::new();
}

#[cfg(feature = "rustls")]
#[test]
fn test_legacy_rustls_alias_compiles() {
    // The legacy rustls alias should still work
    let _ = openrouter_api::OpenRouterClient::<openrouter_api::Unconfigured>::new();
}

#[cfg(feature = "tls-native-tls")]
#[test]
fn test_native_tls_feature_compiles() {
    // The crate should compile with tls-native-tls feature
    let _ = openrouter_api::OpenRouterClient::<openrouter_api::Unconfigured>::new();
}

#[cfg(feature = "native-tls")]
#[test]
fn test_legacy_native_tls_alias_compiles() {
    // The legacy native-tls alias should still work
    let _ = openrouter_api::OpenRouterClient::<openrouter_api::Unconfigured>::new();
}

// Note: Testing that TLS features are mutually exclusive is done via
// compile-time compile_error! macro in src/lib.rs which checks for
// both tls-rustls and tls-native-tls being enabled simultaneously.
// This is enforced at build time, not test time.
