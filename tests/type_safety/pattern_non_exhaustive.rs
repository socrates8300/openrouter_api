//! Compile-Fail Test: Non-Exhaustive Pattern Matching
//!
//! This test verifies that matching on enum types requires handling all
//! variants, ensuring that code won't compile if new enum variants are
//! added without updating all match statements. This is a critical
//! compile-time safety feature that prevents runtime panics from
//! unhandled enum cases.

use openrouter_api::types::{ChatRole, StreamingStatus};

// Example 1: Non-exhaustive match on ChatRole
fn describe_chat_role(role: ChatRole) -> &'static str {
    // This should fail to compile: non-exhaustive patterns
    match role {
        ChatRole::User => "user message",
        ChatRole::Assistant => "assistant response",
        // Missing: ChatRole::System and ChatRole::Tool
    }
}

fn main() {
    let role = ChatRole::System;
    let description = describe_chat_role(role);
    println!("{}", description);

    let _ = description;
}
