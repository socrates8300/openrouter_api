# Maintenance run 2026-05 — blocked findings

Findings from `MAINTENANCE_REPORT_2026-05.md` that could not be resolved as a
single-PR source-tree change during the 2026-05 maintenance run, with reasoning.

## F1.1a — `cargo update -p bytes` (RUSTSEC-2026-0007)

**Status:** `not-applicable-now` (with policy caveat).

**Why blocked:** `Cargo.lock` is in `.gitignore` (line 8), so this repo follows
the "library, no checked-in lockfile" convention. `cargo update -p bytes`
mutates the local `Cargo.lock` but produces no diff against the source tree.
There is therefore no minimal one-PR change that resolves the finding.

Implications:

- **Downstream consumers** (libraries depending on `openrouter_api`) resolve
  `bytes` themselves. Their resolver picks `1.11.1+` automatically on a fresh
  build, so they are not exposed to RUSTSEC-2026-0007 because of this crate's
  lockfile state.
- **Local developers** running `cargo audit` against an old local lockfile
  see the advisory until they `cargo update -p bytes` themselves.
- **CI** generates a lockfile per run, so its `cargo audit` would already pass
  at the latest resolution.

**To actually "fix" the finding in-tree, one of the following policy changes
is required, neither of which is in scope for F1.1a as written:**

1. Remove `Cargo.lock` from `.gitignore` and commit it. Pinning the lockfile
   makes `cargo update -p bytes` a reviewable, in-tree change. Common for
   security-sensitive libraries; would also let `cargo audit` results be
   reproducible across contributors.
2. Add `bytes = "1.11.1"` as a direct dependency in `Cargo.toml`. This forces
   resolution but introduces a code-smell direct dep on a crate the source
   tree does not use directly.

Recommend the maintainer pick (1) as a separate decision and re-run `cargo
audit` after committing the lockfile. That decision is bigger than this one
finding and should be raised on its own.

**"Before" verification (still reproduces on `main` for a developer with the
checked-out lockfile that pins `bytes 1.11.0`):**

```
$ cargo audit
… RUSTSEC-2026-0007 — bytes 1.11.0 — Solution: Upgrade to >=1.11.1
```

**"After" verification was not produced** because no source-tree change was
applied. The local lockfile mutation has been reverted.
