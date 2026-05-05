# Maintenance run 2026-05 — follow-ups

Items noticed during the 2026-05 maintenance run that were intentionally
out of scope for the per-finding PRs in that run, plus deferred sub-items
of in-scope findings. None blocked the run; each is a candidate for a
future maintenance pass or a one-off PR.

## Out of scope of any 2026-05 finding

### `.github/workflows/ci.yml` uses the legacy `rustls` / `native-tls` feature aliases

Lines 80, 83, 145 of `ci.yml` use `--features rustls` / `--features native-tls`. F4.6 covered the same lint in `scripts/pre_quality.sh`; the `ci.yml` occurrences are the same problem in a different file and should be migrated for consistency.

Trivial fix:
```diff
-      - name: Run tests (rustls)
-        run: cargo test --features rustls --verbose
+      - name: Run tests (rustls)
+        run: cargo test --features tls-rustls --verbose
```
…and similarly for the native-tls and coverage steps.

### CHANGELOG: stale dates on 0.3.x / 0.4.x entries

Per F4.5, the dedup of `[0.5.1]` was in scope; cross-checking dates was not. Outstanding suspect dates:

- `[0.3.0] - 2025-01-18` — commit `5880139` ("v0.3.0") is from 2025-10-17 (per the maintenance report's spot check).
- `[0.4.x] - 2025-11-30` (multiple entries all dated the same day) — almost certainly auto-generated rather than reflecting actual release cadence.

A focused cleanup PR could re-derive each `[X.Y.Z]` date from the commit/PR that bumped Cargo.toml to that version.

### CHANGELOG: rename `[Unreleased]` to `[0.5.2]` and bump Cargo.toml

The `[Unreleased]` block describes work that has already merged on `main` (PR #43 + several entries added during this run). Per F4.5 recommendation: rename to `[0.5.2]`, bump `Cargo.toml` to `0.5.2`, then tag (using the new `release.yml` from F4.2 PR #56 and the policy from F4.4 PR #57). Was deliberately deferred from F4.5 because the version bump is a release decision, not a docs cleanup.

### `README.md` "Implementation Status" table not re-verified after this run

The maintenance report (F5.4) re-verified the 16 ✅ items against `src/` and they all checked out at the time of writing. Worth re-confirming after the F7.1 (`ProviderPreferences` dedup) PR merges, since the README references `models::provider_preferences` which now becomes the only `ProviderPreferences` type.

## Deferred sub-items of selected findings

### F3.3 — `cargo llvm-cov --workspace --tests` exit code

F3.3 was bundled with F3.2 in PR #46 (the trybuild stderr drift was the upstream cause). Once #46 merges to `main`, re-run `cargo llvm-cov --features tls-rustls --workspace --tests --summary-only` from the report's V6 to confirm coverage collection now succeeds end-to-end.

### F4.4 — Actually push backfilled tags to `origin`

PR #57 documents the backfill policy and provides a copy-pasteable `git tag -a … && git push origin --tags` script in `TAGGING.md`. The PR intentionally does not push tags. Maintainer to review the table (especially the `v0.4.2` / `v0.4.3` rows flagged as needing extra verification) and run the backfill once comfortable.

### F2.8 — `Plugin`, `ContentPart`, `Tool` family

F2.8 in PR #59 marked `#[non_exhaustive]` on the response surface and the open enums users only ever match. It deliberately left these construction-and-match enums alone:

- `Plugin` (struct, but constructed via `Plugin::web_search()` / `Plugin::file_parser()` / `Plugin::response_healing()` constructors)
- `ContentPart`, `MessageContent`, `ReasoningDetail`
- `Tool`, `ToolType`, `ToolChoice`, `StopSequence`

A future PR could add `#[non_exhaustive]` to each of these PROVIDED constructor methods exist for every "happy path" so users never need struct-literal/enum-literal construction. That's a non-trivial design call separate from the safe one-time pass in F2.8.

### F7.1 — Lost fields from the deleted `types::provider::ProviderPreferences`

The deleted struct had three fields the canonical `models::provider_preferences::ProviderPreferences` doesn't:

- `allow: Option<Vec<String>>`
- `provider_options: Option<HashMap<String, serde_json::Value>>`
- `route_optimizations: Option<Vec<String>>`

Verified by grep that none of these were ever set anywhere (declared but never used). However, the **canonical** `models` version is missing an `allow` field that the OpenRouter API actually documents (provider allow-list). If/when that becomes a user-facing requirement, add `allow: Option<Vec<String>>` to the canonical type with `#[serde(rename_all = "camelCase")]` already in place.

## Items the maintenance report explicitly recommended deferring

For completeness — these are not bugs, just things the report flagged and recommended **not** doing this month. Listed here so they don't get lost.

- **F1.4** — `deny.toml` not present. Recommend adding only if `cargo deny` becomes part of CI (currently `cargo audit` covers the highest-value subset).
- **F1.6** — `cargo machete` not run. Worth running next month to catch genuine unused-dependency drift; manual scan in the report cleared all current direct deps.
- **F5.5** — `SECURITY_ADVISORY.md` is point-in-time and currently scoped only to this crate's history (correct scope for this crate's responsibility).
- **F6.1 / F6.2** — No open issues / PRs at run start; PR review patterns indicate solo maintainership. CODEOWNERS / required reviews can wait until a second contributor lands.
- **F7.2 / F7.3 / F7.4 / F7.5** — Architectural smells flagged as "not worth refactoring on its own" in the report.
