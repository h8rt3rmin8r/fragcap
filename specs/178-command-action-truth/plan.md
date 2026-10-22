# Implementation Plan: Truthful CLI Action Boundaries

**Branch**: `codex/s178-command-action-truth` | **Date**: 2026-10-21 | **Spec**: `specs/178-command-action-truth/spec.md`

**Input**: Owner-approved S178 grouping #489, #491, #492, and #493 at the preflight, side-effect, and actionable-result boundary.

## Summary

Make bundle collection previews offer Apply only for complete eligible proposals and explain their deletion scope; validate missing bundle paths before cleanup/export guidance. Reject conflicting target export selectors at parsing, make missing-selector hints action-specific, and preserve export-all. Prevalidate catalog source and tier compatibility before opening the destination, deduplicate fills, and give help that names the shipped parser. Correct target discovery and scan help to state actual local-store effects. Preserve existing authorization and data ownership mechanics.

## Technical Context

**Language/Version**: Rust 2024, workspace MSRV 1.88.

**Primary Dependencies**: Existing `clap`, `serde_json`, `fragcap`, and `fragcap-cli`; no new package.

**Storage**: Existing local and catalog SQLite files and existing bundle artifacts. No migration or persistent schema change.

**Testing**: Controlled CLI/parser tests, isolated temporary-store and synthetic bundle tests, `cargo xtask ci`, and hosted exact-head gates.

**Target Platform**: Windows is authoritative for exact bundle collection; parser, catalog, target, and help tests run without live capture or a game.

**Project Type**: Native Rust CLI with existing facade and session collection orchestration.

**Performance Goals**: One bounded named-source preflight before catalog store open; no new network call, bundle scan, or target registration.

**Constraints**: Collection proposals retain exact digest authorization and path/ownership checks; private evidence is not printed as a new result field; no direct modification of owner bundles in verification; no focus-stealing console launches.

**Scale/Scope**: Four command families, four GitHub issues, one feature slice. Global JSON/silent restructuring (#488), IGDB (#155), and release publication remain separate.

## Constitution Check

*GATE: passed before research; recheck after design and at analyze.*

- **P-1**: No capture, proxy, trust, target-process, or Windows handle-right change is needed. Collection remains its already authorized local evidence operation.
- **P-2/P-3/P-10**: CLI and existing target facade boundaries remain intact. No new store representation, target registration path, or dependency direction is introduced.
- **P-4/P-9**: Filled tiers count actual distinct work; preview and path failures report observed facts; conflicting selectors cannot silently export the wrong target.
- **P-8**: New Rust and Markdown follow formatting, layout, encoding, and source hygiene. Human multi-column output uses the shared display layout.
- **P-11**: Master specification and current help are corrected together without a claim that an unreleased version has shipped.
- **Workflow**: Specify, clarify, checklist, plan, tasks, analyze, implement, foreground verification, commit, authorized push/PR, CI, and at most two review rounds are required. The owner's explicit push/PR authorization fulfills autopilot's ordinary pre-push approval boundary.

## Research Decisions

1. **Collection recommendation**: Derive whether to print Apply from existing proposal facts: preview mode, `complete=true`, `eligible_sessions>0`, and `unresolved_sessions=0`. Keep the proposal identifier even when not actionable. Render existing row `detail` beside `reason` in human output, and give a read-only next command for non-actionable previews. Preserve exact proposal recomputation and application authority.
2. **Bundle path preflight**: For cleanup and export, distinguish absent path from other failures before suggesting `--yes` or invoking the facade. Name the supplied path and a read-only inspection or path-correction action. Cleanup uses the facade's shared read-only sensitive-path validation so completed and crash-prefix manifests retain the same authority as effectful cleanup.
3. **Target selection**: Add an optional clap selector group with default mutual exclusion to export, mirroring show/remove while preserving its zero-selector case. Retain a defensive conflict check in command code for direct internal construction. Parameterize no-match guidance by requested action rather than changing capture's existing message.
4. **Seed preflight**: Resolve and validate the requested tier set before `Store::open`; refuse explicit launch, deduplicate supported tiers in first-occurrence order, and read/parse named bare-array sources before destination creation. Pass the parsed source into the existing seeder rather than reading it again after open. Default no-source seed remains an offline signature fill with explicit title/engine/launch skip reasons.
5. **Seed guidance**: Name offline `--from` with the matching `--tier` in all builds, and do not advertise `--steam` or `--pcgamingwiki` in default-build messages. Network flags remain separately feature-gated in parser/help. Spell out bare-array input for catalog and engine tiers and reject export envelopes before store creation.
6. **Discovery wording**: Preserve existing local-store opening and volume-eligibility seeding. Correct the inaccurate `read-only` phrase in generated help and section 17.7 of the master specification; describe scan registration in short and full help and bare-target summary.

## Project Structure

### Documentation

```text
specs/178-command-action-truth/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/cli-action-results.md
  quickstart.md
  tasks.md
  checklists/{requirements,action-contract}.md
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/S178-*.md
```

### Source Code

```text
crates/fragcap-cli/src/cli.rs
crates/fragcap-cli/src/commands/{bundle,catalog,target_resolve,targets}.rs
crates/fragcap-cli/src/workflow_help.rs
crates/fragcap-cli/tests/{cli_bundle,cli_catalog,cli_targets,cli_help}.rs
crates/fragcap/src/deep_capture/artifacts.rs
```

**Structure Decision**: Reuse existing command families and temporary result values. Do not add a generic cross-command result framework before #488 defines its global semantics.

## Execution Order

1. Write failing controlled tests for incomplete/zero-work collection previews, path guidance, export selector conflict, target no-match guidance, catalog preflight/deduplication, and discovery effects/help.
2. Implement each preflight before its effect, update result rendering and advice, and preserve existing exact collection and store operations.
3. Update the master specification and chronological slice plan, add an unreleased changelog fragment, and reconcile every contract and checklist item.
4. Run Spec-Kit analyze, focused tests, `cargo xtask ci`, text hygiene, and exact-head checks. Commit only the slice files, push the owner-authorized branch, open the official PR, and resolve every review comment within the two-round cap.

## Complexity Tracking

No constitution deviation, new dependency, migration, or release-marker change is planned.
