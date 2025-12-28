//! Compile-Fail Test: Status Invalid Variant
//!
//! This test verifies that status enums like StreamingStatus and CancellationStatus
//! cannot be constructed with invalid or undefined variants, ensuring that only
//! valid states are representable at compile time.

use openrouter_api::types::status::{CancellationStatus, StreamingStatus};

fn main() {
    // This should fail to compile: StreamingStatus does not have an "Undefined" variant
    let invalid_status = StreamingStatus::Undefined;

    // Expected error:
    //   no variant named `Undefined` found for enum `StreamingStatus`
    //    --> src/types/status/mod.rs:12:5
    //     |
    //   12 | pub enum StreamingStatus {
    //      | --------------- variant `Undefined` not found here
    //
    // Only these valid variants exist:
    //   - StreamingStatus::NotStarted
    //   - StreamingStatus::InProgress
    //   - StreamingStatus::Complete
    //
    // This prevents undefined or ambiguous states from being represented.
    // With Option<bool>, we had: None, Some(true), Some(false)
    // What did None mean? Not started? Unknown? Undefined?
    // Now we have explicit, meaningful states only.

    let _ = invalid_status;

    // Similarly, CancellationStatus also prevents invalid variants:
    // let invalid_cancel = CancellationStatus::Maybe; // ERROR: no such variant
    //
    // Valid CancellationStatus variants:
    //   - CancellationStatus::NotCancelled
    //   - CancellationStatus::Requested
    //   - CancellationStatus::Completed
    //   - CancellationStatus::Failed
    //
    // No ambiguous states like "Maybe" or "Pending" that could cause confusion.
}
