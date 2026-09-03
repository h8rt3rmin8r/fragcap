# Tasks: Generic UDP Observations

**Input**: Design documents from `specs/117-generic-udp-observations/`

**Tests**: Required and written before implementation.

## Phase 1: Specification And Contracts

- [x] T001 Record spec, clarifications, requirements/security checklists, research, data model, evidence contract, plan, quickstart, and tasks
- [x] T002 Confirm no dependency or lockfile change is required

## Phase 2: Generic Datagram Foundation

- [x] T003 Add failing type, retention, boundary, sequence, and omission tests
- [x] T004 Add generic UDP datagram and socket-error types plus protocol accounting
- [x] T005 Implement one-record bounded datagram retention using shared connection and session budgets

## Phase 3: User Story 1 - Exact Routed Datagrams

- [x] T006 Add failing IPv4, available IPv6, domain, empty, duplicate, and reorder tests
- [x] T007 Observe client and upstream ingress at the accepted S115 boundary
- [x] T008 Reconcile directional sequences, endpoints, timestamps, payloads, and forwarding totals

## Phase 4: User Story 2 - Bounds And Independence

- [x] T009 Add failing capture-disabled, partial retention, session exhaustion, and queue-pressure tests
- [x] T010 Preserve complete forwarding under every retention and writer outcome
- [x] T011 Add exact generic UDP observed, retained, omitted, truncated, and queue-loss accounting

## Phase 5: User Story 3 - Errors And Artifacts

- [x] T012 Add failing socket-error, storage-failure, unrouted omission, correlation, and cleanup tests
- [x] T013 Record platform-observed socket errors without ICMP inference
- [x] T014 Serialize `generic.udp_datagram` in application JSON Lines version 2
- [x] T015 Prove bounded localized loss, exact overflow totals, and terminal cleanup

## Phase 6: Documentation And Verification

- [x] T016 Update glossary, architecture, outline, plan status, proxy README, AGENTS, and changelog
- [x] T017 Run focused tests and full `cargo xtask ci`
- [x] T018 Run dependency, lockfile, UTF-8, mojibake, and diff sanity checks
- [x] T019 Complete all task boxes and reconcile issue #313 acceptance

## Execution Order

The type and retention model precede relay integration. Accepted ingress tests precede implementation. Stable evidence precedes serialization and documentation. Full verification finishes the slice.
