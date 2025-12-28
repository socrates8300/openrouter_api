//! Compile-Fail Test: Enum Integer Construction
//!
//! This test verifies that enum types like ChatRole cannot be constructed
//! from integer values, preventing unsafe type coercion and ensuring that
//! only valid enum variants can be created.

use openrouter_api::types::ChatRole;

fn main() {
    // This should fail to compile: cannot construct ChatRole from integer
    let role: ChatRole = 0;

    // Expected error:
    //   expected enum `ChatRole`, found integer
    //   note: an implementation of `From<i32>` might be missing for `ChatRole`
    //
    // This prevents unsafe behavior where integers could be cast to enum variants.
    // Only explicit enum variants are valid:
    //   ChatRole::User
    //   ChatRole::Assistant
    //   ChatRole::System
    //   ChatRole::Tool
    //
    // Using unsafe transmute would work but is strongly discouraged:
    // let role: ChatRole = unsafe { std::mem::transmute(0u8) };
    // This bypasses type safety and could create invalid enum states.

    let _ = role;
}
