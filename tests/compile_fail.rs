//! Compile-Fail Tests using trybuild
//!
//! This test suite validates that code patterns violating type safety
//! guarantees fail to compile. If any of these tests start passing,
//! it indicates a regression in our type safety enforcement.

#[test]
fn compile_fail_tests() {
    let t = trybuild::TestCases::new();

    // ============================================================================
    // SECTION 1: ID Newtype Type Safety
    // ============================================================================

    // Cannot assign ModelId to GenerationId variable
    t.compile_fail("tests/type_safety/id_type_mismatch.rs");

    // Cannot pass wrong ID type to function
    t.compile_fail("tests/type_safety/id_function_arg_mismatch.rs");

    // Cannot add different ID types
    t.compile_fail("tests/type_safety/id_add_mismatch.rs");

    // Cannot compare different ID types directly
    t.compile_fail("tests/type_safety/id_comparison_mismatch.rs");

    // Cannot mix ID types in homogeneous collection
    t.compile_fail("tests/type_safety/id_collection_mismatch.rs");

    // ============================================================================
    // SECTION 2: Enum Type Safety
    // ============================================================================

    // Cannot construct ChatRole from arbitrary string
    t.compile_fail("tests/type_safety/enum_invalid_construction.rs");

    // Cannot match non-exhaustively on ChatRole
    t.compile_fail("tests/type_safety/enum_non_exhaustive_match.rs");

    // Cannot create enum variant from integer
    t.compile_fail("tests/type_safety/enum_integer_construction.rs");

    // ============================================================================
    // SECTION 3: Status Enum Type Safety
    // ============================================================================

    // Cannot construct undefined StreamingStatus
    t.compile_fail("tests/type_safety/status_invalid_variant.rs");

    // Cannot use integer as status without conversion
    t.compile_fail("tests/type_safety/status_integer_without_conversion.rs");

    // Cannot construct undefined CancellationStatus
    t.compile_fail("tests/type_safety/cancellation_invalid_variant.rs");

    // ============================================================================
    // SECTION 4: Price Type Safety
    // ============================================================================

    // Cannot construct Price directly (private field)
    t.compile_fail("tests/type_safety/price_direct_construction.rs");

    // Cannot unwrap None from Price::new(NAN)
    // Note: This compiles but would panic - we test runtime behavior separately
    // t.compile_fail("tests/type_safety/price_nan_unwrap.rs");

    // ============================================================================
    // SECTION 5: Serde Type Safety
    // ============================================================================

    // Serde failures are runtime errors, not compile-time
    // These are tested in integration tests

    // ============================================================================
    // SECTION 6: Generic Type Safety
    // ============================================================================

    // Cannot mix types in homogeneous Vec
    t.compile_fail("tests/type_safety/generic_heterogeneous_vec.rs");

    // ============================================================================
    // SECTION 7: Clone/Copy Safety
    // ============================================================================

    // Cannot use moved value after Into conversion
    t.compile_fail("tests/type_safety/consumed_after_into.rs");

    // ============================================================================
    // SECTION 8: Conversions
    // ============================================================================

    // Cannot pass ModelId where String expected without Into
    t.compile_fail("tests/type_safety/conversion_into_required.rs");

    // ============================================================================
    // SECTION 9: Pattern Matching
    // ============================================================================

    // Cannot have non-exhaustive match on enum
    t.compile_fail("tests/type_safety/pattern_non_exhaustive.rs");
}
