# Tasks: S156 deterministic package certification

## Phase 1: Setup

- [x] T001 Record the post-S155 failure cause and pinned-script decision in `changelog.d/S156-deterministic-package-certification.decisions.md`
- [x] T002 [P] Record the corrected package-certification behavior in `changelog.d/S156-deterministic-package-certification.fixed.md`

## Phase 2: Foundational

- [x] T003 Add schema-4 smoke builders and schema-2/schema-3 compatibility fixtures in `xtask/src/package_certification.rs`
- [x] T004 Add a bounded structured-event parser and closed predicate evaluator in `scripts/Test-PackageCertification.ps1`

## Phase 3: User Story 1 - Certify deterministic controlled reachability (P1)

**Goal**: Make structured native startup and reached-client evidence the positive authority while retaining every containment and ownership gate.

**Independent test**: A schema-4 report with zero sampled endpoints validates when structured evidence is complete, while every observed non-loopback or unexpected-owner mutation fails.

- [x] T005 [US1] Add failing zero-endpoint and deterministic-authority contract tests in `xtask/src/package_certification.rs`
- [x] T006 [US1] Replace free-text and socket-positive assertions with exact NDJSON event validation in `scripts/Test-PackageCertification.ps1`
- [x] T007 [US1] Preserve descendant process, firewall, non-loopback, finite child and cleanup checks in `scripts/Test-PackageCertification.ps1`

## Phase 4: User Story 2 - Diagnose failed predicates safely (P1)

**Goal**: Report every failed smoke invariant using stable bounded identifiers without raw host data.

**Independent test**: Each independently mutated predicate yields its expected stable identifier and public output remains within count, byte and content bounds.

- [x] T008 [US2] Add deterministic closed predicate collection and bounded rendering in `scripts/Test-PackageCertification.ps1`
- [x] T009 [US2] Add static script markers and diagnostic contract checks in `xtask/src/package_certification.rs`
- [x] T010 [US2] Add exhaustive independent mutation coverage for positive authority, containment, ownership, schema, uniqueness and cleanup in `xtask/src/package_certification.rs`

## Phase 5: User Story 3 - Consume a versioned certification report (P2)

**Goal**: Emit strict schema 4 while retaining explicit historical readers.

**Independent test**: Schema 4 positive and mutations behave exactly, schema 2 and schema 3 remain readable under historical rules, and mixed or unknown versions fail.

- [x] T011 [US3] Implement strict schema-4 smoke validation and predicate-specific Rust diagnostics in `xtask/src/package_certification.rs`
- [x] T012 [US3] Emit schema 4 with separate session and socket evidence for portable and installed surfaces in `scripts/Test-PackageCertification.ps1`
- [x] T013 [P] [US3] Update current and historical contract guidance in `docs/maintainers/package-certification.md`
- [x] T014 [P] [US3] Correct the historical package contract boundary in `specs/131-native-packaging/contracts/package-certification.md`

## Phase 6: Reconciliation and Verification

- [x] T015 Add S156 chronological scope and completion boundary in `docs/plans/README.md`
- [x] T016 Reconcile S156 into the architecture of record in `docs/fragcap-specification.md`
- [x] T017 Run focused Rust tests, PowerShell parsing and both PowerShell compliance authorities without executing the product locally
- [x] T018 Run `cargo xtask ci`, inspect all output and perform UTF-8, LF, forbidden-dash and mojibake checks across every changed file
- [ ] T019 Push the authorized branch, open the official pull request closing #425 and move Project 3 Stage to PR review
- [ ] T020 Wait for final-head hosted checks, require both affected workflows green without rerun, address every review within two rounds and update issue evidence

## Dependencies

- Phase 1 precedes every pinned-artifact edit.
- Phase 2 establishes the test and parsing seams used by both P1 stories.
- User Story 1 and User Story 2 share the final script and therefore execute sequentially despite independent acceptance criteria.
- User Story 3 depends on the settled smoke shape from User Stories 1 and 2.
- Reconciliation and verification follow all user stories.

## Parallel Opportunities

- T001 and T002 touch separate changelog fragments.
- T013 and T014 update separate contract documents after schema behavior stabilizes.
- Documentation reconciliation can be drafted while focused unit tests run, but final wording follows verified behavior.

## Implementation Strategy

Write the schema-4 positive fixture and mutations first, including a valid zero-endpoint observation. Implement strict report validation, then change PowerShell event extraction and predicate evaluation until the contract tests and static markers pass. Finish documentation, local static gates and full CI, then rely on the two hosted Windows workflows for product-level proof.
