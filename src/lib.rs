//! # OpenRouter API Client Library
//!
//! A Rust client for interfacing with the OpenRouter API.

pub mod api;
pub mod client;
pub mod error;
pub mod mcp; // Add the MCP module
pub mod models;
pub mod tests;
pub mod types;
pub mod utils;

pub use error::{Error, Result};
pub use types::*;

pub use client::{NoAuth, OpenRouterClient, Ready, Unconfigured};
pub use mcp::client::MCPClient; // Re-export MCPClient
pub use mcp::types as mcp_types; // Re-export MCP types
