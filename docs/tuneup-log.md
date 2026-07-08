# Repo Tune-Up Log

Append-only log of repo-tune-up runs. Each entry records what changed,
why, and the policy tier it was executed under (see the tune-up policy:
**Tier A** = agent-facing scaffolding, execute freely; **Tier B** =
structural moves/deletes with mechanical proof; **Tier C** = semantic
changes, propose only).

---

## 2026-07-07 — `chore/repo-tuneup-2026-07`

**Goal:** a cold-start agent with zero prior context can go from fresh
clone to a correct, verified change while spending minimal tokens on
orientation.

### Tier A — agent-facing scaffolding (executed)

- **D1+D2 — `AGENTS.md` rewritten.** Dropped the two broken commands
  (`cargo test --test integration_tests` — target never existed; and
  `cargo check --all-features` — fails by design via `compile_error!`).
  Corrected the test-layout claim (integration binaries are top-level
  `tests/`, not `src/tests/`). Added four load-bearing invariants (TLS
  mutex, type-state builder, centralized error, brand neutrality) and a
  module map. Moved code-style guidelines out to `CONTRIBUTING.md`
  (single authority).
- **D3 — `OpenCode.md` collapsed to a pointer.** All five of its
  build/test commands used `--all-features` (every one fails). Replaced
  with a pointer to `CONTRIBUTING.md` (tracked→tracked pointer graph)
  plus an inline TLS warning, since a fresh clone ships `OpenCode.md`
  but not the gitignored `AGENTS.md`.
- **D5 — `CLAUDE.md` collapsed to a pointer.** Dropped the
  "context database" protocol (`docs/agent_context.db` — a non-standard
  agent-memory convention not used by the library) and the wrong
  test-layout/TLS claims. Now points to `AGENTS.md`
  (gitignored→gitignored).
- **D4 — MSRV corrected from false 1.70 to real 1.85.** The 1.70 claim
  was verified false: the dependency graph (wiremock 0.6 and the
  edition-2024 crate set) requires Cargo ≥ 1.85 to even parse. Enforced
  in four places: `Cargo.toml` `rust-version = "1.85"` (strongest —
  toolchain rejects older compilers), `README.md`, `CONTRIBUTING.md`,
  and the `msrv` CI job (1.70.0 → 1.85.0).
- **D6 — `README.md` retry examples fixed.** Two code blocks had
  builder methods that return `Self` annotated with `?` (which would
  fail to compile). The `with_retry_config` example also used a
  `RetryConfig` struct literal missing two required fields
  (`total_timeout`, `max_retry_interval`) and an import that is not at
  the crate root. Fixed both examples to match the real signatures:
  `with_retry_config(RetryConfig) -> Self`, `with_retries(u32, u64) ->
  Self`, `without_retries() -> Self`.

### Tier B — structural moves/deletes (executed with proof)

- **D8 — `scripts/pre_quality.sh` stale `--exclude mcp`.** Removed
  `--workspace --exclude mcp` from both test invocations. This is a
  single crate, not a workspace, and `mcp` was never a member; the flag
  emitted `warning: excluded package(s) 'mcp' not found in workspace`
  on every run. Verified the warning is gone after the fix.
- **D9 — `lcov.info` untracked.** The 288 KB coverage artifact was
  committed at repo root. `git rm --cached` (local file kept) and added
  `lcov.info` to `.gitignore` under a coverage-artifacts comment.
- **D10 — `.gitignore` deduped.** Removed the duplicate
  `docs/agent_context.db` line (was at lines 36 and 42); added a
  comment for the previously-unexplained `output.txt` entry.
- **D14 — `MAINTENANCE_REPORT_2026-05.md` relocated.** `git mv` to
  `docs/maint/2026-05-report.md`, matching the naming convention of its
  siblings (`2026-05-blocked.md`, `2026-05-followups.md`). The one
  reference in `CHANGELOG.md:28` is a historical record and left as-is.
- **D11 — `docs/agents/*.json` deleted.** Four v0.4.0-era agent state
  dumps (dated 2026-03-02; current version is 0.7.0) that framed the
  intentional TLS guard as a "critical bug" and otherwise misled. Zero
  references found repo-wide before deletion.

### Tier C — proposed, not executed

- **D12 — `docs/init_db.sql`, `docs/insert_items.sql`,
  `docs/agent_context.db`.** Scaffolding for the non-standard
  agent-memory SQLite convention referenced only by the now-collapsed
  `CLAUDE.md`. Leaving in place pending a maintainer decision on
  whether to retire the convention entirely.

### Deferred to Phase 5 (exam)

- **D7 — `README.md` error example typing.** The `Error::ApiError`
  example loosely types `code`; `Error` is not in scope in the snippet.
  Full exhaustive error-handling example review deferred to the Phase 5
  cold-start exam.

### Verification

- `cargo build` — clean.
- `cargo test` (default features) — 497 tests pass.
- `cargo test --no-default-features --features tls-native-tls` — (run in Phase 5).
- `cargo clippy --all-targets -- -D warnings` — clean.
- `pre_quality.sh` `--exclude mcp` warning — gone.

---

## 2026-07-07 — Phase 5 (exam)

Five cold-start challenges were authored as a zero-context agent would,
oriented via the scaffolding above, then compiled and run in an isolated
scratch crate at `/tmp/exam_scratch` consuming the lib by path.

### Challenge results

| # | Subsystem | Challenge | Outcome |
|---|-----------|-----------|---------|
| A | type-state builder + retry | Build a `Ready` client and set a custom `RetryConfig` | ✅ compile + run (after correction — see D15) |
| B | centralized error | Exhaustive `match` over `Error` (no wildcard) | ✅ compile + run; all 16 variants + fields confirmed real (resolves D7) |
| C | structured output | `client.structured()?.generate::<T>()` with a schema type | ✅ compiles; skips at runtime without a key (compile-only is the point) |
| D | type-state safety | Prove `.chat()` unavailable off `Ready` | ✅ positive case compiles; negative cases documented as commented proof |
| E | TLS mutex | `cargo build --all-features` must fail by design | ✅ `compile_error!` trips: "TLS features … mutually exclusive" |

### Defect found by the exam

- **D15 (Tier A) — `README.md` retry examples used an impossible builder
  chain.** `with_retry_config` / `with_retries` / `without_retries` are
  implemented only on `OpenRouterClient<NoAuth>`, NOT on `Ready`
  (`src/client.rs` line 222 `impl` block). But both README examples
  chained them after `OpenRouterClient::from_env()?`, which returns
  `Ready` — so the documented code **did not compile**:

  ```text
  error[E0599]: no method named `with_retry_config` found for struct
  `OpenRouterClient<Ready>` … the method was found for `OpenRouterClient<NoAuth>`
  ```

  This was missed in Phase 4 (D6 fixed the `?` syntax but preserved the
  broken chain) because README ```rust``` blocks are **not** executed as
  doctests — `cargo test --doc` runs only `src/**/*.rs` doc comments, not
  the top-level `README.md`. The exam's scratch-crate compilation is what
  caught it. **Fixed** both README examples to the verified-correct
  pattern (configure on `NoAuth`, then `with_api_key` → `Ready`) and
  tightened `AGENTS.md` invariant #2 to state that the config builders
  live on `NoAuth` and that `from_env()`/`from_api_key()` return an
  immutable `Ready` client.

### D7 resolution (closed)

The Phase 4 `Error::ApiError` example was re-checked against
`src/error.rs`. The enum is **not** `#[non_exhaustive]`, so a fully
exhaustive `match` (no `_` arm) is both legal and the safest form for
downstream consumers. Challenge B constructs and matches all 16 variants
with correct field destructuring; it compiles and runs.

### Verification (re-run after Phase 5 doc fixes)

- `cargo build` — clean.
- `cargo test` (default features) — 497 passed, 0 failed.
- `cargo test --no-default-features --features tls-native-tls` — 497 passed, 0 failed.
- `cargo test --doc` — 23 passed, 1 ignored (the README examples are
  illustrative, not doctests, so are not run here).
- `cargo clippy --all-targets -- -D warnings` (both TLS features) — clean.
- `cargo fmt --check` — clean.
- `cargo build --all-features` — fails by design (TLS mutex guard).
