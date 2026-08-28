# Implementation Plan: Production Accessibility Remediation

**Branch**: `codex/095-production-accessibility-remediation` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/095-production-accessibility-remediation/spec.md`

## Summary

Correct four related production-site accessibility findings in one coherent slice: establish one shared skip link and exactly one primary landmark per route, normalize generated changelog headings after their category becomes the page title, raise the two failing light-theme text colors above 4.5:1, and give both architecture diagrams Mermaid-native accessible titles. Add a Playwright regression over the built static export so hydrated diagrams, computed contrast, focus transfer, headings, and route-wide landmarks are enforced rather than retained only as audit prose.

## Technical Context

**Language/Version**: TypeScript 5.9.3, JavaScript modules on Node.js 24, React 19.2.8, Next.js 16.3.0

**Primary Dependencies**: Existing Fumadocs 16.14.3 and Mermaid 11.16.1; new development-only `@playwright/test` 1.62.1 (Apache-2.0, Node.js 20 or newer)

**Storage**: Static source, generated documentation, and browser-test evidence only; no runtime persistence

**Testing**: Dependency-free prebuild transformation checks, Playwright against `site/out`, existing deterministic DOM evaluator, `cargo xtask docs check`, `cargo xtask docs build`, and `cargo xtask ci`

**Target Platform**: Statically exported public documentation site in Chromium at 320, 768, and 1440 pixel widths

**Project Type**: Existing static web application and documentation generator

**Performance Goals**: Complete the 54-route, three-viewport accessibility regression within the documentation workflow without changing production runtime behavior beyond the corrections

**Constraints**: Preserve all routes, content, anchors, diagrams, dark-theme values, search behavior, and not-found behavior; no new production dependency; UTF-8 without BOM; LF endings; pinned workflow changes require a dated decision fragment

**Scale/Scope**: Four home-layout routes, 50 documentation routes, every generated changelog page, three affected light-theme color pairs, and two architecture diagrams

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice changes only the public static documentation site and its tests.
- **P-2 Core Stays Platform-Neutral**: Pass. No Rust crate or dependency direction changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution implementation changes.
- **P-4 No Silent Loss**: Pass. No packet path changes; the browser runner also requires nonzero matched contrast and diagram populations so missing subjects cannot become false passes.
- **P-5 Compatibility Outranks Richness**: Pass. No capture output format changes.
- **P-6 Glossary First**: Pass. The slice introduces no new project-domain vocabulary.
- **P-7 Wrappers Stay Thin**: Pass. No shell wrapper changes.
- **P-8 House Standards Apply**: Pass. New source files carry SPDX headers, generated Markdown remains canonical, and all repository gates remain blocking.
- **P-9 The Instrument Does Not Lie**: Pass. The regression observes the built site, fails on missing populations, and does not infer hydrated or computed behavior from source alone.
- **P-10 One Path To A Target**: Pass. No target model or source changes.
- **P-11 The Specification Describes What Shipped**: Pass. The correction does not change product architecture or release claims; changelog fragments carry `spec-impact: none`.

The new Playwright edge is development-only and adds four package records: the Apache-2.0 `@playwright/test`, `playwright`, and `playwright-core` packages, plus Playwright's optional MIT-licensed macOS `fsevents` record. Chromium is downloaded separately for the test runner. The edge is proportional because the acceptance contract requires hydrated SVG semantics, sequential focus behavior, and computed colors, none of which static HTML establishes.

## Project Structure

### Documentation (this feature)

```text
specs/095-production-accessibility-remediation/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── production-accessibility-contract.md
├── checklists/
│   ├── requirements.md
│   └── ux.md
└── tasks.md
```

### Source Code (repository root)

```text
site/
├── app/
│   ├── layout.tsx
│   ├── global.css
│   ├── (home)/
│   │   ├── layout.tsx
│   │   ├── page.tsx
│   │   ├── brand/page.tsx
│   │   ├── disclaimer/page.tsx
│   │   └── license/page.tsx
│   └── docs/[[...slug]]/page.tsx
├── content/docs/architecture.mdx
├── components/skip-link.tsx
├── scripts/
│   ├── prebuild.mjs
│   ├── changelog-headings.mjs
│   └── audit-export-dom.mjs
├── tests/
│   ├── changelog-headings.test.mjs
│   └── production-accessibility.spec.mjs
├── playwright.config.mjs
├── package.json
└── pnpm-lock.yaml

.github/workflows/docs.yml
docs/audits/2026-08-28-production-ux-accessibility.md
changelog.d/263-268-production-accessibility.fixed.md
changelog.d/S095-browser-accessibility-gate.decisions.md
```

**Structure Decision**: Keep the corrections at the existing shared layout, generator, theme, and authored-diagram seams. Use one small client skip-link component because Next's production router intercepts a plain fragment anchor and attempts an unavailable static RSC request. Isolate heading normalization in an import-safe dependency-free module so synthetic hierarchy cases receive focused Node tests. Keep rendered regression code under `site/tests/`, reuse the committed loopback export server and DOM evaluator, and add both test layers to the existing documentation workflow after the static export is built. Do not move production code or introduce a second audit implementation.

## Complexity Tracking

No constitution violation or complexity exception is required.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/production-accessibility-contract.md](contracts/production-accessibility-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. Browser automation is a development-only verification dependency, has an allowlisted license, does not enter the shipped site graph, and prevents source-level assertions from being misreported as rendered behavior. The pinned documentation workflow change receives the required dated decisions fragment.
