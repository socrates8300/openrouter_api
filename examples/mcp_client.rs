//! # MCP Client Example
//!
//! This example demonstrates how to use the `MCPClient` to interact with a
//! Model Context Protocol (MCP) server.
//!
//! **Note:** This example requires a running MCP server. The URL used here
//! (`https://mcp-server.example.com/mcp`) is a placeholder.
//! You will need to replace it with the actual URL of your MCP server.

use openrouter_api::{mcp_types::*, MCPClient, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Create a new MCP client.
    // Replace this URL with the actual URL of your MCP server.
    let client = MCPClient::new("https://mcp-server.example.com/mcp")?;

    // 1. Initialize the client with its capabilities.
    println!("Initializing MCP client...");
    let server_capabilities = client
        .initialize(ClientCapabilities {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            supports_sampling: Some(true),
        })
        .await?;

    println!(
        "Connected to MCP server with capabilities: {:?}",
        server_capabilities
    );

    // 2. Get a resource from the MCP server.
    let resource_id = "document-123".to_string();
    println!("\nAttempting to get resource: {}", resource_id);
    match client
        .get_resource(GetResourceParams {
            id: resource_id,
            parameters: None,
        })
        .await
    {
        Ok(resource) => println!("Retrieved resource: {:?}", resource.contents),
        Err(e) => println!("Failed to get resource: {}", e),
    }

    // 3. Call a tool on the MCP server.
    let tool_id = "search-tool".to_string();
    println!("\nAttempting to call tool: {}", tool_id);
    match client
        .tool_call(ToolCallParams {
            id: tool_id,
            parameters: json!({
                "query": "Rust programming"
            }),
        })
        .await
    {
        Ok(result) => println!("Tool call result: {:?}", result.result),
        Err(e) => println!("Failed to call tool: {}", e),
    }

    Ok(())
}
