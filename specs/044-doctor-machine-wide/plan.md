# Implementation Plan: doctor recognizes machine-wide extcap registration

**Branch**: `044-doctor-machine-wide` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/044-doctor-machine-wide/spec.md`

## Summary

Widen `fragcap doctor`'s `analyzer extcap` check to recognize a machine-wide
registration (the fragcap binary in Wireshark's system extcap directory), not just
the per-user one, and name which scope registered it. Add
`paths::system_extcap_dir()` (env-overridable), extend the probe to read it,
carry two new fields on `Inputs`, rewrite the pure `integration()` classifier as a
four-arm match, and regenerate the doctor goldens. All in `fragcap-cli`; no core,
CLI, or MSI change.

## Technical Context

**Language/Version**: Rust (workspace MSRV 1.82). Change is in `fragcap-cli`.

**Primary Dependencies**: none added. Env-based directory resolution mirrors
`paths::extcap_dir()`.

**Testing**: `cargo test -p fragcap-cli` (unit classifier tests in `checks.rs`,
the golden-driven `cli_doctor.rs`), full `cargo xtask ci`.

**Target Platform**: Windows for the real machine-wide path; the classifier is
platform-neutral and tested via override env vars.

**Project Type**: CLI crate within the Rust workspace.

**Constraints**: `fragcap-core` takes no platform dependency (P-2); the probe is
read-only (P-1); doctor stays truthful (P-9); no em/en dashes including golden
text; the default no-feature build and the Linux neutrality build compile.

**Scale/Scope**: one paths helper, one probe function widened, two `Inputs`
fields, one classifier rewrite, four `Inputs` construction sites updated, goldens
regenerated, one changelog fragment, and a possible one-line docs wording update.

## Constitution Check

- **P-1 Passive Observation**: The probe only reads a directory to see whether a
  file exists; it installs and copies nothing. PASS.
- **P-2 Core Platform-Neutral**: Change is entirely in `fragcap-cli`;
  `fragcap-core` untouched. PASS.
- **P-6 Glossary First**: No new term (extcap, dependency model already defined).
  PASS.
- **P-8 House Standards**: UTF-8, LF, no dashes in code, goldens, and docs. PASS.
- **P-9 The Instrument Does Not Lie**: This is the correctness fix: doctor stops
  reporting a machine-wide registration as absent. PASS.

No violations. Complexity Tracking not required.

## Project Structure

### Source (paths touched)

```text
crates/fragcap-cli/src/paths.rs              # system_extcap_dir() + SYSTEM_EXTCAP_DIR_ENV
crates/fragcap-cli/src/doctor/probe.rs       # extcap_status() reads both dirs; gather()/gather_windows() set the new fields
crates/fragcap-cli/src/doctor/mod.rs         # Inputs: extcap_system_dir, extcap_system_installed
crates/fragcap-cli/src/doctor/checks.rs      # integration() four-arm match; ready_inputs() fixture; new unit tests
crates/fragcap-cli/tests/cli_doctor.rs       # ready() fixture: new fields
crates/fragcap-cli/tests/goldens/doctor-ready.{txt,ndjson}   # regenerated
site/content/docs/getting-started.mdx        # doctor sample detail wording, only if it changes
changelog.d/044-doctor-machine-wide.*.md     # changelog fragment
```

**Structure Decision**: Keep the per-user fields (`extcap_installed`,
`extcap_dir`) as the per-user scope and add parallel `extcap_system_installed`,
`extcap_system_dir`. This keeps the diff small, preserves the pure-classifier
design (all four combinations tested over `Inputs` with no I/O), and confines the
platform code to the thin probe. `integration()` becomes a match over
(per-user, system) with `ok` on either and a scope-naming detail; the neither
arm is the existing optional `Warn` verbatim.

## Complexity Tracking

No constitution violations; no entries.
