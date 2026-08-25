# Implementation Plan: Deep Capture session bundle

**Branch**: `073-deep-capture-session-bundle` | **Date**: 2026-08-25 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/073-deep-capture-session-bundle/spec.md`

## Summary

Issue #216 is a design slice. It defines the durable Deep Capture bundle before doctor cleanup (#218) or MVP implementation (#219) rely on artifact names, sensitivity markings, or correlation anchors. The model keeps `.fcapng` as packet truth, stores decrypted application records in JSONL and HAR sidecars when semantics are available, treats TLS key logs as sensitive analyzer aids, and makes the manifest the bundle index and cleanup handoff contract.

## Technical Context

**Language/Version**: Documentation-only slice; future implementation remains Rust, workspace MSRV 1.82.

**Primary Dependencies**: none added.

**Storage**: No store schema change. This slice defines future session-bundle file contracts and manifest fields.

**Testing**: Documentation and spec gates only for this PR. Future implementation slices must add schema/serializer tests against the contracts defined here.

**Target Platform**: Platform-neutral bundle model. Windows-specific details enter through future proxy, process trace, and doctor implementations.

**Project Type**: Rust Cargo workspace with Markdown specifications and Spec Kit slice artifacts.

**Performance Goals**: The design keeps high-volume packet data in `.fcapng` and high-volume application events in streaming sidecars so the manifest remains small and quick for doctor/status commands to read.

**Constraints**: Do not implement writers or proxy orchestration; do not add dependencies; do not commit local paths, endpoints, account material, or real local title names from fact-finding.

## Constitution Check

- **P-1 (No Covert Target Instrumentation)**: PASS. The slice designs local proxy session outputs and explicitly keeps Deep Capture scoped, visible, and reversible. No injection, hooks, target memory reads, traffic interception drivers, Winsock changes, or target TLS key extraction are introduced.
- **P-2 (Core Stays Platform-Neutral)**: PASS. No code changes.
- **P-3 (Capture And Attribution Stay Separate)**: PASS. The design preserves packet truth and attribution in `.fcapng` while application-layer sidecars join through anchors.
- **P-4 (No Silent Loss)**: PASS. Missing artifacts require omission reasons; unsupported inspection is represented explicitly.
- **P-5 (Compatibility Outranks Richness)**: PASS. `.fcapng` remains ordinary pcapng-compatible packet truth; richer application data stays in sidecars.
- **P-6 (Glossary First)**: PASS. "Session bundle", "session manifest", and "correlation anchor" are defined in the slice. No public glossary file is changed in this design-only PR because the terms are not yet introduced to end-user docs.
- **P-8 (House Standards Apply)**: PASS, gated by `cargo xtask lint`.
- **P-9 (The Instrument Does Not Lie)**: PASS. Artifact omission, metadata-only traffic, failed cleanup, and unavailable attribution are explicit.
- **P-10 / P-11**: PASS. The target store remains the target authority, and the master spec is updated with the design decision.
- **Licensing**: PASS. No dependency.

No violation requires Complexity Tracking.

## Project Structure

```text
specs/073-deep-capture-session-bundle/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── analysis.md
├── quickstart.md
├── contracts/
│   ├── manifest.md
│   ├── application-jsonl.md
│   └── example-bundle.md
├── checklists/
│   └── requirements.md
└── tasks.md
```

Source documentation touched by this slice:

```text
docs/fragcap-specification.md
changelog.d/216-deep-capture-session-bundle.decision.md
```

## Complexity Tracking

No constitution violation requires justification; this section is empty by design.
