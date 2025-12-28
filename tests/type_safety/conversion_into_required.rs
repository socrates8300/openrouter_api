//! Compile-Fail Test: Conversion Into Required
//!
//! This test verifies that type conversions from ModelId (and similar wrapper types)
//! to String require explicit conversion using `.into()`. This prevents accidental
//! type changes and makes conversions intentional and visible in the code.

use openrouter_api::types::ModelId;

fn takes_string(s: String) -> String {
    format!("Received: {}", s)
}

fn main() {
    let id = ModelId::new("gpt-4");

    // This should fail to compile: cannot pass ModelId directly where String is expected
    let result = takes_string(id);

    // Expected error:
    //   expected struct `String`, found struct `ModelId`
    //   note: expected struct `String`
    //          found struct `ModelId`

    let _ = result;
}
