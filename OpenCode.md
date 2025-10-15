# OpenCode Configuration

## Build/Test/Lint Commands
- **Build:** `cargo check --all-targets --all-features`
- **Test all:** `cargo test --all-features`
- **Test single:** `cargo test test_name --all-features`
- **Lint:** `cargo clippy --all-targets --all-features -- -D warnings`
- **Format:** `cargo fmt`
- **Format check:** `cargo fmt --check`
- **Quality gate:** `./scripts/pre_quality.sh`
- **Docs:** `cargo doc --no-deps --all-features`
- **Security audit:** `cargo audit` (requires `cargo install cargo-audit`)

## Code Style Guidelines
- **Imports:** Group std, external crates, then local modules with blank lines between
- **Error handling:** Use `thiserror::Error` for custom errors, `Result<T>` type alias
- **Async:** All API calls are async, use `tokio::test` for async tests
- **Serialization:** Use `serde` with `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- **Security:** Use `zeroize` for sensitive data like API keys, implement `ZeroizeOnDrop`
- **Naming:** snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Documentation:** Use `///` for public APIs, include examples in doc comments
- **Types:** Prefer explicit types over `impl Trait` in public APIs
- **Validation:** Validate inputs early, use descriptive error messages
- **Builder pattern:** Use type-state pattern for compile-time validation (see client.rs)