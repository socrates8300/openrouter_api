//! Compile-Fail Test: Status Integer Without Conversion
//!
//! This test verifies that status enums like StreamingStatus and CancellationStatus
//! cannot be constructed directly from integer values without using the explicit
//! conversion methods provided by the API, ensuring that all status values are
//! intentionally created and validated.

use openrouter_api::types::status::{CancellationStatus, StreamingStatus};

fn main() {
    // This should fail to compile: cannot construct StreamingStatus from integer directly
    let status: StreamingStatus = 1;

    // Expected error:
    //   expected enum `StreamingStatus`, found integer
    //   note: an implementation of `From<i32>` might be missing for `StreamingStatus`
    //
    // This prevents accidental integer-to-status coercion.
    // While the deserializer can handle integer inputs from JSON,
    // direct construction in code requires explicit conversion methods.
    //
    // Correct approaches:
    // 1. Use explicit deserialization from JSON:
    //    let status: StreamingStatus = serde_json::from_str("1").unwrap();
    //    // Maps to StreamingStatus::InProgress
    //
    // 2. Use from_bool() method for legacy compatibility:
    //    let status = StreamingStatus::from_bool(true);
    //    // Maps to StreamingStatus::Complete
    //
    // 3. Construct enum directly (intentional, explicit):
    //    let status = StreamingStatus::InProgress;

    let _ = status;

    // Similarly for CancellationStatus:
    let cancel_status: CancellationStatus = 2;

    // Expected error:
    //   expected enum `CancellationStatus`, found integer
    //
    // Correct approach:
    // let cancel_status = CancellationStatus::Completed; // Explicit and clear
    // Or via deserialization:
    // let cancel_status: CancellationStatus = serde_json::from_str("2").unwrap();

    let _ = cancel_status;
}
