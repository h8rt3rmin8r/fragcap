# Implementation Plan: Consistent User Storage

**Slice**: S192 | **Branch**: `codex/s192-consistent-user-storage` | **Date**: 2026-11-04 | **Spec**: [spec.md](spec.md)

## Summary

Move Windows default persistent state to the current user's `.fragcap` directory outside AppData virtualization. Resolve one account and root for every consumer, preserve legacy AppData state through a non-destructive SQLite-consistent transition, retain old path-bound sessions for discovery and recovery, and report actual file destinations.

## Technical Context

**Language/Version**: Rust 1.88 workspace minimum, PowerShell 7 package harness, Windows APIs through existing `windows-sys`.

**Primary Dependencies**: Existing `fragcap-cli`, `fragcap-targets` SQLite store, native Deep Capture session registry and Doctor; no network dependency.

**Storage**: SQLite `catalog.db` and `local.db`, profiles, native session files and registry, versioned transition metadata.

**Testing**: Focused Rust unit and CLI integration cases, controlled Windows package regression, full applicable workspace checks, exact-head hosted CI.

**Target Platform**: Windows MSI and portable executable, ordinary and supported packaged-host launch contexts.

**Constraints**: Preserve explicit overrides and exact cleanup authority; no private user-data edits during development; no visible console child; no silent account substitution or data replacement.

## Constitution Check

- P-1: No target instrumentation or additional target process rights. The storage resolver operates only on current-user paths.
- P-2/P-3: Windows-specific identity and handle checks remain behind platform boundaries. Core and facade architecture remain unchanged except for a narrow existing-store snapshot API if needed.
- P-4/P-9: Path selection, destination verification, transition, and loss or refusal outcomes remain explicit. An absent destination is not claimed verified.
- P-8: Add synthetic controlled regression, documentation, and changelog. Installed-product field observation remains after release.
- Installed-product governance: Controlled package verification uses the runner account's actual default path and independent opened-file readback, not a relocated synthetic APPDATA that masquerades as default.

## Design and Order

1. Implement shared read-only path functions in `paths.rs` from the current Windows profile identity and exact override precedence. Keep non-Windows behavior stable. Centralize every default path through these functions.
2. Add a bounded exact legacy-root inventory and a versioned transition record. Snapshot one unambiguous database with SQLite consistency into a staged file, validate, and atomically publish; preserve source bytes. Conflicts and active work remain visible.
3. Retain legacy session bundles at original paths when path-bound authority cannot be moved. Make Doctor recovery and history inventory aware of the durable legacy index without broadening deletion.
4. Add selected/effective path observation, using opened handles for existing files and directories, to Doctor and session terminal output. Update structured schema with compatibility fields rather than changing existing field meanings.
5. Align fresh-start, retention, and MSI adapter with the same root inventory. Preserve custom-path and all-users boundaries.
6. Add focused tests, controlled package readback, specification and user documentation, and changelog. Verify formatting, lint, tests, package behavior, encoding, and exact-head CI before owner handoff.

## Project Structure

```text
crates/fragcap-cli/src/paths.rs
crates/fragcap-cli/src/storage_transition.rs
crates/fragcap-cli/src/commands/target_resolve.rs
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/src/commands/steam.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/src/commands/fresh_start.rs
crates/fragcap-cli/src/doctor/{probe,checks,residue,fix}.rs
crates/fragcap-cli/tests/{cli_bootstrap,cli_doctor,cli_fresh_start}.rs
crates/fragcap-targets/src/
scripts/Test-PackageCertification.ps1
docs/fragcap-specification.md
docs/plans/README.md
```

**Structure Decision**: Keep the Windows storage and CLI reporting contract in `fragcap-cli`, reuse the existing target-store authority for SQLite, and avoid a new broker or crate.

## Decisions

- 2026-11-04: The owner selected `%USERPROFILE%\.fragcap` over keeping an AppData default with a separate broker. Microsoft documents package-private new AppData writes and ordinary writes outside AppData; this root is one supported cross-context location for the same user. Separate `roaming` and `local` children retain the existing data-category split and keep cleanup roots non-overlapping, but their names do not assert automatic Windows roaming.
- 2026-11-04: Do not move old session bundles automatically while path-bound recovery records or recipient access could change. Index exact legacy roots and preserve them until a separately proven transfer is possible.
- 2026-11-04: A conflict or unverifiable source refuses automatic migration; no candidate is guessed from timestamps or a host-specific view.
