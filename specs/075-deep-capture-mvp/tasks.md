# Tasks: Deep Capture MVP

**Input**: Design documents from `/specs/075-deep-capture-mvp/`

**Prerequisites**: `spec.md`, `research.md`, `data-model.md`, `analysis.md`, `quickstart.md`, `contracts/deep-capture-cli.md`, `contracts/status-events.md`, `contracts/mitmdump-backend.md`, `contracts/bundle-validation.md`

## Phase 1: Setup

- [X] T001 Re-read `AI_CONTEXT.md`, issue #219, `.specify/memory/constitution.md`, `docs/plans/deep-capture.md`, #216 bundle artifacts, #217 compatibility facts, #218 doctor implementation, and the active CLI/capture code.
- [X] T002 Create branch `codex/219-deep-capture-mvp`.
- [X] T003 Update `.specify/feature.json` to `specs/075-deep-capture-mvp`.
- [X] T004 Create Spec Kit artifacts for #219 before Rust implementation.

## Phase 2: Command Contract

- [ ] T005 Add a first-class Deep Capture CLI command surface in `crates/fragcap-cli/src/cli.rs`.
- [ ] T006 Route the command in `crates/fragcap-cli/src/main.rs` and `crates/fragcap-cli/src/commands/mod.rs`.
- [ ] T007 Implement stored-target-only resolution by reusing `commands/target_resolve`.
- [ ] T008 Refuse raw process input, missing launch ownership, unknown scoped proxy facts, missing backend, and unconfirmed trust before side effects.
- [ ] T009 Add CLI help and refusal tests under `crates/fragcap-cli/tests`.

## Phase 3: Adapter Seams

- [ ] T010 Add narrow proxy backend types for discovery, start, readiness, event ingestion, stop, and cleanup.
- [ ] T011 Implement a fake proxy backend for deterministic tests.
- [ ] T012 Implement the MVP `mitmdump` backend adapter as an owned child process.
- [ ] T013 Add narrow trust manager types for CA material, current-user trust state, confirmation, and cleanup.
- [ ] T014 Implement a fake trust manager for deterministic tests.
- [ ] T015 Implement the Windows current-user trust path only where explicit confirmation is present.

## Phase 4: Session Orchestration

- [ ] T016 Introduce `DeepCaptureSession` coordination for session id, bundle root, target, proxy, trust, launch, capture, observations, status, and cleanup.
- [ ] T017 Reuse the existing capture orchestrator for `.fcapng` packet truth instead of duplicating packet capture.
- [ ] T018 Apply launch-scoped proxy configuration only through the managed launch environment or equivalent target-scoped launch surface.
- [ ] T019 Preserve clean interrupt handling across proxy, launched target, capture pipeline, sidecar writers, and cleanup.
- [ ] T020 Add Deep Capture lifecycle events to `crates/fragcap-cli/src/events.rs`.

## Phase 5: Bundle Outputs

- [ ] T021 Implement bundle directory and manifest writer according to #216.
- [ ] T022 Implement application JSONL writer for inspectable, metadata-only, unsupported, and error records.
- [ ] T023 Implement HAR output when HTTP semantics are observable.
- [ ] T024 Implement proxy and process sidecars with scrubbed structured records.
- [ ] T025 Implement cleanup report writing and manifest cleanup aggregate updates.
- [ ] T026 Ensure secret-adjacent key logs are opt-in and manifest-marked when produced.

## Phase 6: Compatibility Facts

- [ ] T027 Map session observations to `CompatibilityFact` values using existing closed vocabularies.
- [ ] T028 Insert scrubbed facts into `deep_capture_facts` after successful or partially successful sessions.
- [ ] T029 Record refusal-relevant facts only when they were actually observed, never guessed.
- [ ] T030 Add compatibility fact tests for proxy routing, propagation, trust behavior, protocol behavior, inspectability, and final owner role.

## Phase 7: Controlled Target Verification

- [ ] T031 Build a controlled local target harness with placeholder process names and loopback endpoints.
- [ ] T032 Exercise HTTP through the proxy and verify application JSONL plus optional HAR.
- [ ] T033 Exercise HTTPS with the fake trust manager and verify trust-required, trust-declined, and trust-accepted paths.
- [ ] T034 Exercise metadata-only and unsupported protocol records.
- [ ] T035 Add optional ignored demonstration coverage for real `mitmdump` when installed.

## Phase 8: Documentation and Specification

- [ ] T036 Update `docs/fragcap-specification.md` to mark #219 MVP behavior and CLI/output contracts.
- [ ] T037 Update `docs/plans/deep-capture.md` only if implementation decisions refine the standing plan.
- [ ] T038 Add or update command help goldens.
- [ ] T039 Add changelog fragment.

## Phase 9: Verification

- [ ] T040 Run `cargo fmt --check`.
- [ ] T041 Run `git diff --check`.
- [ ] T042 Run `cargo test -p fragcap-cli --quiet`.
- [ ] T043 Run `cargo test -p fragcap-targets --quiet`.
- [ ] T044 Run `cargo test --workspace --quiet`.
- [ ] T045 Run `cargo xtask lint`.
- [ ] T046 Run `cargo xtask deps`.
- [ ] T047 Run `cargo xtask spec`.
- [ ] T048 Run `cargo xtask changelog --check`.
- [ ] T049 Scan added lines for local PII, actual title names, local paths, endpoints, credentials, account material, and captured payloads.
- [ ] T050 Review PR comments from AI agents and respond under each respective comment before final human review.
