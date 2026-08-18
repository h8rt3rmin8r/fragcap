# Implementation Plan: The targets hero command and interactive authoring

**Branch**: `055-targets-hero-command` | **Date**: 2026-08-18 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/055-targets-hero-command/spec.md`

## Summary

Turn `fragcap targets` (and bare `fragcap`) into the product's hero command: a
numbered, handle-ordered listing of the user's own capturable targets with a
CAPTURE readiness column and a neutral KNOWN evidence column, ending by naming the
next command, with an actionable empty case. Make row indices durable by persisting
a listing snapshot so `capture <n>` names the row the user saw. Add interactive
authoring (`targets add`) whose reason for existing is the honest
`Y/n/unsure` socket-holder answer, with a capture run promoting an `unsure` row to
`verified` once it observes the real holder. Add the lifecycle commands the surface
needs: `remove`, and `export`/`import` over a dedicated target-entry array
(operator decision, not the published capture schema). All in `fragcap-cli` and
`fragcap-targets`; no change to core/capture/attr/sink parsing or the published
schema.

Design detail lives in [research.md](research.md), [data-model.md](data-model.md),
the four files under [contracts/](contracts/), and [quickstart.md](quickstart.md).

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82 (local dev builds via
`+1.96.0-x86_64-pc-windows-gnu`; CI uses MSVC).

**Primary Dependencies**: existing only. `rusqlite` (behind `targets`) for the new
`listing_snapshot` table and the delete/promote/update methods; `serde_json` for
the target-entry array mapping (already a runtime dep of `fragcap-targets`). No new
crate, no `Cargo.lock` delta expected.

**Storage**: `local.db` (targets store, SQLite). Schema version 5 -> 6 for the
`listing_snapshot` table. No change to the `targets` table columns.

**Testing**: `cargo test` unit + integration; CLI tests in
`crates/fragcap-cli/tests/` (cli_targets.rs, cli_capture.rs), store/round-trip
tests in `crates/fragcap-targets/tests/`, promotion over the fixture pipeline in
`crates/fragcap/tests/`. Interactive flow tested via a scripted prompt seam (no
terminal).

**Target Platform**: Windows (the product target); the socket-holder promotion
reads attributions from the existing fixture pipeline, so it needs no live driver.

**Project Type**: CLI over a Rust workspace (single project, many crates).

**Performance Goals**: listing completes in a few seconds (hero criterion 4);
non-destructive.

**Constraints**: no fabricated socket holder (P-9); one creation operation and one
stored form for every source (P-10); glossary and master-spec lock-step (P-6,
P-11); every discard counted (P-4).

**Scale/Scope**: tens to low hundreds of registered targets per user; single-file
store.

## Constitution Check

*GATE: must pass before and after design.*

| Principle | Bearing on S055 | Status |
| --- | --- | --- |
| P-1 Passive observation | No process handle, no injection; promotion reads attributions the pipeline already produced. | PASS |
| P-2 Core stays neutral | All work in `fragcap-cli` + `fragcap-targets`; `fragcap-core` untouched. | PASS |
| P-3 Capture/attr separate | No new edge between capture and attribution crates; promotion write-back is CLI-orchestrated. | PASS |
| P-4 No silent loss | scan/discover accounting stays conserved and surfaced; unreadable scan paths surfaced. | PASS |
| P-5 Compatibility | No output-format change; pcapng/JSONL writers untouched. | PASS |
| P-6 Glossary first | New terms (listing snapshot, capture readiness, unresolved launch chain, target-entry export) get glossary entries in this change. | PLANNED (tasks) |
| P-8 House standards | UTF-8 no BOM, LF, no dashes; `cargo xtask lint`/`fmt`/`clippy`. | PLANNED (gate) |
| P-9 Instrument does not lie | The whole `Y/n/unsure` design exists to avoid fabricating a holder; promotion writes only an observed image; KNOWN column is neutral evidence. | PASS (load-bearing) |
| P-10 One path to a target | Interactive add, flag add, scan, import all persist via `insert_target`; one stored form; row-index resolution stays single-sourced in `resolve_positional`. | PASS |
| P-11 Spec describes shipped | Master spec section(s) for the targets command surface updated in lock-step; `cargo xtask spec` Applies-To. | PLANNED (tasks) |

No violation requiring justification. No denylisted technique is anywhere near
this slice. The one operator-level decision (export format) was raised and
resolved before design (research D7).

## Project Structure

### Documentation (this feature)

```text
specs/055-targets-hero-command/
├── plan.md              # this file
├── research.md          # Phase 0
├── data-model.md        # Phase 1
├── quickstart.md        # Phase 1
├── contracts/           # Phase 1
│   ├── targets-command.md
│   ├── listing-and-row-index.md
│   ├── interactive-add-and-promotion.md
│   └── export-import.md
├── checklists/
│   ├── requirements.md
│   └── hero-command.md
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source code (touched)

```text
crates/fragcap-targets/src/
├── schema.rs          # SCHEMA_VERSION 5->6, DDL + MIGRATE_5_TO_6 (listing_snapshot)
├── store.rs           # write_listing_snapshot, listing_snapshot_nth, delete_target,
│                      #   promote_target_launch, update-for-import
├── selector.rs        # row-index branch -> snapshot; share is_row_index
├── entry.rs           # (read) TargetEntry; no column change
├── targets_export.rs  # NEW: TargetEntry <-> target-entry array mapping + validate
└── lib.rs             # facade re-exports for CLI

crates/fragcap-cli/src/
├── cli.rs             # TargetsCommand: add Remove/Export/Import; add args
├── commands/targets.rs# list renderer (CAPTURE/KNOWN), empty case, interactive add,
│                      #   scan (exists), remove, export, import; prompt seam
└── commands/capture.rs# promotion write-back after a run vs an unresolved target

crates/fragcap-cli/tests/   # cli_targets.rs (+ cli_capture.rs) cases + goldens
crates/fragcap-targets/tests/  # snapshot, export/import round-trip, promotion fn
crates/fragcap/tests/          # promotion over the fixture pipeline
docs/                          # master spec section update (P-11), glossary (P-6)
changelog.d/                   # S055 fragment (spec-impact)
```

## Phase sequencing (feeds /speckit-tasks)

1. **Foundational** (blocks the rest): schema 5->6 + `listing_snapshot`;
   `resolve_positional` snapshot branch + shared `is_row_index`; facade re-exports.
2. **US1 (P1, MVP)**: listing snapshot writer; CAPTURE/KNOWN derivation; table
   renderer + empty case; wire bare `fragcap` / `targets` / `list`; `capture <n>`
   inherits snapshot. Tests + goldens.
3. **US2 (P2)**: prompt seam + scripted double; interactive `add` with inline scan;
   `Y/n/unsure` -> stored launch chain; `add --steam` kept; then the heavy item:
   `promote_target_launch` + capture write-back + fixture-pipeline test.
4. **US3 (P3)**: `delete_target` + `remove`; target-entry array mapping +
   `export`/`import` merge-on-id + round-trip test.
5. **Polish**: glossary entries (P-6); master spec section rewrite (P-11) +
   `xtask spec`; changelog fragment; README quickstart; full `cargo xtask ci`.

## Risks

- **Promotion write-back (US2)** is the highest-risk item: it is the first path
  that mutates a `targets` row's fidelity/launch and the first capture->targets
  write. Mitigation: land the store method + pure promotion function first (unit
  tested), then wire capture; if a live backend proves necessary, the Tier-2
  boundary is stated, not hidden (research D6).
- **Snapshot vs S054 `capture <n>`**: behavior change from live order to snapshot;
  covered by the Clarifications entry and an explicit test after a mutation.
- **Slice size**: large (comparable to S054's 26 tasks). MVP is US1 alone; US2/US3
  layer on independently, so the slice degrades gracefully if split.

## Post-design constitution re-check

Design introduces no new dependency, no new crate edge, no platform code in core,
no denylisted technique, and no published-schema change. P-6/P-8/P-11 are tasked
in Polish. Gate remains PASS.
