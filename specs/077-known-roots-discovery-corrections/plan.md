# Implementation Plan: Known-Roots Discovery Corrections

**Branch**: `codex/077-known-roots-discovery-corrections` | **Date**: 2026-08-26 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/077-known-roots-discovery-corrections/spec.md`

## Summary

Correct the tier-2 known-roots walk in two connected places. `SignatureClassifier` will return a container control verdict when a scan observes more than one distinct engine product. `KnownRootsSource` will suppress that directory as a candidate, descend while the existing shallow bound permits, and account separately for traversed and depth-limited containers. `FsDirectoryLister` will convert the shared separator-neutral root to native form exactly at the real filesystem boundary so its returned child paths never preserve a mixed prefix. No dependency, schema migration, depth expansion, or rewrite of existing local rows is introduced.

## Technical Context

**Language/Version**: Rust 2021 edition, minimum supported Rust 1.82

**Primary Dependencies**: Standard library plus the existing `fragcap-profile` signature types; no new dependency

**Storage**: No schema change. Newly discovered candidate paths can later enter the existing SQLite local store through the unchanged persist-on-first-use path.

**Testing**: Rust unit and integration tests, temporary real filesystem trees, repository `cargo xtask ci` gate

**Target Platform**: Windows 10/11 production path; portable fixture and temporary-tree tests on CI hosts

**Project Type**: Rust workspace with library, facade, and CLI crates

**Performance Goals**: Preserve the current depth bound and stop-on-hit behavior for title directories. Container handling adds one distinct-engine count over findings already collected by the bounded signature scan.

**Constraints**: No deep scan, no path canonicalization that resolves links or touches the filesystem beyond the existing walk, no database rewrite, no real local title or PII in fixtures, and total discovery accounting under P-4

**Scale/Scope**: One classifier verdict, two discovery counters, one known-roots control path, one CLI account line, master-spec/glossary reconciliation, and focused regression tests

## Constitution Check

- **P-1 No Covert Target Instrumentation**: PASS. This is read-only filesystem discovery over existing permitted paths and does not interact with a running target.
- **P-2 Core Stays Platform-Neutral**: PASS. Changes remain in `fragcap-targets` and `fragcap-cli`; `fragcap-core` is untouched.
- **P-3 Capture And Attribution Stay Separate**: PASS. No capture or attribution code changes.
- **P-4 No Silent Loss**: ACTION. Add distinct `container_descended` and `container_descent_truncated` outcomes to `DiscoveryAccount`, merge and render both, and emit one warning per truncated container.
- **P-5 Compatibility Outranks Richness**: PASS. No capture format changes.
- **P-6 Glossary First**: ACTION. Update the existing known-roots and stop-on-hit entries and add the container-verdict term before final verification.
- **P-7 Wrappers Stay Thin**: PASS. No wrapper changes.
- **P-8 House Standards Apply**: PASS. Rustfmt, Clippy, Markdown, encoding, and repository lint remain mandatory.
- **P-9 The Instrument Does Not Lie**: ACTION. Never emit a multi-engine aggregate directory as one title; retain incomplete-scan warnings and report depth-limited coverage rather than implying complete discovery.
- **P-10 One Path To A Target**: PASS. Corrected candidates retain the existing `CandidateTarget` and persistence path.
- **P-11 The Specification Describes What Shipped**: ACTION. Reconcile specification section 7.1 with the container-aware descent contract and expanded account outcomes.

Post-design re-check: PASS with the P-4, P-6, P-9, and P-11 actions represented in contracts and tasks. No constitutional exception is requested.

## Project Structure

### Documentation (this feature)

```text
specs/077-known-roots-discovery-corrections/
├── checklists/
│   ├── discovery-correctness.md
│   └── requirements.md
├── contracts/
│   ├── classifier-verdict.md
│   ├── discovery-account.md
│   └── path-composition.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/
├── src/
│   ├── classifier.rs
│   ├── source.rs
│   └── sources/
│       ├── known_roots.rs
│       └── mod.rs
└── tests/
    ├── detection_walk.rs
    └── known_roots.rs

crates/fragcap-cli/
└── src/commands/targets.rs

docs/
├── fragcap-specification.md
└── glossary/
    ├── index.md
    └── process-and-attribution.md

changelog.d/
└── 209-210-known-roots-discovery.fixed.md
```

**Structure Decision**: Extend the existing classifier, discovery-account, walker, filesystem-lister, and CLI rendering boundaries. The container decision belongs in `SignatureClassifier`, where the complete finding set is already available; traversal and accounting remain in `KnownRootsSource`; account aggregation remains in `source.rs`; native separator conversion belongs in `FsDirectoryLister`, the one implementation that crosses the real filesystem boundary. Converting in the generic walker was rejected after design because it would force Windows-native fixture keys and break the separator-neutral test seam. This preserves the established source seam and adds no inter-crate edge.

## Complexity Tracking

No constitution violation or exceptional complexity is introduced.
