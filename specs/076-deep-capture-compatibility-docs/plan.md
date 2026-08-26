# Implementation Plan: Deep Capture Compatibility Documentation

**Branch**: `codex/220-deep-capture-compatibility-docs` | **Date**:
2026-08-26 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from
`specs/076-deep-capture-compatibility-docs/spec.md`

## Summary

Publish an exact traffic-support reference and expose the selected target's
local compatibility evidence through `fragcap targets show`. A pure projection
in `fragcap-targets` will convert stored facts into deterministic matrix rows
with explicit current, stale, and unknown states. The CLI will render that
projection without inference or side effects. Public documentation will explain
Capture and Deep Capture behavior for the seven required traffic families,
evidence provenance, refresh behavior, and privacy boundaries.

## Technical Context

**Language/Version**: Rust 1.82 minimum; MDX documentation

**Primary Dependencies**: Existing `fragcap-targets`, `fragcap-cli`, SQLite
store, serde-based test support, and Fumadocs site; no new dependency

**Storage**: Existing local SQLite `deep_capture_facts` rows; no schema change

**Testing**: Rust unit tests, CLI integration tests, documentation lint and site
build, workspace gates

**Target Platform**: Windows CLI and static documentation site

**Project Type**: Multi-crate Rust CLI with a static documentation site

**Performance Goals**: Render all facts for one selected target in one store
read; compatibility display must not launch processes or perform network work

**Constraints**: Read-only projection, deterministic output, no inferred
verdict, no PII or real local game titles in committed artifacts, no new
dependency, no database migration

**Scale/Scope**: One target detail view, seven traffic-family reference rows,
four evidence sources, and three presentation freshness states

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1 No covert target instrumentation**: Pass. The feature reads existing
  facts and documentation only. Viewing compatibility starts no target, proxy,
  trust mutation, probe, or network operation.
- **P-2 Core stays platform-neutral**: Pass. The projection belongs to
  `fragcap-targets`; no platform code enters `fragcap-core`.
- **P-3 Capture and attribution stay decoupled**: Pass. No capture or
  attribution dependency changes.
- **P-4 No silent loss**: Pass. Empty, repeated, conflicting, stale, and unknown
  facts remain visible rather than being collapsed.
- **P-5 Compatibility over richness**: Pass. The display is additive to the
  existing target detail surface and introduces no new runtime dependency.
- **P-6 Glossary first**: Pass. Deep Capture already has a canonical glossary
  entry. The feature uses existing terms and adds no new formal subsystem term.
- **P-7 Dependency restraint**: Pass. Existing standard-library sorting and
  formatting are sufficient.
- **P-8 Testability is architecture**: Pass. The projection is pure, CLI tests
  use temporary stores, and documentation is checked without a live game or
  proxy.
- **P-9 The instrument does not lie**: Pass. Rows preserve source, launch case,
  and freshness; no aggregate compatibility verdict is invented.
- **P-10 One path to a target**: Pass. `targets show` uses the existing selector
  and target store.
- **P-11 Specification is authoritative**: Pass when section 15 records the
  projection and section 19.6 records the traffic table's exact boundaries.

Post-design re-check: all gates still pass. The pure projection narrows rather
than expands the data path, and the public page contains no local target data.

## Project Structure

### Documentation (this feature)

```text
specs/076-deep-capture-compatibility-docs/
├── checklists/
│   ├── requirements.md
│   └── truth-and-privacy.md
├── contracts/
│   ├── compatibility-matrix.md
│   └── traffic-support.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-targets/src/
├── compatibility.rs
└── lib.rs

crates/fragcap-cli/src/commands/targets.rs
crates/fragcap-cli/tests/cli_targets.rs

site/content/docs/
├── meta.json
└── reference/
    ├── cli.mdx
    └── deep-capture-compatibility.mdx

docs/fragcap-specification.md
changelog.d/220-deep-capture-compatibility.added.md
```

**Structure Decision**: Keep compatibility meaning and ordering in the existing
target-domain crate, keep store access and human formatting in the CLI command,
and publish the protocol reference beside the existing CLI and output-format
reference pages. No new crate or site component is needed.

## Complexity Tracking

No constitution violations require justification.
