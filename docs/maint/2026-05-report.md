# Maintenance Report — May 2026

**Repo:** socrates8300/openrouter_api
**Branch reviewed:** `main` @ `1716acd` (PR #43, "feat: Guardrails API, reasoning config, prompt cache fields, plugin constructors")
**Cargo.toml version:** `0.5.1`
**Toolchain used:** `cargo 1.95.0`, `rustc 1.95.0` (stable, 2026-04-14)
**Date:** 2026-05-04

> Note on the prompt: the brief refers to "current v0.1.6" with a README claim of "100% test coverage, 162 tests." Neither matches reality. Main is at 0.5.1 with ~509 tests and ~78% line coverage. Those specific README claims are no longer present; the README has been softened to "Extensive Test Coverage." This report works against today's repo state.

---

## Summary

- **Repo health: Yellow.** Build, fmt, doc, and the 469-test main suite are green. Two real problems sit in front of users on the current version: four `cargo audit` vulnerabilities (one trivial, three transitive via `reqwest 0.11` → `rustls 0.21`), and four README example blocks that won't compile against the current `Message` API.
- **Top 3 to do this month** (ranked by value/effort)
  1. **Run `cargo update -p bytes`.** Closes RUSTSEC-2026-0007 (integer overflow). 1 minute. **(P0, S)**
  2. **Fix the four broken README example blocks** (`role: "user".to_string()` → `ChatRole::User`). Currently any user copy/pasting hits a type error before they even hit the network. **(P0, S)**
  3. **Decide on the `reqwest 0.11` → `0.12` upgrade.** Three of the four audit advisories sit on `rustls-webpki 0.101.7` and require pulling rustls forward, which means moving reqwest to 0.12. Bigger lift than the other two; needs a half-day. **(P1, M–L)**
- **Top 3 safe to defer**
  1. The five Rust-1.95 `collapsible_match` clippy lints in `src/api/structured.rs` — only fire on the `-D warnings` gate with the newest stable; CI's `dtolnay/rust-toolchain@stable` will start failing within a few months, but right now CI's pre-quality script ran on a slightly older clippy. Defer to next month. **(P2, S)**
  2. The two `trybuild` `.stderr` mismatches in `tests/type_safety/{price_direct_construction,consumed_after_into}.rs` — Rust 1.95 reworded a couple of error messages. Routine maintenance, fix when blessing. **(P2, S)**
  3. The duplicate `ProviderPreferences` (`types::provider` vs `models::provider_preferences`). Real smell, no user impact today. **(P2, M)**

---

## Findings by surface

### 1. Dependency health

#### F1.1 — `cargo audit`: 4 vulnerabilities + 3 unmaintained warnings
- **Severity:** P0 / P1 mix
- **Description:** `cargo audit` (v0.22.1) reports 4 advisories with severity `error` and 3 with severity `warning`.
- **Evidence:** see Verification log §V1. Summary:

  | ID | Crate | Where | Action |
  |----|-------|-------|--------|
  | RUSTSEC-2026-0007 | `bytes 1.11.0` | direct (via reqwest, tokio, h2…) | **`cargo update -p bytes`** → 1.11.1. **P0/S.** |
  | RUSTSEC-2026-0104 | `rustls-webpki 0.101.7` | rustls 0.21 ← reqwest 0.11 | Reachable panic in CRL parsing. Needs rustls ≥ 0.103.13. **P1/M-L.** |
  | RUSTSEC-2026-0098 | `rustls-webpki 0.101.7` | same | URI name constraints incorrectly accepted. Same fix. **P1.** |
  | RUSTSEC-2026-0099 | `rustls-webpki 0.101.7` | same | Wildcard name constraints accepted. Same fix. **P1.** |
  | RUSTSEC-2024-0384 | `instant 0.1.13` | dev-dep via `wiremock` | unmaintained. Not user-reachable. CHANGELOG already documents this. **P2/—.** |
  | RUSTSEC-2025-0134 | `rustls-pemfile 1.0.4` | reqwest 0.11 | unmaintained. Resolved by reqwest upgrade. **P2/—.** |
  | RUSTSEC-2026-0097 | `rand 0.7.3` | dev-dep via `wiremock` → `http-types` | unsoundness. Not user-reachable. **P2/—.** |

- **Recommended action:** F1.1a (bytes) **fix now**; F1.1b–d (rustls-webpki) **fix this month** as the reqwest 0.12 upgrade. The three unmaintained dev-dep warnings can be deferred until `wiremock` cuts a new release (no in-tree fix).
- **Effort:** S for bytes; M–L for the reqwest/rustls migration (also clears `rustls-pemfile`).

#### F1.2 — Pinned versions are 1–2 majors behind on the HTTP stack
- **Severity:** P1
- **Description:** Production HTTP-stack deps are well behind current majors. `Cargo.toml` allows newer minors but not newer majors.
- **Evidence:**
  - `reqwest 0.11.27` (current is 0.12.x)
  - `hyper 0.14.32` (current is 1.x)
  - `rustls 0.21.12` (current is 0.23.x)
  - `thiserror 1.0.69` (current is 2.x)
- **Recommended action:** Bundle the reqwest/hyper/rustls upgrade with F1.1b–d this month. `thiserror 1 → 2` is independent and low-risk; defer until a separate cleanup pass.
- **Effort:** M (HTTP stack), S (thiserror).

#### F1.3 — Tooling for dependency review is missing locally
- **Severity:** Nit
- **Description:** `cargo audit` was not installed when I started this review (had to `cargo install cargo-audit` mid-run). `cargo-outdated` and `cargo-machete` are not installed. CI installs `cargo-audit` per-run.
- **Recommended action:** Note in CONTRIBUTING.md or `scripts/pre_quality.sh` that contributors should `cargo install cargo-audit cargo-outdated cargo-machete`. Or: add a Devbox/flake/justfile entry. Defer.
- **Effort:** S.

#### F1.4 — `deny.toml` does not exist
- **Severity:** Nit
- **Description:** No `deny.toml` in the repo, so `cargo deny check` cannot enforce a license/source/advisory policy. Given the CI already runs `cargo audit`, the marginal value is small for a small-team OSS crate.
- **Recommended action:** Defer.

#### F1.5 — MSRV claim is inconsistent
- **Severity:** P2
- **Description:** README line 75: "tested with Rust v1.83.0". `ci.yml` `msrv` job uses `1.70.0`. The two numbers do not match.
- **Recommended action:** Pick one. The CI MSRV check is the authoritative one; update README to "MSRV: 1.70 (verified in CI)."
- **Effort:** S.

#### F1.6 — Unused dependencies (manual scan, no `cargo machete`)
- **Severity:** —
- **Description:** Spot-checked direct deps in `Cargo.toml` against `src/`. All appear referenced: `tracing` (chat.rs, retry.rs), `chrono` (analytics types, integration tests), `httpdate` (retry.rs Retry-After parsing), `urlencoding` (analytics URL building), `uuid` (mcp/client.rs), `fastrand` (retry.rs jitter), `tokio-util` (chat.rs streaming codec), `zeroize` (security.rs), `regex` (security.rs).
- **Recommended action:** None — but install `cargo-machete` and run it next month for a real check (manual scan misses macro-only or feature-gated uses).

---

### 2. Code health

#### F2.1 — `cargo clippy --all-targets ... -- -D warnings` fails on stable 1.95.0
- **Severity:** P2
- **Description:** Five `collapsible_match` errors in `src/api/structured.rs` between lines 159 and 191. New lint in Rust 1.95. The project's `pre_quality.sh` runs the same flags, so this will block CI as soon as `dtolnay/rust-toolchain@stable` rolls past whatever clippy version was current when PR #43 merged (2026-03-29).
- **Evidence:** See Verification log §V2. Sample:
  ```
  error: this `if` can be collapsed into the outer `match`
    --> src/api/structured.rs:180:25
  ```
- **Recommended action:** Either collapse the matches as clippy suggests, or `#[allow(clippy::collapsible_match)]` at the function level if the explicit form is preferred for readability. Trivial change.
- **Effort:** S.

#### F2.2 — `--all-features` cannot be used: TLS features are mutually exclusive at compile time
- **Severity:** P2
- **Description:** `src/lib.rs:23-26` has `compile_error!` for the case where `tls-rustls` and `tls-native-tls` are both enabled. `cargo build --all-features` and `cargo test --all-features` fail with that error. Affects ad-hoc `--all-features` invocations from contributors and tooling that assumes `--all-features` is safe (e.g., some `cargo-llvm-cov` defaults). CI is unaffected because `ci.yml` enables one TLS feature at a time and `pre_quality.sh` does the same.
- **Evidence:**
  ```
  error: TLS features tls-rustls and tls-native-tls are mutually exclusive...
   --> src/lib.rs:24:1
  ```
- **Recommended action:** Document in CONTRIBUTING and Cargo.toml feature-comment block that `--all-features` is unsupported; do not change the design — the compile-time check itself is correct.
- **Effort:** S.

#### F2.3 — `cargo fmt --check`: clean
- No findings.

#### F2.4 — `cargo doc --no-deps --features tls-rustls`: clean
- No findings; 0 warnings/errors.

#### F2.5 — `unsafe`: zero non-test occurrences
- The single grep hit is the word "unsafe" inside a string literal in `src/utils/validation/web_search.rs:89`. The crate has no `unsafe` blocks.

#### F2.6 — `unwrap()` / `expect()` / `panic!()` in production paths: justified
- **Severity:** —
- **Description:** All 362 hits are concentrated in `mod tests`. The pre-test-block hits in production code:
  - `src/client.rs:129` — `.parse().unwrap()` on the constant `"https://openrouter.ai/api/v1/"`. Compile-time-safe (constant-string parse).
  - `src/utils/security.rs:9,13,19,23` — `Regex::new(...).unwrap()` inside `LazyLock` for redaction patterns. The regex literals are constants; failure would mean the build itself is broken, not a runtime panic risk.
  - `src/api/request.rs:183` — `.expect(...)` inside a doc-comment example, not actual code.
- **Recommended action:** None.

#### F2.7 — `TODO` / `FIXME` / `XXX` / `HACK`: zero in `src/`
- No findings.

#### F2.8 — `#[non_exhaustive]` is not used on any of ~100 public types
- **Severity:** P1
- **Description:** `grep -rEln "#\[non_exhaustive\]" src/` returns nothing. `src/types/` declares ~100 `pub enum`/`pub struct`. For a client wrapping a fast-evolving upstream API (OpenRouter ships new fields/plugins every few weeks), every new field on a public response struct or new variant on a public enum is a SemVer-breaking change. The `[Unreleased]` CHANGELOG entry alone added `cache_creation_input_tokens`, `cache_read_input_tokens`, `cache_discount`, `provider_name`, and four reasoning-related types — each technically a major bump under strict SemVer.
- **Recommended action:** As a one-time pass, mark response/notification structs (Usage, ChatCompletionResponse, ProviderInfo, ModelInfo, etc.) and the open-ended Plugin/ContentType/ChatRole enums `#[non_exhaustive]`. Do this *before* a 1.0 release; doing it after is itself a breaking change.
- **Effort:** M (audit + apply, no behavior change).

#### F2.9 — Public API: no obvious leakage
- Spot-checked `pub use` in `src/lib.rs` and `src/types/mod.rs`: re-exports are intentional and consistent. `pub use types::*` (lib.rs:16) is broad but matches the README's `use openrouter_api::types::chat::...` pattern.

---

### 3. Test reality check

#### F3.1 — Actual test count
- **Verified count** (running `cargo test --features tls-rustls,tracing,allow-http -- --list` on each test target): **388 + 5 + 2 + 50 + 24 = 469** test cases across 5 test binaries, plus **24 doc tests** (23 pass, 1 ignored), plus **16 trybuild compile-fail cases**. **Total ≈ 509.**
- README does not currently quote a specific number (the historical "162 tests" claim lives in `CHANGELOG.md` only). No drift on this dimension.

#### F3.2 — `tests/type_safety/{price_direct_construction,consumed_after_into}` — `.stderr` mismatch on Rust 1.95
- **Severity:** P2
- **Description:** `cargo test --test compile_fail` reports "2 of 16 tests failed" and `test result: FAILED`. The mismatch is in expected-vs-actual diagnostic text — Rust 1.95 reformats the borrow-of-moved-value note (`note: ` indentation and the `into takes ownership` hint changed). Test logic still detects the type error; only the cosmetic stderr fixture is stale.
- **Evidence:** See Verification log §V3.
- **Recommended action:** Re-bless with `TRYBUILD=overwrite cargo test --test compile_fail` and review the diff. Note that this test is **the reason `cargo test` exits non-zero on main**; everything else passes.
- **Effort:** S.

#### F3.3 — Compile-fail failure makes `cargo llvm-cov --workspace --tests` exit non-zero
- **Severity:** P2 (downstream of F3.2)
- **Description:** `cargo llvm-cov --features tls-rustls --workspace --tests --summary-only` aborts before reporting because the trybuild target fails. The Codecov upload in `ci.yml` uses `fail_ci_if_error: false`, so coverage uploads silently zero out without a noisy failure. I had to fall back to `cargo llvm-cov --features tls-rustls --lib` to get a number.
- **Recommended action:** Fix F3.2; that resolves this transitively.

#### F3.4 — Actual coverage: 78.13% line, 72.57% function, 74.70% region (lib only)
- **Severity:** —
- **Description:** Measured with `cargo llvm-cov --features tls-rustls --lib --summary-only`. Total line coverage is **78.13%**, not "100%" as the historical CHANGELOG claimed (current README does not claim a number).
- **Hot spots — modules under 80%:**

  | Module | Line cov | Note |
  |---|---|---|
  | `types/provider.rs` | 0.00% | duplicate of `models/provider_preferences.rs`; only used by internal `chat_request_builder` path. See F7.1. |
  | `types/routing.rs` | 0.00% | likely never instantiated by tests; investigate. |
  | `types/ids.rs` | 30.99% | newtype-id machinery — many display/conversion impls untested. |
  | `utils/validation/chat.rs` | 47.56% | validation framework; many error branches untested. |
  | `utils/validation/common.rs` | 59.56% | same. |
  | `types/guardrails.rs` | 77.65% | new in PR #43; tests added but not full. |

- **Recommended action:** Don't chase 100% — but the two 0% files deserve five minutes' attention to confirm they aren't dead code. The validation framework gaps would benefit from a proptest pass; if that's not on the roadmap, document them as accepted.
- **Effort:** S to investigate; M to close the 0% files; L for full validation coverage.

#### F3.5 — Tests use real wire behavior via `wiremock`, not stubs alone
- **Severity:** —
- **Description:** `wiremock 0.5.22` is a dev-dep and is referenced in `src/tests/integration_tests.rs` and `src/tests/retry_and_streaming_tests.rs`. Streaming and retry/backoff have integration tests that exercise an in-process HTTP server, not just mocked function calls. This is the right shape; flagging only because it's a strong point worth preserving when migrating to reqwest 0.12 (wiremock 0.5 is on the unmaintained dependency tree).

#### F3.6 — Flaky-test smell
- **Severity:** —
- **Description:** Quick scan: tests in `src/tests/retry_and_streaming_tests.rs` use `tokio::time::sleep` and short timeouts to verify backoff; could be flaky on a slow CI runner but uses `tokio::test(start_paused = true)` semantics where it matters. No bare `std::thread::sleep` in non-test code. No findings worth a follow-up.

---

### 4. CI / release hygiene

#### F4.1 — Two GitHub-Actions workflows, only one is current
- **Severity:** P2
- **Description:** Both `.github/workflows/ci.yml` (the comprehensive one) and `.github/workflows/rust.yml` (a stripped-down build+test) run on every push and PR to main. `rust.yml` is older, uses `actions-rs/toolchain@v1` (deprecated/unmaintained), `actions/cache@v3` (latest is v4 and ci.yml uses v4), and exposes `OPENROUTER_API_KEY` to a job that doesn't actually need it (only does `cargo build` + `cargo test`).
- **Recommended action:** Delete `.github/workflows/rust.yml`. ci.yml supersedes it.
- **Effort:** S.

#### F4.2 — No release workflow; publishing is manual
- **Severity:** P2
- **Description:** No `release.yml` or `publish.yml`. crates.io publishing is presumably `cargo publish` from a maintainer's laptop. Combined with F4.4 (no git tags after v0.1.6), this is the main release-hygiene gap.
- **Recommended action:** Add a release workflow that triggers on `v*` tags and runs `cargo publish` with `CARGO_REGISTRY_TOKEN`. Couples with F4.4.
- **Effort:** S.

#### F4.3 — CI matrix matches what the README implies
- **Severity:** —
- **Description:** `ci.yml`'s `test-matrix` job runs both `--features rustls` and `--no-default-features --features native-tls` on stable and beta. MSRV (1.70.0) is checked separately. Matches the README's "tested on stable" implication. No findings.

#### F4.4 — Git tags vs. crates.io: only v0.1.5 and v0.1.6 are tagged
- **Severity:** P1
- **Description:** `git tag --list` returns only `v0.1.5` and `v0.1.6`. There are 51 commits between `v0.1.6` and `HEAD`. CHANGELOG documents released versions 0.2.0, 0.3.0, 0.3.1, 0.3.2, 0.4.0, 0.4.1, 0.4.2, 0.4.3, 0.5.0, 0.5.1 — none of these have git tags. A user reading "what's in v0.4.1?" cannot diff against the source. (I did not check crates.io to confirm which of those numbers were actually published.)
- **Recommended action:** As part of the next release: pick a tagging scheme (`v0.5.1`), backfill at least the most recent 2–3 versions if you can identify the right commit, and add a release-tag step to whatever publish workflow comes out of F4.2.
- **Effort:** S–M depending on backfill ambition.

#### F4.5 — CHANGELOG.md drift
- **Severity:** P2
- **Description:** Multiple structural problems:
  - Two separate `## [0.5.1]` sections with different dates (2025-12-27 and 2025-01-18). The latter looks like a misnumbered entry that should be `0.5.1` or `0.2.x`.
  - The `## [Unreleased] - 2026-03-29` block describes content that **was already merged** (PR #43, also dated 2026-03-29) and is in the `0.5.1`-stamped Cargo.toml on `main`. So either it should become `## [0.5.2] - 2026-03-29` and the version bumped, or the version bump is overdue.
  - Several `0.4.x` entries are all dated 2025-11-30; `0.3.0` is dated 2025-01-18 even though commit `5880139` ("v0.3.0") is from 2025-10-17. Dates are not reliable.
- **Recommended action:** Rename `[Unreleased]` to `[0.5.2]`, bump Cargo.toml, tag, and publish. Separately, take ten minutes to dedupe and cross-check dates against `git log --tags --simplify-by-decoration`.
- **Effort:** S for the unblock; S for the dedupe.

#### F4.6 — `pre_quality.sh` uses legacy feature names
- **Severity:** Nit
- **Description:** Lines 25, 32, 39, 46, 53, 60, 78, 85 use `--features rustls` / `--features native-tls`. These are the *deprecated* legacy aliases per `Cargo.toml:48-49`. Still works, will keep working. Future-proofing only.
- **Recommended action:** Defer.

---

### 5. Doc / reality drift

#### F5.1 — Four README examples won't compile against the current `Message` API
- **Severity:** P0
- **Description:** Today's `Message` is `{ role: ChatRole, content: MessageContent, name, tool_calls, tool_call_id, reasoning, reasoning_details }` (7 fields, role is an enum, content is an enum). Several README examples still construct it the way it looked before the type-safety pass:
  - `README.md:254-260` (Provider Preferences example) — `role: "user".to_string(), content: "Hello with provider preferences!".to_string(), name: None, tool_calls: None,`
  - `README.md:379-389` (Streaming Chat example) — same `role: "user".to_string()` pattern.
  - `README.md:914-928` (Chat Completions snippet, second half of the README) — same.
  - `README.md:954-971` (Tool Calling snippet, second half) — same.
- **Evidence:** `grep -n 'role: "user"' README.md` returns those four lines.
- **Recommended action:** Replace with the same idiom the working examples use (`Message::text(ChatRole::User, "...")`) and `..Default::default()` for the rest. Issue #40 covered the *examples directory* fix; the README missed the second half of the document.
- **Effort:** S.

#### F5.2 — README footer says "Version: 0.1.6"
- **Severity:** P2
- **Description:** `README.md:828` — "_**Version:** 0.1.6 • **License:** MIT / Apache‑2.0_". Cargo.toml is at 0.5.1. Two majors out of date.
- **Recommended action:** Drop the hardcoded version string from the footer (a README isn't where to track versions; CHANGELOG is). Or template it.
- **Effort:** S.

#### F5.3 — README "Available Features" section uses legacy names
- **Severity:** P2
- **Description:** `README.md:71-73` lists `rustls (default)` and `native-tls`. `Cargo.toml:48-49` notes these are deprecated aliases for `tls-rustls` / `tls-native-tls`. The user's first encounter with the feature flags is from a list that points at the legacy names.
- **Recommended action:** Update the README to lead with `tls-rustls` / `tls-native-tls` and mention `rustls` / `native-tls` only as legacy aliases.
- **Effort:** S.

#### F5.4 — Implementation Status checklist: every ✅ item exists in the source
- **Severity:** —
- **Description:** Spot-checked each of the 16 ✅ Core Features against `src/api/`, `src/types/`, `src/mcp/`. All present:
  - `client.rs` (type-state, `from_env`, `from_api_key`, `quick`, `production`, `with_retry_config`)
  - `api/{chat,completion,credits,generation,analytics,providers,models,guardrails,key_info,embeddings,structured,web_search}.rs`
  - `mcp/client.rs` + `mcp/types.rs` (with `MCP_PROTOCOL_VERSION`, `ClientCapabilities`, `GetResourceParams`, `ToolCallParams`)
  - `types/chat.rs` for `Plugin::response_healing()`, `Plugin::web_search()`, `Plugin::file_parser()`, `Plugin::context_compression()`, `ReasoningConfig`, `ReasoningEffort`, `ReasoningSummary`
  - `models/provider_preferences.rs`
- The ✅ list is *not* aspirational. (Whether each endpoint is shaped right vs. the OpenRouter API is out of scope here.)

#### F5.5 — `SECURITY_ADVISORY.md` is point-in-time, doesn't track new advisories
- **Severity:** P2
- **Description:** The file documents v0.4.2, v0.4.1, v0.3.2 fixes. It has no entry for the four open `cargo audit` advisories from F1.1 — but that's fine because those are upstream-supply-chain advisories, not vulnerabilities in this crate's code. The file is correctly scoped to **this crate's** security history. No action needed unless one of the rustls-webpki advisories is judged to be user-reachable in a way that warrants its own coordinated disclosure.
- **Recommended action:** Defer. Optionally add a short "Upstream advisories tracked" subsection pointing at the dependency advisories on the next bump.

#### F5.6 — README "tested with Rust v1.83.0" — stale
- See F1.5.

---

### 6. Issue and PR triage

#### F6.1 — There are no open issues and no open PRs.
- **Severity:** —
- **Description:** `gh issue list --state open` returns nothing. `gh pr list --state open` returns nothing. The most recent activity is PR #43 merged on 2026-03-29. The most recent issue (#40, "[Bug] example code does not match documentation") was closed 2026-03-05 by PR #42. The repo has been quiet for ~1 month before this review.
- **Recommended action:** None on the triage axis. The "examples don't match documentation" issue (#40) was *partially* addressed — examples/*.rs were fixed but README.md wasn't (see F5.1). Worth opening a follow-up issue.

#### F6.2 — Closed-PR pattern: lots of "feat" merges, no review trail visible
- **Severity:** —
- **Description:** Looking at the last ~10 merged PRs (#43, #42, #41, #38, #36, #34, #33, #29, #28, #27): all merge ~hours after creation, almost certainly self-merged by the maintainer. That's normal for a one-person OSS project; flagging only because if/when the project gains contributors, the absence of CODEOWNERS / required reviews is the next thing that will bite.

---

### 7. Architectural smells (lightweight pass)

#### F7.1 — Two `ProviderPreferences` types
- **Severity:** P2
- **Description:** `src/types/provider.rs` defines `pub struct ProviderPreferences` (116 LOC, 0% covered) and `src/models/provider_preferences.rs` defines a *different* `pub struct ProviderPreferences` (158 LOC, used in README examples and `ChatCompletionRequest::provider`). Both reachable from public API:
  - `src/client.rs:359,364` and `src/types/routing.rs:57` use `crate::types::provider::ProviderPreferences`
  - `src/api/request.rs:188`, `src/types/chat.rs:564`, `src/types/embeddings.rs:35` use `crate::models::provider_preferences::ProviderPreferences`
- **Recommended action:** Pick one and migrate the other. The `models/provider_preferences.rs` version is the one users actually see (it's the one in `ChatCompletionRequest`); delete the `types/provider.rs` version and migrate `client.rs` / `types/routing.rs`. Will be a SemVer-minor change; sequence it before a 1.0 release.
- **Effort:** M.

#### F7.2 — Type-state pattern: no obvious bypass
- **Severity:** —
- **Description:** Spot-checked the `OpenRouterClient<NoAuth | Unconfigured | Ready>` markers. The "make a request" entry points (`chat()?`, `completions()?`, `analytics()?`, etc.) return `Result<…>` and are gated through `.with_api_key(...)?` or one of the convenience constructors. I did not find a public method that returns an authenticated `*Api` without going through that path.

#### F7.3 — `Error` enum: large but coherent
- **Severity:** —
- **Description:** `src/error/` defines a single `Error` enum with variants for HTTP, API errors, rate limiting, config, context-length, schema validation, JSON-RPC. It's grown over time but each variant is still pulling its weight. Not a grab bag yet.

#### F7.4 — Module boundaries: `client` does some `api`-shaped work
- **Severity:** Nit
- **Description:** `src/client.rs:493` — `chat_request_builder` constructs requests in `client.rs` rather than delegating to `api/chat.rs`. Minor smell, not worth refactoring on its own.

#### F7.5 — `mcp` module: maintained, not bit-rotting
- **Severity:** —
- **Description:** `src/mcp/client.rs` was touched in v0.5.1 ("JSON-RPC ID Validation") per CHANGELOG. `MCP_PROTOCOL_VERSION = "2025-03-26"` matches the latest spec the README links to. `cargo test` exercises the MCP client in `src/tests/integration_tests.rs`. PR #37 attempted a "Systematic remediation of MCP client" and was closed-without-merge in 2025-11-30 — worth a quick read to confirm nothing was lost from that branch.

---

## Doc / reality drift (consolidated)

| README claim | Reality | Severity |
|---|---|---|
| `Message { role: "user".to_string(), content: "...".to_string(), name: None, tool_calls: None }` (4 places) | `Message { role: ChatRole::User, content: MessageContent::Text("..."), ..Default::default() }` (7 fields, two are enums) | **P0** |
| Footer: "Version: 0.1.6" | Cargo.toml: 0.5.1 | P2 |
| "tested with Rust v1.83.0" | CI MSRV: 1.70.0 | P2 |
| "Available Features: `rustls` (default), `native-tls`, `tracing`" | Current names are `tls-rustls`/`tls-native-tls`; the listed names are deprecated aliases | P2 |
| "100% test coverage, 162 tests" *(prompt-supplied claim, not in current README)* | 78.13% line / ~509 tests | — (already softened in README to "Extensive Test Coverage") |
| Implementation Status ✅ items | All 16 verified present in source | — |

---

## Tooling gaps

To make the next month's review faster:

1. **CI: add `cargo update --locked --dry-run` or a Renovate/Dependabot config.** Right now dependency drift is invisible until someone runs `cargo audit`. Pulling RUSTSEC advisories into PRs automatically would have caught the four findings in F1.1 the day they were published.
2. **Devbox/justfile: bake in `cargo-audit`, `cargo-outdated`, `cargo-machete`, `cargo-llvm-cov`.** The repo has a `devbox.json` per PR #9 history but the toolset for review is not pre-installed.
3. **CI: add a Rust-`beta` clippy gate distinct from the `stable` one.** Then `collapsible_match`-class lints surface a release ahead of breaking the pre-quality script.
4. **CI: a release workflow.** F4.2 / F4.4. Without one, "release" means "the maintainer remembered to `cargo publish` and `git tag`."
5. **CI: separate the `cargo test --doc` / `cargo test --tests` / `cargo test --test compile_fail` steps.** Currently a trybuild stderr drift blocks coverage collection (F3.3). Splitting them lets the green parts stay green.

---

## Verification log

Branch: `main` @ `1716acd`. Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`. macOS / aarch64. All commands run from repo root.

### V1. `cargo audit`
```
$ cargo audit
… (database fetched, 252 deps scanned)
Crate: bytes  Version: 1.11.0  ID: RUSTSEC-2026-0007  Solution: Upgrade to >=1.11.1
Crate: rustls-webpki  Version: 0.101.7  ID: RUSTSEC-2026-0104
Crate: rustls-webpki  Version: 0.101.7  ID: RUSTSEC-2026-0098
Crate: rustls-webpki  Version: 0.101.7  ID: RUSTSEC-2026-0099
Crate: instant  Version: 0.1.13   Warning: unmaintained  ID: RUSTSEC-2024-0384
Crate: rustls-pemfile  Version: 1.0.4   Warning: unmaintained  ID: RUSTSEC-2025-0134
Crate: rand  Version: 0.7.3   Warning: unsound  ID: RUSTSEC-2026-0097
error: 4 vulnerabilities found!
warning: 3 allowed warnings found
```

### V2. `cargo clippy`
```
$ cargo clippy --all-targets --features tls-rustls,tracing,allow-http -- -D warnings
… (5 errors, all collapsible_match in src/api/structured.rs)
error: this `if` can be collapsed into the outer `match` --> src/api/structured.rs:159:25
error: this `if` can be collapsed into the outer `match` --> src/api/structured.rs:166:25
error: this `if` can be collapsed into the outer `match` --> src/api/structured.rs:173:25
error: this `if` can be collapsed into the outer `match` --> src/api/structured.rs:180:25
error: this `if` can be collapsed into the outer `match` --> src/api/structured.rs:187:25
error: could not compile `openrouter_api` (lib) due to 5 previous errors
```

### V3. `cargo test --features tls-rustls,tracing,allow-http`
```
running 388 tests
test result: ok. 388 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.42s

running 5 tests
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 1 test
test compile_fail_tests ... FAILED
2 of 16 tests failed (price_direct_construction.rs, consumed_after_into.rs — stderr fixture drift on Rust 1.95)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

Doc tests:
```
$ cargo test --features tls-rustls,tracing,allow-http --doc
test result: ok. 23 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.99s
```

Per-suite list:
```
$ cargo test --features tls-rustls,tracing,allow-http --tests -- --list
388 tests, 0 benchmarks
5 tests, 0 benchmarks
2 tests, 0 benchmarks
50 tests, 0 benchmarks
24 tests, 0 benchmarks
```

### V4. `cargo fmt --check`
```
$ cargo fmt --check
$ echo $?
0
```

### V5. `cargo doc --no-deps`
```
$ cargo doc --no-deps --features tls-rustls
… (clean, 0 warnings)
```

### V6. `cargo llvm-cov` (lib only — `--workspace --tests` blocked by V3)
```
$ cargo llvm-cov --features tls-rustls --lib --summary-only
… (per-file table; truncated)
TOTAL                       11373    2487    78.13%   999    274    72.57%   8165   2066    74.70%
```

### V7. `cargo build --examples`
```
$ cargo build --examples --features tls-rustls,tracing,allow-http
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
```

### V8. `--all-features` failure
```
$ cargo test --all-features
error: TLS features tls-rustls and tls-native-tls are mutually exclusive. Please choose only one.
 --> src/lib.rs:24:1
```

### V9. Public-API surface scan
```
$ grep -rEln "#\[non_exhaustive\]" src/   # zero hits
$ grep -rEcn "pub enum |pub struct " src/types/ | awk -F: '{s+=$2} END {print s}'
~100 pub items
```

### V10. `unsafe` / `unwrap` / `panic` scan
```
$ grep -rln "unsafe " src/
src/utils/validation/web_search.rs   (string literal only — no unsafe blocks)
$ grep -rEn "TODO|FIXME|XXX|HACK" src/
(no matches)
```

### V11. GitHub state
```
$ gh issue list --repo socrates8300/openrouter_api --state open  # empty
$ gh pr   list --repo socrates8300/openrouter_api --state open   # empty
```

### V12. Tags vs. CHANGELOG
```
$ git tag --list
v0.1.5
v0.1.6
$ grep -E "^## \[" CHANGELOG.md
[Unreleased] - 2026-03-29 ; [0.5.1] - 2025-12-27 ; [0.5.0] - 2025-11-30 ; [0.5.1] - 2025-01-18 ;
[0.4.3] - 2025-11-30 ; [0.4.2] - 2025-11-30 ; [0.4.1] - 2025-11-30 ; [0.4.0] - 2025-10-18 ;
[0.3.1] - 2025-10-18 ; [0.3.0] - 2025-01-18 ; [0.2.0] - 2025-01-16 ; [0.1.6] - 2025-01-14 …
$ git log v0.1.6..HEAD --oneline | wc -l
51
```
