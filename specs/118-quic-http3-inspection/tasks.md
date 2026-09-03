# Tasks: Scoped QUIC And HTTP/3 Inspection

**Input**: Design documents from `specs/118-quic-http3-inspection/`

**Tests**: Required and written before implementation.

## Phase 1: Specification And Dependencies

- [x] T001 Record spec, clarifications, requirements/security checklists, research, data model, evidence contract, plan, quickstart, and tasks in `specs/118-quic-http3-inspection/`
- [x] T002 Promote Quinn and add exact h3 dependencies with MSRV, license, and lockfile review in `Cargo.toml`, `Cargo.lock`, and `crates/fragcap-proxy/Cargo.toml`

## Phase 2: QUIC Foundation

- [x] T003 Add failing admission, identity, 0-RTT, migration, capacity, and accounting tests in `crates/fragcap-proxy/tests/quic_http3.rs`
- [x] T004 Add typed QUIC pair, connection, stream, datagram, refusal, and accounting values in `crates/fragcap-proxy/src/application.rs` and `crates/fragcap-proxy/src/model.rs`
- [x] T005 Implement finite client-facing and upstream QUIC configuration with immutable origin policy in `crates/fragcap-proxy/src/quic.rs`
- [x] T006 Export QUIC configuration and lifecycle seams in `crates/fragcap-proxy/src/lib.rs` and `crates/fragcap-proxy/src/runtime.rs`, reusing the existing upstream policy authority

## Phase 3: User Story 1 - Scoped QUIC Pair

- [x] T007 [US1] Add failing real loopback QUIC identity, authenticated routing, endpoint refusal, and cleanup tests in `crates/fragcap-proxy/tests/quic_http3.rs`
- [x] T008 [US1] Attach exact QUIC admission to authenticated UDP association destinations in `crates/fragcap-proxy/src/socks5.rs` and `crates/fragcap-proxy/src/quic.rs`
- [x] T009 [US1] Implement bounded HTTP/3 stream and QUIC datagram forwarding in `crates/fragcap-proxy/src/quic.rs`
- [x] T010 [US1] Reconcile pair identities, TLS halves, transport-owned key lifecycle, terminal states, and retained or omitted transport evidence in `crates/fragcap-proxy/src/quic.rs` and `crates/fragcap-proxy/src/model.rs`

## Phase 4: User Story 2 - HTTP/3

- [x] T011 [US2] Add two-lineage and authenticated-route HTTP/3 request, response, body, and limit tests in `crates/fragcap-proxy/tests/quic_http3.rs`
- [x] T012 [US2] Implement ALPN-selected HTTP/3 client/server pairing and request forwarding in `crates/fragcap-proxy/src/quic.rs`
- [x] T013 [US2] Map HTTP/3 fields, bodies, timings, and terminals into existing application events in `crates/fragcap-proxy/src/application.rs` and `crates/fragcap-proxy/src/quic.rs`
- [x] T014 [US2] Preserve per-stream task independence and bounded body evidence under pressure in `crates/fragcap-proxy/src/quic.rs`

## Phase 5: User Story 3 - Refusal And Artifacts

- [x] T015 [US3] Add 0-RTT, migration, endpoint-change, finite-capacity, trust-refusal, authenticated-route, and queue-loss tests in `crates/fragcap-proxy/tests/quic_http3.rs` and `crates/fragcap/tests/application_stream.rs`
- [x] T016 [US3] Implement stable QUIC refusal classes with no transparent fallback in `crates/fragcap-proxy/src/quic.rs` and `crates/fragcap-proxy/src/socks5.rs`
- [x] T017 [US3] Serialize and reconcile QUIC and HTTP/3 events in `crates/fragcap/src/deep_capture/application.rs` and `crates/fragcap/src/deep_capture/lifecycle.rs`
- [x] T018 [US3] Extend HAR and bounded application-loss projection in `crates/fragcap/src/deep_capture/har.rs` and confirm that the protocol-neutral manifest, correlation, and cleanup authorities require no parallel QUIC path

## Phase 6: Documentation And Verification

- [x] T019 Update glossary, architecture, outline, plan status, proxy README, AGENTS, and changelog in `docs/`, `crates/fragcap-proxy/README.md`, `AGENTS.md`, and `changelog.d/`
- [x] T020 Run focused QUIC, HTTP/3, application stream, and Deep Capture session tests
- [x] T021 Run full `cargo xtask ci`
- [x] T022 Run dependency, license, MSRV, lockfile, UTF-8, mojibake, Unicode punctuation, and diff sanity checks
- [x] T023 Complete all task boxes and reconcile issue #314 acceptance in `specs/118-quic-http3-inspection/tasks.md`

## Dependencies

QUIC types and configuration block every story. User Story 1 establishes the paired transport. User Story 2 builds semantic HTTP/3 on that pair. User Story 3 completes refusal and artifact authority. Documentation and full verification finish the slice.

## Parallel Opportunities

Documentation and glossary work can proceed after the public model stabilizes. Facade serialization tests can proceed alongside proxy refusal tests once event shapes are fixed. Work touching `quic.rs` remains sequential.

## Implementation Strategy

The minimum deliverable is one authenticated, immutable, bounded HTTP/3 QUIC pair with independent TLS halves and no downgrade. HTTP/3 semantics and artifact integration then complete issue #314 without widening route authority.
