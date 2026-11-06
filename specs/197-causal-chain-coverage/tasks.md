# Tasks: Causal Launch-Chain Coverage

**Input**: Design documents from `specs/197-causal-chain-coverage/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [evidence contract](contracts/evidence-contract.md)

**Tests**: Required by all three issues, the specification, and the repository autopilot protocol. Write each corresponding regression before its implementation and observe the targeted failure.

**Organization**: Tasks follow the three user stories and their shared foundation. Every phase remains a complete reviewable increment, and the final PR closes issues only when all phases and gates complete.

## Phase 1: Setup

**Purpose**: Establish the S197 branch, feature pointer, source context, and controlled failure baselines.

- [x] T001 Confirm clean `codex/s197-causal-chain-coverage`, active `.specify/feature.json`, and exact open issue acceptance in `specs/197-causal-chain-coverage/spec.md`.
- [x] T002 Record current packet, scope, and shutdown contract baselines in `specs/197-causal-chain-coverage/research.md` and controlled test locations in `specs/197-causal-chain-coverage/quickstart.md`.

## Phase 2: Foundational contracts

**Purpose**: Provide one exact process-generation authority and one typed packet decision without changing ordinary Capture.

- [x] T003 Write failing gate-reason and flow-omission regressions in `crates/fragcap/src/session.rs` and `crates/fragcap-core/src/pipeline/mod.rs`.
- [x] T004 Add typed packet admission and actual omission causes in `crates/fragcap-core/src/traits.rs`, `crates/fragcap-core/src/flow.rs`, `crates/fragcap-core/src/pipeline/mod.rs`, and `crates/fragcap/src/session.rs`.
- [x] T005 Write failing exact-generation, delayed-proof, PID-reuse, and unrelated-process membership regressions in `crates/fragcap/src/session.rs` and `crates/fragcap/src/deep_capture/process.rs`.
- [x] T006 Add one bounded causal-generation membership publication path shared by `crates/fragcap/src/session.rs` and `crates/fragcap/src/deep_capture/process.rs`, preserving independent Capture attribution.
- [x] T007 Connect membership to role stamping and endpoint narrowing in `crates/fragcap/src/session.rs`, with controlled loopback and undeclared-descendant regressions.

**Checkpoint**: Typed reasons and exact causal membership pass independently before scope-default changes.

## Phase 3: User Story 1 - Full chain is the ordinary scope (Priority: P1)

**Goal**: Make full-chain application inspection the ordinary pre-effect default and keep packet scope independent from explicit game-only application handling.

**Independent Test**: Controlled default and explicit-game plans select exact expected policy and packet membership without relying on the post-observation lifetime regression.

### Tests for User Story 1

- [x] T008 [US1] Write failing default, explicit game-only, stale-fact, and stale-resume cases in `crates/fragcap-cli/src/commands/deep_capture.rs`, `crates/fragcap-cli/src/commands/calibrate.rs`, and `crates/fragcap/src/deep_capture/proposal.rs`.
- [x] T009 [US1] Write failing unexpected utility and unrelated same-name packet-scope cases in `crates/fragcap/src/session.rs`.

### Implementation for User Story 1

- [x] T010 [US1] Resolve `auto` to full launch-chain inspection consistently in `crates/fragcap-cli/src/commands/deep_capture.rs`, `crates/fragcap/src/deep_capture/inspection.rs`, and `crates/fragcap/src/deep_capture/proposal.rs`.
- [x] T011 [US1] Version exact compatibility policy identity, prevent historical fact promotion, and reject resume policy drift before effects in `crates/fragcap-cli/src/commands/calibrate.rs` and `crates/fragcap-targets/src/compatibility.rs`.
- [x] T012 [US1] Select internal full-chain packet membership for every managed Deep Capture scope while retaining explicit game-only application handling in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/orchestrator.rs`.
- [x] T013 [US1] Reconcile authorization, target guidance, Doctor, human/JSON terminal output, and fallback lineage in `crates/fragcap-cli/src/commands/deep_capture.rs`, `crates/fragcap-cli/src/commands/calibrate.rs`, and `crates/fragcap-cli/src/doctor/`.

**Checkpoint**: Default and explicit scope behavior is independently testable and existing game-only facts cannot qualify the new default.

## Phase 4: User Story 2 - Packet evidence spans the route lifetime (Priority: P1)

**Goal**: Retain eligible packets from launch inception through bounded route-owner release and report actual packet and correlation omissions.

**Independent Test**: A controlled platform/publisher/client/updater fixture emits before acquisition, during observation, and during release; one packet file and application stream reconcile all three windows.

### Tests for User Story 2

- [x] T014 [US2] Write failing three-window packet, proxy-loopback, ordering, and delayed-proof regressions in `crates/fragcap-cli/src/orchestrator.rs` and `crates/fragcap/src/session.rs`.
- [x] T015 [US2] Write failing cause-specific correlation and conservation regressions in `crates/fragcap/src/deep_capture/native.rs` and `crates/fragcap-core/src/pipeline/mod.rs`.

### Implementation for User Story 2

- [x] T016 [US2] Add a finite pending decision for only the writer-held head, reuse the existing single bounded packet buffer, enforce a managed-session byte ceiling, and count saturation in `crates/fragcap-core/src/pipeline/mod.rs` and `crates/fragcap/src/session.rs`.
- [x] T017 [US2] Open managed-chain packet admission at exact launch inception and update causal membership from acquisition and release watcher events in `crates/fragcap-cli/src/orchestrator.rs` and `crates/fragcap/src/session.rs`.
- [x] T018 [US2] Keep one packet pipeline and sink active beyond client observation until verified release or finite deadline in `crates/fragcap-cli/src/orchestrator.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T019 [US2] Carry actual gate cause into flow registry, application correlation, calibration reason lists, manifest, and terminal reports in `crates/fragcap-core/src/flow.rs`, `crates/fragcap/src/deep_capture/native.rs`, and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T020 [US2] Preserve complete versus partial packet-window truth under source, storage, packet-limit, byte-limit, and release-deadline failures in `crates/fragcap/src/deep_capture/manifest.rs` and `crates/fragcap-cli/src/orchestrator.rs`.

**Checkpoint**: The controlled packet file includes exact intermediate traffic in all three windows or names a counted induced failure.

## Phase 5: User Story 3 - Exactly owned helpers exit (Priority: P1)

**Goal**: Apply authorized bounded close and force to every exact session-created helper after client acquisition, and preserve unresolved resources when proof fails.

**Independent Test**: A controlled successful-client run with mixed same-image siblings reconciles process actions, proxy lifetime, route release, journal, and Doctor.

### Tests for User Story 3

- [x] T021 [US3] Write failing post-client mixed-helper, late-child, PID-reuse, and unrelated-process tests in `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs` and `crates/fragcap/src/deep_capture/process.rs`.
- [x] T022 [US3] Write failing journal, listener, route, and Doctor reconciliation cases in `crates/fragcap/src/deep_capture/session.rs` and `crates/fragcap-cli/src/doctor/fix.rs`.

### Implementation for User Story 3

- [x] T023 [US3] Expand bounded process-object receipt acquisition beyond declared stage paths, retaining exact generation, ancestry, path, account, and session proof in `crates/fragcap/src/deep_capture/process.rs` and `crates/fragcap/src/managed_launch.rs`.
- [x] T024 [US3] Show the causal-descendant shutdown class in the pre-effect plan and pass only proven live generations through the existing finite graceful/force sequence in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs`.
- [x] T025 [US3] Verify every applicable exit before route release and preserve exact incomplete state, listener ordering, journal, and Doctor recovery in `crates/fragcap/src/deep_capture/session.rs`, `crates/fragcap-cli/src/commands/deep_capture.rs`, and `crates/fragcap-cli/src/doctor/fix.rs`.

**Checkpoint**: No unowned process receives control; every owned helper exits or produces exact unresolved evidence.

## Phase 6: Polish and cross-cutting acceptance

- [x] T026 Update public behavior, versioned policy, evidence windows, and remaining #572/#574 limits in `docs/fragcap-specification.md`, `docs/plans/README.md`, `site/content/docs/`, and `docs/glossary/`.
- [x] T027 Add the user-visible change and dated architecture decisions to `changelog.d/` without editing published `CHANGELOG.md`.
- [x] T028 Reconcile controlled package certification and exact operator contracts in `.github/workflows/package-certification.yml` or existing package test sources only where S197 changes their assertions.
- [x] T029 Run `cargo xtask ci`, relevant Windows-controlled checks, documentation gates, and encoding/mojibake review at the exact local head; resolve every failure.
- [x] T030 Complete Spec-Kit analysis, update `specs/197-causal-chain-coverage/` artifacts and checklists, and rerun affected checks after any contract correction.
- [x] T031 Commit, push the explicitly authorized `codex/s197-causal-chain-coverage` branch, and open the official PR with issue closure and evidence references.
- [ ] T032 Wait for exact-head CI and all initial external reviews, address every actionable comment, then request no more than one second round with `@Codex` if needed; resolve that round and leave the green PR for owner final review and merge.

## Dependencies and Execution Order

- Setup precedes foundational contracts. Typed reasons and exact membership precede application-scope selection and packet-lifetime integration.
- User Story 1 can prove scope and fact policy independently once the foundation exists. User Story 2 extends packet lifetime and final evidence. User Story 3 extends the same exact membership to release authority. Final reconciliation waits for all three.
- Within each story, its targeted tests are written and observed failing before corresponding implementation. The three stories may use different controlled fixtures, but S197 closes only when the combined fixture reconciles all outcomes.

## Parallel Opportunities

- Read-only policy, packet, and shutdown research may proceed independently. After the common membership contract is fixed, isolated test fixtures in the three owning modules can be prepared in parallel without concurrent edits to shared files.

## Implementation Strategy

First establish the typed gate and exact membership, then deliver default full-chain selection, packet lifetime, and owned shutdown in dependency order. Verify each story independently before the combined controlled journey. Push only after complete local verification; the current owner authorization removes the skill's ordinary pre-push pause but does not authorize merge, tag, or release.
