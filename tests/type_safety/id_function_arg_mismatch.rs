//! Compile-Fail Test: ID Function Argument Mismatch
//!
//! This test verifies that passing the wrong ID type to a function
//! that expects a specific ID type fails to compile.

use openrouter_api::types::{GenerationId, ModelId};

fn process_model(model_id: ModelId) -> String {
    model_id.to_string()
}

fn main() {
    let generation_id = GenerationId::new("gen-123");

    // This should fail to compile: cannot pass GenerationId to function expecting ModelId
    let result = process_model(generation_id);

    // Expected error:
    //   expected struct `ModelId`, found struct `GenerationId`
    //   help: consider converting to `ModelId`
}
