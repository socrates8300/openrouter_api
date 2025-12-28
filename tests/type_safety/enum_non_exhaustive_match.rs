//! Compile-Fail Test: Non-Exhaustive Enum Match
//!
//! This test verifies that matching on enum types requires handling all
//! variants, ensuring that code won't compile if new enum variants are
//! added without updating all match statements.

use openrouter_api::types::ChatRole;

fn describe_role(role: ChatRole) -> &'static str {
    // This should fail to compile: non-exhaustive patterns
    match role {
        ChatRole::User => "user message",
        ChatRole::Assistant => "assistant response",
        // Missing: ChatRole::System and ChatRole::Tool
        // ERROR: non-exhaustive patterns: `ChatRole::System` and `ChatRole::Tool` not covered
    }
}

fn main() {
    let role = ChatRole::System;
    let description = describe_role(role);
    println!("{}", description);

    // Expected error:
    //   error[E0004]: non-exhaustive patterns: `ChatRole::System` and `ChatRole::Tool` not covered
    //    --> tests/type_safety/enum_non_exhaustive_match.rs:13:11
    //     |
    //   13 |     match role {
    //      |           ^^^^ pattern `ChatRole::System` and `ChatRole::Tool` not covered
    //     |
    //   note: `ChatRole` defined here
    //    --> src/types/chat.rs:11:5
    //     |
    //   11 |     pub enum ChatRole {
    //      |     -------------- variant `ChatRole::System` not covered
    //   ...
    //   14 |     Tool,
    //      |     ---- variant `ChatRole::Tool` not covered
    //     = help: ensure that all possible cases are being handled, possibly by adding wildcards or more match arms
    //
    // Correct approaches:
    // 1. Handle all variants explicitly:
    //    match role {
    //        ChatRole::User => "user message",
    //        ChatRole::Assistant => "assistant response",
    //        ChatRole::System => "system instruction",
    //        ChatRole::Tool => "tool call result",
    //    }
    // 2. Use wildcard for future-proofing (less explicit):
    //    match role {
    //        ChatRole::User | ChatRole::Assistant => "chat message",
    //        _ => "special message",
    //    }
}
