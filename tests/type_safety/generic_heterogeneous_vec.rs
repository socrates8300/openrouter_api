//! Compile-Fail Test: Generic Heterogeneous Vector
//!
//! This test verifies that different ID types cannot be mixed in a homogeneous
//! collection like Vec<T> without using an enum wrapper or trait object. This
//! prevents accidental storage of different ID types in the same collection,
//! which could lead to runtime type confusion errors.

use openrouter_api::types::{GenerationId, ModelId};

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
    // The Vec<T> type parameter T is inferred as ModelId from the first element,
    // so subsequent elements must also be ModelId. This is a fundamental guarantee
    // of Rust's type system that prevents type confusion.
    //
    // Similar errors occur with other ID types:
    // let more_ids = vec![
    //     GenerationId::new("gen-1"),
    //     ToolCallId::new("call-1"), // ERROR: expected GenerationId, found ToolCallId
    //     ActivityId::new("act-1"), // ERROR: expected GenerationId, found ActivityId
    // ];
    //
    // Correct approaches for storing different ID types:
    //
    // 1. Use an enum wrapper (explicit, type-safe):
    //    enum AnyId {
    //        Model(ModelId),
    //        Generation(GenerationId),
    //        ToolCall(ToolCallId),
    //        Activity(ActivityId),
    //    }
    //    let ids = vec![
    //        AnyId::Model(ModelId::new("model-1")),
    //        AnyId::Generation(GenerationId::new("gen-1")),
    //    ];
    //
    // 2. Use trait objects (dynamic dispatch, runtime overhead):
    //    trait AsId {
    //        fn as_str(&self) -> &str;
    //    }
    //    impl AsId for ModelId { fn as_str(&self) -> &str { self.0.as_ref() } }
    //    impl AsId for GenerationId { fn as_str(&self) -> &str { self.0.as_ref() } }
    //    // ... implement for other ID types
    //    let ids: Vec<Box<dyn AsId>> = vec![
    //        Box::new(ModelId::new("model-1")),
    //        Box::new(GenerationId::new("gen-1")),
    //    ];
    //
    // 3. Use separate collections for each ID type (zero overhead):
    //    let models: Vec<ModelId> = vec![ModelId::new("model-1")];
    //    let generations: Vec<GenerationId> = vec![GenerationId::new("gen-1")];
    //
    // Each approach has trade-offs, but all require explicit intent - no accidental
    // mixing of different ID types is possible.

    let _ = ids;
}
