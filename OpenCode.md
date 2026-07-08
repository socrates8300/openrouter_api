# OpenCode Configuration

This repo's agent-facing guidance lives in `CONTRIBUTING.md` (workflow,
quality gates, code style, type-design conventions). Follow that file.

> **TLS warning (read first):** this crate has two mutually exclusive
> TLS features, `tls-rustls` (default) and `tls-native-tls`. Enabling
> both trips a `compile_error!` in `src/lib.rs`. **Never use
> `--all-features`** — it will fail the build. Use one TLS feature at a
> time, e.g. `cargo test` (default) or
> `cargo test --no-default-features --features tls-native-tls`.
