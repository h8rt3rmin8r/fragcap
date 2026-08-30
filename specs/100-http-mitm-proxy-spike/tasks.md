# Tasks: Smaller Native Proxy Fallback Spike

**Input**: Design documents from `/specs/100-http-mitm-proxy-spike/`

**Tests**: Required. S100 measures a security-sensitive proxy, fidelity, cleanup, licensing, and toolchain boundary. Contract tests precede implementation.

## Phase 1: Setup

- [x] T001 Freeze root manifest, lock, and released metadata hashes in the local measurement ledger.
- [x] T002 Create exact isolated candidate and audit manifests under `spikes/http-mitm-proxy/`.
- [x] T003 Add repository-equivalent license, source, ban, and advisory policy in `spikes/http-mitm-proxy/deny.toml`.
- [x] T004 Document the non-shipping boundary and private-material lifecycle in `spikes/http-mitm-proxy/README.md`.

## Phase 2: Foundational Contracts

- [x] T005 Add failing normalized-state, complete-row, sanitization, and parity tests in `spikes/http-mitm-proxy/tests/matrix.rs`.
- [x] T006 Add failing loopback, deadline, trust-separation, and temporary-path tests in `spikes/http-mitm-proxy/tests/matrix.rs`.
- [x] T007 Implement deterministic evidence rows and three-way comparison in `spikes/http-mitm-proxy/src/evidence.rs`.
- [x] T008 Adapt S099 fixed payloads and scenario identities in `spikes/http-mitm-proxy/src/scenario.rs`.
- [x] T009 Implement bounded candidate command dispatch in `spikes/http-mitm-proxy/src/main.rs`.

## Phase 3: User Story 1 - Measure the Fallback

- [x] T010 [US1] Add candidate tests for HTTP/1.1 request and response fidelity.
- [x] T011 [US1] Add candidate tests for HTTPS CONNECT, HTTP/2, WebSocket handshake/messages, and HAR-source fields.
- [x] T012 [US1] Add candidate tests for ten lifecycle trials, active-connection cancellation, deadlines, and residue.
- [x] T013 [US1] Add candidate tests for private CA separation, bounded certificate cache, and client-facing key logging.
- [x] T014 [US1] Adapt controlled loopback origin servers and clients in `spikes/http-mitm-proxy/src/scenario.rs`.
- [x] T015 [US1] Implement request, response, protocol, and HAR-source observation in `spikes/http-mitm-proxy/src/candidate.rs`.
- [x] T016 [US1] Implement public upgraded-stream WebSocket observation in `spikes/http-mitm-proxy/src/candidate.rs`.
- [x] T017 [US1] Implement loopback listener ownership, cancellation, deadline, and cleanup evidence in `spikes/http-mitm-proxy/src/candidate.rs`.
- [x] T018 [US1] Implement session-private CA generation and explicit client-only trust.
- [x] T019 [US1] Implement bounded cache evidence and public-interface key-log classification.
- [x] T020 [US1] Run the candidate matrix and record sanitized results in `specs/100-http-mitm-proxy-spike/evidence.md`.

## Phase 4: User Story 2 - Compare Three Backends

- [x] T021 [US2] Add tests proving every S099 key appears for all three backends.
- [x] T022 [US2] Add tests that unsupported, failed, and not-measured rows never imply parity.
- [x] T023 [US2] Review the committed S099 normalized evidence and align it by proof-point key in `specs/100-http-mitm-proxy-spike/evidence.md`.
- [x] T024 [US2] Build and record the three-way comparison in `specs/100-http-mitm-proxy-spike/evidence.md`.

## Phase 5: User Story 3 - Audit Compatibility

- [x] T025 [P] [US3] Record exact normal and all-target dependency paths, counts, features, sources, licenses, and root-store paths.
- [x] T026 [P] [US3] Run `cargo deny` and record allowlist, advisory, source, and duplicate results.
- [x] T027 [P] [US3] Run parse, check, and build under Rust 1.82 and Rust 1.96.
- [x] T028 [P] [US3] Measure clean and warm build times and target size.
- [x] T029 [US3] Prove root manifest, lock, metadata, and release graph isolation.

## Phase 6: User Story 4 - Close the Decision

- [x] T030 [US4] Map every issue criterion to complete, fail, unsupported, or not-measured evidence.
- [x] T031 [US4] Select exactly one backend outcome in `docs/plans/deep-capture-proxy-backends.md` with no new candidate path.
- [x] T032 [US4] Update `docs/fragcap-specification.md` and `docs/fragcap-spec-outline.md` without implying product adoption.
- [x] T033 [US4] Add `changelog.d/S100-http-mitm-proxy-spike.decisions.md`.

## Phase 7: Verification and Commit

- [x] T034 Run isolated formatting, clippy, tests, candidate, comparison, and audit commands.
- [x] T035 Run root formatting, full clippy, locked tests, and `cargo xtask ci`.
- [x] T036 Audit UTF-8 without BOM, LF, mojibake, dash characters, private material, non-loopback addresses, prohibited capabilities, and product graph isolation.
- [x] T037 Review every S100 requirement and checklist item, mark tasks complete, create one local commit, and halt before push.

## Dependencies

Setup and foundational contracts block candidate work. Candidate evidence blocks comparison. All measurements block the decision. Verification follows documentation.

## Parallel Opportunities

T025 through T028 are independent once the locks exist. Source and documentation files are otherwise kept sequential because the shared evidence record is load-bearing.

## Implementation Strategy

Adapt the proven S099 harness contract, then replace only the candidate adapter. Treat unsupported behavior as a valid result, never as permission to weaken a test. No task authorizes product adoption, another backend search, push, tag, release, or trust mutation.
