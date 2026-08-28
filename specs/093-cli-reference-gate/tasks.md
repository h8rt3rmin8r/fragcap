# Tasks: CLI Reference Gate

**Input**: Design documents from `specs/093-cli-reference-gate/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/cli-reference-contract.md`

**Tests**: Test-first command, option, sink, and example contract coverage is required, followed by documentation and complete repository gates.

**Organization**: Tasks are grouped by user story. The CLI reference and its single integration test are shared contract surfaces, so most implementation tasks are sequential.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel because it affects a different file and has no dependency on an incomplete task.
- **[Story]**: Maps a task to its user story.

## Phase 1: Setup and Specification

**Purpose**: Establish the clean feature branch, behavioral authorities, and complete S093 design set.

- [X] T001 Confirm `codex/093-cli-reference-gate` starts clean from synchronized `main` and issue #246 is the next open documentation slice
- [X] T002 Point `.specify/feature.json` at `specs/093-cli-reference-gate` without staging the local pointer
- [X] T003 Read `AI_CONTEXT.md`, the constitution, repository conventions, contributor workflow, CLI source and tests, task-runner documentation gate, master-specification command sections, and current public CLI reference
- [X] T004 Create and validate `specs/093-cli-reference-gate/spec.md` and `specs/093-cli-reference-gate/checklists/requirements.md`
- [X] T005 Run clarification coverage analysis and confirm issue #246 plus shipped source resolve every material question without an unnecessary clarification section
- [X] T006 Create and complete `specs/093-cli-reference-gate/checklists/cli-contract.md`
- [X] T007 Create the S093 plan and design set in `specs/093-cli-reference-gate/plan.md`, `research.md`, `data-model.md`, `contracts/cli-reference-contract.md`, and `quickstart.md`

**Checkpoint**: S093 has an implementation-ready, constitution-compliant contract with no unresolved clarification.

---

## Phase 2: Foundational Failing Contract

**Purpose**: Build the hermetic comparison harness and prove the stale reference fails before correcting it.

- [X] T008 Add MDX command-section and option-table parsing helpers to `crates/fragcap-cli/tests/cli_reference.rs`, including duplicate and malformed-contract diagnostics with source lines
- [X] T009 Add recursive visible clap command and locally owned option extraction to `crates/fragcap-cli/tests/cli_reference.rs`, with structural hidden-item filtering and generated-control policy
- [X] T010 Add exact command, long option, short alias, finite value, parser-default, and feature-availability comparisons to `crates/fragcap-cli/tests/cli_reference.rs`
- [X] T011 Run the default and `net` focused tests against the current page and record the expected failures for missing command sections and stale option contracts

**Checkpoint**: The new test fails deterministically against the known stale page and identifies owning paths and mismatched values.

---

## Phase 3: User Story 1 - Command Drift Fails Before Merge (Priority: P1)

**Goal**: Make every public command and option a two-sided checked contract and integrate it into the standing documentation gate.

**Independent Test**: A synthetic command, option, value, or default present on only one side causes a focused failure that names the owning command and mismatch.

- [X] T012 [US1] Add test specimens for duplicate sections, stale sections, missing commands, option drift, alias drift, value drift, and default drift in `crates/fragcap-cli/tests/cli_reference.rs`
- [X] T013 [US1] Add default-versus-`net` availability filtering and prove conditional `catalog seed` flags are checked only in the enabled tree
- [X] T014 [US1] Update `xtask/src/docs.rs` so `cargo xtask docs check` runs the glossary linter and the CLI-reference test in default and `net` variants without a crate dependency
- [X] T015 [US1] Run `cargo xtask docs check` and confirm the stale page fails through the contributor-facing gate

**Checkpoint**: The standing docs command exercises every public command variant and catches synthetic drift without runtime side effects.

---

## Phase 4: User Story 2 - Operators Read the Shipped Surface (Priority: P2)

**Goal**: Replace the stale page with one exact v0.7.0 command and option contract.

**Independent Test**: Every visible command has one section, every owning option agrees with clap, and managed store paths are described as optional overrides.

- [X] T016 [US2] Restructure `site/content/docs/reference/cli.mdx` with exactly one stable heading for every public top-level and nested command path
- [X] T017 [US2] Add complete option tables with aliases, finite values, parser defaults, and `net` availability to `site/content/docs/reference/cli.mdx`
- [X] T018 [US2] Correct managed path prose so `--db`, `--local-db`, `--catalog-db`, `--bundle`, and `--dir` are optional overrides with their applicable environment or per-user fallback
- [X] T019 [US2] Derive accepted sink schemes, aliases, and modifier names from `crates/fragcap-cli/src/args.rs` and compare them with the `--sink` contract in `crates/fragcap-cli/tests/cli_reference.rs`
- [X] T020 [US2] Document every accepted sink scheme, alias, modifier, value rule, and transport-specific platform constraint in `site/content/docs/reference/cli.mdx`
- [X] T021 [US2] Run the default and `net` contract tests and resolve every command, option, availability, and sink mismatch

**Checkpoint**: Operators can compose any shipped public invocation from the reference, and the gate proves the visible contract agrees with source.

---

## Phase 5: User Story 3 - Examples and JSON Routing Are Trustworthy (Priority: P3)

**Goal**: Parse every executable reference example without dispatch and make terminal and sink routing explicit.

**Independent Test**: Every discovered `fragcap` invocation parses through clap only, an invalid specimen fails with its source line, and all output classes have an explicit destination.

- [X] T022 [US3] Add fenced-example discovery, supported shell comment handling, quoted-token parsing, and PowerShell and shell line-continuation handling to `crates/fragcap-cli/tests/cli_reference.rs`
- [X] T023 [US3] Add focused example-parser specimens for quoted Windows paths, inline comments, continuations, and one invalid invocation with source-line diagnostics
- [X] T024 [US3] Correct every executable example in `site/content/docs/reference/cli.mdx` so it parses under its documented feature availability without execution
- [X] T025 [US3] Document command-result stdout, Capture and Deep Capture lifecycle stderr, capture sink bytes, warning and error diagnostics, `--quiet`, and `--silent` routing in `site/content/docs/reference/cli.mdx`
- [X] T026 [US3] Run both focused variants and confirm every page example is discovered and accepted through `try_get_matches_from()` only

**Checkpoint**: Worked invocations and stream routing are reliable for automation authors, with no command execution in the gate.

---

## Phase 6: Polish and Cross-Cutting Validation

**Purpose**: Close traceability, documentation, source hygiene, and complete repository gates before the pre-push halt.

- [X] T027 Add `changelog.d/246-cli-reference-gate.fixed.md` with a valid specification-impact marker and concise user-visible correction
- [X] T028 Run `cargo xtask docs check` and resolve every glossary, link, command-tree, sink, and example finding
- [X] T029 Run `cargo xtask docs build` and resolve every production static-export failure
- [X] T030 Run `cargo fmt --all -- --check`, `cargo xtask lint`, `git diff --check`, UTF-8 and mojibake checks, and prohibited-punctuation audits
- [X] T031 Run `cargo xtask ci` in the foreground and resolve every failure
- [X] T032 Re-run the complete contract audit in `specs/093-cli-reference-gate/contracts/cli-reference-contract.md` and validate both checklists remain complete
- [X] T033 Review the final diff for issue #246 scope, no runtime or workflow changes, no forbidden dependency edge, and exclusion of `.specify/feature.json`
- [X] T034 Mark all completed tasks in `specs/093-cli-reference-gate/tasks.md`, stage only S093 files, commit locally with the repository co-author trailer, and halt before `git push`

---

## Dependencies and Execution Order

### Phase Dependencies

- **Setup and Specification (Phase 1)**: Starts from the clean branch.
- **Foundational Failing Contract (Phase 2)**: Depends on the completed design and blocks reference corrections.
- **User Story 1 (Phase 3)**: Depends on the comparison harness and establishes the contributor-facing gate.
- **User Story 2 (Phase 4)**: Depends on User Story 1 so the rewritten page is continuously checked.
- **User Story 3 (Phase 5)**: Depends on the stable page structure and shares the integration test.
- **Polish (Phase 6)**: Depends on all three stories.

### User Story Dependencies

- **User Story 1 (P1)**: The minimum viable gate and prerequisite for durable corrections.
- **User Story 2 (P2)**: Uses the gate to correct all current command, option, store, and sink drift.
- **User Story 3 (P3)**: Adds parsing-only examples and output routing after the command contract is stable.

### Parallel Opportunities

- T014 affects `xtask/src/docs.rs` while T012 and T013 affect the integration test, but the tasks remain sequential for a clear failing-gate record.
- Documentation checks are sequential because each validates the same completed tree.
- No shared `cli.mdx` or `cli_reference.rs` task is marked parallel.

## Implementation Strategy

### Contract First

1. Parse the human-visible reference and real clap tree.
2. Record the expected stale-page failure.
3. Integrate the failing contract into the docs command.
4. Correct command, option, store, and sink documentation.
5. Add parsing-only example coverage and routing prose.
6. Run the focused, documentation, and repository-wide gates.

### Review Boundary

The slice ends with one local commit on `codex/093-cli-reference-gate`. Do not push until the operator explicitly authorizes it after reviewing the autopilot breakdown.

## Notes

- Exact machine tokens remain inline code; explanatory prose uses existing glossary terms.
- `.specify/feature.json` is local state and must never be staged.
- The task order intentionally captures a red test before page corrections.
