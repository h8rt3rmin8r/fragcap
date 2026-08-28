# Implementation Plan: Public Entry Point Reconciliation

**Branch**: `codex/088-public-entry-points` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/088-public-entry-points/spec.md`

## Summary

Fix issue #244 by reconciling the repository landing page, contributor guides, public documentation index, GitHub issue forms, and repository description with v0.7.0. All surfaces will share the mode boundary established by the constitution and master specification: Capture is passive process-attributed packet capture; Deep Capture is shipped, explicit, target-scoped, reversible local proxy inspection for authorized sessions and compatible traffic. The implementation also corrects narrow v0.7.0 status contradictions in the master specification discovered during planning, while leaving the deeper walkthrough, CLI reference, architecture diagrams, bundle reference, and rendered-site audit to issues #245 through #249.

## Technical Context

**Language/Version**: Markdown, MDX, GitHub issue-form YAML, and GitHub repository metadata; Rust workspace version 0.7.0 supplies the release baseline

**Primary Dependencies**: Existing Fumadocs and Next.js site toolchain, repository documentation gates, GitHub CLI for repository metadata

**Storage**: Committed documentation files plus one GitHub repository-description field; no runtime storage changes

**Testing**: Focused phrase and link audits, issue-form YAML parsing, current CLI help comparison, `cargo xtask docs check`, `cargo xtask docs build`, `cargo xtask spec`, and `cargo xtask ci`

**Target Platform**: GitHub repository surfaces and the statically exported documentation site; product claims describe Windows v0.7.0

**Project Type**: Documentation and repository-metadata correctness slice

**Performance Goals**: No runtime performance impact; the existing production static export must complete successfully

**Constraints**: Preserve historical slice records; no runtime, command grammar, dependency, workflow, toolchain, or release-configuration changes; no universal attribution or inspection claims; no Npcap bundling or redistribution implication; UTF-8 without BOM and no mojibake

**Scale/Scope**: Six committed public-entry-point files, narrow master-spec status corrections, one repository-description field, S088 artifacts, and one changelog fragment

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. The shared definition keeps Capture passive and describes Deep Capture only as explicit, target-scoped, reversible local proxy inspection. The complete technique denylist remains intact.
- **P-2 Core Stays Platform-Neutral**: Pass. No runtime crate changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution implementation changes.
- **P-4 No Silent Loss**: Pass. No packet or discovery accounting changes; public wording retains compatibility limits rather than claiming complete inspection.
- **P-5 Compatibility Outranks Richness**: Pass. No output format changes.
- **P-6 Glossary First**: Pass. Capture, Deep Capture, Npcap, target, and local proxy already have glossary entries.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. Markdown, MDX, and YAML remain UTF-8 without BOM, soft-wrapped where edited, structurally valid, and free of prohibited punctuation.
- **P-9 The Instrument Does Not Lie**: Pass. Public claims distinguish supported inspection from universal decryption and preserve the authority difference between packet capture and proxy observations.
- **P-10 One Path To A Target**: Pass. Current examples use stored-target selectors and do not introduce a second target shape.
- **P-11 The Specification Describes What Shipped**: Pass with required correction. The implementation updates both public entry points and the narrow stale current-status statements found in the v0.7.0 master specification.

## Project Structure

### Documentation (this feature)

```text
specs/088-public-entry-points/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── public-entry-points.md
├── checklists/
│   ├── public-claims.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
README.md
CONTRIBUTING.md
site/content/docs/index.mdx
site/content/docs/contributing.mdx
.github/ISSUE_TEMPLATE/bug_report.yml
.github/ISSUE_TEMPLATE/feature_request.yml
docs/fragcap-specification.md
changelog.d/244-public-entry-points.fixed.md
```

**External metadata**: The GitHub repository description is changed from its passive-only sentence to the exact contract recorded in `contracts/public-entry-points.md`.

**Structure Decision**: Correct each first-contact surface in place and keep the repository contributor guide canonical. The site contributor page stays a concise public summary linked to that canonical guide. The master-spec change is limited to stale present-tense status and release-roadmap statements; issue #247 retains the Deep Capture execution and trust-boundary documentation.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/public-entry-points.md](contracts/public-entry-points.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The design still passes every constitution gate. The narrow master-spec correction is a required P-11 repair discovered during planning, not a takeover of the architecture or reference work owned by later issues.
