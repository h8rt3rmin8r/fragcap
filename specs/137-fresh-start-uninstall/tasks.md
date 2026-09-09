# Tasks: Explicit Fresh-Start Uninstall

**Input**: Design documents from `/specs/137-fresh-start-uninstall/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/fresh-start-cli.md

**Tests**: Destructive-action, security, lifecycle, and clean-reinstall tests are mandatory.

## Phase 1: Setup and Contract Guards

- [x] T001 Add S137 feature artifacts, completed requirement checklists, and the active feature pointer in `specs/137-fresh-start-uninstall/` and `.specify/feature.json`
- [x] T002 Add failing fresh-start CLI parse and help guards in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/tests/cli_fresh_start.rs`
- [x] T003 Add failing pinned-installer static guards for opt-in defaults, lifecycle conditions, and exact executable delegation in the existing repository validation authority

---

## Phase 2: Foundational Inventory Authority

- [x] T004 Add failing isolated-root tests for canonical pair validation, deterministic inventory, category coverage, exclusions, aliases, overlap, roots, and changed inventories in `crates/fragcap-cli/src/commands/fresh_start.rs`
- [x] T005 Implement typed scope, profile, root, entry, category, inventory, digest, and report models in `crates/fragcap-cli/src/commands/fresh_start.rs`
- [x] T006 Implement canonical current-user root resolution and explicit installer-root handling in `crates/fragcap-cli/src/paths.rs` and `crates/fragcap-cli/src/commands/fresh_start.rs`
- [x] T007 Implement link-aware bounded traversal and refusal without following symlinks, junctions, mount points, or reparse points in `crates/fragcap-cli/src/commands/fresh_start.rs`

**Checkpoint**: Preview authority is deterministic, read-only, exact, and contained.

---

## Phase 3: User Story 1 - Preserve Data Unless Explicitly Selected (Priority: P1)

**Goal**: No ordinary lifecycle path can authorize fresh-start cleanup.

**Independent Test**: Seed isolated state, omit or vary opt-in values across uninstall and maintenance conditions, and prove byte identity plus zero cleanup invocation.

- [x] T008 [US1] Add failing preserve-by-default, cancellation, stale-property, and non-removal lifecycle tests in `crates/fragcap-cli/tests/cli_fresh_start.rs` and package certification
- [x] T009 [US1] Add the additive command surface and dispatch with no implicit execution in `crates/fragcap-cli/src/cli.rs`, `crates/fragcap-cli/src/lib.rs`, and `crates/fragcap-cli/src/commands/mod.rs`
- [x] T010 [US1] Add exact silent-property and explicit-removal-only guards in `crates/fragcap-cli/wix/main.wxs`

**Checkpoint**: Every unopted or non-uninstall lifecycle path preserves all user data.

---

## Phase 4: User Story 2 - Reset the Initiating User Safely (Priority: P2)

**Goal**: Preview, confirm, recover, and remove one initiating user's exact canonical data with truthful outcomes.

**Independent Test**: Run the command over isolated canonical roots containing all categories and exclusions, then verify exact deletion, retained recovery evidence on failure, and clean reinitialization.

- [x] T011 [US2] Add failing preview, exact-confirmation, deletion-order, redirection, locked-file, recovery-failure, partial-report, and clean-state tests in `crates/fragcap-cli/src/commands/fresh_start.rs` and `crates/fragcap-cli/tests/cli_fresh_start.rs`
- [x] T012 [US2] Expose Doctor's existing exact recovery implementation through a narrow crate-private entry point in `crates/fragcap-cli/src/doctor/fix.rs`
- [x] T013 [US2] Implement preview, identifier validation, safe recursive deletion, recovery-first session handling, and bounded JSON report in `crates/fragcap-cli/src/commands/fresh_start.rs`
- [x] T014 [US2] Add an unchecked current-user maintenance dialog, exact initiating-user root transfer, hidden deferred action, local report, and failure propagation in `crates/fragcap-cli/wix/main.wxs`
- [x] T015 [US2] Extend `scripts/Test-PackageCertification.ps1` for preserved ordinary uninstall, confirmed current-user cleanup, failure truth, and clean reinstall using certification-owned roots

**Checkpoint**: Current-user fresh start is explicit, exact, contained, recovery-aware, and truthful.

---

## Phase 5: User Story 3 - Perform Auditable Administrative Cleanup (Priority: P3)

**Goal**: All-users authority is distinct, administrator-only, complete before deletion, and bound to an unchanged inventory.

**Independent Test**: Preview an injected multi-profile authority, execute the exact digest, mutate one fact and prove zero deletion, and verify no profile is reached by current-user scope.

- [x] T016 [US3] Add failing multi-profile sorting, authority, preview-binding, changed-inventory, and current-user isolation tests in `crates/fragcap-cli/src/commands/fresh_start.rs`
- [x] T017 [US3] Implement Windows profile-authority enumeration and administrator refusal plus injected test inventory in `crates/fragcap-cli/src/commands/fresh_start.rs`
- [x] T018 [US3] Require preview identifier and report path for all-users execution and add distinct MSI guidance in `crates/fragcap-cli/wix/main.wxs`
- [x] T019 [US3] Add package/static certification for explicit all-users handling and absence of implicit cross-profile cleanup

**Checkpoint**: Broader cleanup is independently previewed and cannot inherit current-user or MSI consent.

---

## Phase 6: Documentation and Verification

- [x] T020 Reconcile `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, `docs/plans/README.md`, `docs/maintainers/package-certification.md`, and `site/content/docs/getting-started.mdx`
- [x] T021 Add `changelog.d/377-fresh-start-uninstall.added.md` and dated `changelog.d/s137-fresh-start-uninstall.decisions.md`
- [x] T022 Mark tasks complete, validate the quickstart, run focused tests, `cargo fmt --all -- --check`, and `cargo xtask ci`
- [x] T023 Verify no dependency drift, no forbidden punctuation, UTF-8 without BOM, LF, no trailing whitespace, no mojibake, and a scope-clean final diff

## Dependencies and Execution Order

- Phase 1 establishes the specification and failing surface guards.
- Phase 2 blocks every destructive path and must complete before execution work.
- User Story 1 establishes the preserve-default lifecycle boundary before any wipe is wired.
- User Story 2 depends on the inventory authority and preserve guard.
- User Story 3 reuses the same inventory and deletion contract but adds broader profile authority and stronger preview binding.
- Documentation and full verification follow all executable stories.

## Implementation Strategy

Start with the read-only inventory and its negative tests. Wire preservation guards before the cleanup action. Add current-user execution and recovery next. Add all-users authority last through the same contract, with no alternate deletion implementation. Run the full gate only after every focused security case passes.
