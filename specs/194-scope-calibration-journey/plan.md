# Implementation Plan: Scope-Aware Calibration and Operator Journey

**Branch**: `codex/s194-scope-calibration-journey` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/194-scope-calibration-journey/spec.md`

## Summary

Extend the exact S121 compatibility case and S145 durable workflow with the S191/S193 inspection-scope contract, migrate existing history without cross-promoting it, and let guided calibration execute one bounded linked game-scope retry or packet-only fallback only when the initial immutable plan permits it. Expose the same scope vocabulary through calibration and advanced Deep Capture, then reconcile target detail, Doctor, authorization, human and JSON output, documentation, and CI package artifact certification.

## Technical Context

**Language/Version**: Rust 1.88 workspace, PowerShell 7 and Bash repository tooling, MDX documentation site

**Primary Dependencies**: Existing workspace dependencies only: clap, rusqlite, serde_json, Tokio, native fragcap proxy and facade crates

**Storage**: Additive SQLite local-store schema version 14; versioned JSON Lines and manifest projections remain additive

**Testing**: Rust unit and controlled integration tests, CLI parser/help/output matrices, store migration fixtures, repository xtask checks, Windows package certification, hosted exact-head CI

**Target Platform**: Windows 11 product, Linux and Windows CI for platform-neutral contracts

**Project Type**: Multi-crate Rust library and CLI with Windows packaging and static documentation

**Performance Goals**: Scope resolution remains pure and bounded; no additional per-packet work; retry remains one successor; existing queue, connection, workflow, and phase limits remain finite

**Constraints**: No new dependency, system-wide proxy effect, target instrumentation, middleware content retention, mutable resume intent, or pre-release installed-product completion requirement

**Scale/Scope**: Two open child issues (#560 and #561), one parent epic (#557), one additive store migration, existing thirteen protocol candidates, one public scope enum, two optional fallback authorities

## Constitution Check

*GATE: Passed before research and rechecked after design.*

- **P-1**: Scope is resolved before effects, game inspection uses S191 exact ownership, middleware uses S193 authenticated blind relay, and no process memory, injection, hook, target key extraction, or system-wide proxy behavior is added.
- **P-2/P-3**: Scope and compatibility vocabulary stay in the targets/facade layers. The proxy remains independent of CLI and attribution implementations. Capture and attribution seams remain unchanged.
- **P-4/P-9**: Legacy data, failed attempts, passive evidence, omissions, and every named loss remain explicit. Retry and fallback never rewrite the predecessor or turn absent game evidence into success.
- **P-5**: Additive JSON and manifest fields preserve existing readers; packet output remains ordinary pcapng.
- **P-6**: New public scope, middleware-handling, attempt-lineage, and passive-fallback terms receive glossary coverage.
- **P-7/P-8**: No wrapper logic is added. All files follow repository encoding, prose, source, and layout conventions.
- **P-10**: Target recommendations and readiness remain projections of the existing target row and local store, with no second target shape or resolver.
- **P-11**: The master specification and public documentation describe only behavior delivered in S194.
- **Workflow**: Full Spec-Kit sequence, foreground verification, changelog fragments, exact-head CI, at most two hosted review rounds, and owner-reserved merge.
- **Installed-product policy**: CI certifies package bytes and identity. Installed-product and private-title checks occur only after release and do not condition implementation completion.

Post-design recheck: passed. Schema 14 is additive, public intent is immutable, and controlled evidence covers all behavior introduced by this slice.

## Project Structure

### Documentation (this feature)

```text
specs/194-scope-calibration-journey/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── operator-contract.md
├── checklists/
│   ├── requirements.md
│   └── scope-and-fallback.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/
├── compatibility.rs
├── workflow.rs
├── schema.rs
└── store.rs

crates/fragcap/src/deep_capture/
├── inspection.rs
├── proposal.rs
├── policy.rs
├── model.rs
├── manifest.rs
└── api.rs

crates/fragcap-cli/src/
├── cli.rs
├── events.rs
├── session_ux.rs
├── workflow_help.rs
├── commands/calibrate.rs
├── commands/deep_capture.rs
├── commands/targets.rs
└── doctor/

docs/
├── fragcap-specification.md
├── plans/README.md
└── glossary/

site/content/docs/
```

**Structure Decision**: Extend the existing targets store and facade contracts in place. CLI presentation consumes those contracts and does not create parallel policy or persistence.

## Complexity Tracking

No constitution violation requires justification.
