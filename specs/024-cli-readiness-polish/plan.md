# Implementation Plan: CLI readiness, help, and output-contract polish

**Branch**: `024-cli-readiness-polish` | **Date**: 2026-08-12 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/024-cli-readiness-polish/spec.md`

## Summary

Nine post-v0.2.0 defects in the `fragcap` command-line surface are corrected as
one slice: `doctor` is made honest about compiled-in capture/attribution
backends (and about npcap version and loopback severity), the release binary is
built with its capability features, the `--json` stream is extended to the
`profile` subcommands, the exit-code contract is made consistent, help text is
stripped of internal leakage, and live capture refuses cleanly without
elevation. All work is confined to `crates/fragcap-cli` plus one master-spec
update and one pinned-workflow change; no new capture or attribution capability
is added, and no new runtime crate dependency is introduced.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (toolchain pinned by
`rust-toolchain.toml`).

**Primary Dependencies**: `clap` (derive) for the command grammar and help text;
`windows-sys` 0.36 (already a `fragcap-cli` dependency) for the current-process
elevation token and for reading the npcap version; the existing in-crate
`emit`/`events` structured-event machinery; `serde_json` (dev-only) to parse
NDJSON in tests. No new runtime crate is added; the npcap version read may
require enabling an additional `windows-sys` feature (registry or version-info),
which is a feature addition on an existing dependency, not a new crate.

**Storage**: N/A.

**Testing**: `cargo test` - pure-function unit tests in `doctor/checks.rs`,
CLI/integration tests in `crates/fragcap-cli/tests`, and NDJSON assertions using
the dev-only `serde_json`. Windows-only paths tested behind `#[cfg(windows)]` or
via a platform-neutral predicate seam.

**Target Platform**: Windows for capture, elevation, and npcap probing; the
crate still compiles for a backend-less target (capability checks report
"absent" there).

**Project Type**: CLI (single crate, `fragcap-cli`, facaded by `fragcap`).

**Performance Goals**: N/A - no capture hot-path code changes.

**Constraints**: P-1 (no handle against any target process; elevation reads only
the current process token), P-2 (no Windows leakage into `fragcap-core`; all new
platform code lives in `fragcap-cli` behind `#[cfg(windows)]`/`#[cfg(feature)]`),
and the pinned-artifact rule for `.github/workflows/release.yml` (dated decision
fragment required).

**Scale/Scope**: 9 issues, one crate, one spec section pair (17 and 26.3), one
workflow file, one decision fragment.

## Constitution Check

*GATE: evaluated before Phase 0 and re-checked after Phase 1 design.*

| Principle | Verdict | Basis |
| --- | --- | --- |
| P-1 Passive observation | PASS | The elevation gate reuses the existing `is_elevated()` which reads only the current-process pseudo-handle token (`probe.rs:163-189`); it opens no handle against any target and requests no memory rights. The npcap version read is a registry/version-info read, not a process handle. `cargo xtask lint` continues to assert no `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`. |
| P-2 Core stays neutral | PASS | Every new line lives in `fragcap-cli`; capability presence is expressed via `#[cfg(feature = ...)]` and elevation/version reads via `#[cfg(windows)]`. `fragcap-core` is untouched, so the backend-less CI build still passes. |
| P-3 Capture/attribution separate | PASS (N/A) | No `PacketSource`/`FlowAttributor` code is touched; the elevation gate sits in CLI assembly ahead of both. |
| P-4 No silent loss | PASS | No new packet discard path. The elevation refusal happens before the driver is opened, so no packet is ever observed-then-dropped; nothing to count. |
| P-5 Compatibility | PASS (N/A) | No `.fcapng`/output-format change. |
| P-6 Glossary first | PASS w/ action | Any newly surfaced term (e.g. "capability backend" as a reported readiness fact) gets a section 4.3 glossary entry in this slice if not already present; verified in Phase 1. |
| P-7 Wrappers stay thin | PASS (advances) | Moving the elevation check into the binary is exactly the capability-in-Rust direction P-7 prescribes; it removes the wrapper's elevation role rather than growing a wrapper. |
| P-8 House standards | PASS | The decision fragment is house-standard Markdown; the `release.yml` edit stays within existing YAML conventions; `cargo xtask ci` enforces the rest. |
| P-9 Instrument does not lie | PASS (advances) | This slice exists to end a false "ready" verdict over a binary that cannot capture - a direct P-9 correction. |
| Pinned artifacts | PASS w/ action | `.github/workflows/release.yml` changes under a dated `changelog.d/<key>.decisions.md` fragment (FR-021). |

No gate is violated; the Complexity Tracking table stays empty.

## Project Structure

### Documentation (this feature)

```text
specs/024-cli-readiness-polish/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI behavior contracts)
│   ├── doctor.md
│   ├── profile-json.md
│   ├── exit-codes.md
│   └── help-and-elevation.md
├── checklists/
│   └── requirements.md  # Spec-quality checklist (from /speckit-specify)
└── tasks.md             # /speckit-tasks output (not created here)
```

### Source Code (repository root)

```text
crates/fragcap-cli/
├── src/
│   ├── cli.rs                 # help text: roles note (#66), slice IDs / --launch (#67)
│   ├── args.rs                # RingWindow slice-ID doc (#67)
│   ├── exit.rs                # From<ResolveError> alignment; contract doc (#68)
│   ├── emit.rs                # structured emitter reused for profile (#65)
│   ├── events.rs              # add diagnostic/summary + profile-list events (#65)
│   ├── lib.rs                 # thread --json into profile dispatch (#65)
│   ├── assemble.rs            # elevation refusal gate ahead of the driver (#56)
│   ├── commands/
│   │   ├── profile.rs         # validate path dedup (#70.1); --json (#65); exit (#68)
│   │   ├── doctor.rs          # render/exit wiring for new checks
│   │   └── stub.rs            # replay "not yet implemented" copy (#67)
│   └── doctor/
│       ├── mod.rs             # Inputs/Status/Report: new capability fields
│       ├── checks.rs          # live/socket-table checks; loopback warn (#63,#69)
│       └── probe.rs           # cfg(feature) availability; real npcap version (#63,#70.2)
└── tests/                     # CLI integration + NDJSON assertions

crates/fragcap/Cargo.toml       # (reference only) feature declarations
docs/fragcap-specification.md   # sections 17 + 26.3 updates (FR-020)
.github/workflows/release.yml   # capability feature build + npcap SDK step (#62)
changelog.d/
├── S024-cli-readiness-polish.md          # slice changelog fragment
└── release-features.decisions.md         # dated pinned-artifact decision (#62)
```

**Structure Decision**: Single-crate change. All runtime code lands in
`crates/fragcap-cli`; the only files touched outside it are the master
specification (governance requirement), the release workflow (pinned artifact),
and the two changelog fragments. This matches the crate map produced during
exploration and keeps the slice inside one compilation unit for `cargo xtask
deps`.

## Complexity Tracking

No constitution violations; no entries.
