//! Validation utilities module for all API endpoints
//!
//! This module provides comprehensive input validation across all OpenRouter API endpoints.
//! It includes common validation utilities as well as endpoint-specific validation functions.
//!
//! # Structure
//!
//! - [`common`] - Shared validation utilities used across all endpoints
//! - [`chat`] - Chat completion request validation
//! - [`completion`] - Text completion request validation
//! - [`web_search`] - Web search request validation
//! - [`analytics`] - Analytics request validation
//! - [`models`] - Models API request validation
//! - [`credits`] - Credits API request validation
//! - [`providers`] - Providers API request validation
//! - [`generation`] - Generation API request validation
//! - [`structured`] - Structured output request validation
//!
//! # Usage
//!
//! ```rust
//! use openrouter_api::utils::validation::{
//!     validate_chat_request, check_token_limits,
//!     validate_completion_request, validate_web_search_request
//! };
//!
//! // Validate chat request
//! validate_chat_request(&chat_request)?;
//! check_token_limits(&chat_request)?;
//!
//! // Validate completion request
//! validate_completion_request(&completion_request)?;
//!
//! // Validate web search request
//! validate_web_search_request(&search_request)?;
//! ```

pub mod chat;
pub mod common;
pub mod completion;
pub mod web_search;

// Re-export commonly used validation functions for convenience
pub use chat::{check_token_limits, validate_chat_request};
pub use common::{
    validate_date_format, validate_date_range, validate_enum_value, validate_model_id,
    validate_non_empty_collection, validate_non_empty_string, validate_numeric_range,
    validate_sampling_parameters, validate_string_length, validate_url,
};
pub use completion::{check_prompt_token_limits, validate_completion_request};
pub use web_search::{
    estimate_query_complexity, validate_and_suggest_query_improvement,
    validate_results_for_complexity, validate_web_search_request,
};

// Additional endpoint validation modules (to be implemented)
// pub mod analytics;
// pub mod models;
// pub mod credits;
// pub mod providers;
// pub mod generation;
// pub mod structured;

// Include test modules
#[cfg(test)]
mod tests;

// Include endpoint-specific test modules
#[cfg(test)]
mod completion_tests;
#[cfg(test)]
mod web_search_tests;
