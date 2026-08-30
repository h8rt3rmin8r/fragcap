# Tasks: Native Proxy Backend Spike

**Input**: Design documents from `/specs/099-native-proxy-spike/`

**Tests**: Required. S099 measures a security-sensitive proxy, fidelity, cleanup, licensing, and toolchain boundary. Contract and scenario tests precede harness implementation.

## Phase 1: Setup

- [x] T001 Freeze root `Cargo.toml`, `Cargo.lock`, and package metadata hashes in the local measurement ledger before adding spike files.
- [x] T002 Create the isolated nested workspace and exact candidate feature set in `spikes/native-proxy/Cargo.toml` and `spikes/native-proxy/Cargo.lock`.
- [x] T003 Add the repository-equivalent license, ban, advisory, and source policy in `spikes/native-proxy/deny.toml`.
- [x] T004 Document the non-shipping boundary, prerequisites, private-material lifecycle, and safe commands in `spikes/native-proxy/README.md`.

## Phase 2: Foundational Evidence Contracts

- [x] T005 Add failing tests for the normalized status vocabulary, complete row invariant, sanitization, and parity rules in `spikes/native-proxy/tests/matrix.rs`.
- [x] T006 Add failing tests for loopback-only endpoints, bounded deadlines, no trust mutation, and temporary sensitive paths in `spikes/native-proxy/tests/matrix.rs`.
- [x] T007 Implement normalized scenarios, observations, backend runs, comparisons, and deterministic JSON output in `spikes/native-proxy/src/evidence.rs`.
- [x] T008 Implement fixed synthetic payloads, exact byte accounting, stable digests, and scenario identities in `spikes/native-proxy/src/scenario.rs`.
- [x] T009 Implement bounded command dispatch for `candidate`, `baseline`, and `compare` in `spikes/native-proxy/src/main.rs`.

## Phase 3: User Story 1 - Measure the Native Candidate

**Goal**: Exercise the exact native candidate on Windows with complete lifecycle, protocol, CA, cache, and key-log evidence.

**Independent Test**: The candidate command runs the controlled matrix on loopback, records every required row, cancels cleanly, and leaves no trust or sensitive-path residue.

- [x] T010 [US1] Add failing candidate tests for HTTP/1.1 request and response body fidelity in `spikes/native-proxy/tests/matrix.rs`.
- [x] T011 [US1] Add failing candidate tests for HTTPS CONNECT, HTTP/2 negotiation, WebSocket messages, and HAR-source fields in `spikes/native-proxy/tests/matrix.rs`.
- [x] T012 [US1] Add failing candidate tests for ten startup and shutdown trials, active-connection cancellation, deadline failure, and residue reporting in `spikes/native-proxy/tests/matrix.rs`.
- [x] T013 [US1] Add failing candidate tests for private CA separation, bounded session cache, cache diagnostics, and client-facing proxy-owned key logging in `spikes/native-proxy/tests/matrix.rs`.
- [x] T014 [US1] Implement controlled loopback HTTP/1.1 and TLS origin servers and clients in `spikes/native-proxy/src/scenario.rs`.
- [x] T015 [US1] Implement controlled HTTP/2 and WebSocket exchanges in `spikes/native-proxy/src/scenario.rs`.
- [x] T016 [US1] Implement the `hudsucker` handler with lossless fixed-body observation and reconstruction in `spikes/native-proxy/src/candidate.rs`.
- [x] T017 [US1] Implement loopback listener ownership, readiness, cancellation, bounded drain, and cleanup in `spikes/native-proxy/src/candidate.rs`.
- [x] T018 [US1] Implement session-private CA generation/import and the public-API client-facing key-log wrapper in `spikes/native-proxy/src/candidate.rs`.
- [x] T019 [US1] Implement bounded cache configuration and observable cache evidence without reimplementing candidate internals in `spikes/native-proxy/src/candidate.rs`.
- [x] T020 [US1] Run the candidate matrix and ten lifecycle trials on Windows, then record sanitized results in `specs/099-native-proxy-spike/evidence.md`.

## Phase 4: User Story 2 - Compare the Shipped Baseline

**Goal**: Run the same local matrix through installed `mitmdump` and compare normalized results without inference.

**Independent Test**: Candidate and baseline output contain exactly one row per shared scenario and proof point, and only complete matching rows count as parity.

- [x] T021 [US2] Add failing baseline tests for bounded process startup, loopback binding, private configuration, child-scoped key logging, and cleanup in `spikes/native-proxy/tests/matrix.rs`.
- [x] T022 [US2] Add failing baseline tests for HTTP, HTTPS, HTTP/2, WebSocket, HAR-source, and normalized missing-result behavior in `spikes/native-proxy/tests/matrix.rs`.
- [x] T023 [US2] Implement the bounded external `mitmdump` adapter and private configuration lifecycle in `spikes/native-proxy/src/baseline.rs`.
- [x] T024 [US2] Implement normalized baseline observation ingestion and child-scoped HAR and key-log evidence in `spikes/native-proxy/src/baseline.rs`.
- [x] T025 [US2] Implement comparison joins and parity classification in `spikes/native-proxy/src/evidence.rs`.
- [x] T026 [US2] Run the baseline and comparison matrix on Windows and append sanitized results to `specs/099-native-proxy-spike/evidence.md`.

## Phase 5: User Story 3 - Audit Repository Compatibility

**Goal**: Produce reproducible dependency, license, MSRV, build-cost, and product-isolation evidence.

**Independent Test**: Every isolated package and target-conditional path is accounted for, both toolchains have explicit results, and the root graph and lock remain candidate-free.

- [x] T027 [P] [US3] Record exact Cargo metadata, active normal trees, all-target trees, package counts, features, sources, licenses, and root-store paths in `specs/099-native-proxy-spike/evidence.md`.
- [x] T028 [P] [US3] Run `cargo deny` against `spikes/native-proxy/deny.toml` and record every allowlist, advisory, source, or duplicate result in `specs/099-native-proxy-spike/evidence.md`.
- [x] T029 [P] [US3] Run locked checks and builds under Rust 1.82 and Rust 1.96, recording declared and effective minimum failures in `specs/099-native-proxy-spike/evidence.md`.
- [x] T030 [P] [US3] Measure reproducible clean and warm build times and target sizes in `specs/099-native-proxy-spike/evidence.md`.
- [x] T031 [US3] Compare root manifest, lock, package metadata, and release graph hashes before and after in `specs/099-native-proxy-spike/evidence.md`.

## Phase 6: User Story 4 - Record One Backend Decision

**Goal**: Close the native backend question with one dated outcome and one bounded follow-up.

**Independent Test**: Every issue criterion maps to explicit evidence, exactly one decision is selected, and the shipping backend remains unchanged.

- [x] T032 [US4] Build the 12-criterion pass, fail, unsupported, or not-measured table and identify every deciding limitation in `specs/099-native-proxy-spike/evidence.md`.
- [x] T033 [US4] Select one permitted outcome and record dated rationale, maintenance implications, and one follow-up boundary in `docs/plans/deep-capture-proxy-backends.md`.
- [x] T034 [US4] Correct the backend question status and non-shipping decision in `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md` without implying product adoption.
- [x] T035 [US4] Add the issue-linked research decision fragment in `changelog.d/S099-native-proxy-spike.decisions.md`.

## Phase 7: Verification and Local Commit

- [x] T036 Run isolated formatting, lint, tests, candidate, baseline, comparison, and audit commands in the foreground.
- [x] T037 Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all --locked`, and `cargo xtask ci` in the foreground.
- [x] T038 Audit all changed files for UTF-8 without BOM, LF endings, mojibake, disallowed dash characters, private evidence, secret material, non-loopback addresses, prohibited capabilities, and unintended product graph changes.
- [x] T039 Review the final diff against every S099 requirement and checklist item, mark all tasks complete, create one local feature commit, and halt before push.

## Dependencies and Execution Order

- Setup and foundational contracts block both backend runs.
- User Story 1 establishes the native evidence shape and controlled matrix.
- User Story 2 reuses that matrix and can begin after the foundational contract, but comparison completes only after User Story 1 has results.
- User Story 3 can run after the isolated lock exists and is parallel with protocol implementation except for final root-graph comparison.
- User Story 4 depends on every measured result and audit finding.
- Verification and commit follow the decision record.

## Parallel Opportunities

- T027 through T030 are independent audit measurements after the lock is frozen.
- Candidate protocol work and baseline process-adapter work touch separate modules, but shared scenario and evidence files remain sequential.
- Documentation updates T033 through T035 touch separate files after the decision is fixed.

## Implementation Strategy

Use test-driven measurement. Establish the normalized evidence and safety boundaries first, then make one HTTP/1.1 candidate path complete before adding TLS, HTTP/2, WebSocket, lifecycle repetition, and key logging. Run the external baseline only through the same scenario definitions. Treat failed and unsupported behavior as evidence, never as a reason to weaken assertions. No task authorizes product adoption, issue creation, push, tag, release, or trust mutation.
