# Implementation Plan: Launch-and-observe promotion

**Branch**: `059-launch-and-observe` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/059-launch-and-observe/spec.md`

## Summary

Let a stored target whose launch chain is unresolved (a `no`/`unsure` authoring
answer) be captured in an observe mode, and promote it once a run observes the real
socket-holding process. Four moving parts: (1) a fourth branch on the shared
`commands/target_resolve.rs` seam that synthesizes a valid two-stage observe-mode
profile from the stored observed executable; (2) an ordered per-owner tally on
`fragcap-core` `CaptureStats`, incremented at the attribution site and folded in
`absorb`, kept out of every golden-pinned surface; (3) a small `CaptureOutcome {
exit, observed_holder }` returned from `orchestrator::capture` so `capture.rs::run`
learns the dominant image; (4) the write-back in `capture.rs::run` via the existing
`Store::promote_target_launch`. Observe-nothing leaves the target untouched (P-9).
No new direct-exe launcher; live launch stays on the Steam path. No new dependency.

## Technical Context

**Language/Version**: Rust (workspace MSRV 1.82; local build/test on the GNU
toolchain `cargo +1.96.0-x86_64-pc-windows-gnu ...` since there is no MSVC linker
here; CI runs the MSVC `--all-features` gate).

**Primary Dependencies**: none added. Reuses `fragcap-core` `Attribution`/
`CaptureStats`, `fragcap-targets` `authoring`/`Store`, `fragcap::profile` types, and
the existing `target_resolve`/`orchestrator`/`assemble` machinery in `fragcap-cli`.

**Storage**: SQLite `local.db` via the existing `Store` (no schema change;
`promote_target_launch` already exists from S055).

**Testing**: `cargo test -p fragcap-core` (stats.rs), `cargo test -p fragcap-cli`
(cli_capture.rs, cli_extcap.rs), `cargo test -p fragcap` (pipeline tally), `cargo
test -p fragcap-targets` (authoring accessor), `cargo xtask ci`, `cargo xtask spec`,
glossary linter, docs build. GNU locally; MSVC `--all-features` in CI.

**Target Platform**: Windows (the tool); the docs site is static-exported.

**Project Type**: CLI (Rust workspace) with a co-located docs site.

**Performance Goals**: N/A. The tally is one `BTreeMap` increment per attributed
packet, off the drop path.

**Constraints**: fragcap-core gains one additive field and one increment; no drop
counter changes (P-4). The tally never reaches a writer, the completion summary, or
a golden. No new direct-exe launcher (hard boundary). UTF-8 no BOM, LF, no em/en
dashes (including comments). No `Cargo.lock` delta.

**Scale/Scope**: one field + one increment + one fold on `CaptureStats`; a
`dominant_holder` helper + `CaptureOutcome` in the orchestrator (both call sites
updated, extcap ignores the holder); one observe-mode branch + profile synthesizer
in `target_resolve`; a `ResolvedTarget`/`Promotion` carrier so `run` can write back;
a small `observed_executable` accessor in `fragcap-targets` `authoring`; new tests;
three glossary entries; spec 17.2/17.7 reconcile.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 (technique denylist / observe-only)**: The tally reads the already-attributed
  process image; it opens no handle, injects nothing, reads no process memory. The
  observe-mode profile matches the same socket-table/ETW attribution every capture
  uses. No new launch surface: live launch stays on the Steam protocol handler. PASS.
- **P-4 (every discard counted)**: The tally is additive and touches no drop counter;
  the conservation identity is unchanged (it is not a discard path). PASS.
- **P-6 (new term -> glossary same change)**: Three new terms ("launch-and-observe",
  "observed socket-holder", "capture-time promotion") get glossary entries in this
  change. PASS (verified in tasks).
- **P-9 (no fabrication / honest reporting)**: Promotion happens only on an observed
  dominant image; observe-nothing writes nothing. The observe-mode profile is not a
  wildcard (it names the observed executable and a descends-from client). The Tier 2
  `steam://run` launch is labeled as not exercised in CI. PASS.
- **P-11 (spec describes what shipped)**: Spec sections 17.2/17.7 are reconciled and
  `cargo xtask spec` keeps the Applies-To lockstep. PASS.
- **Architecture (fragcap-core takes no platform dep; deps flow concrete->abstract)**:
  The `CaptureStats` field is a `BTreeMap<Arc<str>, u64>`, std-only; no platform
  dependency enters core. The write-back and observe branch live in `fragcap-cli`,
  the top of the graph. `cargo xtask deps` unaffected. PASS.
- **Compatibility / wrappers thin**: extcap wire contract unchanged; it resolves the
  observe-mode profile and streams, ignoring the holder. PASS.
- **Pinned artifacts**: none touched (no workflows/toolchain/release.toml/scripts).
  PASS.
- **Encoding / no dashes**: enforced across edited files. PASS (verified in tasks).

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/059-launch-and-observe/
├── plan.md              # This file
├── spec.md              # Feature spec
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   ├── observe-mode-profile.md   # The synthesized two-stage profile shape
│   ├── dominant-holder.md        # Tally + dominant-image selection contract
│   └── capture-time-promotion.md # Resolve -> observe -> promote/leave contract
└── checklists/
    ├── requirements.md
    └── launch-observe.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── stats.rs                     # CaptureStats: + holder_tally BTreeMap; absorb folds it; sample()/set_nth untouched-by-drop-tests
└── pipeline/mod.rs              # increment holder_tally at the Resolved arm (where packets_attributed increments)

crates/fragcap-targets/src/
└── authoring.rs                 # + observed_executable(&TargetEntry) -> Option<&str> accessor (reads observed_exe/executable)

crates/fragcap-cli/src/
├── commands/
│   ├── target_resolve.rs        # 4th branch: launch_is_unresolved -> synthesize_observe_profile; resolve_stored returns ResolvedTarget { profile, promotion }
│   ├── capture.rs               # run(): take ResolvedTarget; after capture, promote when observed_holder is Some
│   └── extcap.rs                # capture(): take ResolvedTarget.profile; ignore promotion + observed_holder
└── orchestrator.rs              # capture() -> CaptureOutcome { exit, observed_holder }; dominant_holder(&CaptureStats); both drivers compute it

crates/fragcap-cli/tests/
├── cli_capture.rs               # observe-mode: register unsure target, capture over a launcher+child fixture, assert promotion; and no-observe leaves it
├── data/*.procscript            # NEW fixture: launcher spawns child socket holder
crates/fragcap/tests/
└── pipeline.rs (or new)         # the tally rides in PipelineReport.stats over a fixture

docs/fragcap-specification.md    # sections 17.2/17.7 reconcile + glossary terms; `cargo xtask spec`
changelog.d/S059-*.md            # changelog fragment
```

**Structure Decision**: The observe branch and the promotion carrier live in
`target_resolve.rs` (the seam S058 extracted for exactly this), so `capture` and
`extcap` share one resolution and only `capture` writes back. The tally is the
minimal additive change to core that makes the dominant holder a property of the
run's own stats rather than something a caller reconstructs.

## Complexity Tracking

No constitution violations; no entries.
