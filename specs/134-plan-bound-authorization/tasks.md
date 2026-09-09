# Tasks: Plan-Bound Deep Capture Authorization

**Input**: Design documents from `/specs/134-plan-bound-authorization/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/deep-capture-authorization.md, quickstart.md

**Tests**: Security, lifecycle, CLI, structured-output, migration, and controlled integration tests are mandatory and are written before their implementation tasks.

**Organization**: Tasks are grouped by independently testable user story after shared canonical-plan and certificate foundations.

## Phase 1: Setup

**Purpose**: Establish direct dependency and change-record scaffolding without changing runtime behavior.

- [x] T001 Add direct `blake3` and `subtle` workspace dependencies already present in `Cargo.lock` to `crates/fragcap-cli/Cargo.toml`
- [x] T002 [P] Add the user-visible S134 outcome fragment in `changelog.d/382-plan-bound-authorization.added.md`
- [x] T003 [P] Add the S134 architecture decision fragment covering canonical identifiers, process-local CA preparation, post-authorization listener reservation, and one-release migration in `changelog.d/s134-plan-bound-authorization.decisions.md`

---

## Phase 2: Foundational Authorization Ownership

**Purpose**: Make the exact certificate and canonical plan available before any user-story authorization flow.

**Critical**: No user story proceeds until the displayed certificate identity can be consumed by the native runtime and the canonical plan identifier is deterministic.

- [x] T004 Add failing prepared-authority tests proving public thumbprints are stable, private material stays opaque, and the native runtime uses the exact prepared CA in `crates/fragcap/tests/native_proxy.rs`
- [x] T005 Add a failing native runtime unit test proving a supplied session CA is consumed once and reported unchanged in `crates/fragcap-proxy/src/runtime.rs`
- [x] T006 Implement process-local prepared session CA generation, public identity access, and exact runtime handoff in `crates/fragcap/src/deep_capture/native.rs`
- [x] T007 Implement optional exact prepared CA consumption while retaining the existing generated fallback for library compatibility in `crates/fragcap-proxy/src/runtime.rs`
- [x] T008 Add failing canonical serialization and digest sensitivity tests for every authorization-relevant field in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T009 Implement the versioned `AuthorizationPlan`, deterministic field ordering, complete `plan-v1` digest, target launch-authority snapshot, trust summary, and exact drift comparison in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T010 Bind `LibraryIdentifierAdapter` and the prepared native proxy adapter to the authorization plan id, session id, and exact prepared CA in `crates/fragcap-cli/src/commands/deep_capture.rs`

**Checkpoint**: The process can produce one exact plan and certificate identity without binding a listener or writing a bundle, and later runtime ownership cannot substitute either identity.

---

## Phase 3: User Story 1 - Interactive Complete-Plan Authorization (Priority: P1)

**Goal**: Display every effect-bearing field and accept one interactive plan decision before any session effect.

**Independent Test**: A controlled interactive authorizer sees one complete plan; exact approval runs it; decline, EOF, interruption, output failure, and drift record zero effects and no duplicate trust prompt.

### Tests for User Story 1

- [x] T011 [US1] Add failing interactive authorization tests for complete human plan content, one prompt, affirmative approval, decline, EOF, and no duplicate trust confirmation in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T012 [US1] Add failing no-effect ordering and target launch-authority drift tests around listener, bundle, proxy, trust, launch, capture, and fact adapters in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T013 [US1] Add failing exact library `PlanId` single-use and mismatch coverage required by the canonical CLI plan in `crates/fragcap/tests/deep_capture_session.rs`

### Implementation for User Story 1

- [x] T014 [US1] Implement complete human authorization-plan rendering and narrow-terminal non-truncation behavior in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T015 [US1] Implement the interactive plan authorizer with plan flush, affirmative grammar, decline, EOF, I/O failure, and interruption outcomes in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T016 [US1] Reorder ordinary and calibration execution so plan construction and authorization precede facade preflight, listener reservation, bundle protection, proxy, trust, launch, capture, artifacts, and facts in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T017 [US1] Revalidate the exact target and launch-authority snapshot after approval and refuse drift before facade preflight in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T018 [US1] Remove the duplicate calibration and post-warm-restart session prompts while preserving one final freshly prepared plan decision in `crates/fragcap-cli/src/commands/deep_capture.rs`

**Checkpoint**: Interactive Deep Capture has one complete effect authorization and every pre-approval refusal is demonstrably effect-free.

---

## Phase 4: User Story 2 - Structured Exact-Identifier Authorization (Priority: P1)

**Goal**: Let automation return only the exact emitted plan identifier through prompt-free dedicated input.

**Independent Test**: Structured controlled execution emits the plan, accepts its exact identifier, never prompts, and refuses absent, malformed, whitespace-altered, case-changed, stale, replayed, extra, or mismatched input with zero effects.

### Tests for User Story 2

- [x] T019 [US2] Add failing parser and structured contract tests for `--authorize-stdin`, JSON gating, and forbidden flag combinations in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T020 [US2] Add failing bounded exact-input tests for matching, EOF, malformed UTF-8, whitespace, abbreviation, case, stale id, replay, extra bytes, and I/O failure in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T021 [US2] Add failing structured event tests for the complete plan and one terminal authorization outcome without secrets in `crates/fragcap-cli/tests/cli_deep_capture.rs`

### Implementation for User Story 2

- [x] T022 [US2] Add public `--authorize-stdin` grammar and validation to `crates/fragcap-cli/src/cli.rs`
- [x] T023 [US2] Add an injectable bounded input and exact-plan authorizer path while preserving existing `run` and `run_with` callers in `crates/fragcap-cli/src/lib.rs`
- [x] T024 [US2] Implement prompt-free exact identifier input with bounded reading and authorization-sensitive exact comparison in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T025 [US2] Add versioned `deep_capture.authorization_plan` and `deep_capture.authorization` event variants and serialization in `crates/fragcap-cli/src/events.rs`
- [x] T026 [US2] Emit and flush the complete structured plan before reading dedicated input, then emit the exact outcome before execution or refusal in `crates/fragcap-cli/src/commands/deep_capture.rs`

**Checkpoint**: Automation can authorize one ephemeral exact plan without a generic boolean, prompt, plan file, persisted private key, or replayable preference.

---

## Phase 5: User Story 3 - Trust-Free Reachability Authorization (Priority: P2)

**Goal**: Authorize reachability launch and routing effects without requesting or performing certificate trust.

**Independent Test**: A controlled reachability plan explicitly contains no trust action, HAR, or key log; exact approval runs the route; the trust adapter remains untouched.

### Tests for User Story 3

- [x] T027 [US3] Add failing human and structured reachability-plan tests for absent trust, HAR, and key log plus retained launch and routing scope in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T028 [US3] Add failing controlled adapter coverage proving authorized reachability never invokes current-user trust in `crates/fragcap-cli/src/commands/deep_capture.rs`

### Implementation for User Story 3

- [x] T029 [US3] Route reachability through the common authorization plan while rendering trust and TLS-sensitive artifacts as absent in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T030 [US3] Preserve early refusal for trust, HAR, and key-log options on reachability before plan authorization in `crates/fragcap-cli/src/commands/deep_capture.rs`

**Checkpoint**: Reachability uses the same exact plan authority without misrepresenting or invoking trust.

---

## Phase 6: User Story 4 - Legacy Flag Migration (Priority: P3)

**Goal**: Hide and reject insecure Deep Capture booleans with actionable one-release migration guidance while leaving unrelated confirmations intact.

**Independent Test**: Normal help has neither legacy flag; either hidden input exits 2 with replacement guidance and zero effects; Doctor, bundle cleanup, and target reconciliation keep their existing `--yes` behavior.

### Tests for User Story 4

- [x] T031 [US4] Update the help contract to require `--authorize-stdin` and reject visible Deep Capture `--trust-ca` or `--yes` in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T032 [US4] Add failing legacy invocation tests for exact human and JSON migration errors, conflicts, and zero effects in `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T033 [P] [US4] Retain existing Doctor, bundle cleanup, and target reconciliation `--yes` contract assertions in `crates/fragcap-cli/tests/cli_doctor.rs`, `crates/fragcap-cli/tests/cli_bundle.rs`, and `crates/fragcap-cli/tests/cli_targets.rs`

### Implementation for User Story 4

- [x] T034 [US4] Hide legacy Deep Capture `--trust-ca` and `--yes` parser inputs, conflict them with `--authorize-stdin`, and stop mapping them to trust intent in `crates/fragcap-cli/src/cli.rs`
- [x] T035 [US4] Return one-release actionable migration errors before store access, certificate generation, plan emission, or effects in `crates/fragcap-cli/src/commands/deep_capture.rs`

**Checkpoint**: No Deep Capture boolean can authorize a session, while existing separately scoped destructive-action confirmations remain stable.

---

## Phase 7: Documentation, Convergence, and Verification

**Purpose**: Synchronize the architecture of record, user references, evidence, and full gates across all stories.

- [x] T036 [P] Update the shipped CLI and security contract in `docs/fragcap-specification.md`, `docs/fragcap-spec-outline.md`, and `docs/plans/README.md`
- [x] T037 [P] Update focused user documentation and examples in `site/content/docs/architecture.mdx`, `site/content/docs/getting-started.mdx`, `site/content/docs/reference/cli.mdx`, and `site/content/docs/reference/deep-capture-compatibility.mdx`
- [x] T038 Re-run `/speckit-analyze`, resolve every finding, and record complete requirement-to-task coverage against `specs/134-plan-bound-authorization/spec.md`, `specs/134-plan-bound-authorization/plan.md`, and `specs/134-plan-bound-authorization/tasks.md`
- [x] T039 Run focused authorization, native proxy, lifecycle, migration, and unrelated confirmation tests from `specs/134-plan-bound-authorization/quickstart.md`
- [x] T040 Run `/speckit-converge`, append and implement any remaining tasks, then rerun until the implementation satisfies every S134 requirement in `specs/134-plan-bound-authorization/tasks.md`
- [x] T041 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, and `cargo xtask ci` from the repository root
- [x] T042 Verify the lockfile package set is unchanged, changed text is UTF-8 without BOM and LF-only, no mojibake or forbidden dash exists, every checklist is complete, and the final diff matches issue #382 scope

---

## Phase 8: First-Round Review Remediation

**Purpose**: Close every first-round Codex finding without widening the authorized session.

- [x] T043 Propagate authorization plan, prompt, and terminal-outcome write failures and prove a writer whose write fails but flush succeeds never reaches input or effects in `crates/fragcap-cli/src/emit.rs`, `crates/fragcap-cli/src/commands/deep_capture.rs`, and `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T044 Require one complete LF-terminated interactive authorization line and add affirmative-at-EOF refusal coverage in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/tests/cli_deep_capture.rs`
- [x] T045 Enforce the exact reviewed target authority inside the facade's final resolver before endpoint and Capture preparation in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T046 Refuse expired prepared authorities and pending prior-session recovery, delegating the latter to Doctor through a bounded read-only inspection in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/doctor/fix.rs`
- [x] T047 Synchronize S134 contracts and user documentation, then rerun focused and full repository gates

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Phase 1 and blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on the canonical plan and prepared CA foundation.
- **User Story 2 (Phase 4)**: Depends on User Story 1 plan rendering and authorization ordering.
- **User Story 3 (Phase 5)**: Depends on the common authorization flow from User Stories 1 and 2.
- **User Story 4 (Phase 6)**: Depends on the replacement command contract being functional.
- **Documentation and Verification (Phase 7)**: Depends on all user stories.
- **First-Round Review Remediation (Phase 8)**: Depends on the first published review and Phase 7 completion.

### User Story Dependencies

- **User Story 1**: Delivers the interactive minimum viable authorization boundary.
- **User Story 2**: Reuses the same canonical plan and adds prompt-free exact input without changing lifecycle authority.
- **User Story 3**: Reuses the same plan while proving trust-free calibration semantics.
- **User Story 4**: Activates migration errors only after the replacement flows exist.

### Within Each User Story

- Add and observe failing tests before implementation.
- Prepare public certificate identity before canonical plan construction.
- Emit and flush the complete plan before reading authorization.
- Revalidate authorization before facade preflight and every session effect.
- Complete focused tests before advancing to the next story.

### Parallel Opportunities

- T002 and T003 affect separate changelog files.
- Documentation tasks T036 and T037 affect separate document sets after behavior settles.
- T033 only verifies unaffected command suites and can run beside Deep Capture migration implementation.

---

## Implementation Strategy

### Minimum Viable Security Boundary

1. Complete dependency and changelog setup.
2. Complete prepared CA and canonical identifier foundations.
3. Complete interactive exact-plan authorization and zero-effect refusal.
4. Run focused User Story 1 tests before adding automation.

### Incremental Delivery

1. Add the interactive boundary.
2. Add structured exact-identifier automation.
3. Prove trust-free reachability.
4. Activate legacy migration refusals.
5. Synchronize documentation and run convergence plus full gates.

## Notes

- Every checked task must be marked `[x]` only after its implementation or verification evidence has been observed.
- No test may perform a real current-user Root-store mutation, launch a real game, require Npcap, or need elevation.
- If implementation reveals an architecture deviation, record it in the S134 decision fragment and master specification before completion.
