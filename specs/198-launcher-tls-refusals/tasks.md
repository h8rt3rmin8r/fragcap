# Tasks: Launcher TLS Refusal Diagnosis

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [refusal evidence contract](contracts/refusal-evidence.md).

**Tests**: Required by issue #574, the specification, and the repository autopilot protocol. Observe each focused regression fail before implementing its corresponding behavior.

**Organization**: Setup and classifier foundation precede the controlled proxy and correlation journeys; artifact and documentation reconciliation follow the behavioral tests.

## Phase 1: Setup and specification

- [x] T001 Confirm clean `codex/s198-launcher-tls-refusals`, open issue #574, the S198 feature pointer, and no extension hooks.
- [x] T002 Complete [spec.md](spec.md), resolve clarifications, and check the requirements-quality [checklist](checklists/requirements.md).
- [x] T003 Complete [research.md](research.md), [plan.md](plan.md), [data-model.md](data-model.md), [refusal evidence contract](contracts/refusal-evidence.md), and [quickstart.md](quickstart.md).
- [x] T004 Run Spec-Kit analysis across requirements, plan, tasks, constitution, and master specification; resolve every critical finding before implementation.

## Phase 2: Stable refusal foundation

**Goal**: Classify only observable TLS error structure and preserve original transport evidence.

- [x] T005 [US1] Write failing EOF/reset, structured invalid-message, wrapped-source, and generic invalid-data regressions in `crates/fragcap-proxy/src/upstream.rs`; retain the existing trust, pinning-signal, client-auth, and protocol-mismatch cases.
- [x] T006 [US1] Add transport-ended and invalid-message stable classes and finite rustls source-chain lookup in `crates/fragcap-proxy/src/upstream.rs`, retaining existing alert and protocol precedence.
- [x] T007 [US1] Assert client-facing and upstream boundary handling in `crates/fragcap-proxy/tests/https_proxy.rs` without adding handshake reads or fallback.

## Phase 3: Controlled launch-chain journey

**Goal**: Demonstrate a compatible inspected launcher/updater pair and exact failure truth.

- [x] T008 [US1] Add failing real proxy cases for authenticated CONNECT followed by EOF and malformed TLS in `crates/fragcap-proxy/tests/https_proxy.rs`, checking accepted ID, tunnel response, client-TLS stage, terminal, alert, category, and absent inspected HTTP.
- [x] T009 [US2] Add a controlled pair of independent launcher/updater-like CONNECT, TLS, and HTTP exchanges in `crates/fragcap-proxy/tests/https_proxy.rs`, checking both TLS stages, distinct connection IDs, and complete application evidence.
- [x] T010 [US2] Strengthen the exact owner join regression in `crates/fragcap/src/deep_capture/native.rs` with the pair's accepted connection IDs, distinct non-client roles, and unavailable ownership; retain the existing process-generation and ambiguous-owner regressions.
- [x] T011 [US3] Preserve after-completed-exchange HTTP timeout integration coverage and add a before-first-request diagnostic regression in `crates/fragcap-proxy/src/runtime.rs`.
- [x] T012 [US1] Verify an `Inspect` failure stays failed with no blind relay or full HTTP observation in the controlled proxy cases.

## Phase 4: Artifacts and documentation

- [x] T013 Reconcile `ConnectionFailureCategory`, bounded diagnostic loss, `proxy.jsonl`, `application.jsonl`, facade summary, and calibration JSON with new stable codes; add exact category assertions where a contract change occurred and retain unchanged schema projections.
- [x] T014 Update `docs/fragcap-specification.md`, `docs/plans/README.md`, and the operator glossary with the source-level classifications, field-evidence limit, and #572 boundary.
- [x] T015 Add a user-visible `changelog.d/` fix fragment and a dated decisions fragment for the classification authority.
- [x] T016 Re-run the requirements checklist and Spec-Kit analysis; ensure the final plan and implementation agree.

## Phase 5: Verification and publication

- [x] T017 Run focused Rust tests, `cargo xtask ci`, applicable Windows-controlled checks, formatting, encoding/mojibake review, and `git diff --check` on the exact local head.
- [x] T018 Commit the verified S198 slice, push the explicitly authorized branch, and open an official PR with `Closes #574` and concrete evidence.
- [ ] T019 Wait for exact-head hosted CI and all arriving external reviews, fix and answer every actionable comment, request at most one second `@codex` round, and leave the green PR for owner final review and merge.

## Dependencies and Execution Order

The classifier regression and its implementation precede real proxy expectations. The real proxy produces accepted connection IDs and stage evidence before facade correlation or calibration assertions. Documentation and changelog follow the measured behavior. Exact-head hosted checks and review closeout follow publication.

## Implementation Strategy

Reuse the existing bounded diagnostic schema and exact process-correlation authority. Add only stable codes supported by structured rustls or transport evidence, then demonstrate the compatible and refused controlled paths end to end. Do not infer the private installed-title cause or route direct sockets in this slice.
