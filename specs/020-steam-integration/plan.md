# Implementation Plan: Steam integration and managed launch

**Branch**: `020-steam-integration` | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/020-steam-integration/spec.md` (roadmap
slice S17, specification section 16; depends on S05, S12; gated by Q-4, resolved).

## Summary

Fill in the `fragcap-steam` crate (today a skeleton) and make its two command surfaces
real. Three capabilities from specification section 16:

1. **Library discovery** (16.2): locate the Steam installation through its Windows
   registry entry, read `libraryfolders.vdf`, and read every `appmanifest_*.acf` across
   every library, yielding the set of installed titles with app_id and install path. The
   Valve key-value (VDF) text format is parsed by a small hand-rolled module in the crate.
2. **Profile scaffolding** (16.3): `fragcap steam profile <app_id>` scans an installed
   title's install directory, classifies executables heuristically (launcher-token images
   are launcher stages; the largest non-launcher image is the client), and emits a profile
   skeleton to standard output. The skeleton is built as TOML text and then parsed back
   through `fragcap_profile::Profile::parse` before it is printed, so a scaffold that would
   fail section 15.4 validation is caught in-process rather than shipped.
3. **Managed launch** (16.4): `fragcap run --profile <ref> --launch` starts the title
   through Steam's protocol handler (`steam://run/<app_id>`) via `ShellExecuteW`, issued
   *after* the session reaches `Watching` and the sinks/capture handle are open, which is
   what removes the acquisition race. Absent `game.platform`/`game.app_id`, `--launch` is a
   named configuration error raised in run assembly before capture starts.

The load-bearing structural facts: the crate carries no capture and no attribution logic
(P-3, G-4); `fragcap-core` gains no notion of Steam (P-2). Discovery and launch touch the
OS only through the already-resolved `windows-sys` 0.36 binding, adding zero packages.
Everything pure - the VDF parser, the scaffolding classifier, the launch-URL and
launch-config decision - is portable and unit-tested on the CI host whatever its OS; only
the registry read and `ShellExecuteW` are `#[cfg(windows)]`, and the live launch itself is
tier-2/manual and never asserted as run in CI.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (pinned toolchain per
`rust-toolchain.toml`).

**Primary Dependencies**: `fragcap-profile` (schema, `Profile::parse` validation) and the
workspace-shared `windows-sys` 0.36, adding two additive features - `Win32_System_Registry`
(the Steam install-path read) and `Win32_UI_Shell` (`ShellExecuteW` for the protocol
handler). **No new runtime dependency, no new package in `Cargo.lock`.** This deviates from
the specification crate table, which names `winreg`; see Decision D1.

**Storage**: reads Steam's local metadata files (`libraryfolders.vdf`,
`appmanifest_*.acf`) and scans install directories. Writes nothing to disk: the scaffold
goes to stdout.

**Testing**: `cargo test`, tier 1 (offline, no Steam installed, no capture driver, no
elevation, no game). VDF parsing, the scaffolding classifier, scaffold-validates-through-
`Profile::parse`, launch-config validation, and launch-URL construction are unit-tested.
Library discovery is tested against a fixture Steam root laid out in a tempdir. The
physical Steam launch is tier-2/manual (Decision D5).

**Target Platform**: the crate compiles on every target. Its public API is
cfg-independent, so the facade and CLI build on the neutral non-Windows target (P-2, FR-014);
the Windows-only registry read and `ShellExecuteW` live behind `#[cfg(windows)]`, with a
non-Windows arm returning a structured "only supported on Windows" error.

**Project Type**: Rust workspace (library crates plus a CLI), single repository.

**Performance Goals**: discovery is a bounded walk of a handful of manifest files and one
directory scan per scaffold; no hot path. Not performance-sensitive.

**Constraints**: opens no process handle (P-1; the `OpenProcess`/`ReadProcessMemory`/
`WriteProcessMemory` lint stays green - section 16.5, which would need one, is deferred);
scaffolded profiles must pass section 15.4 unedited (P-9 / FR-008); nothing bundles,
downloads, or installs Steam (FR-015, the npcap posture generalized); core stays neutral
(P-2).

**Scale/Scope**: one new crate body (~5 modules), two CLI wiring points, one fixture Steam
tree for tests. No change to the pipeline, the session driver's shape, or the sinks.

## Constitution Check

*GATE: evaluated before Phase 0 and re-checked after design. All pass.*

- **P-1 Passive Observation (NON-NEGOTIABLE)**: PASS. No packet interception, injection,
  hooking, or process handle. Managed launch *starts* a title through the OS shell; it does
  not reach into any process. Section 16.5 (which would require a memory-read handle) is
  explicitly deferred - Decision D6.
- **P-2 Core Stays Platform-Neutral**: PASS. All new code lands in `fragcap-steam`,
  `fragcap` (facade re-export), and `fragcap-cli`. `fragcap-core` is untouched; the neutral
  build still compiles because the crate's public API is not cfg-gated.
- **P-3 Capture And Attribution Stay Separate**: PASS. The crate contains neither. It reads
  metadata and issues a launch; it never touches the capture or attribution paths.
- **P-4 No Silent Loss**: PASS (n/a to packet loss). The discovery analogue - a malformed
  manifest - is reported and skipped, never silently dropped (FR-004).
- **P-5 Compatibility Outranks Richness**: PASS. Scaffolded profiles are ordinary profiles
  an unmodified `Profile::parse` accepts; the scaffold changes no output format.
- **P-6 Glossary First**: PASS with action. New terms (`managed launch`, `library
  discovery`, `profile scaffolding`, `VDF`) get `docs/glossary.md` entries in this change.
- **P-7 Wrappers Stay Thin**: PASS. No output parsing; the CLI calls typed crate APIs.
- **P-8 House Standards Apply**: PASS. UTF-8 no BOM, LF, no em/en dashes; `cargo xtask ci`
  is the gate.
- **P-9 The Instrument Does Not Lie (NON-NEGOTIABLE)**: PASS. The scaffold proves its own
  validity by round-tripping through `Profile::parse` before emission; the live launch is
  labeled tier-2 and is not asserted as run in CI (Decision D5); the heuristic scaffold
  carries a header comment stating it is a guess.

**Dependency direction (`cargo xtask deps`)**: `fragcap-core`'s allowlist is unchanged;
`fragcap-steam` is not core and may take `windows-sys`. `fragcap-steam` depends only on
`fragcap-profile` (concrete toward abstract holds). PASS.

## Decision Log

- **D1 - Registry and protocol handler via `windows-sys`, not `winreg`** (architecture-of-
  record deviation; dated changelog decision). The spec crate table names `winreg`. Reuse
  the workspace's already-resolved `windows-sys` 0.36 with two additive features instead.
  Rationale mirrors the recorded S10 decision: `winreg` would add a second Windows-binding
  package tree to `Cargo.lock` for declarations `windows-sys` already carries; additive
  features on the existing 0.36 pin add no package and change no resolved version, and
  `fragcap-attr` already writes `windows-sys` FFI for the socket table and ETW, so the
  pattern and the `unsafe` review surface already exist. Alternative (`winreg`): more
  ergonomic, but a second binding tree the project has twice chosen to avoid.
- **D2 - Platform gating by `#[cfg(windows)]` internals, cfg-independent public API.** The
  crate builds everywhere (FR-014, P-2); non-Windows discovery/launch return a structured
  "only supported on Windows" error. Alternative (whole crate `#[cfg(windows)]`): would
  force every dependent to cfg-gate its use and break the neutral facade build.
- **D3 - Hand-rolled VDF parser.** The format is small and stable (spec 16.2 says it is not
  worth a dependency). Parse the nested-quoted-block subset; malformed input yields a
  positioned error and the caller reports-and-skips (FR-004). Alternative (a VDF crate):
  a dependency for a format smaller than its own parser.
- **D4 - Scaffold proves validity by round-trip.** Build TOML text, parse it through
  `fragcap_profile::Profile::parse`, emit only if it validates. Makes FR-008/SC-001 true by
  construction rather than by a separate assertion. Emit to stdout (clarification): a
  scaffold is a reviewed starting point, so file placement is the operator's.
- **D5 - Managed launch URL `steam://run/<app_id>`** (mutable plan-level detail; alternative
  `steam://rungameid/<app_id>` noted). Issued via `ShellExecuteW` after `session.attach()`
  ([session.rs:221](../../crates/fragcap/src/session.rs)) reaches `Watching` and the sinks
  are open. The live launch is tier-2/manual; CI asserts the *decision* (URL, ordering,
  refusals), never the syscall.
- **D6 - Section 16.5 environment inheritance deferred.** It requires a process handle with
  memory-read rights, which P-1's denylist and the `OpenProcess` lint forbid. It is a
  corroborating signal only; section 10 ancestry already attributes reliably, so deferral
  costs no capability. Recorded, not implemented.
- **D7 - Stage rules are `exe` image-name predicates, never inferred `descends_from`.**
  Runtime topology (the Div2 three-processes-one-image case) is invisible to a static scan;
  the heuristic header comment and the existing section 15.4 runtime warning cover it. Where
  two proposed stages would share a basename, add a `path_contains` predicate so the output
  passes the ambiguous-image-match check.

## Project Structure

### Documentation (this feature)

```text
specs/020-steam-integration/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI command + crate API contracts)
├── checklists/          # requirements.md, compliance.md
└── tasks.md             # /speckit-tasks output (next phase)
```

### Source code (repository root)

```text
crates/fragcap-steam/
├── Cargo.toml           # + windows-sys (Win32_System_Registry, Win32_UI_Shell), cfg(windows)
├── src/
│   ├── lib.rs           # public API: discover(), scaffold(), launch(); error type
│   ├── vdf.rs           # hand-rolled Valve key-value parser (portable, unit-tested)
│   ├── library.rs       # registry -> libraryfolders.vdf -> appmanifest_*.acf; InstalledTitle
│   ├── scaffold.rs      # executable scan + heuristic classifier + TOML emit + self-validate
│   └── launch.rs        # launch-URL construction (portable) + ShellExecuteW (cfg windows)
└── tests/
    └── discovery.rs     # fixture Steam tree in a tempdir

crates/fragcap/           # facade: re-export fragcap-steam public API (already a dependency)

crates/fragcap-cli/
├── src/cli.rs           # Steam(StubArgs) -> Steam(SteamArgs { profile <app_id> })
├── src/lib.rs           # dispatch steam::run instead of commands::stub::run(Stub::Steam)
├── src/assemble.rs      # replace the --launch "not yet supported" refusal with real
│                        # platform/app_id validation reading the loaded profile
└── src/commands/steam.rs# new: profile subcommand -> fragcap-steam scaffold, to stdout
```

**Neutral-build note**: `fragcap-steam`'s `Cargo.toml` scopes `windows-sys` under
`[target.'cfg(windows)'.dependencies]`, so the neutral non-Windows build pulls no Windows
binding and the pure modules (`vdf`, classifier, URL construction) still compile and test.

## Phase 0 - Research

See [research.md](research.md). All Technical Context items are resolved (no NEEDS
CLARIFICATION remains); the autopilot decision log above supplied the calls research would
otherwise open.

## Phase 1 - Design & Contracts

- [data-model.md](data-model.md): `InstalledTitle`, `SteamLibrary`, `ExecutableImage`,
  `StageProposal`, `ScaffoldedProfile`, `LaunchRequest`, `SteamError`.
- [contracts/](contracts/): the `fragcap steam profile` and `fragcap run --launch` command
  contracts, and the `fragcap-steam` crate API contract.
- [quickstart.md](quickstart.md): the offline validation scenarios that prove the slice.

## Complexity Tracking

No constitution gate is violated, so no complexity justification is required. The one
deviation (D1, `windows-sys` over `winreg`) reduces complexity (zero new packages) rather
than adding it, and is recorded as a dated changelog decision per the pinned-artifact and
dependency conventions.
