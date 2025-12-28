//! Tests for ID newtypes (ModelId, GenerationId, ActivityId, ToolCallId)
//!
//! This test suite ensures:
//! - ID types are strongly typed and cannot be mixed up
//! - All ID types have proper traits (Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)
//! - IDs can be created from strings
//! - IDs implement Display for logging
//! - IDs can convert back to strings when needed
//! - IDs serialize/deserialize correctly as strings (transparent)

use openrouter_api::types::ids::{ActivityId, GenerationId, ModelId, Price, ToolCallId};
use serde_json::{from_str, from_value, to_value};
use std::collections::HashSet;

/// Test that ModelId can be created
#[test]
fn test_model_id_creation() {
    let id = ModelId::new("openai/gpt-4o");
    assert_eq!(id.as_str(), "openai/gpt-4o");
}

/// Test that ModelId can be created from &str
#[test]
fn test_model_id_from_str() {
    let id = ModelId::from("anthropic/claude-3-opus");
    assert_eq!(id.as_str(), "anthropic/claude-3-opus");
}

/// Test that ModelId implements Debug
#[test]
fn test_model_id_debug() {
    let id = ModelId::new("test-model");
    let debug_str = format!("{:?}", id);
    assert!(debug_str.contains("test-model"));
}

/// Test that ModelId implements Clone
#[test]
fn test_model_id_clone() {
    let id1 = ModelId::new("model-1");
    let id2 = id1.clone();
    assert_eq!(id1, id2);
}

/// Test that ModelId implements PartialEq and Eq
#[test]
fn test_model_id_equality() {
    let id1 = ModelId::new("same-model");
    let id2 = ModelId::new("same-model");
    let id3 = ModelId::new("different-model");

    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

/// Test that ModelId implements Hash
#[test]
fn test_model_id_hash() {
    let id1 = ModelId::new("model-a");
    let id2 = ModelId::new("model-b");

    let mut set = HashSet::new();
    set.insert(id1);
    set.insert(id2);

    assert_eq!(set.len(), 2);
    set.insert(ModelId::new("model-a"));
    assert_eq!(set.len(), 2); // No duplicate
}

/// Test that ModelId implements Display
#[test]
fn test_model_id_display() {
    let id = ModelId::new("my-model");
    let display_str = format!("{}", id);
    assert_eq!(display_str, "my-model");
}

/// Test that ModelId serializes as string
#[test]
fn test_model_id_serialization() {
    let id = ModelId::new("openai/gpt-4o");
    let json = to_value(&id).unwrap();

    assert_eq!(json, "openai/gpt-4o");
    assert!(json.is_string());
}

/// Test that ModelId deserializes from string
#[test]
fn test_model_id_deserialization() {
    let json = r#""openai/gpt-4o""#;
    let id: ModelId = from_str(json).unwrap();

    assert_eq!(id.as_str(), "openai/gpt-4o");
}

/// Test that ModelId can convert to String
#[test]
fn test_model_id_into_string() {
    let id = ModelId::new("test-model");
    let string: String = id.into();
    assert_eq!(string, "test-model");
}

/// Test that different ID types cannot be mixed (compile-time safety)
#[test]
fn test_id_types_not_interchangeable() {
    // This should compile - they're different types
    let model_id: ModelId = ModelId::new("model");
    let gen_id: GenerationId = GenerationId::new("gen-123");

    // This should not compile if we accidentally swap them
    // But we can test that they're not equal
    assert_ne!(format!("{:?}", model_id), format!("{:?}", gen_id));
}

/// Test that GenerationId works correctly
#[test]
fn test_generation_id_creation() {
    let id = GenerationId::new("gen-abc123");
    assert_eq!(id.as_str(), "gen-abc123");
}

/// Test that GenerationId implements all traits
#[test]
fn test_generation_id_traits() {
    let id = GenerationId::new("gen-test");

    // Debug
    let _ = format!("{:?}", id);

    // Clone
    let _cloned = id.clone();

    // PartialEq
    assert_eq!(id, GenerationId::new("gen-test"));
    assert_ne!(id, GenerationId::new("gen-other"));

    // Hash
    let mut set = HashSet::new();
    set.insert(id.clone());

    // Display
    let _ = format!("{}", id);
}

/// Test that GenerationId serializes/deserializes
#[test]
fn test_generation_id_roundtrip() {
    let original = GenerationId::new("gen-xyz789");
    let json = to_value(&original).unwrap();
    let deserialized: GenerationId = from_value(json).unwrap();

    assert_eq!(original, deserialized);
}

/// Test that ActivityId works correctly
#[test]
fn test_activity_id_creation() {
    let id = ActivityId::new("activity-123");
    assert_eq!(id.as_str(), "activity-123");
}

/// Test that ActivityId implements all traits
#[test]
fn test_activity_id_traits() {
    let id = ActivityId::new("act-456");

    // Clone
    let _cloned = id.clone();

    // PartialEq
    assert_eq!(id, ActivityId::new("act-456"));

    // Hash
    let mut set = HashSet::new();
    set.insert(id.clone());

    // Display
    let _ = format!("{}", id);
}

/// Test that ActivityId serializes/deserializes
#[test]
fn test_activity_id_roundtrip() {
    let original = ActivityId::new("act-abc");
    let json = to_value(&original).unwrap();
    let deserialized: ActivityId = from_value(json).unwrap();

    assert_eq!(original, deserialized);
}

/// Test that ToolCallId works correctly
#[test]
fn test_tool_call_id_creation() {
    let id = ToolCallId::new("call_12345");
    assert_eq!(id.as_str(), "call_12345");
}

/// Test that ToolCallId implements all traits
#[test]
fn test_tool_call_id_traits() {
    let id = ToolCallId::new("call_xyz");

    // Clone
    let _cloned = id.clone();

    // PartialEq
    assert_eq!(id, ToolCallId::new("call_xyz"));
    assert_ne!(id, ToolCallId::new("call_abc"));

    // Hash
    let mut set = HashSet::new();
    set.insert(id.clone());

    // Display
    let _ = format!("{}", id);
}

/// Test that ToolCallId serializes/deserializes
#[test]
fn test_tool_call_id_roundtrip() {
    let original = ToolCallId::new("call_test");
    let json = to_value(&original).unwrap();
    let deserialized: ToolCallId = from_value(json).unwrap();

    assert_eq!(original, deserialized);
}

/// Test that IDs are transparent in serialization (no wrapping)
#[test]
fn test_id_transparent_serialization() {
    let model_id = ModelId::new("model");
    let gen_id = GenerationId::new("gen");

    let json = to_value(&(model_id, gen_id)).unwrap();

    // Should serialize as array of strings, not objects
    assert!(json.is_array());
    assert_eq!(json[0], "model");
    assert_eq!(json[1], "gen");
}

/// Test that IDs can be used as HashMap keys
#[test]
fn test_id_as_hashmap_key() {
    use std::collections::HashMap;

    let mut map: HashMap<ModelId, String> = HashMap::new();
    map.insert(ModelId::new("model-1"), "Description 1".to_string());
    map.insert(ModelId::new("model-2"), "Description 2".to_string());

    assert_eq!(
        map.get(&ModelId::new("model-1")),
        Some(&"Description 1".to_string())
    );
    assert_eq!(
        map.get(&ModelId::new("model-2")),
        Some(&"Description 2".to_string())
    );
    assert_eq!(map.get(&ModelId::new("model-3")), None);
}

/// Test that IDs can be used in Vec and Set operations
#[test]
fn test_id_collections() {
    let ids: Vec<ModelId> = vec![
        ModelId::new("model-1"),
        ModelId::new("model-2"),
        ModelId::new("model-1"), // Duplicate
    ];

    // Vec allows duplicates
    assert_eq!(ids.len(), 3);

    // Set eliminates duplicates
    let set: HashSet<ModelId> = ids.into_iter().collect();
    assert_eq!(set.len(), 2);
}

/// Test that IDs work with pattern matching
#[test]
fn test_id_pattern_matching() {
    fn describe_id(id: &ModelId) -> &'static str {
        match id.as_str() {
            s if s.starts_with("openai/") => "OpenAI model",
            s if s.starts_with("anthropic/") => "Anthropic model",
            _ => "Other model",
        }
    }

    assert_eq!(describe_id(&ModelId::new("openai/gpt-4")), "OpenAI model");
    assert_eq!(
        describe_id(&ModelId::new("anthropic/claude-3")),
        "Anthropic model"
    );
    assert_eq!(describe_id(&ModelId::new("custom/model")), "Other model");
}

/// Test that IDs implement AsRef<str>
#[test]
fn test_id_as_ref() {
    let id = ModelId::new("test");
    let s: &str = id.as_ref();
    assert_eq!(s, "test");
}

/// Test that Price can be created from valid numeric values
#[test]
fn test_price_creation() {
    let price = Price::new(0.001);
    assert!(price.is_some());
    assert_eq!(price.unwrap().as_f64(), 0.001);

    let zero_price = Price::new(0.0);
    assert!(zero_price.is_some());
    assert!(zero_price.unwrap().is_zero());
}

/// Test that Price::new accepts negative values (API compatibility)
/// But is_valid_business_logic correctly identifies them as invalid
#[test]
fn test_price_new_accepts_negative() {
    let price = Price::new(-0.001);
    assert!(
        price.is_some(),
        "Price::new should accept negative values for API compatibility"
    );
    assert_eq!(price.clone().unwrap().as_f64(), -0.001);
    assert!(price.clone().unwrap().is_negative());
    assert!(
        !price.unwrap().is_valid_business_logic(),
        "Negative prices should be invalid business logic"
    );
}

/// Test that Price::new_unchecked accepts negative values (changed for API compatibility)
#[test]
fn test_price_new_unchecked_accepts_negative() {
    let price = Price::new_unchecked(-0.001);
    assert_eq!(price.as_f64(), -0.001);
}

/// Test that Price rejects NaN and infinite values
#[test]
fn test_price_rejects_nan_infinite() {
    let nan_price = Price::new(f64::NAN);
    assert!(nan_price.is_none());

    let inf_price = Price::new(f64::INFINITY);
    assert!(inf_price.is_none());

    let neg_inf_price = Price::new(f64::NEG_INFINITY);
    assert!(neg_inf_price.is_none());
}

/// Test Price::new_unchecked still panics on NaN
#[test]
#[should_panic]
fn test_price_new_unchecked_panics_on_nan() {
    Price::new_unchecked(f64::NAN);
}

/// Test that Price serializes as transparent number
#[test]
fn test_price_serialization() {
    let price = Price::new(0.001).unwrap();
    let json = to_value(&price).unwrap();

    assert_eq!(json, 0.001);
    assert!(json.is_number());
}

/// Test that Price deserializes from number
#[test]
fn test_price_deserialization() {
    let json = r#"0.001"#;
    let price: Price = from_str(json).unwrap();

    assert_eq!(price.as_f64(), 0.001);
}

/// Test that Price deserializes from string representation (for API compatibility)
#[test]
fn test_price_deserialization_from_string() {
    let json = r#""0.001""#;
    let price: Price = from_str(json).unwrap();

    assert_eq!(price.as_f64(), 0.001);
}

/// Test that Price deserializes negative strings (API compatibility)
#[test]
fn test_price_deserialization_negative_string() {
    let json = r#""-1""#;
    let price: Price = from_str(json).unwrap();

    assert_eq!(price.as_f64(), -1.0);
    assert!(price.is_negative());
}

/// Test Price::as_usd formatting
#[test]
fn test_price_as_usd() {
    let price = Price::new(0.0012345).unwrap();
    assert_eq!(price.as_usd(), "0.001234");

    let zero_price = Price::new(0.0).unwrap();
    assert_eq!(zero_price.as_usd(), "0.000000");
}

/// Test Price::is_zero and ::is_positive
#[test]
fn test_price_zero_and_positive() {
    let zero_price = Price::new(0.0).unwrap();
    assert!(zero_price.is_zero());
    assert!(!zero_price.is_positive());

    let positive_price = Price::new(0.001).unwrap();
    assert!(!positive_price.is_zero());
    assert!(positive_price.is_positive());
}

/// Test Price Display implementation
#[test]
fn test_price_display() {
    let price = Price::new(0.001).unwrap();
    let display_str = format!("{}", price);
    assert_eq!(display_str, "0.001000");
}

/// Test Price From<f64> implementation
#[test]
fn test_price_from_f64() {
    let price: Price = 0.001.into();
    assert_eq!(price.as_f64(), 0.001);
}

/// Test Price From<f32> implementation
#[test]
fn test_price_from_f32() {
    let price: Price = 0.001_f32.into();
    // f32 to f64 conversion has precision loss, use approximate comparison
    assert!((price.as_f64() - 0.001).abs() < 0.0001);
}

/// Test Price AsRef<f64> implementation
#[test]
fn test_price_as_ref() {
    let price = Price::new(0.001).unwrap();
    let f64_ref: &f64 = price.as_ref();
    assert_eq!(*f64_ref, 0.001);
}

/// Test Price Into<f64> implementation
#[test]
fn test_price_into_f64() {
    let price = Price::new(0.001).unwrap();
    let f64_val: f64 = price.into();
    assert_eq!(f64_val, 0.001);
}

/// Test Price with very large values
#[test]
fn test_price_large_values() {
    let price = Price::new(9999.99).unwrap();
    assert_eq!(price.as_f64(), 9999.99);
    assert!(price.is_positive());
}

/// Test Price with very small values
#[test]
fn test_price_small_values() {
    let price = Price::new(0.000001).unwrap();
    assert_eq!(price.as_f64(), 0.000001);
    assert!(price.is_positive());
}

/// Test Price::abs method
#[test]
fn test_price_abs() {
    let positive = Price::new(0.001).unwrap();
    let negative = Price::new_allow_negative(-0.001).unwrap();

    assert_eq!(positive.abs().as_f64(), 0.001);
    assert_eq!(negative.abs().as_f64(), 0.001);
}

/// Test Price::is_negative method
#[test]
fn test_price_is_negative() {
    let positive = Price::new(0.001).unwrap();
    let zero = Price::new(0.0).unwrap();
    let negative = Price::new_allow_negative(-0.001).unwrap();

    assert!(!positive.is_negative());
    assert!(!zero.is_negative());
    assert!(negative.is_negative());
}

/// Test Price::is_valid_business_logic method
#[test]
fn test_price_is_valid_business_logic() {
    let positive = Price::new(0.001).unwrap();
    let zero = Price::new(0.0).unwrap();
    let negative = Price::new_allow_negative(-0.001).unwrap();

    assert!(positive.is_valid_business_logic());
    assert!(zero.is_valid_business_logic());
    assert!(!negative.is_valid_business_logic());
}

/// Test Price PartialEq
#[test]
fn test_price_equality() {
    let price1 = Price::new(0.001).unwrap();
    let price2 = Price::new(0.001).unwrap();
    let price3 = Price::new(0.002).unwrap();

    assert_eq!(price1, price2);
    assert_ne!(price1, price3);
}

/// Test Price Clone
#[test]
fn test_price_clone() {
    let price1 = Price::new(0.001).unwrap();
    let price2 = price1.clone();

    assert_eq!(price1, price2);
}

/// Test Price Debug
#[test]
fn test_price_debug() {
    let price = Price::new(0.001).unwrap();
    let debug_str = format!("{:?}", price);
    assert!(debug_str.contains("0.001"));
}

/// Test Price round-trip serialization
#[test]
fn test_price_roundtrip() {
    let original = Price::new(0.025).unwrap();
    let json = to_value(&original).unwrap();
    let deserialized: Price = from_value(json).unwrap();

    assert_eq!(original, deserialized);
}
