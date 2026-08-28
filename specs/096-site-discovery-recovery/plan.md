# Implementation Plan: Site Discovery And Recovery

**Branch**: `codex/096-site-discovery-recovery` | **Date**: 2026-08-28 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/096-site-discovery-recovery/spec.md`

## Summary

Correct the two remaining S094 production-site findings in one slice. Add concise current migration guidance for retired `fragcap run` and `fragcap tap` commands, then use the installed static search engine's exact-query pinning API to promote that current command-reference page without deleting historical changelog matches. Replace Next's generic exported not-found body with a branded server-rendered recovery page that reuses the shared skip target and links to the homepage and current getting-started journey. Extend the existing Playwright production-export gate to enforce search ordering, preserved history, current-query relevance, HTTP 404 semantics, recovery navigation, responsive reachability, and browser-error hygiene.

## Technical Context

**Language/Version**: TypeScript 5.9.3, JavaScript modules on Node.js 24, React 19.2.8, Next.js 16.3.0

**Primary Dependencies**: Existing Fumadocs 16.14.3 and `zbsearch` 3.3.4 search engine; promote the already-resolved Apache-2.0 `zbsearch` package to a direct site dependency so its public pinning API is owned explicitly

**Storage**: Static documentation content, serialized client search index, and exported not-found HTML only; no runtime persistence

**Testing**: Existing Playwright 1.62.1 production-export suite, `cargo xtask docs check`, `cargo xtask docs build`, and `cargo xtask ci`

**Target Platform**: Statically exported public documentation site in Chromium at 320 and 1440 pixel recovery widths, plus the existing three-viewport public-route accessibility matrix

**Project Type**: Existing static web application and documentation generator

**Performance Goals**: Exact retired-command searches continue to resolve within the existing client search interaction and production regressions remain within the current documentation workflow

**Constraints**: Preserve historical changelog searchability, all 54 public routes, the real 404 status, current query relevance, existing brand system, shared skip behavior, and all runtime Capture and Deep Capture behavior; no new lockfile package; UTF-8 without BOM; LF endings

**Scale/Scope**: Two exact retired-command queries, four current-query baselines, two absent-path shapes, two recovery destinations, and two not-found viewports

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice changes only the public static site, its search index configuration, and browser tests.
- **P-2 Core Stays Platform-Neutral**: Pass. No Rust crate or dependency direction changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution implementation changes.
- **P-4 No Silent Loss**: Pass. No packet path changes; search tests require nonempty current and historical result populations so removal cannot masquerade as correct ordering.
- **P-5 Compatibility Outranks Richness**: Pass. No capture output format changes.
- **P-6 Glossary First**: Pass. The slice introduces no new project-domain term.
- **P-7 Wrappers Stay Thin**: Pass. No shell wrapper changes.
- **P-8 House Standards Apply**: Pass. New source carries SPDX, the site reuses existing style tokens, and the full conventions gate remains blocking.
- **P-9 The Instrument Does Not Lie**: Pass. Search and not-found claims are observed through the hydrated production export, including actual result activation and HTTP response status.
- **P-10 One Path To A Target**: Pass. No target source or storage changes.
- **P-11 The Specification Describes What Shipped**: Pass. The correction changes no product architecture or release claim; changelog fragments carry `spec-impact: none`.

`zbsearch` 3.3.4 is already in the lock graph through Fumadocs. The direct production edge adds no package or version, is Apache-2.0, requires Node.js 20 or newer against the pinned Node.js 24 build, and exposes the exact-query pinning primitive the static index needs. Exact pinning is preferred to score manipulation because measured current migration prose alone cannot outrank the dense historical page reliably.

## Project Structure

### Documentation (this feature)

```text
specs/096-site-discovery-recovery/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── site-discovery-recovery-contract.md
├── checklists/
│   ├── requirements.md
│   └── discovery-recovery.md
└── tasks.md
```

### Source Code (repository root)

```text
site/
├── app/
│   ├── not-found.tsx
│   └── static.json/route.ts
├── content/docs/reference/cli.mdx
├── tests/production-accessibility.spec.mjs
├── package.json
└── pnpm-lock.yaml

docs/audits/2026-08-28-production-ux-accessibility.md
changelog.d/266-267-site-discovery-recovery.fixed.md
changelog.d/S096-search-pinning.decisions.md
```

**Structure Decision**: Keep current migration guidance in the checked command reference, configure exact query promotion at the existing static index boundary, and use Next's stable root `app/not-found.tsx` convention inside the existing root layout. Do not customize the search dialog, globally demote changelog content, alter generated history, add a second layout, or change the loopback server. Extend the existing production accessibility suite because it already owns Chromium installation, export hosting, route inventory, shared skip behavior, and fatal browser errors.

## Complexity Tracking

No constitution violation or complexity exception is required.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/site-discovery-recovery-contract.md](contracts/site-discovery-recovery-contract.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The completed design still passes all eleven principles. The direct search dependency makes an already-shipped transitive engine API explicit without changing the resolved package graph. Exact-query rules are narrow, preserve historical results, and are exercised through the same serialized index and client dialog deployed to production. The not-found design reuses the root provider and shared skip link, adds no client state, and retains the host-owned HTTP 404 contract.
