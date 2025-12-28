//! Compile-Fail Test: ID Type Mismatch
//!
//! This test verifies that different ID types cannot be assigned
//! to each other without explicit conversion.

use openrouter_api::types::{GenerationId, ModelId};

fn main() {
    let model_id: ModelId = ModelId::new("gpt-4");

    // This should fail to compile: cannot assign ModelId to GenerationId variable
    let generation_id: GenerationId = model_id;

    // Expected error:
    //   expected struct `GenerationId`, found struct `ModelId`
}
