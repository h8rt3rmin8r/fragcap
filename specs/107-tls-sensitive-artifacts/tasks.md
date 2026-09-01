# Tasks: TLS Evidence and Sensitive Artifact Lifecycle

**Input**: Design documents from `specs/107-tls-sensitive-artifacts/`

## Phase 1: Setup and Contracts

- [x] T001 Record S107 issue traceability and dependency decisions in `specs/107-tls-sensitive-artifacts/`
- [x] T002 [P] Add failing key-log and mutual-TLS contract tests in `crates/fragcap-proxy/tests/`
- [x] T003 [P] Add failing sensitive lifecycle tests in `crates/fragcap/src/deep_capture/artifacts.rs`
- [x] T004 [P] Add failing CLI argument and bundle-command tests in `crates/fragcap-cli/tests/`

## Phase 2: Foundational Types and Lifecycle

- [x] T005 Define stable key-log status, client identity, and TLS refusal models in `crates/fragcap-proxy/src/`
- [x] T006 Define retention, sensitive action, cleanup, and share transformation models in `crates/fragcap/src/deep_capture/`
- [x] T007 Move authorized artifact preparation before proxy startup through `ArtifactSink` in `crates/fragcap/src/deep_capture/adapters.rs` and `session.rs`

## Phase 3: User Story 1 - Authorized TLS Key Logs

- [x] T008 [US1] Implement the allowlisted, redacted, serialized, live-flushed session key logger in `crates/fragcap-proxy/src/key_log.rs`
- [x] T009 [US1] Attach the logger only to client-facing server configurations through `tls.rs` and `runtime.rs`
- [x] T010 [US1] Create and protect the final bundle path before proxy start and surface exact final status through `native.rs` and CLI bundle assembly
- [x] T011 [US1] Prove TLS 1.2/TLS 1.3 analyzer format, concurrency, no-authorization, upstream exclusion, and failure reporting

## Phase 4: User Story 2 - Explicit Mutual TLS and Refusal Evidence

- [x] T012 [US2] Parse a paired explicit client certificate chain and private key without target discovery
- [x] T013 [US2] Configure only intended upstream connections with the validated identity
- [x] T014 [US2] Classify rustls variants and alerts into stable non-secret refusal records
- [x] T015 [US2] Serialize refusal evidence through the application stream and compatibility observations
- [x] T016 [US2] Prove accepted, missing, rejected, expired, mismatched, protocol, validation, and ambiguous cases

## Phase 5: User Story 3 - Sensitive Artifact Lifecycle

- [x] T017 [US3] Implement protected bundle and strict sensitive-file creation with Windows ACLs and portable owner-only permissions
- [x] T018 [US3] Implement the bounded synced sensitive action journal and pending-action replay
- [x] T019 [US3] Implement exact, confirmed, idempotent completed-bundle sensitive cleanup
- [x] T020 [US3] Implement protected atomic share-on-copy with exhaustive transformation manifest and immutable source
- [x] T021 [US3] Restrict doctor cleanup to unfinished or pending residue
- [x] T022 [US3] Prove permission, traversal, fault, partial failure, replay, cleanup, and sharing contracts

## Phase 6: User Story 4 - Complete Platform Trigger

- [x] T023 [US4] Remove path filters from `.github/workflows/platform.yml`
- [x] T024 [US4] Add an xtask lint contract that refuses filtered whole-workspace platform triggers
- [x] T025 [US4] Record the dated pinned-workflow decision in `changelog.d/`

## Phase 7: Integration and Documentation

- [x] T026 Add CLI flags for explicit paired client credentials and expose the fixed retain-until-explicit-cleanup policy
- [x] T027 Add confirmed `bundle cleanup` and atomic `bundle export` command surfaces
- [x] T028 Update master specification, outline, plans index, public CLI documentation, README surfaces, and changelog
- [x] T029 Complete `checklists/security.md` from verified evidence

## Phase 8: Verification and Delivery

- [x] T030 Run focused tests after each story and repair every failure
- [x] T031 Run Spec-kit analyze and remediate all blocking findings
- [x] T032 Run formatting, lint, dependency, test, MSRV, documentation, advisory, license, denylist, encoding, and diff audits
- [x] T033 Verify issue closure mapping for #300, #304, and #322 while leaving #320 open
- [x] T034 Commit the complete S107 slice locally and halt before push

## Dependencies and Execution Order

- T001 through T007 establish the shared contracts and lifecycle boundary.
- US1 and US2 share TLS configuration but are independently testable after T005 and T007.
- US3 depends on T007 and supplies the protected file handle required by US1.
- US4 is independent after setup.
- Integration, analysis, and delivery follow all four stories.

## Implementation Strategy

Implement tests before each production capability, keep key-log and private-key bytes out of displayable types, and treat every artifact or refusal outcome as data. The deliberate deviation from the previous architecture is T007: finalization-only protection is too late because the native application writer opens during proxy startup.
