# Implementation Plan: Verified First Capture and Deep Capture Journeys

**Branch**: `codex/090-getting-started-journeys` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/090-getting-started-journeys/spec.md`

## Summary

Rewrite the v0.5-era getting-started page as two connected v0.7.0 journeys: a first bounded Capture followed by a known-compatible Deep Capture session. Derive specimens from current synthetic CLI contracts, keep database locations optional, state exact traffic and artifact limits, stop unknown targets at the read-only compatibility check, and validate the page through focused CLI suites, documentation checks, a production static export, and the complete repository gate.

## Technical Context

**Language/Version**: MDX and Markdown; Rust workspace version 0.7.0 supplies the shipped behavior baseline

**Primary Dependencies**: Existing Fumadocs and Next.js site toolchain, current clap command tree, committed doctor golden, target-listing tests, and Deep Capture compatibility reference

**Storage**: One committed guide page, S090 specification artifacts, and one unreleased changelog fragment; no runtime storage changes

**Testing**: Focused CLI argument, help, target, doctor, and Deep Capture suites; command and phrase audits; `cargo xtask docs check`; `cargo xtask docs build`; and `cargo xtask ci`

**Target Platform**: Statically exported public documentation for the Windows v0.7.0 release

**Project Type**: Documentation correctness slice

**Performance Goals**: No runtime performance impact; the production static export completes under the existing documentation build gate

**Constraints**: Documentation-only implementation; synthetic examples only; no command, runtime, dependency, workflow, toolchain, or release changes; no automatic compatibility-bootstrap claim; no universal traffic-inspection claim; UTF-8 without BOM; soft-wrapped MDX prose; no prohibited punctuation

**Scale/Scope**: One public guide page, one changelog fragment, the complete S090 artifact set, five focused CLI validation surfaces, and the existing documentation and repository gates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. The guide requires explicit Deep Capture selection, target-scoped managed launch, visible CA trust confirmation, and auditable cleanup. It rejects system-wide fallback, pinning bypass, target key extraction, and every denylisted technique.
- **P-2 Core Stays Platform-Neutral**: Pass. No runtime crate changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. The guide distinguishes packet acquisition from process attribution and changes neither implementation.
- **P-4 No Silent Loss**: Pass. Capture scope and unsupported inspection remain visible; the guide does not reinterpret absent application observations as complete coverage.
- **P-5 Compatibility Outranks Richness**: Pass. The Capture journey ends with ordinary pcapng in an unmodified analyzer.
- **P-6 Glossary First**: Pass. Capture, Deep Capture, target, local proxy, certificate authority, TLS key log, HAR, and attribution fidelity already exist in the glossary.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. New Markdown and edited MDX use one logical line per paragraph or list item, valid fences and links, UTF-8 without BOM, and no prohibited punctuation.
- **P-9 The Instrument Does Not Lie**: Pass. The guide separates packet truth from proxy observations, states payload scope exactly, preserves unknown and partial states, and does not infer compatibility.
- **P-10 One Path To A Target**: Pass. Both journeys use the existing stored-target listing and selector path; no alternate target storage or identity is introduced.
- **P-11 The Specification Describes What Shipped**: Pass. The page is reconciled specifically to v0.7.0 and does not describe issue #251's future compatibility bootstrap as current behavior.

## Project Structure

### Documentation (this feature)

```text
specs/090-getting-started-journeys/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── journey-contract.md
├── checklists/
│   ├── journeys.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
site/content/docs/getting-started.mdx
changelog.d/245-getting-started.fixed.md

crates/fragcap-cli/tests/
├── cli_args.rs
├── cli_deep_capture.rs
├── cli_doctor.rs
├── cli_help.rs
├── cli_targets.rs
└── goldens/doctor-ready.txt
```

**Structure Decision**: Replace the getting-started page in place and treat current synthetic CLI tests and goldens as validation authorities, not implementation surfaces. Do not add a new documentation command extractor because issue #246 owns the command-tree gate. Keep detailed architecture and bundle contracts in issues #247 and #248; this guide carries the minimum facts required to complete the two first-run journeys safely and does not direct operators to the currently stale output-format page for bundle handling.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/journey-contract.md](contracts/journey-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. The contract keeps the Deep Capture path fact-backed and consent-forward, leaves packet truth in `.fcapng`, uses only stored targets, and adds no runtime or dependency surface.
