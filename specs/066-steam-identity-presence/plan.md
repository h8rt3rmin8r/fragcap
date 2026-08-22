# Implementation Plan: Steam install-path resolution, target presence, and multi-name identity

**Branch**: `066-steam-identity-presence` | **Date**: 2026-08-21 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `specs/066-steam-identity-presence/spec.md`

**Closes**: #166, #167, #173

## Summary

Three fixes to the same Steam-discovery-to-target-store surface, sequenced
together because each touches the same `TargetEntry`/discovery/registration code
path and the same rendering function:

1. Resolve a Steam title's install directory by its actual app type (read once
   from `appcache/appinfo.vdf`, already parsed for launch entries) instead of
   unconditionally assuming `steamapps/common/`; a `Music`-typed app resolves
   under `steamapps/music/` instead and is excluded from registration entirely,
   counted under the existing `considered_not_a_game` discovery outcome.
2. Derive, at listing time, whether a registered target's `install_root` still
   exists on disk, and render a recorded-but-missing row with a short note in
   the existing warning color, without ever mutating the registration.
3. Store a Steam title's raw installdir and observed launch executable as two
   new verbatim, optional fields alongside its display name, extend selector
   resolution to match on all three, and surface a genuinely divergent pair of
   names in `targets show`; expand `&` to `and` in handle derivation as part of
   the same handle-fidelity work.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82, pinned toolchain in
`rust-toolchain.toml`

**Primary Dependencies**: none added. The appinfo `common/type` and
`config/launch` reads reuse the existing hand-rolled binary VDF parser
(`fragcap-steam::appinfo`); the store already uses `rusqlite`.

**Storage**: `local.db` (and, structurally, `catalog.db`, which leaves the new
columns empty like every other `targets`-table column) through
`fragcap-targets::store::Store`. Schema version moves 7 to 8 for two additive
nullable columns.

**Testing**: `cargo test --workspace --locked`, plus `cargo xtask ci`. Every new
behavior is testable with no game, no capture driver, and no live Steam
install: fixture Steam trees (manifest plus a hand-built `appinfo.vdf` byte
sequence, reusing the existing appinfo test-fixture builders), an in-memory
`Store`, and CLI golden fixtures.

**Target Platform**: Windows for the shipped tool; every test in this slice is
platform-neutral (the Steam walk and the target store are portable by
construction, per their own module docs) and runs on any target the workspace
builds for.

**Project Type**: Rust workspace, eight crates, CLI plus libraries.

**Performance Goals**: one appinfo read per `discover_in` call (already paid for
launch entries; no new file read), one `Path::exists()` per registered row per
listing (bounded by the operator's own registration count, already the same
order of magnitude as the existing per-row detection-scan and readiness
derivations).

**Constraints**: passive observation only (P-1); no dependency added; no silent
loss (P-4) across the new Music-type exclusion path; existing CLI goldens for
unaffected rows stay byte-identical (FR-009); the 80-column terminal budget
`render_table` already documents is untouched (the note lands in the
free-running SENSITIVITIES column).

**Scale/Scope**: three source issues, one schema migration, one CLI rendering
change, one selector-resolution extension, one handle-derivation rule change.

## Constitution Check

*GATE: passed before Phase 0, re-checked after Phase 1 design.*

| Principle | Bearing on this slice | Verdict |
| --- | --- | --- |
| **P-1 Passive Observation** | The appinfo read is a file the operator's own Steam client already wrote; the presence check is a `Path::exists()` call. No process handle, no launch, no network. | Pass |
| **P-2 Core Stays Platform-Neutral** | Nothing lands in `fragcap-core`. The appinfo type/launch reads live in `fragcap-steam`; the schema, selector, and rendering changes live in `fragcap-targets` and `fragcap-cli`. | Pass |
| **P-3 Capture And Attribution Stay Separate** | Neither is touched; `launch_entries` (the capture-chain field) is deliberately left alone (R-7) so this slice cannot leak into capture behavior. | Pass |
| **P-4 No Silent Loss** | The Music-type exclusion is a new discard path and must be counted (FR-004, FR-018); it reuses the existing `considered_not_a_game` outcome rather than adding one, so `DiscoveryAccount::is_conserved` needs no change and the existing conservation test still exercises it. | Pass, load-bearing |
| **P-5 Compatibility Outranks Richness** | `targets export`'s two new keys are optional and additive, following the `detection_scan` precedent; an existing consumer reading the array is unaffected. | Pass |
| **P-6 Glossary First** | No new term is introduced; "handle", "anchor", "install root" are already glossary entries. | Not engaged |
| **P-7 Wrappers Stay Thin** | No wrapper changes. | Not engaged |
| **P-8 House Standards Apply** | UTF-8 without BOM, LF, no em-dashes or en-dashes, 80-column wrapping in prose. Enforced by `cargo xtask lint`. | Pass |
| **P-9 The Instrument Does Not Lie** | Central to all three fixes: the install path reflects the app's real type rather than an assumption; a missing install root is reported rather than presented as healthy; the three names are stored verbatim, none reconstructed from another; a cosmetic divergence stays silent so only a genuine one is asserted. | Pass, load-bearing |
| **P-10 One Path To A Target** | `register_candidate` remains the single registration path for every source; the two new `CandidateTarget` fields flow through it unchanged in shape, so a future platform source (Epic, GOG) implementing `TargetSource` inherits the same storage with no new special case. | Pass |
| **P-11 Specification Describes What Shipped** | The schema version (7 → 8) and any Appendix A handle vectors touched by the `&` decision are updated in this change, not later. | Obligation, tracked as a task |

No violation requires justification, so the Complexity Tracking table is omitted.

## Project Structure

### Documentation (this feature)

```text
specs/066-steam-identity-presence/
├── plan.md              # This file
├── spec.md              # Requirements and clarifications
├── research.md          # Phase 0: nine questions and their answers
├── data-model.md        # Phase 1: the type, schema, and presentation changes
├── quickstart.md         # Phase 1: how to verify the slice
├── checklists/
│   ├── requirements.md  # Spec quality gate
│   └── data-safety.md   # Requirements-quality gate for schema/conservation
└── tasks.md             # Phase 2 output, not created here
```

### Source code

```text
crates/fragcap-steam/src/
├── appinfo.rs   # + common/type and first launch-entry extraction alongside
│                #   the existing launch-entries extraction (no second parse)
└── library.rs   # + InstalledTitle::{installdir, app_type, launch_executable};
                 #   read_manifest joins common/ vs music/ from app_type

crates/fragcap-targets/src/
├── entry.rs         # + TargetEntry::{folder_name, executable_hint}
├── schema.rs         # SCHEMA_VERSION 7 -> 8, + MIGRATE_7_TO_8
├── store.rs          # + migration step, + column read/write, + targets_by_substring
├── source.rs         # + CandidateTarget::{folder_name, executable_hint}
├── register.rs       # plumb the two new fields onto the stored entry
├── selector.rs       # + the third (substring) resolution tier
├── handle.rs          # + the "&" -> " and " normalization step
├── targets_export.rs # + folder_name/executable_hint export keys
└── readiness.rs       # + InstallPresence, + NameDivergence derivations

crates/fragcap/src/discovery.rs   # SteamSource: Music-type exclusion (considered_not_a_game),
                                  # folder_name/executable_hint on the candidate

crates/fragcap-cli/src/
├── color.rs (new)                # shared use_color()/WARN/RESET, pub(crate)
├── commands/doctor.rs             # re-point at crate::color instead of its own copy
├── doctor/mod.rs                  # re-point at crate::color instead of its own copy
└── commands/targets.rs            # render_table: missing-install-root note in
                                    # SENSITIVITIES; hero listing's next-command
                                    # skips a missing row; print_target: divergence note
crates/fragcap-cli/tests/cli_targets.rs   # missing-install-root golden, unaffected-row
                                           # byte-identity assertion

docs/fragcap-specification.md   # schema version reference, Appendix A vectors
changelog.d/                    # one feature fragment, one decisions fragment
                                 # (the & expansion and the color-module extraction
                                 #  are the two architecture-adjacent choices)
```

**Structure Decision**: the existing crate layout is used unchanged. The one new
file, `crates/fragcap-cli/src/color.rs`, exists because the missing-install-root
note is the first place outside `doctor` that needs the same warning palette;
extracting it there (rather than duplicating the ANSI codes in `targets.rs`)
keeps the palette single-sourced, matching the issue's own instruction to
"follow the doctor convention" literally rather than by copy.

## Phase 0: Research

Complete. See [research.md](./research.md): eleven items (R-1 through R-11),
covering the appinfo read, the type-to-subdirectory mapping and its fallback,
the new `InstalledTitle` fields, the Music-type discovery-account bucket, why
`launch_entries` is deliberately not reused for the executable hint, the
three-tier selector resolution (and why a naive single-tier substring match was
rejected), the shared color module, the SENSITIVITIES-column note placement, and
the handle-derivation vector update.

## Phase 1: Design

Complete. See [data-model.md](./data-model.md) for the type, schema, and
presentation changes, and [quickstart.md](./quickstart.md) for how to verify
them.

### Contracts

This slice touches two contracts that outlive it.

**The target-entry export object** (`targets export`/`targets import`) gains two
optional keys:

```text
"folder_name": <string>        // present only when observed
"executable_hint": <string>    // present only when observed
```

Absent means not recorded, following the `detection_scan` precedent exactly. An
import that omits either leaves the corresponding field `None` on insert/merge.

**The `targets` schema** (`local.db`/`catalog.db`, `crates/fragcap-targets/src/schema.rs`).
Version 8 adds two nullable `TEXT` columns to `targets`, `folder_name` and
`executable_hint`, applied as one additive transaction alongside the version
stamp, exactly like every prior migration in this table's history. No CHECK
constraint is added for either (unlike `detection_scan`'s enum CHECK): both are
free-form observed strings with no closed vocabulary to enforce.

### Post-design Constitution re-check

Re-run after the design above. One obligation surfaced by the design rather than
the spec: the P-11 specification update for the schema version and any `&`-bearing
Appendix A vectors, carried as a task. No new violation appeared; in particular the
design adds no dependency, touches no platform-specific call beyond the existing
Windows-gated code this slice does not add to, and leaves `fragcap-core` and the
capture path (`launch_entries`, `capture_readiness`) untouched.

## Decision log

Decisions taken under the autopilot policy during this slice, beyond the five
already recorded in the spec's Clarifications section. The two marked
*architecture* also get a `changelog.d/*.decisions.md` fragment.

| # | Decision | Why | Alternative rejected |
| --- | --- | --- | --- |
| D-1 | Only `Music` redirects off `common/`; every other and unknown type keeps the current assumption | The reported defect and its only confirmed real-world instance are both Music apps; widening the change to every non-`Game` type risks moving the dominant case's behavior with no evidence for it (FR-003) | Mapping every known Steam type to its own subdirectory rule |
| D-2 | Fall back to `common/` when the app type is unknown or unreadable | Matches current behavior exactly for the dominant, already-working case; a fallback that instead errored would turn a benign appinfo-read miss into a new discovery-wide failure | Treating an unreadable appinfo cache as a hard discovery error |
| D-3 (*architecture*) | Two new dedicated columns (`folder_name`, `executable_hint`) rather than reusing `launch_entries` for the executable observation | `launch_entries` carries the socket-holder decision's P-9 guarantee (never claims a holder fragcap did not observe); repurposing it for a bare findability hint would either fabricate that claim or require touching `capture_readiness`/the real capture path, both outside this slice's scope | Storing the executable hint inside `launch_entries` under the existing `observed_exe` shape the `Unsure` answer already writes |
| D-4 | Three-tier selector resolution (exact handle, exact name, then substring) rather than a single substring pass | A single substring pass turns today's unambiguous exact-name selection into an ambiguity the moment a superstring name exists (e.g. "Portal 2" vs "Portal 2 Beta"), a regression FR-009's no-drift guarantee forbids | Collapsing exact-name and substring matching into one tier |
| D-5 (*architecture*) | Extract `use_color()` and the Warn/Reset ANSI constants into a shared `crate::color` module, refactoring `doctor` to use it too | The issue explicitly asks to follow "the doctor convention"; keeping two independently maintained copies of the same palette is exactly the drift risk the constitution's P-8/P-10 posture warns against | Duplicating the ANSI escape codes directly in `targets.rs` |
| D-6 | The missing-install-root note is a prefix on the existing SENSITIVITIES cell, not a new column | `render_table`'s own documentation already reserves this one column as free-running and exempt from the 80-column budget calculation; a new column would recompute that budget and risk the exact truncation P-4 forbids | A new "STATUS" or "PRESENCE" column; a marker glyph beside the row number with a legend line |
| D-7 | The `&` → `and` expansion is forward-looking only; no migration of already-derived handles | The spec's Clarifications already settle this; recorded here only to note the mechanical consequence: existing rows in a user's `local.db` keep their current handle, so no store migration touches `targets.handle` | Recomputing and re-disambiguating every existing handle on schema upgrade |
