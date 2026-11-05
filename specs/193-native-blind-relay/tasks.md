# Tasks: Native Blind Relay

**Slice**: S193 | **Branch**: `codex/s193-native-blind-relay` | **Issue**: #559

## Phase 1: Decision and HTTP/1

- [x] T001 Confirm S193 selector, branch, dependencies, and cross-artifact analysis before implementation.
- [x] T002 [US1] Add controlled same-hostname pinned-launcher and inspected-game regression using one authenticated listener.
- [x] T003 [US1] Implement HTTP/1 CONNECT blind relay before leaf creation, including bounded half-close, inactivity, cancellation, and terminal byte totals.
- [x] T004 [US2] Implement non-exporting plaintext HTTP forwarding and metadata-only relay diagnostics.

## Phase 2: HTTP/2 and SOCKS

- [x] T005 [US2] Add cleartext HTTP/2 concurrent request and CONNECT coverage with excluded-content assertions.
- [x] T006 [US2] Propagate frozen action through HTTP/2 streams and implement policy-checked CONNECT streaming.
- [x] T007 [US2] Add SOCKS TCP and UDP excluded-owner regressions for IPv4/IPv6, half-close, timeout, cancellation, policy, and peer pinning.
- [x] T008 [US2] Suppress SOCKS generic chunk export and route excluded QUIC Initial through original datagram forwarding before native QUIC termination.

## Phase 3: Evidence and lifetime

- [x] T009 [US3] Add explicit relay, inspection, uncertainty, omission, loss, failure, and cleanup reconciliation to diagnostics/artifacts without mixing connection and datagram populations.
- [x] T010 [US3] Add progressing-download, idle-relay, writer-failure, saturation, failed-game-TLS, and ancestor-after-game-exit cases; fix any bounded-lifetime or route-owner mismatch.
- [x] T011 Reconcile protocol, artifact, API, glossary, architecture, chronology, and changelog documentation.
- [x] T012 Run focused and full applicable repository checks, Windows/package checks, mojibake scan, and `git diff --check`; resolve failures.
- [x] T013 Run Spec-Kit convergence and complete every remaining buildable task before commit.

## Dependencies

T001 precedes implementation. T002 precedes T003, and T005 precedes T006. T007 precedes T008. T009 follows every transport. T010 through T013 close the slice. #560 and #561 consume this runtime work; #542 remains independent.
