# Implementation Plan: Target Discovery Integrity

**Branch**: `codex/133-target-discovery-integrity` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/133-target-discovery-integrity/spec.md`

## Summary

Close issue #375 by separating discovery production from automatic-registration eligibility, excluding the exact Steam client subtree from the generic known-roots walk, and routing both the hero and Doctor paths through one high-precision registration operation. Extend explicit discovery with per-candidate eligibility and a privacy-safe count-only view. Add an operator-invoked reconciliation preview that deletes only exact fragcap-owned infrastructure, duplicate, or aggregate rows after confirmation and an atomic unchanged-row check.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: Existing workspace crates only (`fragcap-targets`, `fragcap-steam`, `fragcap` facade, `fragcap-cli`); standard library and existing `serde_json`/`rusqlite`

**Storage**: Existing version 10 `local.db`; no schema migration or new persistent record

**Testing**: Pure policy and reconciliation unit tests, synthetic filesystem and Steam metadata integration tests, CLI contract tests, transactional store tests, privacy checks, and full repository gates

**Target Platform**: Policy, reconciliation, and fixture tests are cross-platform; real fixed-volume enumeration and Steam-root behavior remain Windows production paths

**Project Type**: Rust workspace library plus thin CLI consumer

**Performance Goals**: One bounded pass over each discovery candidate and stored target; no deeper filesystem walk, additional full-volume enumeration, or unbounded collection

**Constraints**: No automatic deletion, no path-based ownership inference, no store-shape change, no new dependency, no exhaustive executable scan, no #374/#376 presentation scope, and no release action

**Scale/Scope**: One candidate policy, one exact-subtree exclusion, one combined registration account, one reconciliation plan/apply path, two CLI additions (`discover --summary`, `reconcile --yes`), synthetic fixtures, specification correction, and count-only real-machine evidence

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. Work reads platform metadata, bounded known roots, and the existing user-owned store. It opens no target process and changes no target or network state.
- **P-2 Core stays platform-neutral**: Pass. No change enters `fragcap-core`; platform discovery remains in the facade and platform crates.
- **P-3 Capture and attribution stay separate**: Pass. No packet or attribution path changes.
- **P-4 No silent loss**: Pass. Produced candidates reconcile to accepted plus refused decisions; existing source loss counters remain conserved; cleanup is atomic and reports every decision.
- **P-5 Compatibility outranks richness**: Pass. No capture output or artifact format changes.
- **P-6 Glossary first**: Pass. Existing terms cover target, discovery source, fidelity, and evidence. New command terms are defined in the specification and contract before user documentation.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes.
- **P-8 House standards apply**: Pass. UTF-8 without BOM, LF, formatting, lint, tests, docs, dependency, privacy, and changelog gates remain blocking.
- **P-9 The instrument does not lie**: Pass. Location alone loses automatic-registration authority; ambiguous historical rows are preserved; missing evidence never becomes a clean or game claim.
- **P-10 One path to a target**: Pass. Accepted candidates continue through the existing `register_candidates` entry shape; policy narrows admission without creating another store form.
- **P-11 The specification describes what shipped**: Pass. Sections 7.1 and 17.7, the outline, roadmap, and changelog change with implementation.

Post-design re-check: passed. The registration policy belongs beside the existing candidate-to-entry operation, exact platform-root exclusions remain an injected decision on the known-roots source, and reconciliation operates over existing `TargetEntry` values plus an injected platform inventory. No constitution exception is required.

## Project Structure

### Documentation (this feature)

```text
specs/133-target-discovery-integrity/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── real-machine-validation.md
├── contracts/
│   └── discovery-integrity.md
├── checklists/
│   ├── requirements.md
│   └── discovery-integrity.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/
├── source.rs                       # candidate and source accounting remain unchanged
├── register.rs                     # automatic-registration policy and combined account
├── reconcile.rs                    # pure preview model and exact ownership decisions
├── sources/known_roots.rs          # exact excluded-subtree pruning
├── store.rs                        # atomic compare-and-delete operation
└── lib.rs                          # facade-ready exports

crates/fragcap-targets/tests/
├── known_roots.rs                  # direct and nested Steam-root exclusion
├── detection_walk.rs               # positive local evidence admission
└── discovery_integrity.rs          # policy and reconciliation fixture matrix

crates/fragcap/src/
├── discovery.rs                    # authoritative Steam inventory and root composition
└── lib.rs                          # target-domain re-exports

crates/fragcap-cli/src/
├── cli.rs                          # discover --summary and reconcile --yes grammar
└── commands/targets.rs             # shared registration, rendering, preview, confirmation

crates/fragcap-cli/tests/
└── cli_targets.rs                  # non-persistence, privacy, preview, and apply contracts

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md

changelog.d/
├── s133-target-discovery-integrity.fix.md
└── s133-target-discovery-integrity.decisions.md
```

**Structure Decision**: Keep candidate eligibility and reconciliation classification in `fragcap-targets`, which owns discovery values, entry provenance, registration, and the store. The facade supplies exact Steam installation facts without adding a sibling dependency to `fragcap-targets`. The CLI owns only command grammar, presentation, and confirmation.

## Implementation Sequence

1. Add failing policy, excluded-subtree, combined-account, reconciliation, transactional-delete, and CLI contract tests.
2. Add exact path normalization and excluded-subtree pruning before known-roots classification or descent.
3. Add the pure automatic-registration decision and route hero plus Doctor registration through one combined operation.
4. Annotate explicit discovery with eligibility and add a count-only output that suppresses names, application ids, paths, and detailed warnings.
5. Add the pure reconciliation plan, exact platform inventory composition, immutable preview, confirmation grammar, and transactional unchanged-row apply.
6. Synchronize specification, outline, roadmap deviation, changelog, and count-only validation record.
7. Run analyze, focused tests, full CI parity, privacy/encoding hygiene, dependency stability, and diff checks.

## Complexity Tracking

No constitution violations require justification.
