//! Compile-Fail Test: Enum Invalid Construction from String
//!
//! This test verifies that enum types like ChatRole cannot be constructed
//! from arbitrary string values, preventing typos and undefined roles.

use openrouter_api::types::ChatRole;

fn main() {
    // This should fail to compile: ChatRole does not implement From<String>
    let role: ChatRole = "admin".into();

    // Expected error:
    //   the trait `From<&str>` is not implemented for `ChatRole`
    //   the trait `From<String>` is not implemented for `ChatRole`
    //
    // Only the four valid variants can be constructed:
    //   ChatRole::User
    //   ChatRole::Assistant
    //   ChatRole::System
    //   ChatRole::Tool
    //
    // This prevents typos like "admnin" or undefined roles like "moderator"
    // from being represented at compile time.

    let _ = role;
}
