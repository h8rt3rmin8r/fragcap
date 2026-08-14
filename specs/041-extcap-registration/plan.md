# Implementation Plan: extcap registration

**Branch**: `041-extcap-registration` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/041-extcap-registration/spec.md`

## Summary

Add `fragcap extcap install` and `fragcap extcap uninstall` subcommands that copy
the running binary into (or remove it from) the per-user Wireshark extcap
directory the readiness check already probes. The analyzer protocol path is
untouched; the two new subcommands hang off the existing `extcap` command as an
optional subcommand, chosen precisely because `ExtcapArgs` is all-flags so the
`install`/`uninstall` barewords cannot collide with the protocol flags or with
the top-level `route_extcap` shim.

The Windows installer option is split to a dedicated follow-up slice (operator
direction 2026-08-14) so it gets a real WiX build and a multi-user install test.
This slice therefore touches no WiX and no pinned or release-adjacent artifact;
the machine-wide registration path an administrator needs is reachable now via
`--dir`, and the CLI reference documents it.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82.

**Primary Dependencies**: existing only. `std::fs` (copy/remove/create_dir_all),
`std::env::current_exe`, `crate::paths`. No new crate.

**Storage**: the filesystem (the extcap directory); read-only elsewhere.

**Testing**: `cargo test`. A new integration test drives register/uninstall over
a scratch directory via the `--dir` override (and/or the existing
`FRAGCAP_EXTCAP_DIR` env override), and asserts the doctor integration check
flips. The existing `cli_extcap.rs` already covers the four protocol invocations
in both forms; this slice adds an explicit parser-regression assertion.

**Target Platform**: Windows (the analyzer target); the command also runs on
other platforms against the platform extcap location.

**Project Type**: CLI (the `fragcap-cli` crate) plus the WiX installer source.

**Constraints**: FR-006 is load-bearing: the four analyzer protocol invocations
must keep working in both the bare top-level form (`route_extcap` injects the
`extcap` subcommand) and the explicit `extcap <flags>` form. The register target
name and location must match the readiness check exactly (R-6), single-sourced.

**Scale/Scope**: one CLI grammar addition, one command module of register logic,
one shared binary-name constant, one WiX component, one reference-doc entry, one
integration test, changelog fragments.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive observation / Licensing**: registration copies fragcap's own
  binary only; it never downloads, installs, or modifies npcap or Wireshark
  (FR-010). No denylisted technique, no process handle. PASS.
- **P-2 Core neutral**: all changes are in `fragcap-cli` and the WiX source;
  `fragcap-core` is untouched. PASS.
- **P-3 / P-4 / P-5**: capture and attribution untouched; no discard path; output
  compatibility unaffected. PASS.
- **P-6 Glossary first**: extcap already has a glossary entry; no new term is
  introduced. The CLI reference documents the new subcommand same-change (FR-009).
  PASS.
- **P-9 The instrument does not lie**: a failed registration reports an error and
  never claims success (FR-008). PASS.
- **Pinned artifacts**: none touched. The installer change (`wix/main.wxs`,
  release-adjacent) is split to a dedicated slice; nothing under
  `.github/workflows/**`, `release.toml`, `scripts/**`, or `wix/` is modified
  here.
- **Text hygiene**: UTF-8, LF, no em/en dashes, SPDX headers. PASS.

No violations; Complexity Tracking empty.

## Project Structure

### Documentation (this feature)

```text
specs/041-extcap-registration/
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- extcap-cli.md
|-- checklists/
|   |-- requirements.md
|   `-- registration.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/cli.rs          # ExtcapArgs gains `action: Option<ExtcapAction>`; new ExtcapAction + ExtcapInstallArgs
crates/fragcap-cli/src/commands/extcap.rs   # dispatch action first; install()/uninstall() register logic
crates/fragcap-cli/src/paths.rs        # shared EXTCAP_BINARY name constant (single-sources R-6)
crates/fragcap-cli/src/doctor/probe.rs # use the shared EXTCAP_BINARY constant instead of its private copy
crates/fragcap-cli/tests/cli_extcap.rs # install/uninstall integration tests + explicit parser-regression assertion
site/content/docs/reference/cli.mdx    # document `fragcap extcap install`/`uninstall` (per-user + machine-wide via --dir)
changelog.d/104-extcap-registration.added.md
# (Windows installer option: deferred to a dedicated slice; no WiX touched here)
```

**Structure Decision**: keep the analyzer protocol on `Command::Extcap`'s flags
and add register/uninstall as an optional subcommand under it. The register
behavior lives in one place (`commands/extcap.rs`); the deferred installer slice
will reuse the same command so the installer and the command cannot diverge.

## Complexity Tracking

No constitution violations; no entries.
