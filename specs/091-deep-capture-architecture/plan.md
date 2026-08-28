# Implementation Plan: Deep Capture Architecture and Trust Boundaries

**Branch**: `codex/091-deep-capture-architecture` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/091-deep-capture-architecture/spec.md`

## Summary

Rewrite the passive-only architecture page around two distinct shipped execution views. Keep Capture as passive packet truth with external process attribution, then show Deep Capture as an explicit compatibility-gated composition of prepared Capture, target-scoped launch configuration, a loopback mitmdump child, proxy observations, correlation, manifest-indexed outputs, and audited cleanup. Ground trust, Npcap acquisition, artifact authority, and refusal language in current v0.7.0 behavior and validate the result with focused phrase and link audits, documentation checks, a production static export, and the complete repository gate.

## Technical Context

**Language/Version**: MDX and Mermaid; Rust workspace version 0.7.0 supplies the shipped behavior baseline

**Primary Dependencies**: Existing Fumadocs, Next.js, and Mermaid site toolchain; current `fragcap capture`, `fragcap deep-capture`, and `fragcap doctor --fix` implementations; master specification and glossary

**Storage**: One committed architecture page, S091 specification artifacts, and one unreleased changelog fragment; no runtime storage changes

**Testing**: Architecture phrase and link audits; Mermaid node-count review; `cargo xtask docs check`; `cargo xtask docs build`; `cargo fmt --all -- --check`; and `cargo xtask ci`

**Target Platform**: Statically exported public documentation for the Windows v0.7.0 release

**Project Type**: Documentation correctness slice

**Performance Goals**: No runtime performance impact; both diagrams remain concise and the production export completes under the existing documentation gate

**Constraints**: Documentation-only implementation; no runtime, dependency, workflow, toolchain, or release change; at most twelve primary nodes per diagram; synthetic content only; UTF-8 without BOM; soft-wrapped MDX prose; no prohibited punctuation; no universal inspection or covert target capability claim

**Scale/Scope**: One public architecture page, one changelog fragment, the complete S091 artifact set, two Mermaid execution views, and existing documentation and repository gates

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. The design requires explicit Deep Capture selection, compatibility-gated managed launch, current-user trust authorization, target-scoped proxy configuration, and audited cleanup. It excludes system-wide fallback and every denylisted technique.
- **P-2 Core Stays Platform-Neutral**: Pass. No runtime crate changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. The Capture diagram presents packet acquisition and external attribution as separate evidence paths.
- **P-4 No Silent Loss**: Pass. Default target-scope omissions and named loss accounting remain visible, and absent proxy observations are never promoted to observed application truth.
- **P-5 Compatibility Outranks Richness**: Pass. `.fcapng` remains packet truth for an unmodified analyzer; Deep Capture sidecars add separate evidence.
- **P-6 Glossary First**: Pass. Capture mode, Deep Capture, local inspection proxy, capture scope, session bundle, local development certificate authority, and proxy-owned TLS key-log export already have glossary entries.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. New Markdown and edited MDX use one logical line per paragraph or list item, valid links and fences, UTF-8 without BOM, and no prohibited punctuation.
- **P-9 The Instrument Does Not Lie**: Pass. The design distinguishes packet truth, proxy observations, analyzer aids, compatibility evidence, and audit records, and states inspection and correlation limits directly.
- **P-10 One Path To A Target**: Pass. Deep Capture begins from the existing stored-target selector and prepared managed-launch path.
- **P-11 The Specification Describes What Shipped**: Pass. The page is reconciled to current v0.7.0 code and does not describe future compatibility bootstrap, native proxy, or universal protocol support.

## Project Structure

### Documentation (this feature)

```text
specs/091-deep-capture-architecture/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── architecture-page-contract.md
├── checklists/
│   ├── architecture.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
site/content/docs/architecture.mdx
changelog.d/247-deep-capture-architecture.fixed.md

crates/fragcap-cli/src/
├── commands/deep_capture.rs
├── doctor/action.rs
└── doctor/fix.rs

docs/
├── fragcap-specification.md
└── glossary/
    ├── anti-cheat-and-security.md
    ├── capture-and-networking.md
    ├── file-and-wire-formats.md
    ├── platform-and-distribution.md
    └── process-and-attribution.md
```

**Structure Decision**: Replace the architecture page in place and treat current implementation, tests, the constitution, and the master specification as behavioral authorities. Link the output-format reference only for ordinary Capture formats because issue #248 owns its Deep Capture correction; give the architecture page enough bundle authority detail to remain truthful without endorsing that page as a complete Deep Capture artifact reference.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/architecture-page-contract.md](contracts/architecture-page-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. It keeps passive Capture and active Deep Capture visibly separate, makes every security-sensitive side effect explicit and auditable, preserves packet truth as the analyzer-compatible baseline, and adds no runtime or dependency surface.
