# Compile-Fail Test Suite

This directory contains compile-fail tests that verify the type safety guarantees provided by the OpenRouter API library. These tests ensure that code patterns violating type safety fail to compile, preventing regressions in our type safety enforcement.

## Overview

Unlike traditional unit tests that verify code *does* work correctly, compile-fail tests verify that code *doesn't* compile when it shouldn't. This is a powerful technique for ensuring that:

- Different ID types cannot be mixed without explicit conversion
- Enum types only accept valid variants
- Status enums prevent invalid states that `Option<bool>` would allow
- Price types validate numeric inputs and reject NaN/Infinity
- Type conversions require explicit intent

## Architecture

The test suite uses [`trybuild`](https://github.com/dtolnay/trybuild), a compile-time testing framework for Rust. Here's how it works:

1. **Test Files**: Each file in this directory (e.g., `id_type_mismatch.rs`) contains code that should fail to compile
2. **Stderr Files**: Corresponding `.stderr` files contain the expected compiler error messages
3. **Validation**: When you run `cargo test --test compile_fail`, trybuild compiles each test file and compares the actual errors against the expected `.stderr` files
4. **Pass/Fail**: A test "passes" if the code fails to compile with the expected errors

## Running the Tests

Run the entire compile-fail test suite:

```bash
cargo test --test compile_fail
```

Run a specific compile-fail test:

```bash
# Run only the ID type mismatch test
cargo test --test compile_fail -- id_type_mismatch
```

## Test Categories

### 1. ID Newtype Type Safety

Tests that verify different ID types cannot be mixed:

- `id_type_mismatch.rs` - Cannot assign `ModelId` to `GenerationId` variable
- `id_function_arg_mismatch.rs` - Cannot pass wrong ID type to function
- `id_add_mismatch.rs` - Cannot add different ID types
- `id_comparison_mismatch.rs` - Cannot compare different ID types directly
- `id_collection_mismatch.rs` - Cannot mix ID types in homogeneous collection

### 2. Enum Type Safety

Tests that verify enum types prevent invalid values:

- `enum_invalid_construction.rs` - Cannot construct `ChatRole` from arbitrary string
- `enum_non_exhaustive_match.rs` - Cannot match non-exhaustively on enum
- `enum_integer_construction.rs` - Cannot create enum variant from integer

### 3. Status Enum Type Safety

Tests that verify status enums replace `Option<bool>` confusion:

- `status_invalid_variant.rs` - Cannot construct undefined `StreamingStatus`
- `status_integer_without_conversion.rs` - Cannot use integer as status without conversion
- `cancellation_invalid_variant.rs` - Cannot construct undefined `CancellationStatus`

### 4. Price Type Safety

Tests that verify `Price` type validates inputs:

- `price_direct_construction.rs` - Cannot construct `Price` directly (private field)

### 5. Generic Type Safety

Tests that verify generics preserve type information:

- `generic_heterogeneous_vec.rs` - Cannot mix types in homogeneous `Vec<T>`

### 6. Clone/Copy Safety

Tests that verify ownership semantics:

- `consumed_after_into.rs` - Cannot use moved value after `Into` conversion

### 7. Conversions

Tests that verify type conversions require explicit intent:

- `conversion_into_required.rs` - Cannot pass `ModelId` where `String` expected without `Into`

### 8. Pattern Matching

Tests that verify exhaustive matching:

- `pattern_non_exhaustive.rs` - Cannot have non-exhaustive match on enum

## Adding New Tests

To add a new compile-fail test:

### Step 1: Create the Test File

Create a new `.rs` file in this directory with code that should fail to compile:

```rust
//! Compile-Fail Test: Your Test Description
//!
//! Brief explanation of what this test verifies and why it's important.

use openrouter_api::types::SomeType;

fn main() {
    // Write code that should fail to compile
    let invalid = SomeType::InvalidVariant;
}
```

### Step 2: Generate Expected Stderr

Run the test suite once to generate the `.stderr` file:

```bash
cargo test --test compile_fail
```

This will create a file in the `wip/` directory with the actual compiler output.

### Step 3: Move and Review the Stderr File

Move the generated stderr file to this directory:

```bash
mv wip/your_test.stderr tests/type_safety/your_test.stderr
```

Review the `.stderr` file to ensure it contains the expected error messages and documentation.

### Step 4: Register the Test

Add your test to `../compile_fail.rs`:

```rust
#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();
    
    // Your new test
    t.compile_fail("tests/type_safety/your_test.rs");
    
    // ... existing tests
}
```

### Step 5: Verify the Test

Run the test suite to verify everything works:

```bash
cargo test --test compile_fail
```

## Test File Template

```rust
//! Compile-Fail Test: [Brief Title]
//!
//! [Detailed description of what this test verifies]
//!
//! [Explanation of why this pattern should fail to compile]
//!
//! [Discussion of the type safety principle being enforced]
//!
//! Expected error:
//!   [Copy the actual error from the stderr file]
//!
//! [Optional: Correct approaches and examples]

use openrouter_api::types::YourType;

fn main() {
    // Code that should fail to compile
    let invalid = YourType::some_invalid_construction();
    
    let _ = invalid;
}
```

## Troubleshooting

### Test Fails Because Code Compiles

If a compile-fail test starts passing (i.e., the code successfully compiles), you have a type safety regression. Common causes:

1. **Enum variant added**: A new variant was added without updating all non-exhaustive matches
2. **Implicit conversion added**: A `From` or `Into` implementation was added that should not exist
3. **Field made public**: A private field was made public, bypassing validation

**Action**: Review the change that caused the regression and revert or fix it.

### Test Fails Because Stderr Mismatched

If the compiler error output changes (e.g., due to Rust version update), you may need to update the `.stderr` file:

1. Run `cargo test --test compile_fail` to regenerate in `wip/`
2. Review the new errors to ensure they still reflect the same underlying issue
3. Replace the old `.stderr` with the new one

### Test File Has Unclosed Delimiter

Ensure all test files have complete `fn main()` functions and all braces are closed:

```rust
fn main() {
    // Your test code
    let _ = result;
}  // Don't forget this!
```

## Best Practices

### Documentation

Each test file should include:

- **Clear title**: What pattern is being tested
- **Purpose**: Why this pattern should fail to compile
- **Error message**: The expected compiler error
- **Correct approaches**: How to write code that does compile

### Minimal Reproductions

Keep test code minimal and focused on the specific pattern being tested. Avoid:

- Unnecessary imports
- Unused variables (use `let _ = variable` if needed)
- Complex control flow
- External dependencies

### Expected Error Format

Include the exact compiler error in comments at the end of the test file:

```rust
// Expected error:
//   error[E0308]: mismatched types
//    --> tests/type_safety/id_type_mismatch.rs:12:39
//     |
//   12 |     let generation_id: GenerationId = model_id;
//      |                        ------------   ^^^^^^^^
```

This serves as documentation and helps verify the test is checking the right thing.

## Type Safety Principles

This test suite enforces the following type safety principles:

### 1. Strong Typing Prevents Bugs

Different types cannot be mixed without explicit conversion. This prevents:
- Passing wrong ID types to functions
- Comparing unrelated entities
- Storing different types in homogeneous collections

### 2. Enums Make Invalid States Unrepresentable

Enums replace primitive types with explicit variants, eliminating:
- Ambiguous `Option<bool>` states (is `None` "not started" or "unknown"?)
- Invalid string values (typos like `"admnin"` instead of `"admin"`)
- Magic numbers and constants

### 3. Validation at Construction

Types with validation (like `Price`) cannot be constructed directly, ensuring:
- All values go through validation logic
- Invalid inputs (NaN, Infinity) are rejected
- The API is explicit about accepting special cases (e.g., negative prices)

### 4. Explicit Conversions

Type conversions require explicit intent:
- `Into<T>` consumes the value, making the cost visible
- `as_ref()` provides temporary read-only access
- No implicit conversions that hide type changes

### 5. Exhaustive Pattern Matching

All enum variants must be handled in match statements, preventing:
- Runtime panics from unhandled cases
- Bugs when new variants are added
- Silent failures from ignored states

## Related Documentation

- [Type Design Guidelines](../../CONTRIBUTING.md#type-design) - Guidelines for creating type-safe APIs
- [OpenRouter API Library](../../README.md) - Main library documentation
- [trybuild Documentation](https://docs.rs/trybuild) - Official trybuild documentation

## Contributing

When adding new types or modifying existing ones, ensure that:

1. **Compile-fail tests exist** for all type safety guarantees
2. **Existing tests still pass** after your changes
3. **New tests are added** for any new type safety features
4. **Documentation is updated** to explain the new patterns

Remember: The goal is to make invalid states unrepresentable at compile time. Every compile-fail test represents a bug that our type system prevents!