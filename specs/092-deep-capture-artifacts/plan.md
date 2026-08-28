# Implementation Plan: Deep Capture Bundle and Artifact Reference

**Branch**: `codex/092-deep-capture-artifacts` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/092-deep-capture-artifacts/spec.md`

## Summary

Rewrite the public output-formats reference around two distinct surfaces: ordinary Capture sinks and the Deep Capture session bundle. Preserve the existing packet-format contract, then add an implementation-grounded manifest guide, state and omission vocabularies, a complete artifact authority and sensitivity matrix, correlation guidance, synthetic examples, lifecycle and sharing guidance, and direct links from the CLI and compatibility references. Validate copied contract tokens against the v0.7.0 implementation, build the production site, and run the complete repository gate.

## Technical Context

**Language/Version**: MDX and JSON examples; Rust workspace version 0.7.0 supplies the shipped behavior baseline

**Primary Dependencies**: Existing Fumadocs and Next.js site; Deep Capture manifest, sidecar, HAR, doctor cleanup, and controlled-test implementations; master specification and glossary

**Storage**: One expanded public reference page, two cross-page link edits, S092 specification artifacts, and one unreleased changelog fragment; no runtime storage changes

**Testing**: Focused artifact, state, omission, link, and synthetic-data phrase audits; `cargo xtask docs check`; `cargo xtask docs build`; `cargo fmt --all -- --check`; and `cargo xtask ci`

**Target Platform**: Statically exported public documentation for the Windows v0.7.0 release

**Project Type**: Documentation correctness and security-handling slice

**Performance Goals**: No runtime impact; the reference remains scannable through compact tables and the production export completes under the existing documentation gate

**Constraints**: Documentation-only implementation; preserve stable page URL; no runtime, dependency, workflow, toolchain, release, or master-specification change; exact shipped tokens only; synthetic examples only; UTF-8 without BOM; soft-wrapped prose; no prohibited punctuation; no claim of universal inspection, automatic completed-bundle deletion, or target TLS key extraction

**Scale/Scope**: One authoritative output page covering two Capture formats, nine Deep Capture artifact roles, three manifest states, four current omission reasons, correlation anchors, and handling guidance; two narrow inbound-link edits

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. The plan describes only explicit proxy-owned observations and proxy-owned TLS material, and it explicitly excludes target key extraction and silent cleanup.
- **P-2 Core Stays Platform-Neutral**: Pass. No runtime crate changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. Packet truth, proxy observations, and process evidence remain distinct artifacts with separate authorities.
- **P-4 No Silent Loss**: Pass. Capture loss accounting remains in packet outputs, and missing application artifacts are documented through explicit omissions rather than inferred absence.
- **P-5 Compatibility Outranks Richness**: Pass. `.fcapng` remains ordinary pcapng readable by an unmodified analyzer; application observations remain sidecars.
- **P-6 Glossary First**: Pass. Session bundle, HAR, local development certificate authority, proxy-owned TLS key-log export, capture scope, and attribution fidelity already have glossary entries.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. New Markdown and edited MDX follow the repository and ShruggieTech authoring standards.
- **P-9 The Instrument Does Not Lie**: Pass. The plan assigns each claim to the artifact that owns it, preserves original observations, and treats absent anchors or artifacts as unavailable evidence rather than negative observations.
- **P-10 One Path To A Target**: Pass. The bundle reference begins from one existing stored target and does not introduce a second target form.
- **P-11 The Specification Describes What Shipped**: Pass. Exact v0.7.0 implementation and tests take precedence over broader forward-looking omission examples in the master specification.

## Project Structure

### Documentation (this feature)

```text
specs/092-deep-capture-artifacts/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── output-reference-contract.md
├── checklists/
│   ├── artifact-contract.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
site/content/docs/reference/
├── output-formats.mdx
├── deep-capture-compatibility.mdx
└── cli.mdx

changelog.d/248-deep-capture-artifacts.fixed.md

crates/fragcap-cli/src/
├── commands/deep_capture.rs
├── doctor/fix.rs
├── doctor/probe.rs
└── har.rs

crates/fragcap-cli/tests/cli_deep_capture.rs
docs/fragcap-specification.md
docs/glossary/file-and-wire-formats.md
```

**Structure Decision**: Expand `output-formats.mdx` in place because it is already the linked output authority. Keep the first half focused on ordinary Capture pcapng and packet JSON Lines, then introduce the Deep Capture bundle as a separate output family. Link the CLI and compatibility pages to that contract instead of copying its matrix. Treat current source and controlled tests as the authority for exact v0.7.0 values; use the master specification for intent where it agrees with shipped behavior.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/output-reference-contract.md](contracts/output-reference-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. It keeps packet truth, proxy observations, projections, compatibility evidence, and cleanup evidence separate; documents sensitive proxy-owned material without normalizing it into target extraction; and binds all copied tokens to current tests and source rather than future vocabulary.
