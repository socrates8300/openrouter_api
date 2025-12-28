//! Compile-Fail Test: ID Collection Type Mismatch
//!
//! This test verifies that different ID types cannot be mixed in homogeneous
//! collections like Vec<T>, preventing accidental storage of wrong ID types.

use openrouter_api::types::{ActivityId, GenerationId, ModelId, ToolCallId};

fn main() {
    // This should fail to compile: Vec<T> requires all elements to be the same type
    let ids = vec![
        ModelId::new("model-1"),
        GenerationId::new("gen-1"), // ERROR: type mismatch with previous element
    ];

    // Expected error:
    //   expected struct `ModelId`, found struct `GenerationId`
    //   note: expected due to this
    //
    // Similar error would occur with HashSet or HashMap keys:
    // let mut set = HashSet::new();
    // set.insert(ModelId::new("model-1"));
    // set.insert(GenerationId::new("gen-1")); // ERROR: type mismatch
    //
    // Correct approaches:
    // 1. Use enum to wrap different ID types: enum Id { Model(ModelId), Generation(GenerationId) }
    // 2. Use trait object: Vec<Box<dyn AsId>>
    // 3. Use separate collections for each ID type

    let _ = ids;
}
