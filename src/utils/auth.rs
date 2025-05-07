//! Authentication utilities for managing API keys and authorization tokens.

use crate::error::{Error, Result};
use dotenvy::dotenv;
use std::env;

/// Attempts to load an API key from environment variables.
/// Checks for OPENROUTER_API_KEY and OR_API_KEY.
/// Also loads from .env file if present.
pub fn load_api_key_from_env() -> Result<String> {
    // Load from .env file if present
    dotenv().ok();

    // Try to read the API key from common environment variables
    if let Ok(key) = env::var("OPENROUTER_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    
    if let Ok(key) = env::var("OR_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(key);
        }
    }
    
    Err(Error::MissingCredential(
        "API key not found in environment variables OPENROUTER_API_KEY or OR_API_KEY".into()
    ))
}

/// Validates an API key format.
/// Basic validation to check if the key is non-empty and has a reasonable length.
pub fn validate_api_key(key: &str) -> Result<()> {
    let key = key.trim();
    
    if key.is_empty() {
        return Err(Error::ConfigError("API key cannot be empty".into()));
    }
    
    if key.len() < 10 {
        return Err(Error::ConfigError("API key is too short".into()));
    }
    
    // Most OpenRouter keys are in a specific format, but we don't check exact patterns
    // to allow for different key formats in the future
    
    Ok(())
}

