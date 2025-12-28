//! Compile-Fail Test: Cancellation Invalid Variant
//!
//! This test verifies that CancellationStatus enum cannot be constructed
//! with invalid or undefined variants, ensuring that only valid cancellation
//! states are representable at compile time.

use openrouter_api::types::status::CancellationStatus;

fn main() {
    // This should fail to compile: CancellationStatus does not have a "Maybe" variant
    let invalid_cancel = CancellationStatus::Maybe;

    // Expected error:
    //   no variant named `Maybe` found for enum `CancellationStatus`
}
