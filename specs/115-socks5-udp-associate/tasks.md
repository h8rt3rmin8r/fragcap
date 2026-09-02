# Tasks: Scoped SOCKS5 UDP Association

**Input**: Design documents from `specs/115-socks5-udp-associate/`

**Tests**: Required and written before implementation.

## Phase 1: Specification And Contracts

- [x] T001 Record spec, clarifications, requirements/security checklists, research, data model, wire/evidence contracts, plan, quickstart, and tasks
- [x] T002 Confirm no dependency or lockfile change is required

## Phase 2: Foundational Wire And State

- [x] T003 Add failing UDP frame parser, encoder, malformed, fragment, and bounds unit tests
- [x] T004 Generalize authenticated request parsing for CONNECT and UDP ASSOCIATE
- [x] T005 Add bounded UDP association limits, accounting, and typed application events

## Phase 3: User Story 1 - Authorized Datagram Relay

- [x] T006 Add failing IPv4, available IPv6, domain, and control-lifetime loopback tests
- [x] T007 Bind fixed relay/upstream sockets and implement policy-checked frame-preserving forwarding
- [x] T008 Prove exact response source framing and control EOF revocation

## Phase 4: User Story 2 - Security And Loss

- [x] T009 Add failing spoofed client, unsolicited reply, local policy, malformed, fragment, oversized, peer saturation, and timeout tests
- [x] T010 Implement immutable client pinning, exact peer validation, finite bounds, and named drop accounting
- [x] T011 Prove reflection and local hijack attempts produce zero unauthorized delivery

## Phase 5: User Story 3 - Evidence And Cleanup

- [x] T012 Add failing typed event, conservation, mapping cleanup, cancellation, and forced-shutdown tests
- [x] T013 Serialize metadata-only UDP events and aggregate protocol accounting
- [x] T014 Reconcile exact observed endpoints and leave unavailable facts uninvented
- [x] T015 Prove every terminal path releases sockets and mappings

## Phase 6: Documentation And Verification

- [x] T016 Update glossary, architecture, outline, plan status, proxy README, AGENTS, and changelog
- [x] T017 Run focused tests and full `cargo xtask ci`
- [x] T018 Run dependency, lockfile, UTF-8, mojibake, and diff sanity checks
- [x] T019 Complete all task boxes and reconcile issue #311 acceptance

## Execution Order

Wire/state foundations precede the valid relay. The valid relay precedes adversarial hardening. Stable transport outcomes precede evidence finalization. Documentation and full verification finish the slice.
