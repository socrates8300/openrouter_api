//! Compile-Fail Test: ID Comparison Mismatch
//!
//! This test verifies that different ID types cannot be compared directly
//! without explicit conversion, preventing accidental mixing of entity IDs.

use openrouter_api::types::{GenerationId, ModelId};

fn main() {
    let model_id = ModelId::new("same-id");
    let gen_id = GenerationId::new("same-id");

    // This should fail to compile: cannot compare ModelId with GenerationId directly
    if model_id == gen_id {
        println!("IDs match");
    }

    // Expected error:
    //   can't compare `ModelId` with `GenerationId`
    //   note: an implementation of `PartialEq<GenerationId>` might be missing for `ModelId`
    //
    // Correct approach: Compare string representations explicitly
    // if model_id.as_str() == gen_id.as_str() { ... }
}
