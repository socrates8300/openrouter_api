//! Compile-Fail Test: Price Direct Construction
//!
//! This test verifies that the Price type cannot be constructed directly
//! because its inner field is private. This forces users to use the
//! provided constructor methods that validate input values, preventing
//! creation of invalid prices (NaN, Infinity) from bypassing validation.

use openrouter_api::types::Price;

fn main() {
    // This should fail to compile: cannot construct Price directly
    // The inner field is private to ensure all Price values go through
    // validation via constructor methods.
    let invalid_price = Price(f64::NAN);

    // Expected error:
    //   field `0` of struct `Price` is private
    //    --> src/types/ids.rs
    //     |
    //   pub struct Price(f64);
    //      ------------------ field `0` is private
    //
    // This prevents bypassing validation. For example:
    // - Price(f64::NAN) would create an invalid price
    // - Price(f64::INFINITY) would create an infinite price
    // - Price(f64::NEG_INFINITY) would create a negative infinite price
    //
    // Correct approaches:
    // 1. Use Price::new() - returns Option<Price>, rejects NaN/Infinity:
    //    let price = Price::new(10.99).unwrap();
    //    let invalid = Price::new(f64::NAN); // Returns None
    //
    // 2. Use Price::new_unchecked() - panics on invalid values:
    //    let price = Price::new_unchecked(10.99);
    //    // Price::new_unchecked(f64::NAN); // PANICS at runtime
    //
    // 3. Use Price::new_allow_negative() - accepts negative values for API compatibility:
    //    let price = Price::new_allow_negative(-1.0).unwrap();
    //    // Some APIs use -1 as a sentinel value for "free" or "unknown"

    let _ = invalid_price;

    // Other direct construction attempts should also fail:
    // let zero_price = Price(0.0); // ERROR: field `0` is private
    // let positive_price = Price(10.5); // ERROR: field `0` is private
    // let negative_price = Price(-1.0); // ERROR: field `0` is private

    // This design ensures that:
    // 1. All Price values are validated at construction time
    // 2. Invalid values (NaN, Infinity) cannot be created accidentally
    // 3. The API is explicit about accepting negative values (via new_allow_negative)
    // 4. Users are forced to handle the Option returned by new() for safe construction
}
