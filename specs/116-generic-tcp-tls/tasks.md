# Tasks: Generic TCP And Non-HTTP TLS Evidence

**Input**: Design documents from `specs/116-generic-tcp-tls/`

**Tests**: Required and written before implementation.

## Phase 1: Specification And Contracts

- [x] T001 Record spec, clarifications, requirements/security checklists, research, data model, evidence contract, plan, quickstart, and tasks
- [x] T002 Confirm no dependency or lockfile change is required

## Phase 2: Generic Stream Foundation

- [x] T003 Add failing chunk model, retention, offset, omission, and truncation tests
- [x] T004 Add generic stream types and protocol accounting
- [x] T005 Implement fixed-buffer full-duplex observation independent from retention

## Phase 3: User Story 1 - Plain TCP

- [x] T006 Add failing authenticated SOCKS plain TCP, bounds, half-close, and cancellation tests
- [x] T007 Emit directional plaintext chunks from accepted opaque TCP tunnels
- [x] T008 Reconcile chunk totals with SOCKS transfer and terminal accounting

## Phase 4: User Story 2 - TLS Outcomes

- [x] T009 Add failing opaque SOCKS TLS, intercepted no-ALPN TLS, and HTTP preservation tests
- [x] T010 Preserve encrypted provenance for byte-transparent TLS
- [x] T011 Add bounded no-ALPN discrimination and intercepted protocol-unknown TLS relay
- [x] T012 Preserve independent upstream verification and negotiation evidence

## Phase 5: User Story 3 - Refusal And Artifacts

- [x] T013 Add failing trust, client-auth, retention exhaustion, queue pressure, correlation, and cleanup tests
- [x] T014 Preserve structured refusal with no downgrade fallback
- [x] T015 Serialize generic chunks in application and lifecycle JSON Lines
- [x] T016 Prove finite retention, exact omission, event loss, and terminal cleanup

## Phase 6: Documentation And Verification

- [x] T017 Update glossary, architecture, outline, plan status, proxy README, AGENTS, and changelog
- [x] T018 Run focused tests and full `cargo xtask ci`
- [x] T019 Run dependency, lockfile, UTF-8, mojibake, and diff sanity checks
- [x] T020 Complete all task boxes and reconcile issue #312 acceptance

## Execution Order

The chunk model and bounded observer precede transport integration. Plain TCP precedes TLS outcome routing. Stable outcomes precede artifact serialization and documentation. Full verification finishes the slice.
