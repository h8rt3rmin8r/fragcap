# Implementation Plan: Anti-cheat detection and machine-scope presence

**Branch**: `068-anticheat-machine-scope` | **Date**: 2026-08-22 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/068-anticheat-machine-scope/spec.md`

## Summary

Issue #170 measured that `fragcap targets` never reports anti-cheat, even for
two titles that demonstrably ship Easy Anti-Cheat, for two independent
reasons: the in-tree signature rows miss the real bootstrapper artifacts, and
modern EAC installs machine-wide (a service and driver outside any game's
tree), invisible to a directory scan no matter how many rows are added. This
slice: (1) adds signature rows matching the measured bootstrapper artifacts;
(2) adds a pure classifier over Steam's already-parsed launch-entry
`arguments`/`description`/`executable` fields, a second, zero-new-I/O
evidence source, deliberately narrow (specific tokens, not substring
matching, to avoid the issue's own measured false-positive trap); (3) adds a
machine-wide anti-cheat presence check behind an injectable trait seam, with
one real Windows registry-key-existence implementation, rendered separately
from every target row so a machine-wide fact is never conflated with a
title-specific one. No schema change: every new finding rides the existing
`DetectionFinding`/evidence-JSON machinery.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (pinned toolchain per
`rust-toolchain.toml`)

**Primary Dependencies**: `fragcap-profile` (`Signature`, `DetectionFinding`,
`SignatureSet`), `fragcap-steam` (`SteamLaunchEntry`, appinfo reading,
already depends on `fragcap-profile` and `windows-sys` with
`Win32_System_Registry`), `fragcap-targets` (the new probe trait, no
`windows-sys` dependency added there), the `fragcap` facade (the Windows
registry adapter, `windows-sys` feature list gains `Win32_System_Registry`,
an additive feature on the already-resolved 0.36 pin per the S17/S10
precedent, no `Cargo.lock` delta)

**Storage**: No schema change. New signature rows use existing valid `kind`
values; launch-entry and machine-wide findings ride the existing per-target
`evidence` JSON column and a fresh-per-run computation, respectively.

**Testing**: `cargo test --workspace --locked`; unit tests in
`fragcap-profile` (signature fixture extension), `fragcap-steam` (the new
classifier, including the issue's own MCC negative example), `fragcap-targets`
(the probe trait via a fixture implementation), `fragcap` facade
(merge-at-discovery-time behavior), `fragcap-cli` (the rendered machine-scope
section)

**Target Platform**: Windows (the machine-wide probe is Windows-only,
following the existing `#[cfg(windows)]` adapter pattern already used for
`WindowsVolumeInventory`; the seam itself is platform-neutral and lives in
`fragcap-targets`)

**Project Type**: CLI (single Cargo workspace)

**Performance Goals**: N/A. The launch-entry classifier is a linear scan over
a title's own (small) launch-entry list, already held in memory during
appinfo indexing. The machine-wide probe is one registry-key-existence check,
run once per `fragcap targets` invocation, not per title.

**Constraints**: A launch-entry finding and a directory-scan finding for the
same product on the same title must combine into one finding (FR-005), reusing
the existing dedup-by-`(category, product)`-keep-strongest-fidelity rule
currently inlined inside `SignatureSet::detect`, extracted into a shared
function so the two call sites cannot drift (research.md decision). A
machine-wide finding must never reach a title's evidence array (FR-007).

**Scale/Scope**: One product (Easy Anti-Cheat) implemented end to end across
all three new evidence paths; the probe trait and classifier structure are
designed to add more products without a structural change, but only EAC is
backed by measured evidence in this slice (spec Assumptions).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 (passive observation, denylisted techniques)**: Held. Every new read
  is a directory listing, a file read, or a registry key read (already an
  established, denylist-compliant technique in this codebase for the Steam
  install path). No process handle, no service-control-manager handle (the
  issue's own text prefers the registry route over SCM for exactly this
  reason), no driver interaction.
- **Any process handle states its access rights explicitly**: N/A, no
  process handle is opened by this slice.
- **fragcap-core takes no platform-specific dependency**: Held. The new
  `windows-sys` feature addition lands in the `fragcap` facade, mirroring the
  existing `WindowsVolumeInventory` adapter; `fragcap-core` is untouched.
- **Every discard path has a named counter**: Addressed by FR-008: a probe
  that could not run renders nothing (an absent section), never a false
  "clean" claim; this is the same posture `ScanOutcome::coverage_warnings`
  already establishes for a directory scan, applied to a probe that has no
  install root to report against.
- **P-4 (conserved accounting)**: N/A in the strict discovery-account sense;
  no new candidate/considered/produced counter is introduced, since the
  machine-wide probe does not correspond to a discovery source. FR-008
  carries the equivalent honesty obligation for this probe's own scope.
- **P-9 (the instrument does not lie)**: This is the slice's central
  concern. FR-002, FR-004, and the MCC counter-example test all exist
  specifically to keep a finding from asserting more than its evidence
  supports; FR-007 keeps a machine-wide fact from being silently attributed
  to a title.
- **A new term gets a glossary entry**: "Machine-scope" (as opposed to
  title/install-root scope) is a new term this slice introduces to the
  detection vocabulary; a glossary entry is added in the same change
  (`docs/glossary`, checked by the `docs check` xtask gate).
- **Wrappers stay thin**: N/A, no wrapper script involved.
- **UTF-8 no BOM, LF, no em/en dashes**: Applies to all new source and docs.

No violations requiring the Complexity Tracking table.

## Project Structure

### Documentation (this feature)

```text
specs/068-anticheat-machine-scope/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks, not this command)
```

### Source Code (repository root)

```text
crates/fragcap-targets/
├── assets/signatures.json        # new EAC bootstrapper rows (filename, directory-shape)
├── tests/signatures.rs           # all_markers_tree() extended with the new artifacts
└── src/
    └── machine_probe.rs          # NEW: MachineAntiCheatProbe trait, MachineAntiCheatFinding,
                                   #      FixtureMachineAntiCheatProbe, KNOWN_MACHINE_PRODUCTS

crates/fragcap-profile/src/signature.rs
    # merge_finding() extracted from SignatureSet::detect's inline dedup, made pub(crate)
    # or pub, reused by the facade's discovery merge site

crates/fragcap-steam/src/
├── anti_cheat.rs                 # NEW: classify_launch_entries(&[SteamLaunchEntry]) -> Vec<DetectionFinding>
└── library.rs                    # appinfo_index tuple gains a third element (Vec<DetectionFinding>);
                                   # InstalledTitle gains `pub anti_cheat: Vec<DetectionFinding>`

crates/fragcap/
├── Cargo.toml                    # windows-sys gains the Win32_System_Registry feature
└── src/
    ├── discovery.rs              # SteamSource::discover merges title.anti_cheat into
    │                             # each candidate's evidence via merge_finding()
    └── machine_probe.rs          # NEW: WindowsMachineAntiCheatProbe (#[cfg(windows)])

crates/fragcap-cli/src/commands/targets.rs
    # hero listing runs the probe once, renders a "Machine:" section (only if non-empty)
    # separate from the per-target table

docs/glossary/                    # new entry: "machine-scope" (vs title/install-root scope)
```

**Structure Decision**: No new crate. The probe seam (trait + result type +
a small fixture implementation for tests) lives in `fragcap-targets`, which
already owns other injectable-adapter traits (`VolumeInventory`) and has no
platform dependency to protect. The one real (Windows) adapter lives in the
`fragcap` facade, the established home for platform adapters that compose
Steam and targets data (the `WindowsVolumeInventory`/`SteamSource`
precedent). The launch-entry classifier lives in `fragcap-steam`, which
already depends on `fragcap-profile` (the `DetectionFinding` vocabulary), so
it can produce findings directly with no new cross-crate edge.

## Complexity Tracking

Not applicable, no constitution violation.
