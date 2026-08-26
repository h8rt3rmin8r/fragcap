# Implementation Plan: Homepage Positioning And Next-Command Footer

**Branch**: `codex/078-homepage-positioning-footer` | **Date**: 2026-08-26 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/078-homepage-positioning-footer/spec.md`

## Summary

Correct two connected presentation defects. The Rust target listing will keep its existing row-selection algorithm and replace the bare suggested command with one exact labelled footer. The static homepage will replace stale passive-only and absolute claims with precise Capture and Deep Capture positioning, correct dependency roles, and a synthetic specimen matching the current CLI. The master specification will supersede S057's frozen-copy requirement without rewriting historical artifacts.

## Technical Context

**Language/Version**: Rust 1.82 minimum; TypeScript 5.9 and React 19 in the existing static site

**Primary Dependencies**: Existing Rust standard-library formatting, `fragcap-cli`, Next.js/Fumadocs site dependencies; no new dependency

**Storage**: N/A; no database or schema change

**Testing**: Rust unit and CLI integration tests, repository documentation checks, production static-site build, source and generated-output claim scans, full workspace CI

**Target Platform**: Windows CLI and cross-platform static documentation build

**Project Type**: Multi-crate Rust CLI plus static Next.js documentation site

**Performance Goals**: No measurable runtime change; one formatting literal changes and the site remains statically generated

**Constraints**: Preserve next-row selection, bare-versus-explicit output invariant, empty listing, site structure, one primary action, Npcap obligations, and mode-specific safety; no real title names or local data

**Scale/Scope**: One Rust rendering line and focused tests; one homepage; three master-specification sections; one changelog fragment

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1 No covert target instrumentation**: Pass. The slice changes text rendering and documentation only. It adds no capture, proxy, trust, launch, or process behavior.
- **P-2 Core stays platform-neutral**: Pass. `fragcap-core` is untouched.
- **P-3 Capture and attribution stay separate**: Pass. Neither interface changes.
- **P-4 No silent loss**: Pass. The target row and machine findings are unchanged; the next command becomes more explicit.
- **P-5 Compatibility outranks richness**: Pass. Analyzer-compatible output and Capture behavior are untouched.
- **P-6 Glossary first**: Pass. The slice uses existing terms and introduces no new domain term.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes.
- **P-8 House standards apply**: Pass. Rust, TypeScript, Markdown, and changelog files remain under existing lint and build gates.
- **P-9 The instrument does not lie**: Pass. The homepage removes absolute claims, and the CLI labels rather than alters its existing recommendation.
- **P-10 One path to a target**: Pass. Target creation, storage, resolution, and row selection are unchanged.
- **P-11 Specification describes what shipped**: Pass when sections 17.7, 23.1, and 23.3 match the CLI and site in the same change.

Post-design re-check: all gates pass. The presentation contracts narrow claims to shipped behavior and add no capability.

## Project Structure

### Documentation (this feature)

```text
specs/078-homepage-positioning-footer/
├── checklists/
│   ├── positioning-correctness.md
│   └── requirements.md
├── contracts/
│   ├── homepage-positioning.md
│   ├── homepage-specimen.md
│   └── target-next-command.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/tests/cli_targets.rs
site/app/(home)/page.tsx
docs/fragcap-specification.md
changelog.d/208-232-homepage-footer.fixed.md
```

**Structure Decision**: Keep selection and rendering in the existing target command, keep homepage content in the existing page component, and reconcile the architecture of record directly. No shared generator is introduced for a two-row static specimen because doing so would couple the Node site build to a Windows-oriented CLI invocation and local target store.

## Complexity Tracking

No constitution violation requires justification.
