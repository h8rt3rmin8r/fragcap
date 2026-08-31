# Tasks: HTTP/2, Metadata, and Streaming Bodies

**Input**: Design documents from `/specs/105-http2-metadata-bodies/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Protocol, metadata, body, application-stream, security, lifecycle, accounting, and regression tests are mandatory and precede implementation.

## Phase 1: Setup

- [x] T001 Add exact async-compression and direct runtime h2 and bytes dependency declarations in Cargo.toml and crates/fragcap-proxy/Cargo.toml
- [x] T002 [P] Add protocol, metadata, body, HTTP/2, and application module declarations in crates/fragcap-proxy/src/lib.rs
- [x] T003 Verify Rust ignore patterns and dependency-direction policy for S105 paths in .gitignore and xtask/src/deps.rs

## Phase 2: Shared Foundations

- [x] T004 Define separate forwarding, stream, observation, storage, decoder, idle, and shutdown limits in crates/fragcap-proxy/src/model.rs
- [x] T005 Define connection and stream identities, protocol versions, terminal outcomes, and reconciliation counters in crates/fragcap-proxy/src/model.rs
- [x] T006 Define binary-safe metadata blocks, fields, ordering provenance, body segments, transformations, and typed application events in crates/fragcap-proxy/src/metadata.rs, crates/fragcap-proxy/src/body.rs, and crates/fragcap-proxy/src/application.rs
- [x] T007 Add bounded nonblocking event sink dispositions and accounting tests in crates/fragcap-proxy/tests/application_events.rs
- [x] T008 Implement bounded nonblocking event sink production and retirement accounting in crates/fragcap-proxy/src/application.rs and crates/fragcap-proxy/src/runtime.rs

## Phase 3: User Story 1 - Inspect Multiplexed HTTP/2 Correctly (Priority: P1)

**Goal**: Proxy authorized HTTP/2 with deterministic stream pairing, finite flow control, and exact stream and connection outcomes.

**Independent Test**: One controlled connection overlaps 32 streams, resets one, completes out of order, sends trailers and GOAWAY, and leaves every accepted stream and connection with exactly one terminal outcome.

- [x] T009 [P] [US1] Add ALPN agreement, mismatch refusal, h2c preface, and single-authority tests in crates/fragcap-proxy/tests/http2_proxy.rs
- [x] T010 [US1] Add 32-stream overlap, out-of-order response, reset isolation, trailer, GOAWAY, and push-refusal tests in crates/fragcap-proxy/tests/http2_proxy.rs
- [x] T011 [US1] Add flow-control pressure, header-list, stream-count, reset-retention, idle, cancellation, and shutdown bound tests in crates/fragcap-proxy/tests/http2_proxy.rs
- [x] T012 [US1] Advertise both supported ALPN values and coordinate one exact selected protocol across client and verified origin TLS in crates/fragcap-proxy/src/tls.rs, crates/fragcap-proxy/src/upstream.rs, and crates/fragcap-proxy/src/runtime.rs
- [x] T013 [US1] Dispatch TLS ALPN and cleartext prior-knowledge traffic without regressing HTTP/1.1 in crates/fragcap-proxy/src/protocol.rs and crates/fragcap-proxy/src/runtime.rs
- [x] T014 [US1] Implement bounded direct-h2 client and server handshakes, stream mapping, request and response relay, and flow-control release in crates/fragcap-proxy/src/http2.rs
- [x] T015 [US1] Implement HTTP/2 reset, refusal, GOAWAY, push rejection, transport error, and forced-shutdown terminal evidence in crates/fragcap-proxy/src/http2.rs and crates/fragcap-proxy/src/runtime.rs

## Phase 4: User Story 2 - Preserve Complete HTTP Metadata (Priority: P1)

**Goal**: Retain every metadata value available at each protocol boundary without normalization loss or fabricated fidelity.

**Independent Test**: HTTP/1.1 and HTTP/2 exchanges round-trip duplicates, binary values, repeated queries, cookies, informational blocks, and trailers with exact protocol-specific provenance.

- [x] T016 [P] [US2] Add raw HTTP/1.1 order, casing, duplicate, empty-value, informational, and trailer metadata tests in crates/fragcap-proxy/tests/http1_proxy.rs
- [x] T017 [US2] Add HTTP/2 pseudo-field, binary-value, duplicate-order, trailer, and unavailable-HPACK-order tests in crates/fragcap-proxy/tests/http2_proxy.rs
- [x] T018 [US2] Add repeated query, cookie, decode-uncertainty, and sensitive-diagnostic leak tests in crates/fragcap-proxy/tests/metadata.rs
- [x] T019 [US2] Retain protocol-faithful request, informational, response, and trailer metadata blocks in crates/fragcap-proxy/src/http1.rs and crates/fragcap-proxy/src/http2.rs
- [x] T020 [US2] Implement traceable query and cookie conveniences plus unavailable-representation provenance in crates/fragcap-proxy/src/metadata.rs
- [x] T021 [US2] Extend observation accounting and terminal snapshots for metadata outcomes in crates/fragcap-proxy/src/model.rs and crates/fragcap-proxy/src/runtime.rs

## Phase 5: User Story 3 - Retain Bounded Streaming Bodies (Priority: P1)

**Goal**: Observe authorized bodies incrementally while forwarding remains bounded and byte-correct independently of retention capacity.

**Independent Test**: Fixed, chunked, compressed, oversized, indefinite, malformed, cancelled, and metadata-only transfers preserve forwarded bytes and reconcile all observed, retained, omitted, truncated, decode-failed, storage-failed, and dropped bytes.

- [x] T022 [P] [US3] Add fixed, chunked, close-delimited, indefinite, early-response, partial, and cancelled streaming tests in crates/fragcap-proxy/tests/body_streams.rs
- [x] T023 [US3] Add gzip, zlib-deflate, Brotli, unsupported, malformed, truncated, output-limit, ratio-limit, time-limit, and decoder-concurrency tests in crates/fragcap-proxy/tests/body_streams.rs
- [x] T024 [US3] Add metadata-only, message-limit, session-limit, queue-pressure, and byte-reconciliation tests in crates/fragcap-proxy/tests/body_streams.rs
- [x] T025 [US3] Split forwarding memory bounds from observation retention and remove total valid-body refusal in crates/fragcap-proxy/src/model.rs and crates/fragcap-proxy/src/http1.rs
- [x] T026 [US3] Tee bounded ordered raw body segments from HTTP/1.1 and HTTP/2 relay paths without whole-message buffering in crates/fragcap-proxy/src/http1.rs and crates/fragcap-proxy/src/http2.rs
- [x] T027 [US3] Implement bounded transfer and gzip, zlib-deflate, and Brotli content transformations with raw authority in crates/fragcap-proxy/src/body.rs
- [x] T028 [US3] Reconcile body completion, omission, truncation, decoder, queue, and storage outcomes in crates/fragcap-proxy/src/body.rs, crates/fragcap-proxy/src/model.rs, and crates/fragcap-proxy/src/runtime.rs

## Phase 6: User Story 4 - Consume a Crash-Readable Application Stream (Priority: P2)

**Goal**: Write a versioned application stream during capture that remains honestly readable after orderly completion or interruption.

**Independent Test**: A consumer reads records during traffic, validates a complete reconciling trailer after orderly stop, and reads an explicitly incomplete prefix after writer or process interruption.

- [x] T029 [P] [US4] Add schema version 2 golden, round-trip, legacy version 1, unknown-version, malformed-line, and torn-prefix tests in crates/fragcap/tests/application_stream.rs
- [x] T030 [US4] Add live-read, queue-saturation, injected I/O failure, single-trailer, repeated-stop, and no-final-overwrite tests in crates/fragcap/tests/application_stream.rs and crates/fragcap-cli/tests/cli_deep_capture.rs
- [x] T031 [US4] Define version 2 header, evidence, gap, trailer, correlation, binary encoding, reserved-family, and reader models in crates/fragcap/src/deep_capture/application.rs
- [x] T032 [US4] Implement the bounded dedicated application writer, sequence assignment, per-record flush, gap accounting, atomic retirement, and one-trailer finalization in crates/fragcap/src/deep_capture/application.rs
- [x] T033 [US4] Implement version 1 and version 2 prefix readers with explicit unknown-version and incomplete outcomes in crates/fragcap/src/deep_capture/application.rs
- [x] T034 [US4] Open the approved application artifact before proxy start and pass its event sink through facade orchestration in crates/fragcap/src/deep_capture/adapters.rs, crates/fragcap/src/deep_capture/native.rs, and crates/fragcap/src/deep_capture/session.rs
- [x] T035 [US4] Remove CLI final projection overwrite and finalize the live lease through existing bundle ownership in crates/fragcap-cli/src/commands/deep_capture.rs

## Phase 7: Integration and Cross-Cutting Gates

- [x] T036 Add end-to-end native HTTP/2, metadata, body, live-artifact, and ten-cycle cleanup coverage in crates/fragcap-proxy/tests/protocol_lab.rs and crates/fragcap/tests/native_proxy.rs
- [x] T037 [P] Update architecture, protocol boundaries, artifact schema, and incomplete-status truth in AGENTS.md, docs/fragcap-specification.md, and docs/fragcap-spec-outline.md
- [x] T038 [P] Update public Deep Capture support language and proxy crate documentation in README.md, crates/fragcap-proxy/README.md, and site/content/docs/
- [x] T039 Add user-visible and dated architecture decision fragments in changelog.d/S105-http2-metadata-bodies.added.md and changelog.d/S105-http2-metadata-bodies.decisions.md
- [x] T040 Run focused verification from specs/105-http2-metadata-bodies/quickstart.md and resolve every failure
- [x] T041 Run the spec-kit convergence audit and append any discovered requirement-driven work to specs/105-http2-metadata-bodies/tasks.md
- [x] T042 Run cargo xtask ci and resolve every format, lint, test, dependency, specification, MSRV, package, documentation, encoding, mojibake, and platform failure
- [x] T043 Mark every completed task and set S105 status complete in specs/105-http2-metadata-bodies/tasks.md and specs/105-http2-metadata-bodies/spec.md
- [x] T044 Commit the verified slice locally with the repository conventional message and co-author trailer

## Phase 8: Convergence

- [x] T045 Complete ALPN mismatch, single-authority, and explicitly scoped cleartext HTTP/2 behavior and tests per FR-001 and FR-020 (partial)
- [x] T046 Emit and verify distinct reset, refusal, GOAWAY, push-refusal, transport-error, and forced-shutdown outcomes per FR-004, FR-006, and FR-017 (partial)
- [x] T047 Enforce and test HTTP/2 flow-control, header-list, stream-count, reset-retention, idle, cancellation, and shutdown bounds per FR-003, FR-005, and FR-019 (partial)
- [x] T048 Complete HTTP/1.1 and HTTP/2 raw metadata, duplicate, binary, informational, trailer, query, cookie, uncertainty, and sensitive-diagnostic coverage per FR-007 through FR-010 (partial)
- [x] T049 Complete fixed, chunked, close-delimited, indefinite, partial, cancelled, compressed, malformed, bounded, metadata-only, and queue-pressure body tests and reconciliation per FR-011 through FR-018 (partial)
- [x] T050 Complete application writer saturation, injected failure, interruption, single-trailer, repeated-stop, no-overwrite, and exact trailer reconciliation coverage per FR-022 through FR-027 (partial)
- [x] T051 Add the combined offline protocol lab and ten-cycle no-residue lifecycle proof per FR-020 and SC-002 (missing)
- [x] T052 Update the public documentation site with the exact S105 shipped and deferred boundary per plan: documentation (partial)

## Dependencies and Execution Order

```text
Setup -> Shared Foundations -> US1 HTTP/2 -> US2 Metadata -> US3 Bodies
      -> US4 Application Stream -> Integration -> Convergence -> Full CI -> Commit
```

The shared typed event and accounting foundation blocks every story. HTTP/2 establishes the multiplexed stream lifecycle consumed by metadata and body evidence. The application stream depends on all typed event families, but its schema and reader tests can be authored once those models stabilize. Tests precede each corresponding implementation.

## Parallel Opportunities

- T002, T007, T009, T016, T022, T029, T037, and T038 affect distinct files at their phase boundary.
- HTTP/1.1 metadata tests and HTTP/2 lifecycle tests can be authored independently after shared models exist.
- Application schema tests can proceed while body transformation implementation is completed because the typed event contract is foundational.

## Implementation Strategy

Deliver all four issues before the autopilot pre-push halt. User Story 1 is the minimum independently demonstrable increment, but it is not the slice boundary because metadata and body evidence require the durable live artifact supplied by User Story 4. Production claims change only after reconciliation, lifecycle, regression, documentation, and full repository gates pass.
