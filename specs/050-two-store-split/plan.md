# Implementation Plan: The two-store split (catalog.db + local.db)

**Branch**: `050-two-store-split` | **Date**: 2026-08-16 | **Spec**: [spec.md](spec.md)

**Slice**: S050 (GitHub issue #137, milestone v0.5.0). Depends on S049 (merged).

## Summary

Split the single `hint.db` into `catalog.db` (ShruggieTech-shipped, disposable)
and `local.db` (user-owned, durable), both in the per-user AppData root. First run
bootstraps both; learned launch accumulation writes to `local.db`; resolution
reads both (local first). The MSI and release packaging install the seed as
`catalog.db`. No migration. The change is confined to `fragcap-targets` (the
layered provider) and `fragcap-cli` (paths, flags, bootstrap, doctor), plus the
WiX source, the release workflow, the specification, and the glossary.

## Technical Context

**Language/Version**: Rust, pinned toolchain (MSRV 1.82). SQLite via `rusqlite`
(bundled), behind the `targets` feature at the facade.

**Primary Dependencies**: none added. Reuses `rusqlite`, the existing `Store`, and
the existing resolver.

**Storage**: two SQLite files, same version-2 schema.

**Testing**: `cargo test` for the targets crate (provider layering, bootstrap) and
the CLI crate (paths, flags, bootstrap, doctor). Run under
`cargo +1.96.0-x86_64-pc-windows-gnu` in this environment (R-11); CI runs the
pinned msvc build.

**Target Platform**: Windows 11 x64 for the store locations; the store logic is
platform-neutral.

**Project Type**: single Rust workspace; changes in `fragcap-targets`,
`fragcap-cli`, `crates/fragcap-cli/wix`, `.github/workflows/release.yml`, `docs/`.

**Constraints**: no elevation for either store; text hygiene (P-8); pinned-artifact
rule for `release.yml`; the S049 spec lock-step and `spec-impact` fragment fields.

**Scale/Scope**: two files, one crate provider change, CLI path/flag/bootstrap
rework, one workflow, one WiX source, spec + glossary edits.

## Constitution Check

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | N/A; no capture technique. The accumulation still reads only the appinfo cache and opens no process handle. Pass. |
| P-2 Core Platform-Neutral | No change to `fragcap-core`; no new dependency. Pass. |
| P-3 Capture/Attribution Separate | N/A. Pass. |
| P-4 No Silent Loss | Bootstrap failures and accumulation faults stay warnings/surfaced, unchanged. Pass. |
| P-5 Compatibility | N/A; no output format change. Pass. |
| P-6 Glossary First | New terms `catalog.db`, `local.db` get glossary entries + index (R-10). Pass by inclusion. |
| P-7 Wrappers Stay Thin | Logic stays in Rust; no wrapper parsing added. Pass. |
| P-8 House Standards | All edited files obey text hygiene; changelog fragments carry `spec-impact` (S049). Pass with care. |
| P-9 Instrument Does Not Lie | The split changes where data is stored, never what is observed or reported. Pass. |
| P-10 One Path To A Target | The split is the storage foundation P-10 builds on; one store shape (reused type) for both files. Pass. |
| P-11 / S049 gate | Specification store sections are edited in-slice; fragments name them via `spec-impact`. `release.yml` (pinned) carries a `decisions` fragment. Pass. |

**Gate result**: PASS. No Complexity Tracking entries.

## Project Structure

```text
crates/fragcap-targets/src/hint_provider.rs   # HintDatabaseProvider -> Vec<Store>, new_layered
crates/fragcap-cli/src/paths.rs               # catalog/local path helpers + env; drop hint-db
crates/fragcap-cli/src/cli.rs                 # --catalog-db / --local-db; drop --hint-db
crates/fragcap-cli/src/commands/run.rs        # bootstrap both; accumulate to local; layered resolver
crates/fragcap-cli/src/commands/targets.rs    # maintainer commands target catalog.db
crates/fragcap-cli/src/doctor/*               # report both stores
crates/fragcap-cli/wix/main.wxs               # seed installed as catalog.db
.github/workflows/release.yml                 # build/stage/archive/checksum catalog.db (pinned)
docs/fragcap-specification.md                 # store sections (15.x, 24.5)
docs/glossary/platform-and-distribution.md    # catalog.db, local.db entries
docs/glossary/index.md                        # regenerated
site/content/docs/*.mdx                        # hint.db references reconciled
changelog.d/S050-*.md                          # added/changed + release.yml decisions
```

**Structure Decision**: No new crate. The provider change is the only
`fragcap-targets` edit; everything else is CLI, packaging, and docs.

## Phase sketch (for /speckit-tasks)

1. Provider layering in `fragcap-targets` (unit tested with two in-memory stores).
2. Paths + flags in `fragcap-cli` (unit tested).
3. Bootstrap both + accumulate-to-local + layered resolver in `run.rs` (tested).
4. Maintainer `targets` command + doctor report reconciled to the split.
5. WiX + release.yml rename (decisions fragment).
6. Specification store sections + glossary entries + index.
7. Changelog fragments (with `spec-impact`) and the release.yml decisions fragment.
8. Verify: build/test/clippy/fmt/xtask (spec, lint, docs) under the gnu-host
   toolchain; quickstart scenarios.

## Complexity Tracking

No violations; no entries.
