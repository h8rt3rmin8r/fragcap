# Implementation Plan: Readable Steam Title Listing

**Branch**: `codex/136-steam-list-layout` | **Date**: 2026-09-09 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/136-steam-list-layout/spec.md`

## Summary

Close issue #374 by replacing `steam list` tab separators with one width-aware human listing. Reuse S135's shared display-cell accounting and terminal-width policy, render an aligned four-column table only when every complete row fits, and otherwise render the complete result as labeled vertical records. Preserve every value, visibly escape layout controls, and leave Steam discovery, target identity, ordering, JSON, diagnostics, exits, and local storage untouched.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.88

**Primary Dependencies**: Existing `fragcap-cli`, `fragcap` facade Steam and target APIs, and Windows-only transitive `terminal_size`; no new dependency or lockfile package

**Storage**: Existing local target store and listing snapshot are read only; no schema or write-path change

**Testing**: Renderer unit tests, CLI contract tests, existing Steam identity and read-only regressions, full Cargo workspace and xtask gates

**Target Platform**: Windows interactive terminals plus deterministic redirected and unsupported-host behavior

**Project Type**: Rust workspace CLI plus facade library

**Performance Goals**: Linear width calculation and rendering over enumerated rows and text length; at most one terminal-width query per human invocation

**Constraints**: No tab output, no value truncation, 40-display-column minimum, 80-column maximum and fallback, one layout per listing, JSON compatibility, deterministic ordering, read-only storage behavior, no new package

**Scale/Scope**: One Steam human renderer, shared width selection and control-display helpers, focused contracts and tests, documentation and changelog synchronization

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1 No covert target instrumentation**: Pass. S136 only formats already enumerated titles and resolved identities; it adds no process, capture, routing, trust, or effect capability.
- **P-2 Core stays platform-neutral**: Pass. Presentation remains in `fragcap-cli`; no change enters `fragcap-core`.
- **P-3 Capture and attribution stay separate**: Pass. Neither boundary changes.
- **P-4 No silent loss**: Pass. Human values are never truncated or omitted, and JSON observations remain unchanged.
- **P-5 Compatibility outranks richness**: Pass. The stable JSON interface, diagnostics, exits, and storage behavior remain unchanged. Only the defective human byte shape changes.
- **P-6 Glossary first**: Pass. Steam title, app id, target, display width, and terminal are established vocabulary; no new domain term is introduced.
- **P-7 Wrappers stay thin**: Pass. No wrapper changes.
- **P-8 House standards apply**: Pass. UTF-8 without BOM, LF, format, lint, tests, docs, changelog, and shell checks remain blocking.
- **P-9 The instrument does not lie**: Pass. Every value is preserved; the three layout controls are represented explicitly rather than interpreted as hidden syntax, and JSON stays verbatim.
- **P-10 One path to a target**: Pass. Target resolution, identity, and storage do not change.
- **P-11 The specification describes what shipped**: Pass. The master specification, outline, S067 contract, roadmap, site reference, and changelog move with the renderer.

Post-design re-check: passed. The design creates no new data entity or state transition. One pure presentation row feeds either complete layout, shared helpers own display-cell and width policy, and the existing JSON renderer is not modified. No constitution exception is introduced.

## Project Structure

### Documentation (this feature)

```text
specs/136-steam-list-layout/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- steam-list-human.md
|-- checklists/
|   |-- requirements.md
|   `-- ux.md
`-- tasks.md
```

### Source Code (repository root)

```text
crates/fragcap-cli/src/
|-- display.rs                         # shared display-cell, escaping, and stdout-width policy
|-- commands/
|   |-- doctor.rs                     # consume shared stdout-width selection
|   `-- steam.rs                      # pure table-or-vertical human renderer
`-- lib.rs

crates/fragcap-cli/tests/
`-- cli_steam.rs                      # human and JSON surface regressions

specs/067-steam-list-identity-json/contracts/
`-- steam-list-cli.md                 # original output contract reconciled to S136

docs/
|-- fragcap-specification.md
|-- fragcap-spec-outline.md
`-- plans/README.md

site/content/docs/reference/
`-- cli.mdx

changelog.d/
|-- 374-steam-list-layout.fixed.md
`-- s136-steam-list-layout.decisions.md
```

**Structure Decision**: Keep enumeration, identity resolution, ordering, and JSON in their current command module. Add a small presentation row so width calculation and both human layouts consume identical strings. Extend the existing crate-private display module with the control representation and width-selection policy already needed by Doctor, then make both commands consume that single policy.

## Implementation Sequence

1. Add failing renderer tests for aligned short rows, vertical long and narrow rows, localized display widths, embedded controls, all identity states, ordering, and absence of tabs.
2. Move stdout-width selection from Doctor into the shared display module without changing Doctor output or width behavior.
3. Add one presentation-row mapping and implement the complete-table fit decision, aligned table, and labeled vertical listing.
4. Preserve the existing JSON renderer, diagnostics, empty state, ordering, and storage paths; strengthen integration assertions against human tabs and JSON drift.
5. Reconcile the S067 contract, master specification, outline, roadmap, site CLI reference, changelog, and quickstart.
6. Run artifact analysis, focused tests, full CI parity, dependency stability, encoding and mojibake checks, and final diff review.

## Complexity Tracking

No constitution violations require justification.
