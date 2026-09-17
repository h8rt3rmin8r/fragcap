# Implementation Plan: S157 complete native documentation contract

**Branch**: `codex/s157-native-documentation-contract` | **Date**: 2026-09-17 UTC | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/157-native-documentation-contract/spec.md`

## Summary

Complete the agent-runnable #331 documentation baseline under child #427. Upgrade S154's page-level readiness inventory to a version-two thirteen-topic contract with required sections and example authorities, rewrite the stable native-documentation route as the current product contract index, reconcile current pages against published v0.10.1 and merged verification work, and prove commands, artifact examples, site quality, and status boundaries through existing production authorities.

## Technical Context

**Language/Version**: Existing Rust workspace at MSRV 1.88, JSON contract data, MDX/Next.js documentation site, pinned Node and pnpm tools.

**Primary Dependencies**: Existing standard library, serde_json, shared Cargo test discovery, fragcap CLI command tree, product artifact readers, Fumadocs/Next.js site, and Playwright accessibility harness. No new dependency.

**Storage**: One versioned committed JSON documentation registry and authored Markdown/MDX. No product persistence change.

**Testing**: Focused xtask contract unit tests, CLI reference parser in default and network-capable configurations, manifest readers, capture goldens, native bundle conformance, public API tests, documentation lint, static site unit/build/accessibility/search/link checks, and `cargo xtask ci`.

**Target Platform**: Offline and controlled Linux/Windows CI plus static browser tests. No installed product or real target execution.

**Project Type**: Rust workspace, Windows CLI/library product, and statically exported documentation site.

**Performance Goals**: Documentation validation stays bounded to repository files smaller than the existing one-megabyte limit and adds no network dependency to ordinary CI.

**Constraints**: Preserve P-1, P-6, P-8, P-9, and P-11; no runtime/dependency/schema/version/release/policy behavior change; no installed sensitive product, game, live capture, elevation, or trust mutation; no independent acceptance claim.

**Scale/Scope**: Thirteen closed documentation topics, one command corpus, a bounded artifact-authority set, current README and site pages, one xtask validator module, existing CLI and site test surfaces, slice/changelog/specification records.

## Constitution Check

Pre-research PASS. P-1 is preserved because verification is static or controlled and dispatches no product effects. P-5 keeps packet examples tied to unmodified analyzer formats. P-6 introduces no unexplained domain term and uses existing glossary vocabulary. P-8 governs Markdown and text hygiene. P-9 requires exact observed artifact, loss, refusal, and status language. P-11 binds published v0.10.1 separately from current main. Spec Kit sequence and foreground verification remain mandatory.

Post-design PASS. Registry version two strengthens truth and traceability without changing product architecture. Actual reviewer independence and installed QUIC evidence remain external, so #333/#413/#331/#334/#278 cannot close in this slice. No constitution exception or complexity waiver is needed.

## Phase 0: Research

[Research](research.md) records the published/current boundary, registry version, command and artifact authorities, stable reader-facing route, and host-execution limits. All material choices resolve from current issue, specification, source, tests, and S154 through S156 evidence; no clarification remains.

## Phase 1: Design

The [data model](data-model.md) defines the registry, topic, test, example, and completion-boundary entities. The [version-two contract](contracts/documentation-contract-v2.md) defines closure, section ownership, command parsing, artifact authority, status truth, and exclusions. The [quickstart](quickstart.md) supplies bounded validation commands.

Test-driven implementation starts by expressing version-two validation and mutation failures against the current version-one registry. Then replace the registry, update the stable contract page and bounded current guidance, extend site search/link assertions, and reconcile specification and delivery records. Existing parser and product readers remain the execution authorities.

## Project Structure

```text
specs/157-native-documentation-contract/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/documentation-contract-v2.md
├── checklists/requirements.md
├── checklists/documentation.md
└── tasks.md

docs/audits/native-documentation-coverage.v2.json
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
site/content/docs/
├── index.mdx
├── architecture.mdx
├── getting-started.mdx
├── guides/
└── reference/
xtask/src/docs_coverage.rs
crates/fragcap-cli/tests/cli_reference.rs
site/tests/production-accessibility.spec.mjs
README.md
CONTRIBUTING.md
changelog.d/
```

**Structure Decision**: Keep documentation content, static-site tests, and repository validators in their existing ownership boundaries. The registry is data under `docs/audits`; xtask owns its structural validation; the production CLI test owns command parsing; product tests own artifact readers; Playwright owns exported-site behavior.

## Execution and Verification

Generate dependency-ordered tasks, run the mandatory read-only cross-artifact analysis, and resolve any finding before implementation. Implement tests before the version-two registry and prose updates. Run focused gates, production site checks, then complete `cargo xtask ci` in the foreground. Commit locally and halt before push because this kickoff did not explicitly authorize publication.

## Complexity Tracking

No constitution violations or new product architecture.
