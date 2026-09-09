# Tasks: Target Readiness Groups

**Input**: Design documents from `/specs/138-target-readiness-groups/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/hero-listing.md

**Tests**: Listing-shape, selector-integrity, and stable-interface tests are mandatory and precede implementation.

## Phase 1: Setup and Contract Guards

- [x] T001 Add and validate S138 specification, plan, research, data model, contract, quickstart, and requirement checklists in `specs/138-target-readiness-groups/`
- [x] T002 Record the exact grouped-listing contract and stable-interface boundaries in `specs/138-target-readiness-groups/contracts/hero-listing.md`

---

## Phase 2: Foundational Ordering Authority

- [x] T003 Add failing focused helper tests for readiness rank, ready-first final order, and deterministic within-group handle order in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T004 Implement one final target ordering and readiness boundary in `crates/fragcap-cli/src/commands/targets.rs`

**Checkpoint**: One complete vector authorizes rendering, numbering, snapshot persistence, and footer selection.

---

## Phase 3: User Story 1 - See Capturable Targets First (Priority: P1)

**Goal**: Present ready targets first under plain-language headings without hiding or distorting any row.

**Independent Test**: Render a mixed fixture with crossing handle order and wide cells, then prove exact headings, group order, within-group order, independent widths, and complete evidence.

- [x] T005 [US1] Add failing mixed-listing integration coverage for exact headings, ready-first grouping, handle order, independent widths, and complete cells in `crates/fragcap-cli/tests/cli_targets.rs`
- [x] T006 [US1] Add failing renderer unit coverage for global row offsets and independent number, target, and engine widths in `crates/fragcap-cli/src/commands/targets.rs`
- [x] T007 [US1] Implement non-empty group headings, separation, and offset-aware per-group table rendering in `crates/fragcap-cli/src/commands/targets.rs`

**Checkpoint**: Mixed human output makes the immediate action class visually first and preserves every target fact.

---

## Phase 4: User Story 2 - Keep Numeric Selection Truthful (Priority: P2)

**Goal**: Keep row numbers, snapshot selectors, and next-command selection aligned with grouped presentation.

**Independent Test**: Render a mixed store, resolve every row number after mutation, and exercise present and missing installs in both readiness classes.

- [x] T008 [US2] Add failing continuous-number and snapshot-resolution tests over grouped order in `crates/fragcap-cli/tests/cli_targets.rs`
- [x] T009 [US2] Replace the prior cross-readiness missing-install expectation with failing ready-first footer cases in `crates/fragcap-cli/tests/cli_targets.rs`
- [x] T010 [US2] Write the snapshot from final order and implement readiness-scoped install-presence fallback in `crates/fragcap-cli/src/commands/targets.rs`

**Checkpoint**: Every numeric selector names the displayed row and every populated footer respects readiness priority.

---

## Phase 5: User Story 3 - Preserve Single-State and Machine Interfaces (Priority: P3)

**Goal**: Keep empty and single-state output concise and all machine interfaces stable.

**Independent Test**: Exercise empty, all-ready, and all-setup-needed stores, then compare export and stable identity before and after listing.

- [x] T011 [US3] Add focused empty, all-ready, and all-setup-needed heading-omission coverage in `crates/fragcap-cli/tests/cli_targets.rs`
- [x] T012 [US3] Add export-byte and stable-identifier preservation coverage around grouped listing in `crates/fragcap-cli/tests/cli_targets.rs`
- [x] T013 [US3] Preserve machine-findings and bare-help footer composition while updating renderer unit calls in `crates/fragcap-cli/src/commands/targets.rs`

**Checkpoint**: Every required listing shape is covered and non-human interfaces remain unchanged.

---

## Phase 6: Documentation and Verification

- [x] T014 Reconcile `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, `docs/glossary/command-line-and-diagnostics.md`, `README.md`, and `site/content/docs/getting-started.mdx`
- [x] T015 Add `changelog.d/376-target-readiness-groups.changed.md` with the exact specification impact
- [x] T016 Mark tasks complete, validate `specs/138-target-readiness-groups/quickstart.md`, run focused tests, `cargo fmt --all -- --check`, and `cargo xtask ci`
- [x] T017 Verify no dependency drift, no forbidden punctuation, UTF-8 without BOM, LF, no trailing whitespace, no mojibake, and a scope-clean final diff

## Dependencies and Execution Order

- Phase 1 establishes the complete requirements and public contract.
- Phase 2 creates the single ordering authority that blocks grouped rendering and selector work.
- User Story 1 depends on Phase 2 and establishes grouped output plus independent widths.
- User Story 2 depends on User Story 1's final order and connects numbering, snapshots, and footer selection.
- User Story 3 validates the remaining listing shapes and stable interfaces after the grouped path exists.
- Documentation and full verification follow every executable story.

## Parallel Opportunities

- T005 and T006 affect separate test surfaces and can be authored independently after T004.
- T008 and T009 cover separate selector and footer contracts and can be authored independently after T007.
- Documentation research for T014 and changelog drafting for T015 can proceed after the implementation contract stabilizes.

## Implementation Strategy

Start with a failing ordering test and one final ordered vector. Add the mixed grouped renderer next, preserving all existing cell authorities. Bind snapshot and footer behavior to that exact order, then cover empty and single-state forms plus export stability. Reconcile all public examples only after focused output is green, and finish with the complete repository gate.
