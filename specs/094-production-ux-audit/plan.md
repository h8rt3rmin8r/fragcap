# Implementation Plan: Production UX And Accessibility Audit

**Branch**: `codex/094-production-ux-audit` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `specs/094-production-ux-audit/spec.md`

## Summary

Build the locked production documentation export, serve that immutable output locally, reconcile and exercise its complete route inventory, audit representative and route-specific behavior across keyboard, semantic, search, theme, zoom, and required viewport modes, then commit a durable evidence report. Material defects become narrow GitHub issues only after overlap search. S094 records findings and limitations; it does not fold corrective site changes into the audit branch.

## Technical Context

**Language/Version**: Existing Next.js 16.3.0 and React 19.2.8 site, Rust workspace pinned by `rust-toolchain.toml`, Markdown audit artifacts
**Primary Dependencies**: Locked pnpm dependency graph, Fumadocs 16.14.3, Mermaid 11.16.1, in-app Chromium browser inspection
**Storage**: One durable Markdown report plus S094 specification artifacts and one changelog fragment; no runtime storage
**Testing**: `pnpm install --frozen-lockfile`, production static export, local static server, browser route and interaction inspection, DOM and computed-style evidence, existing documentation checks, `cargo xtask ci`
**Target Platform**: Production static documentation site, Chromium at 320 px, 768 px, and 1440 px viewports
**Project Type**: Documentation audit over an existing static web application
**Performance Goals**: No performance optimization claim; record production build completion and observable usability only
**Constraints**: Audit-only scope; no unrelated UI fixes; no dependency or lockfile drift; no claim for an unperformed native screen-reader or operating-system high-contrast session; UTF-8 without BOM; LF endings
**Scale/Scope**: Homepage, three informational home routes, every generated documentation and glossary route, not-found behavior, shared navigation/search/theme/footer surfaces, every complex-content instance

## Constitution Check

*GATE: Must pass before research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. S094 exercises a local static documentation export and touches no target process, network capture path, trust store, or proxy session.
- **P-2 Core Stays Platform-Neutral**: Pass. No crate dependency or platform abstraction changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution implementation changes.
- **P-4 No Silent Loss**: Pass. Every required audit check receives passed, failed, or not-run state, preventing silent evidence loss.
- **P-5 Compatibility Outranks Richness**: Pass. No output format changes.
- **P-6 Glossary First**: Pass. The audit introduces no product-domain term; any finding reuses existing vocabulary.
- **P-7 Wrappers Stay Thin**: Pass. No wrapper changes.
- **P-8 House Standards Apply**: Pass. Markdown, encoding, and repository gates cover the committed report and artifacts.
- **P-9 The Instrument Does Not Lie**: Pass. Limitations and unavailable checks are disclosed, and failed observations cannot be rewritten as passes.
- **P-10 One Path To A Target**: Pass. No target entry or discovery change.
- **P-11 The Specification Describes What Shipped**: Pass. The production v0.7.0 site is the audit subject; S094 does not change release claims.

## Project Structure

### Documentation (this feature)

```text
specs/094-production-ux-audit/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── audit-report-contract.md
├── checklists/
│   ├── requirements.md
│   └── ux.md
└── tasks.md
```

### Source Code (repository root)

```text
docs/audits/2026-08-28-production-ux-accessibility.md
changelog.d/249-production-ux-audit.fixed.md
```

**Structure Decision**: Keep the final report under `docs/audits/`, where durable project evidence can be reviewed without entering the public documentation route tree. Keep browser screenshots as transient working evidence unless a compact image is necessary to reproduce a finding; route, viewport, DOM, computed-style, and issue evidence belongs directly in the report. Do not change the site application during the audit.

## Complexity Tracking

No constitution violation or complexity exception is required.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/audit-report-contract.md](contracts/audit-report-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. The report contract makes coverage reconciliation and limitations explicit, keeps findings separate from corrections, adds no dependency or runtime behavior, and binds every material defect to one searched disposition.
