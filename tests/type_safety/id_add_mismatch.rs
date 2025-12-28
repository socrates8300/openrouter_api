//! Compile-Fail Test: ID Add Operation Mismatch
//!
//! This test verifies that different ID types cannot be added or combined
//! together in arithmetic-like operations without explicit conversion.

use openrouter_api::types::{GenerationId, ModelId};

fn combine_ids<T>(a: T, b: T) -> T {
    a
}

fn main() {
    let model_id = ModelId::new("model-1");
    let gen_id = GenerationId::new("gen-1");

    // This should fail to compile: cannot combine different ID types
    let _ = combine_ids(model_id, gen_id);

    // Expected error:
    //   mismatched types: expected `ModelId`, found `GenerationId`
    //   note: expected type parameter `T`
    //        found type parameter `T`
    //
    // Or similar error indicating type mismatch between the two IDs
}
