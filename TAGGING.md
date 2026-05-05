# Tagging Policy

This crate publishes to crates.io as `openrouter_api`. As of the 2026-05
maintenance review, only `v0.1.5` and `v0.1.6` are tagged in git — every
release after that exists in `CHANGELOG.md` but has no corresponding tag.
A user reading "what changed in v0.4.1?" cannot diff against the source.

This document describes the tagging convention going forward and a
recommended backfill of past releases.

## Convention going forward

- **Tag format:** `vMAJOR.MINOR.PATCH` (no `v0.5.2-rc.1`-style pre-release
  suffixes for now; revisit when we cut a 1.0).
- **What gets tagged:** the commit on `main` that bumps `Cargo.toml` to
  the version being released. The commit message should be
  `chore: bump version to X.Y.Z` and should land *after* the CHANGELOG
  entry for that version is finalized.
- **Annotated tags only.** Use `git tag -a vX.Y.Z -m "release vX.Y.Z"`,
  not lightweight tags. Annotated tags carry the tagger's name/date and
  show up correctly in `git describe`.
- **Sign the tag if you have a key configured:**
  `git tag -s vX.Y.Z -m "release vX.Y.Z"`. Optional but recommended.

## Cutting a release (operator runbook)

```bash
# 1. Make sure you are on a clean main with the version bump merged.
git checkout main
git pull --ff-only origin main

# 2. Confirm Cargo.toml is on the new version.
grep '^version' Cargo.toml

# 3. Tag.
git tag -a v0.5.2 -m "release v0.5.2"

# 4. Push the tag. This triggers .github/workflows/release.yml,
#    which runs the test matrix, `cargo publish --dry-run`, then
#    `cargo publish`, then opens a GitHub Release.
git push origin v0.5.2
```

If the publish workflow fails, fix the issue on `main`, **delete the tag**
locally and remotely (`git tag -d v0.5.2 && git push --delete origin v0.5.2`),
then re-tag the new HEAD. Do not amend a published tag.

## Backfilling past releases

The following table maps each `CHANGELOG.md` entry without a tag to the
most-likely source commit. This was assembled from `git log --oneline`
and is **best-effort** — the actual `cargo publish` invocation that
released to crates.io may have run from a slightly different commit.

| Version | Likely commit | Commit summary                                                 |
| ------- | ------------- | -------------------------------------------------------------- |
| v0.2.0  | `04f1cef`     | docs: update README.md with new API examples and bump version  |
| v0.3.0  | `5880139`     | feat: Enterprise-Grade Error Handling Standardization (v0.3.0) |
| v0.3.1  | `5081201`     | feat: validation framework + CI cleanup (v0.3.1)               |
| v0.3.2  | `b8c6afb`     | Security Release v0.3.2: Fix Critical Vulnerabilities          |
| v0.4.0  | `fde47af`     | Quality & Security Improvements - v0.4.0                       |
| v0.4.1  | `e0d1fc8`     | chore: release v0.4.1 with security fixes (latest of two)      |
| v0.4.2  | (unverified)  | not separately listed; CHANGELOG dated 2025-11-30              |
| v0.4.3  | (unverified)  | not separately listed; CHANGELOG dated 2025-11-30              |
| v0.5.0  | `96f8bd7`     | feat: OpenRouter API 2025 Updates (v0.5.0)                     |
| v0.5.1  | `345b924`     | chore: bump version to 0.5.1                                   |

To apply the backfill (run as the maintainer, after confirming each row
against the actual crates.io publish history if possible):

```bash
git tag -a v0.2.0 04f1cef -m "release v0.2.0 (backfilled 2026-05)"
git tag -a v0.3.0 5880139 -m "release v0.3.0 (backfilled 2026-05)"
git tag -a v0.3.1 5081201 -m "release v0.3.1 (backfilled 2026-05)"
git tag -a v0.3.2 b8c6afb -m "release v0.3.2 (backfilled 2026-05)"
git tag -a v0.4.0 fde47af -m "release v0.4.0 (backfilled 2026-05)"
git tag -a v0.4.1 e0d1fc8 -m "release v0.4.1 (backfilled 2026-05)"
git tag -a v0.5.0 96f8bd7 -m "release v0.5.0 (backfilled 2026-05)"
git tag -a v0.5.1 345b924 -m "release v0.5.1 (backfilled 2026-05)"
git push origin --tags
```

`v0.4.2` and `v0.4.3` need additional research before a tag can be
backfilled with confidence — the CHANGELOG dates them both 2025-11-30
but no commit message clearly maps to either. Recommend skipping
those two unless you can corroborate against crates.io history.

**Backfilled tags will not retroactively trigger** the release workflow
(`release.yml`) because the relevant commits already exist; only newly
pushed tags fire the workflow. That is desirable here — we don't want
to re-publish historical versions to crates.io.
