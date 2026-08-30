# Tasks: Secure Native Proxy Foundation

**Input**: Design documents from `/specs/103-secure-native-proxy/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Security, lifecycle, protocol, loss-accounting, and platform-seam tests are mandatory and precede implementation.

## Phase 1: Setup

- [x] T001 Add exact direct ring, subtle, zeroize, dev-only x509-parser, and dev-only Quinn dependencies in Cargo.toml and crates/fragcap-proxy/Cargo.toml
- [x] T002 Add required windows-sys cryptography, authorization, filesystem, and memory features to crates/fragcap-proxy/Cargo.toml
- [x] T003 [P] Create module declarations and public boundary exports in crates/fragcap-proxy/src/lib.rs

## Phase 2: Shared Foundations

- [x] T004 Define stable identifiers, typed stage errors, finite stage budgets, and invariant helpers in crates/fragcap-proxy/src/model.rs
- [x] T005 Define injected resolver, root-loader, certificate-store, ACL, clock, and cancellation seams in crates/fragcap-proxy/src/model.rs
- [x] T006 Verify ignore files and repository dependency-direction policy for the new module and test paths in .gitignore and xtask/src/deps.rs

## Phase 3: User Story 1 - Admit Only the Selected Session (Priority: P1)

**Goal**: Authenticate one session before allocating upstream or payload work.

**Independent Test**: IPv4/IPv6 loopback, invalid capability, replay, port reuse, and race tests reconcile all admitted/refused attempts.

- [x] T007 [P] [US1] Add capability secrecy, entropy, constant-time equality, invalidation, and encoding tests in crates/fragcap-proxy/tests/authentication.rs
- [x] T008 [US1] Add listener pre-authentication, unauthorized payload refusal, saturation, stop-race, and port-reuse tests in crates/fragcap-proxy/tests/authentication.rs
- [x] T009 [US1] Implement SessionCapability and protocol authorization adapters in crates/fragcap-proxy/src/auth.rs
- [x] T010 [US1] Authenticate before spawning connection tasks and add admitted/refused accounting in crates/fragcap-proxy/src/runtime.rs

## Phase 4: User Story 2 - Connect Upstream Without Weakening Policy (Priority: P1)

**Goal**: Parse, resolve, policy-check, connect, verify, cancel, and report upstream work under finite budgets.

**Independent Test**: Controlled resolver/origin tests cover valid authorities, recursion, rebinding, private grants, IPv4/IPv6, verification failures, timeouts, and cancellation.

- [x] T011 [P] [US2] Add authority grammar and address-policy table tests in crates/fragcap-proxy/tests/upstream.rs
- [x] T012 [US2] Add deterministic DNS, rebinding, connect, TLS verification, empty-root, timeout, and cancellation tests in crates/fragcap-proxy/tests/upstream.rs
- [x] T013 [US2] Implement validated DestinationAuthority and DestinationPolicy in crates/fragcap-proxy/src/upstream.rs
- [x] T014 [US2] Implement bounded resolution, TCP connection, read/write stream operations, native-root loading, and verified TLS connection in crates/fragcap-proxy/src/upstream.rs
- [x] T015 [US2] Emit typed upstream attempt outcomes without application-status fabrication in crates/fragcap-proxy/src/event.rs

## Phase 5: User Story 3 - Own Certificate and Trust State Exactly (Priority: P1)

**Goal**: Generate, protect, issue, cache, trust, inspect, and clean only session-owned certificate state.

**Independent Test**: Synthetic certificate and store-adapter tests prove SAN/usage/validity, count/byte/lifetime bounds, rotation, permissions, exact store mutation, and partial-failure recovery.

- [x] T016 [P] [US3] Add independent CA and leaf DER semantic tests in crates/fragcap-proxy/tests/certificates.rs
- [x] T017 [US3] Add leaf-cache concurrency, bounds, eviction, CA rotation, policy rotation, and malformed-identity tests in crates/fragcap-proxy/tests/certificates.rs
- [x] T018 [US3] Add protected-storage inventory, partial-write, cleanup, and ACL-seam tests in crates/fragcap-proxy/tests/certificates.rs
- [x] T019 [US3] Add exact current-user Root add/query/remove, duplicate, mismatch, wrong-store, denial, and idempotency tests in crates/fragcap-proxy/tests/trust.rs
- [x] T020 [US3] Implement zeroized per-session CA generation, fingerprints, protected inventory, and cleanup in crates/fragcap-proxy/src/certificate.rs
- [x] T021 [US3] Implement bounded concurrent LeafCache issuance and invalidation in crates/fragcap-proxy/src/certificate.rs
- [x] T022 [US3] Implement injected trust state machine and typed exact-mutation outcomes in crates/fragcap-proxy/src/trust.rs
- [x] T023 [US3] Implement cfg-isolated current-user CryptoAPI store effects in crates/fragcap-proxy/src/windows/trust.rs
- [x] T024 [US3] Implement cfg-isolated DPAPI persistence, plaintext zeroization, and protected ACL effects in crates/fragcap-proxy/src/windows/acl.rs
- [x] T025 [US3] Expose shared native trust through crates/fragcap/src/deep_capture/native.rs and replace session/doctor certutil calls in crates/fragcap-cli/src/commands/deep_capture.rs and crates/fragcap-cli/src/doctor/fix.rs

## Phase 6: User Story 4 - Preserve Every Native Observation and Gap (Priority: P1)

**Goal**: Carry every native event family through one bounded, ordered, loss-accounted raw stream.

**Independent Test**: Round-trip, ordering, overflow, truncation, unknown, malformed, refusal, and projection-gap tests prove conservation and completeness behavior.

- [x] T026 [P] [US4] Add event-family, round-trip, ordering, and correlation tests in crates/fragcap-proxy/tests/events.rs
- [x] T027 [US4] Add drop-oldest, payload-bound, refusal, unparsed, projection-gap, and conservation tests in crates/fragcap-proxy/tests/events.rs
- [x] T028 [US4] Implement RawObservation, event families, payload state, and typed provenance in crates/fragcap-proxy/src/event.rs
- [x] T029 [US4] Implement bounded ObservationStream, drop-oldest behavior, snapshots, and completeness in crates/fragcap-proxy/src/event.rs
- [x] T030 [US4] Connect listener and upstream lifecycle outcomes to the shared event stream in crates/fragcap-proxy/src/runtime.rs and crates/fragcap-proxy/src/upstream.rs

## Phase 7: User Story 5 - Prove the Foundation in a Controlled Protocol Lab (Priority: P1)

**Goal**: Provide deterministic local truth and failure fixtures for every required protocol family.

**Independent Test**: The complete protocol-by-case matrix runs offline with synthetic data and every endpoint task reaches a named terminal state.

- [x] T031 [P] [US5] Define scenario, fidelity, output-expectation, and truth-ledger support in crates/fragcap-proxy/tests/protocol_lab_support/model.rs and crates/fragcap-proxy/tests/protocol_lab_support/truth.rs
- [x] T032 [US5] Implement deterministic TCP, UDP, TLS, and Quinn loopback transports with channel barriers in crates/fragcap-proxy/tests/protocol_lab_support/transport.rs
- [x] T033 [US5] Implement HTTP/1.1, HTTPS, HTTP/2, streaming, WebSocket, gRPC, raw TCP, non-HTTP TLS, SOCKS, UDP, and QUIC scenarios in crates/fragcap-proxy/tests/protocol_lab_support/scenarios.rs
- [x] T034 [US5] Add positive, refusal, malformed, timeout, cancellation, disconnect, and cleanup-failure matrix assertions in crates/fragcap-proxy/tests/protocol_lab.rs
- [x] T035 [US5] Add synthetic-data rejection, unavailable packet truth, output separation, determinism, and resource-conservation assertions in crates/fragcap-proxy/tests/protocol_lab.rs

## Phase 8: Polish and Cross-Cutting Gates

- [x] T036 [P] Update dependency policy and native foundation architecture in Cargo.toml, AGENTS.md, docs/fragcap-specification.md, and docs/fragcap-spec-outline.md
- [x] T037 [P] Add or update glossary entries and public incomplete-status documentation in docs/glossary/, README.md, CONTRIBUTING.md, and site/content/docs/
- [x] T038 Add user-visible and dated architecture decision fragments in changelog.d/S103-native-proxy-foundation.added.md and changelog.d/S103-native-proxy-foundation.decisions.md
- [x] T039 Run focused tests and resolve every failure from specs/103-secure-native-proxy/quickstart.md
- [x] T040 Run format, clippy, full locked tests, xtask lint/deps/spec/msrv, Cargo deny, package, Windows feature, documentation, encoding, and mojibake checks
- [x] T041 Mark all completed tasks and the S103 spec status complete in specs/103-secure-native-proxy/tasks.md and specs/103-secure-native-proxy/spec.md
- [x] T042 Commit the verified slice locally with the repository conventional message and co-author trailer

## Dependencies and Execution Order

```text
Setup -> Shared Foundations -> US1 -> US2 -> US3 -> US4 -> US5 -> Polish
```

US1 through US4 expose independently testable contracts, but the implementation order is chronological because later stories consume earlier owners. US5 depends on every preceding story. Tests in each story are written and observed failing before implementation.

## Parallel Opportunities

- T003 can proceed independently after dependency declarations are known.
- T007, T011, T016, T026, T031, T036, and T037 affect separate files and can be prepared in parallel at their phase boundary.
- Portable certificate/store tests and Windows effect modules remain separate so ordinary CI never mutates real trust.

## Implementation Strategy

Complete the whole ordered slice before the pre-push halt because #289 is the foundation proof and #290 depends on every earlier contract. Validate each user story independently at its checkpoint, then run the complete repository gate once all seven issues are satisfied.
