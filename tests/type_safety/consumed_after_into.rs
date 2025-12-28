//! Compile-Fail Test: Consumed Value After Into Conversion
//!
//! This test verifies that values consumed by `Into::into()` cannot be
//! used afterwards, preventing use-after-move errors. This is a fundamental
//! Rust ownership guarantee that ensures memory safety and prevents
//! undefined behavior from accessing moved values.

use openrouter_api::types::ModelId;

fn main() {
    let id = ModelId::new("test-id");

    // This consumes `id` and converts it to a String
    let s: String = id.into();

    // This should fail to compile: `id` was moved in the previous line
    // and is no longer accessible
    let _ = id.as_str();

    // Expected error:
    //   use of moved value: `id`
    //    --> tests/type_safety/consumed_after_into.rs:XX:YY
    //     |
    //   XX |     let s: String = id.into();
    //      |                        -- value moved here
    //   XX |     let _ = id.as_str();
    //      |              ^^^ value used here after move
    //
    // This is a fundamental Rust ownership rule:
    // 1. `into()` takes ownership of the value
    // 2. The original variable is no longer valid
    // 3. Any attempt to use it results in a compile error
    //
    // Correct approaches:
    //
    // 1. Use `as_str()` for temporary read-only access (doesn't consume):
    //    let id = ModelId::new("test-id");
    //    let s: &str = id.as_str();
    //    let _ = id.as_str(); // Still valid - id wasn't moved
    //
    // 2. Clone before converting if you need both:
    //    let id = ModelId::new("test-id");
    //    let id_copy = id.clone(); // Explicit clone
    //    let s: String = id.into(); // Original is moved
    //    let _ = id_copy.as_str(); // Clone is still valid
    //
    // 3. Use `as_ref()` for reference conversion (doesn't consume):
    //    let id = ModelId::new("test-id");
    //    let s: &str = id.as_ref();
    //    let _ = id.as_str(); // Still valid
    //
    // 4. Chain operations to avoid intermediate moves:
    //    let id = ModelId::new("test-id");
    //    let s: String = id.clone().into(); // Clone is moved, original stays
    //    let _ = id.as_str(); // Original is still valid
    //
    // The key insight: `Into::into()` is a consuming transformation.
    // If you need to keep the original, either clone it first or use
    // a non-consuming method like `as_ref()` or `as_str()`.
    //
    // This compile-time guarantee prevents:
    // - Use-after-move bugs
    // - Double-free errors
    // - Data races from concurrent access
    // - Undefined behavior from accessing invalid memory
}
