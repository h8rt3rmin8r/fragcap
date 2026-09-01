# Tasks: Correlated Native Evidence

**Input**: Design documents from `specs/108-correlated-native-evidence/`

## Phase 1: Setup

- [x] T001 Confirm no-new-package baseline and affected workspace graph in `Cargo.toml` and `Cargo.lock`
- [x] T002 [P] Add failing manifest schema drift fixtures under `crates/fragcap/assets/` and `docs/schema/`

## Phase 2: Foundational

- [x] T003 Add failing timestamped flow-history and permutation tests in `crates/fragcap-core/src/flow.rs`
- [x] T004 Add failing accepted-connection and native timing event tests in `crates/fragcap-proxy/src/application.rs`
- [x] T005 Implement timestamped immutable flow summaries in `crates/fragcap-core/src/flow.rs` and pipeline publication in `crates/fragcap-core/src/pipeline/mod.rs`
- [x] T006 Implement accepted connection descriptors and native timing evidence across `crates/fragcap-proxy/src/runtime.rs`, `http1.rs`, and `http2.rs`

## Phase 3: User Story 1 - Exact correlation (P1)

**Independent Test**: Permuted IPv4/IPv6, endpoint reuse, retained ownership, late publication, and multiplexed streams reconcile identically.

- [x] T007 [US1] Add failing final-reconciliation and conservation tests in `crates/fragcap/tests/application_stream.rs` and `native_proxy.rs`
- [x] T008 [US1] Implement connection-aware deferred records and ordered final correlation in `crates/fragcap/src/deep_capture/application.rs`
- [x] T009 [US1] Replace latest-owner lookup and remove fabricated controlled flow ids in `crates/fragcap/src/deep_capture/native.rs`
- [x] T010 [US1] Propagate typed correlation state, reasons, and accounting through `crates/fragcap/src/deep_capture/model.rs` and `policy.rs`
- [x] T011 [US1] Verify stream/message identity and forward-only UDP/QUIC representation in facade and proxy tests

## Phase 4: User Story 2 - Truthful HAR 1.2 (P2)

**Independent Test**: Complete transactions parse as HAR 1.2, partials remain in the namespaced extension, and no field lacks source evidence.

- [x] T012 [US2] Add failing field-provenance, partial, binary, bound, and atomic-failure tests in `crates/fragcap/tests/har.rs`
- [x] T013 [US2] Add required URL, head-size, response-cookie, body-terminal, timing, and transformation-direction evidence in `crates/fragcap-proxy/src/metadata.rs`, `body.rs`, `http1.rs`, and `http2.rs`
- [x] T014 [US2] Implement streaming bounded transaction assembly and HAR projection in `crates/fragcap/src/deep_capture/har.rs`
- [x] T015 [US2] Integrate authoritative application input and atomic HAR publication in `crates/fragcap/src/deep_capture/mod.rs` and CLI `commands/deep_capture.rs`
- [x] T016 [US2] Remove placeholder CLI projection in `crates/fragcap-cli/src/har.rs` and update CLI integration tests

## Phase 5: User Story 3 - Versioned manifest authority (P3)

**Independent Test**: V2 examples round-trip and validate, contradictions fail, v1 bytes remain unchanged, and crash prefixes never claim completion.

- [x] T017 [US3] Add failing v1 compatibility, v2 round-trip, contradiction, path, and fault tests in `crates/fragcap/tests/manifest.rs`
- [x] T018 [US3] Publish byte-identical machine-readable v2 schemas in `crates/fragcap/assets/` and `docs/schema/`
- [x] T019 [US3] Implement typed v1 reader, v2 model, validator, safe paths, and serializer in `crates/fragcap/src/deep_capture/manifest.rs`
- [x] T020 [US3] Implement protected crash-prefix and atomic final publication in facade session artifact lifecycle
- [x] T021 [US3] Replace CLI-local v1 serialization with facade v2 assembly in `crates/fragcap-cli/src/commands/deep_capture.rs`
- [x] T022 [US3] Align cleanup, share export, and doctor readers with the common compatibility view in `crates/fragcap/src/deep_capture/artifacts.rs` and CLI doctor modules

## Phase 6: Polish and verification

- [x] T023 Update master specification, outline, plans, schema docs, glossary, and AGENTS dependency/current-state prose
- [x] T024 Add feature and dated decision fragments under `changelog.d/`
- [x] T025 Mark completed tasks and run quickstart focused verification
- [x] T026 Run formatting, clippy, all locked tests, xtask CI, MSRV, dependency, encoding, mojibake, and diff checks

## Dependencies and Execution Order

T001-T006 establish source evidence. US1 completes before US2 because HAR consumes final correlation. US2 completes before US3 because manifest authority consumes HAR and correlation outcomes. T023-T026 follow all stories.

## Parallel Opportunities

Schema drift fixtures can be prepared while source-evidence tests are authored. Proxy event evidence and core flow history affect separate crates. Documentation can be drafted after contracts stabilize while independent focused tests run.

## Implementation Strategy

Complete the three stories in dependency order under test-first development. Each checkpoint must pass focused tests before the next consumer is implemented. The deliverable is the whole slice, not US1 alone.
