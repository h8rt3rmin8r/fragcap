# Implementation Plan: Doctor Residue Guidance

**Branch**: `codex/135-doctor-residue-guidance` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/135-doctor-residue-guidance/spec.md`

## Summary

Close issue #373 by separating Doctor's human residue presentation from its stable machine identity. Retain the existing dynamic check name and cleanup action contract for compatibility, add exact structured native-resource context to JSON, and give human output a short stable label plus health-specific plain-language diagnosis. Make the report select an aligned or compact layout from the output width, count Unicode display cells, and preserve every identity without truncation. The S124 inventory and shared recovery planner remain the sole classification and cleanup authorities.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: Existing `fragcap-cli`, `fragcap` facade residue model, `serde_json`, and transitive `terminal_size`; no new package

**Storage**: Existing Deep Capture bundle and cleanup journals are read only; no schema or write-path change

**Testing**: Doctor unit tests, CLI contract tests, controlled residue fixtures, full Cargo workspace and xtask gates

**Target Platform**: Cross-platform renderer and controlled tests; production residue discovery and cleanup remain Windows-aware through existing adapters

**Project Type**: Rust workspace CLI plus facade library

**Performance Goals**: Linear rendering over bounded Doctor findings and text length; at most one terminal-width query per human Doctor invocation

**Constraints**: No cleanup-policy change, no new effect in read-only Doctor, no truncation, 40-display-column minimum, 80-column default, machine compatibility, no secret or new local-path disclosure, no new lockfile package

**Scale/Scope**: One human residue diagnosis layer, one optional structured JSON context, two width-aware report layouts, shared display-width helpers, focused documentation and regression coverage

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. S135 only presents already discovered residue and adds no capture, process, routing, trust, or cleanup capability.
- **P-2 Core stays platform-neutral**: Pass. Rendering remains in `fragcap-cli`; no change enters `fragcap-core`.
- **P-3 Capture and attribution stay separate**: Pass. No packet or attribution boundary changes.
- **P-4 No silent loss**: Pass. Observation and loss paths are untouched, and human values are never truncated.
- **P-5 Compatibility outranks richness**: Pass. Existing common JSON check fields, machine check names, verdicts, and action selection remain intact; the structured residue object is additive.
- **P-6 Glossary first**: Pass. Doctor, residue, session owner, recovery, and Deep Capture are established terms. The implementation introduces no new domain term.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes.
- **P-8 House standards apply**: Pass. UTF-8 without BOM, LF, formatting, lint, tests, docs, privacy, and changelog gates remain blocking.
- **P-9 The instrument does not lie**: Pass. Wording is derived from the exact S124 state, health, ownership authority, and recoverability facts; structured eligibility follows the actual Doctor action, and ambiguous records remain explicitly unproven.
- **P-10 One path to a target**: Pass. Target resolution and storage do not change.
- **P-11 The specification describes what shipped**: Pass. Master specification, outline, roadmap, site reference, and changelog move with the implementation without claiming Deep Capture completion.

Post-design re-check: passed. A presentation-only value is attached to the existing `Check`, while the original machine name and action remain authoritative. The structured context copies only non-secret facts already held by the residue finding. Terminal-width selection changes layout, not diagnosis or cleanup eligibility. No constitution exception is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/135-doctor-residue-guidance/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── doctor-residue-output.md
├── checklists/
│   ├── requirements.md
│   └── ux.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
├── display.rs                         # shared display-cell width and padding helpers
├── doctor/
│   ├── checks.rs                      # exact residue-to-diagnosis mapping
│   ├── fix.rs                         # width-aware pre/post-fix reports
│   └── mod.rs                         # typed context, layouts, wrapping, JSON
├── commands/
│   ├── doctor.rs                      # one stdout width selection per invocation
│   └── targets.rs                     # consume shared display helpers
└── lib.rs                             # register shared display module

crates/fragcap-cli/tests/
└── cli_doctor.rs                      # human, JSON, action, escaping, and residue regressions

docs/
├── fragcap-specification.md
├── fragcap-spec-outline.md
└── plans/README.md

site/content/docs/reference/
└── cli.mdx                            # Doctor human and JSON residue contract

changelog.d/
├── 373-doctor-residue-guidance.fixed.md
└── s135-doctor-residue-guidance.decisions.md
```

**Structure Decision**: Keep classification in the existing residue inventory and translate it into presentation in Doctor checks. Preserve the machine-facing check identity while attaching an optional human presentation and structured native-resource context. Extract the already proven target-table display-cell helpers into one crate-private module so Doctor does not duplicate width logic.

## Implementation Sequence

1. Add failing controlled tests for abandoned, active, healthy, stale, cleanup-failed, unknown, and unsupported residue findings, including exact cleanup-action parity.
2. Extract shared Unicode display-cell helpers and retain target-discovery table behavior.
3. Add typed native-resource JSON context and separate human presentation while preserving common check fields and stable machine names.
4. Add exact health-specific diagnoses and plain-language remediation without changing inventory classification or recovery planning.
5. Add width selection, aligned and compact layouts, display-cell wrapping, color parity, and default plus 40-column regressions.
6. Synchronize the specification, outline, roadmap, CLI site reference, changelog, and controlled validation guide.
7. Run analysis, focused tests, full CI parity, dependency stability, encoding and mojibake checks, and final diff review.

## Complexity Tracking

No constitution violations require justification.
