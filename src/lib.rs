//! # OpenRouter API Client Library
//!
//! A Rust client for interfacing with the OpenRouter API.
//!
//! ## Model Context Protocol (MCP)
//!
//! This library includes a client implementation for the Model Context Protocol (MCP),
//! allowing connection to MCP servers for resource retrieval, tool execution, and prompt handling.
//!
//! See [`MCPClient`] for more details.

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

// Ensure TLS features are mutually exclusive
#[cfg(all(feature = "rustls", feature = "native-tls"))]
compile_error!("rustls and native-tls features are mutually exclusive. Please choose one.");
