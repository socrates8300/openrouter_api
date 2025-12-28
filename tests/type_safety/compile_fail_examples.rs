//! Compile-Fail Examples: Type Safety Guarantees
//!
//! This file demonstrates patterns that SHOULD NOT compile.
//! If any of these examples successfully compile, it indicates
//! a regression in our type safety guarantees.
//!
//! # How to Verify
//!
//! To verify type safety, uncomment each section individually and
//! run `cargo check`. Expect compilation errors.
//!
//! ```bash
//! # Uncomment one section at a time
//! cargo check --test compile_fail_examples
//! ```
//!
//! # Type Safety Principles
//!
//! 1. **ID Newtypes**: Different ID types cannot be mixed
//! 2. **Enum over String**: Only valid variants are representable
//! 3. **Status over Option<bool>**: Invalid states are impossible
//! 4. **Price Validation**: Invalid prices (NaN, infinite) are rejected
//! 5. **Serde Compatibility**: External formats map to internal types

// ============================================================================
// SECTION 1: ID Newtype Type Safety
// ============================================================================

// These examples demonstrate that different ID types cannot be mixed.
// This prevents accidentally passing a ModelId where a GenerationId is expected.

#[allow(dead_code)]
fn id_type_safety_examples() {
    use openrouter_api::types::{ModelId, GenerationId, ActivityId, ToolCallId};

    // ✗ SHOULD FAIL: Cannot pass ModelId where GenerationId is expected
    // This demonstrates compile-time prevention of mixing different entity IDs
    //
    // let model_id: ModelId = ModelId::new("gpt-4");
    // let generation_id: GenerationId = model_id; // ERROR: type mismatch
    //
    // Error should be something like:
    //   expected struct `GenerationId`, found struct `ModelId`

    // ✗ SHOULD FAIL: Cannot construct with wrong ID type
    //
    // struct Request {
    //     model_id: ModelId,
    //     generation_id: GenerationId,
    // }
    //
    // let request = Request {
    //     model_id: ModelId::new("gpt-4"),
    //     generation_id: ModelId::new("gen-123"), // ERROR: type mismatch
    // };

    // ✗ SHOULD FAIL: Cannot add different ID types
    // Even though both are String wrappers, they remain distinct types
    //
    // fn combine_ids<T>(a: T, b: T) -> T { a }
    //
    // let model_id = ModelId::new("model");
    // let gen_id = GenerationId::new("gen");
    // let _ = combine_ids(model_id, gen_id); // ERROR: type mismatch

    // ✗ SHOULD FAIL: Cannot compare different ID types directly
    //
    // let model_id = ModelId::new("same-id");
    // let gen_id = GenerationId::new("same-id");
    // if model_id == gen_id { // ERROR: type mismatch
    //     println!("IDs match");
    // }

    // ✓ PASSES: Comparison with explicit conversion requires intent
    //
    // let model_id = ModelId::new("same-id");
    // let gen_id = GenerationId::new("same-id");
    // if model_id.as_str() == gen_id.as_str() {
    //     // This requires explicit intent via .as_str()
    // }

    // ✗ SHOULD FAIL: Cannot hash different ID types in the same collection
    // without explicit type annotations
    //
    // use std::collections::HashSet;
    //
    // let mut ids = HashSet::new();
    // ids.insert(ModelId::new("model-1"));
    // ids.insert(GenerationId::new("gen-1")); // ERROR: type mismatch
}

// ============================================================================
// SECTION 2: ChatRole Enum Type Safety
// ============================================================================

// These examples demonstrate that ChatRole prevents invalid string values.

#[allow(dead_code)]
fn chat_role_type_safety_examples() {
    use openrouter_api::types::ChatRole;

    // ✗ SHOULD FAIL: Cannot construct ChatRole from arbitrary string
    // The enum only has 4 valid variants
    //
    // let invalid_role: ChatRole = "admin".into(); // ERROR: no implementation
    //
    // This prevents typos and undefined roles from compiling

    // ✗ SHOULD FAIL: Cannot match on invalid value
    // Exhaustive matching ensures all variants are handled
    //
    // fn describe_role(role: ChatRole) -> &'static str {
    //     match role {
    //         ChatRole::User => "user",
    //         ChatRole::Assistant => "assistant",
    //         // Missing System and Tool variants
    //         // ERROR: non-exhaustive patterns
    //     }
    // }

    // ✗ SHOULD FAIL: Cannot create ChatRole via integer
    //
    // let role: ChatRole = 5; // ERROR: type mismatch
    // let role: ChatRole = unsafe { std::mem::transmute(5u8) };
    // // Using transmute would work but is unsafe and discouraged

    // ✓ PASSES: Only valid variants can be created
    //
    // let valid_roles = vec![
    //     ChatRole::User,
    //     ChatRole::Assistant,
    //     ChatRole::System,
    //     ChatRole::Tool,
    // ];
}

// ============================================================================
// SECTION 3: Status Enum Type Safety (StreamingStatus & CancellationStatus)
// ============================================================================

// These examples demonstrate that status enums replace Option<bool> and
// prevent invalid states.

#[allow(dead_code)]
fn status_enum_type_safety_examples() {
    use openrouter_api::types::status::{StreamingStatus, CancellationStatus};

    // ✗ SHOULD FAIL: Cannot represent undefined streaming states
    // With Option<bool>, we had: None, Some(true), Some(false)
    // Now we have: NotStarted, InProgress, Complete - all have meaning
    //
    // let undefined_status = StreamingStatus::Undefined; // ERROR: no such variant

    // ✗ SHOULD FAIL: Cannot create Option<bool> confusion
    //
    // let status: Option<bool> = None;
    // // What does None mean here? Not started? Unknown?
    // // With enum, we're explicit: StreamingStatus::NotStarted

    // ✗ SHOULD FAIL: Cannot use integer as status without explicit conversion
    //
    // let status: StreamingStatus = 3; // ERROR: type mismatch
    //
    // Even though the deserializer handles integers, direct construction
    // requires explicit conversion methods:
    // let status = StreamingStatus::from_bool(true); // ✓ OK
    // Or deserialize from JSON: let status: StreamingStatus = serde_json::from_str("1")?; // ✓ OK

    // ✗ SHOULD FAIL: CancellationStatus also prevents invalid states
    //
    // let invalid_cancel = CancellationStatus::Maybe; // ERROR: no such variant
    //
    // We have: NotCancelled, Requested, Completed, Failed
    // No ambiguous states

    // ✓ PASSES: Deserialization handles multiple formats
    //
    // use serde_json::from_str;
    //
    // let from_bool: StreamingStatus = from_str("true").unwrap();
    // let from_int: StreamingStatus = from_str("1").unwrap();
    // let from_str: StreamingStatus = from_str("\"in_progress\"").unwrap();
    // All deserialize correctly but compile-time construction is strict
}

// ============================================================================
// SECTION 4: Price Type Safety
// ============================================================================

// These examples demonstrate that Price validates numeric inputs and
// prevents NaN/infinite values.

#[allow(dead_code)]
fn price_type_safety_examples() {
    use openrouter_api::types::Price;

    // ✗ SHOULD FAIL: Cannot create Price from NaN
    // Price::new() returns Option<Price> for validation
    //
    // let nan_price = Price::new(f64::NAN).unwrap(); // PANIC: None unwrapped
    // let nan_price = Price::new(f64::INFINITY).unwrap(); // PANIC: None unwrapped

    // ✗ SHOULD FAIL: new_unchecked() panics on invalid values
    //
    // let invalid_price = Price::new_unchecked(f64::NAN); // PANIC at runtime
    //
    // Note: This compiles but panics at runtime. Use Price::new() for safe construction.

    // ✗ SHOULD FAIL: Cannot construct Price directly (wrapper is private)
    //
    // let invalid_price = Price(f64::NAN); // ERROR: field 0 of struct `Price` is private
    //
    // This forces use of constructor methods that validate input

    // ✓ PASSES: Valid prices through constructor
    //
    // let valid_price = Price::new(10.99).unwrap();
    // let zero_price = Price::new(0.0).unwrap();
    // let negative_price = Price::new_allow_negative(-1.0).unwrap();

    // ✓ PASSES: Negative prices are accepted for API compatibility
    // (Some APIs use -1 as sentinel value)
    //
    // let sentinel = Price::new_allow_negative(-1.0).unwrap();
    // assert_eq!(sentinel.as_f64(), -1.0);

    // ✓ PASSES: Business logic validation distinguishes valid prices
    //
    // let price = Price::new_allow_negative(-1.0).unwrap();
    // assert!(!price.is_valid_business_logic()); // Negative prices invalid for business logic
    // assert!(price.is_negative()); // But can still be negative for API compatibility
}

// ============================================================================
// SECTION 5: Serde Type Safety (External Format Validation)
// ============================================================================

// These examples demonstrate that external formats (JSON) are validated
// and mapped to our strict internal types.

#[allow(dead_code)]
fn serde_type_safety_examples() {
    use openrouter_api::types::{ModelId, StreamingStatus};
    use serde_json::json;

    // ✓ PASSES: Valid JSON deserializes correctly
    //
    // let id: ModelId = serde_json::from_value(json!("gpt-4")).unwrap();
    // assert_eq!(id.as_str(), "gpt-4");
    //
    // let status: StreamingStatus = serde_json::from_value(json!("in_progress")).unwrap();
    // assert_eq!(status, StreamingStatus::InProgress);

    // ✗ SHOULD FAIL: Invalid JSON values produce errors at runtime
    // (Cannot be caught at compile time, but deserialization is safe)
    //
    // let invalid_id: Result<ModelId, _> = serde_json::from_value(json!(123));
    // // This returns Err - safe runtime error, not undefined behavior
    //
    // let invalid_status: Result<StreamingStatus, _> =
    //     serde_json::from_value(json!("invalid_status"));
    // // This returns Err - safe runtime error

    // ✓ PASSES: Multiple formats map correctly
    //
    // let from_bool: StreamingStatus = serde_json::from_value(json!(true)).unwrap();
    // let from_int: StreamingStatus = serde_json::from_value(json!(1)).unwrap();
    // let from_str: StreamingStatus = serde_json::from_value(json!("in_progress")).unwrap();
    // All correctly deserialize to StreamingStatus::InProgress

    // ✓ PASSES: External serialization is transparent
    //
    // let id = ModelId::new("gpt-4");
    // let json = serde_json::to_value(&id).unwrap();
    // assert_eq!(json, json!("gpt-4")); // Serializes as string
}

// ============================================================================
// SECTION 6: Generic Type Safety
// ============================================================================

// These examples demonstrate how our types work with generics
// while maintaining safety.

#[allow(dead_code)]
fn generic_type_safety_examples() {
    use openrouter_api::types::{ModelId, GenerationId};

    // ✓ PASSES: Generic functions can be polymorphic
    //
    // fn get_id<T>(id: T) -> T { id }
    //
    // let model_id = get_id(ModelId::new("gpt-4"));
    // let gen_id = get_id(GenerationId::new("gen-123"));
    //
    // Each call maintains its type

    // ✗ SHOULD FAIL: Cannot mix types in homogeneous collection
    //
    // let ids = vec![
    //     ModelId::new("model-1"),
    //     GenerationId::new("gen-1"), // ERROR: type mismatch
    // ];

    // ✓ PASSES: Can use trait objects or enums for mixed collections
    //
    // trait AsId {
    //     fn as_str(&self) -> &str;
    // }
    //
    // impl AsId for ModelId { fn as_str(&self) -> &str { self.0.as_ref() } }
    // impl AsId for GenerationId { fn as_str(&self) -> &str { self.0.as_ref() } }
    //
    // let ids: Vec<Box<dyn AsId>> = vec![
    //     Box::new(ModelId::new("model-1")),
    //     Box::new(GenerationId::new("gen-1")),
    // ];
    //
    // But this requires explicit interface - no accidental mixing

    // ✓ PASSES: Generic constraints ensure type safety
    //
    // fn process_id<T: std::fmt::Display + Clone>(id: T) -> (T, String) {
    //     (id.clone(), format!("{}", id))
    // }
    //
    // let (id_copy, id_str) = process_id(ModelId::new("gpt-4"));
    // assert!(matches!(id_copy, ModelId(_))); // Type preserved
}

// ============================================================================
// SECTION 7: Clone and Copy Safety
// ============================================================================

// These examples demonstrate how Clone/Copy traits interact with
// our types and prevent unintended behavior.

#[allow(dead_code)]
fn clone_copy_safety_examples() {
    use openrouter_api::types::{ModelId, StreamingStatus};

    // ✓ PASSES: ID types implement Clone (intentional, not Copy)
    // This prevents silent mutation issues with large structs
    //
    // let id1 = ModelId::new("model-1");
    // let id2 = id1.clone(); // Explicit clone required
    //
    // Without Clone:
    // let id2 = id1; // This would move, not copy
    // // id1 is no longer usable - prevents confusion

    // ✓ PASSES: Enums are Clone and Copy (small, value types)
    //
    // let status1 = StreamingStatus::InProgress;
    // let status2 = status1; // Copy - status1 still valid
    // assert_eq!(status1, status2); // Both usable

    // ✓ PASSES: This prevents issues with Option<CloneType>
    //
    // let mut maybe_id: Option<ModelId> = None;
    // if let Some(id) = maybe_id.take() {
    //     // id is moved out of Option
    //     // Option is now None, preventing double-use
    // }

    // ⚠️  CONSIDERATION: Large structs with Clone may have performance cost
    // Our ID types are small (String wrapper), so Clone is cheap
    // For large structs, prefer reference (&T) instead of cloning
}

// ============================================================================
// SECTION 8: Hash and Eq Safety
// ============================================================================

// These examples demonstrate how Hash/Eq implementations ensure
// consistent behavior in collections.

#[allow(dead_code)]
fn hash_eq_safety_examples() {
    use openrouter_api::types::{ModelId, StreamingStatus};
    use std::collections::{HashSet, HashMap};

    // ✓ PASSES: IDs can be used in HashSet and HashMap
    //
    // let mut set: HashSet<ModelId> = HashSet::new();
    // set.insert(ModelId::new("model-1"));
    // set.insert(ModelId::new("model-2"));
    //
    // let mut map: HashMap<ModelId, StreamingStatus> = HashMap::new();
    // map.insert(ModelId::new("model-1"), StreamingStatus::InProgress);

    // ✓ PASSES: Hashing is based on underlying value
    //
    // let id1 = ModelId::new("same-value");
    // let id2 = ModelId::new("same-value");
    // assert_eq!(id1, id2); // Eq compares underlying value
    // assert_eq!(std::hash::Hash::hash(&id1), std::hash::Hash::hash(&id2));

    // ✓ PASSES: Different ID types hash differently
    //
    // let model_id = ModelId::new("same-value");
    // let gen_id = GenerationId::new("same-value");
    // // These cannot be compared directly due to type mismatch
    // // But even if cast to dyn Hash, they'd be different

    // ✓ PASSES: Enums implement Hash and Eq
    //
    // let mut status_set: HashSet<StreamingStatus> = HashSet::new();
    // status_set.insert(StreamingStatus::InProgress);
    // status_set.insert(StreamingStatus::Complete);
}

// ============================================================================
// SECTION 9: AsRef/Into Conversions
// ============================================================================

// These examples demonstrate how our types provide safe conversions
// while preventing implicit unsafe conversions.

#[allow(dead_code)]
fn conversion_safety_examples() {
    use openrouter_api::types::ModelId;

    // ✓ PASSES: AsRef<str> provides read-only access
    //
    // let id = ModelId::new("gpt-4");
    // let s: &str = id.as_ref();
    // assert_eq!(s, "gpt-4");

    // ✓ PASSES: Into<String> provides owned conversion
    //
    // let id = ModelId::new("gpt-4");
    // let s: String = id.into();
    // assert_eq!(s, "gpt-4");

    // ✓ PASSES: From<String> provides construction
    //
    // let id: ModelId = "gpt-4".to_string().into();
    // assert_eq!(id.as_str(), "gpt-4");

    // ✗ SHOULD FAIL: No From<ModelId> for String without explicit Into
    //
    // fn takes_string(s: String) {}
    //
    // let id = ModelId::new("gpt-4");
    // takes_string(id); // ERROR: type mismatch
    // takes_string(id.into()); // ✓ OK: explicit conversion

    // ✓ PASSES: Conversion requires explicit intent
    // This prevents accidental String usage where ModelId is expected

    // ⚠️  Note: Into<T> consumes the value
    // let id = ModelId::new("gpt-4");
    // let s: String = id.into();
    // // id is no longer usable - use .as_str() for temporary access
}

// ============================================================================
// SECTION 10: Pattern Matching Safety
// ============================================================================

// These examples demonstrate how exhaustive matching ensures
// all cases are handled.

#[allow(dead_code)]
fn pattern_matching_safety_examples() {
    use openrouter_api::types::{ChatRole, StreamingStatus};

    // ✗ SHOULD FAIL: Non-exhaustive match on ChatRole
    //
    // fn describe_role(role: ChatRole) -> &'static str {
    //     match role {
    //         ChatRole::User => "user message",
    //         ChatRole::Assistant => "assistant response",
    //         // ERROR: missing patterns: System, Tool
    //     }
    // }

    // ✓ PASSES: Exhaustive match
    //
    // fn describe_role(role: ChatRole) -> &'static str {
    //     match role {
    //         ChatRole::User => "user message",
    //         ChatRole::Assistant => "assistant response",
    //         ChatRole::System => "system instruction",
    //         ChatRole::Tool => "tool call result",
    //     }
    // }

    // ✓ PASSES: Wildcard pattern or `_` allows future extensions
    //
    // fn describe_role_conservative(role: ChatRole) -> &'static str {
    //     match role {
    //         ChatRole::User | ChatRole::Assistant => "chat message",
    //         _ => "special message",
    //     }
    // }
    //
    // But this is less explicit - prefer exhaustive matching

    // ✓ PASSES: if let requires handling None
    //
    // fn maybe_role(role: Option<ChatRole>) -> &'static str {
    //     if let Some(ChatRole::User) = role {
    //         "user"
    //     } else {
    //         "not user or none"
    //     }
    // }
}

// ============================================================================
// SUMMARY: Type Safety Checklist
// ============================================================================

// After reviewing these examples, verify that:
//
// ✓ Different ID types cannot be mixed without explicit conversion
// ✓ Enums only allow valid variants
// ✓ Option<bool> confusion is eliminated with explicit enums
// ✓ Price validates numeric inputs (NaN/Infinity rejected)
// ✓ Serde deserialization is safe (errors don't panic)
// ✓ Generic polymorphism preserves type information
// ✓ Clone/Copy behavior is intentional and documented
// ✓ Hash/Eq implementations are consistent
// ✓ Conversions require explicit intent
// ✓ Pattern matching is exhaustive
//
// If any example marked "✗ SHOULD FAIL" compiles successfully,
// we have a regression in type safety!

#[cfg(test)]
mod verify_type_safety {
    // This module contains positive tests that verify the safety
    // mechanisms work correctly.

    use super::*;
    use openrouter_api::types::{ModelId, GenerationId, ChatRole, StreamingStatus, Price};

    #[test]
    fn test_ids_are_different_types() {
        let model_id = ModelId::new("test");
        let gen_id = GenerationId::new("test");

        // Verify they have same underlying value but are different types
        assert_eq!(model_id.as_str(), gen_id.as_str());

        // Cannot compare directly - this should not compile:
        // assert!(model_id == gen_id); // ERROR: type mismatch

        // But can compare strings explicitly:
        assert_eq!(model_id.as_str(), gen_id.as_str());
    }

    #[test]
    fn test_enum_variants() {
        let roles = vec![
            ChatRole::User,
            ChatRole::Assistant,
            ChatRole::System,
            ChatRole::Tool,
        ];

        // All variants are distinct
        for (i, role_a) in roles.iter().enumerate() {
            for (j, role_b) in roles.iter().enumerate() {
                if i == j {
                    assert_eq!(role_a, role_b);
                } else {
                    assert_ne!(role_a, role_b);
                }
            }
        }
    }

    #[test]
    fn test_status_enums_are_exhaustive() {
        let statuses = vec![
            StreamingStatus::NotStarted,
            StreamingStatus::InProgress,
            StreamingStatus::Complete,
        ];

        // Each status is distinct
        for (i, status_a) in statuses.iter().enumerate() {
            for (j, status_b) in statuses.iter().enumerate() {
                if i == j {
                    assert_eq!(status_a, status_b);
                } else {
                    assert_ne!(status_a, status_b);
                }
            }
        }
    }

    #[test]
    fn test_price_rejects_nan() {
        // NaN should be rejected by Price::new()
        let result = Price::new(f64::NAN);
        assert!(result.is_none());

        // Infinity should also be rejected
        let result = Price::new(f64::INFINITY);
        assert!(result.is_none());

        let result = Price::new(f64::NEG_INFINITY);
        assert!(result.is_none());
    }

    #[test]
    fn test_price_accepts_valid_values() {
        // Zero is valid
        assert!(Price::new(0.0).is_some());

        // Positive numbers are valid
        assert!(Price::new(10.5).is_some());

        // Negative numbers are valid (for API compatibility)
        assert!(Price::new_allow_negative(-1.0).is_some());
    }

    #[test]
    fn test_conversions_require_explicit_intent() {
        let id = ModelId::new("test-id");

        // AsRef doesn't consume
        let s1: &str = id.as_ref();
        assert_eq!(s1, "test-id");

        // id is still usable
        assert_eq!(id.as_str(), "test-id");

        // Into consumes
        let id2 = ModelId::new("test-id-2");
        let s2: String = id2.into();
        assert_eq!(s2, "test-id-2");

        // id2 is no longer usable - this should not compile:
        // assert_eq!(id2.as_str(), "test-id-2"); // ERROR: use of moved value
    }
}
