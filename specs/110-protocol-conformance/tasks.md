# Tasks: Native Protocol Conformance

**Input**: Design documents from `specs/110-protocol-conformance/`

**Prerequisites**: spec.md, plan.md, research.md, data-model.md, contracts/

## Phase 1: Evidence Contract and Failing Gates

- [x] T001 Persist S110 feature state and complete specification, clarification, checklist, research, data model, contract, and quickstart artifacts under `specs/110-protocol-conformance/`
- [x] T002 Add failing closed-matrix validation tests for missing, duplicate, skipped, not-run, stale, unresolved, and same-lineage required rows in `xtask/src/conformance.rs`
- [x] T003 Add failing deterministic evidence, sanitization, and analyzer semantic-output tests in `xtask/src/conformance.rs`
- [x] T004 Define the versioned matrix and report fixtures under `conformance/native-http-tls/`

## Phase 2: User Story 1 - Complete Independent Matrix

**Goal**: Prove every required protocol with two independent client and origin implementations and explicit failure boundaries.

**Independent Test**: Run the conformance matrix validator and focused proxy harness tests, then prove exact computed coverage with zero skip state.

- [x] T005 [US1] Implement matrix and report parsing, validation, lineage counting, result reconciliation, and diagnostics in `xtask/src/conformance.rs`
- [x] T006 [US1] Register `cargo xtask conformance` and update xtask help and unit coverage in `xtask/src/main.rs`
- [x] T007 [US1] Add missing bounded raw HTTP/2, Hyper HTTP/1.1 and HTTPS, wire SSE, and h2 SSE and gRPC peers in `crates/fragcap-proxy/tests/conformance.rs`
- [x] T008 [US1] Bind positive HTTP/1.1, HTTPS TLS 1.2 and TLS 1.3, HTTP/2, WebSocket HTTP/1.1 and RFC 8441, SSE, and gRPC rows to independent executable proxy tests
- [x] T009 [US1] Bind authentication, malformed framing, wrong-name, untrusted-chain, disconnect, timeout, cancellation, and cleanup failure rows to executable tests
- [x] T010 [US1] Prove implementation lineage and exact version identities against Cargo.lock and executing tools in xtask tests
- [x] T011 [US1] Run focused proxy conformance and matrix gates and mark US1 complete

## Phase 3: User Story 2 - Integrated Synthetic Evidence

**Goal**: Reconcile protocol behavior with the full native Deep Capture evidence bundle.

**Independent Test**: Generate a controlled bundle, validate every production artifact authority, and reproduce the committed normalized report byte for byte.

- [x] T012 [US2] Add failing full-bundle conformance and evidence drift tests in `crates/fragcap/tests/native_conformance.rs`
- [x] T013 [US2] Reuse executable native success and expected-failure rows and add one controlled cross-artifact authority session in `crates/fragcap/tests/native_conformance.rs`
- [x] T014 [US2] Validate application JSON Lines and HAR with production readers and retain existing exact body, timing, omission, and loss tests as row evidence
- [x] T015 [US2] Validate lifecycle, resource journal, and manifest version 2 together while binding existing key-log, correlation, cleanup summary, and artifact tests into the report
- [x] T016 [US2] Commit the normalized report and implement exact matrix, result, executable-reference, and analyzer-fixture drift comparison in xtask
- [x] T017 [US2] Add capability, credential, private-key, path, endpoint, timestamp, UTF-8, BOM, and mojibake sanitization gates
- [x] T018 [US2] Run focused facade conformance, drift, and sanitization gates and mark US2 complete

## Phase 4: User Story 3 - Unmodified Analyzer Proof

**Goal**: Require unmodified TShark consumption of committed packet and key-log artifacts.

**Independent Test**: Run analyzer mode with TShark and require nonzero packets plus every declared protocol fact.

- [x] T019 [US3] Commit deterministic synthetic `analyzer.pcapng` and `tls-keylog.log` fixtures under `conformance/native-http-tls/`
- [x] T020 [US3] Implement required TShark discovery, version capture, pcapng read, key-log preference, frame count, protocol field, and diagnostic checks in `xtask/src/conformance.rs`
- [x] T021 [US3] Add portable Windows and Linux conformance execution and a dedicated required Ubuntu TShark job in `.github/workflows/ci.yml`
- [x] T022 [US3] Prove missing TShark, unreadable input, zero packet, and absent field cases fail instead of skip
- [x] T023 [US3] Run the local analyzer gate when TShark is available and validate the CI command contract when it is not

## Phase 5: Documentation and Verification

- [x] T024 Correct stale S110 generic-transport prose and record the #305 milestone exit in `AGENTS.md`, `docs/plans/README.md`, and `docs/fragcap-specification.md`
- [x] T025 Document matrix reproduction, analyzer requirements, evidence review, and incomplete Deep Capture status in repository documentation
- [x] T026 Add S110 feature and dated decision fragments under `changelog.d/`
- [x] T027 Run spec-kit analysis, remediate every finding, and rerun until clean
- [x] T028 Run spec-kit convergence, append and implement any missing tasks, and rerun until converged
- [x] T029 Verify issue closure mapping for #305 while leaving #310 through #334 open as applicable
- [x] T030 Run formatting, clippy, locked tests, xtask CI, MSRV, dependency, platform, encoding, mojibake, evidence drift, and diff checks

## Dependencies and Execution Order

1. T001-T004 establish the closed evidence contract and failing gates.
2. T005-T011 complete independent protocol interoperability.
3. T012-T018 consume protocol truth to complete cross-artifact evidence.
4. T019-T023 add external analyzer proof after stable fixtures exist.
5. T024-T030 reconcile repository narrative and complete all gates.

## Implementation Strategy

Implement in strict user-story and test-first order. A row is complete only when its executable behavior, normalized observation, artifact assertions, and required tier all pass. No required row may be deferred, skipped, or converted to informational status.
