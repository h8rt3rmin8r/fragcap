# Tasks: Evidence and Diagnostic Debt Reconciliation

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [http-head-failure-count.md](contracts/http-head-failure-count.md), and [disposition-ledger.md](contracts/disposition-ledger.md).

**Tests**: Required controlled read cases and native/debug-report readback are specified in FR-007. Focused head-reader tests preceded the source change and were observed red. Downstream assertions follow the typed-field additions because they require that field to compile; they are not claimed to have been red first.

## Phase 1: Setup

**Goal**: Establish the exact public work and extraction inventory without disturbing the original worktree.

- [ ] T001 Verify `specs/212-evidence-diagnostic-reconciliation/spec.md`, quality checklists, and `.specify/feature.json` point to this branch and that analyze precedes Rust edits.
- [ ] T002 Record the original 20-file modified-path inventory and planned hunk decisions in `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md`.

## Phase 2: Foundational evidence boundary

**Goal**: Confirm the contracts against existing source and the public tracker before changing either.

- [ ] T003 Review existing head-read, connection, facade, native, and CLI fields against `specs/212-evidence-diagnostic-reconciliation/contracts/http-head-failure-count.md` and the six listed Rust source paths.
- [ ] T004 [P] Audit the active epic and related issues for completed evidence, independent defects, metadata, dependencies, and unobservable completion gates; record the sanitized issue set in `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md`.
- [x] T005 Run the Spec Kit analyze gate over `specs/212-evidence-diagnostic-reconciliation/` and resolve blocking cross-artifact gaps before implementation.

## Phase 3: User Story 1 - Trust the public work queue (P1)

**Goal**: Make the epic and child issues state what is complete and what still needs product work.

**Independent Test**: Read back every affected issue and compare its body, status, labels, milestone, and dependencies to the sanitized disposition ledger.

- [ ] T006 [US1] Reconcile completed comparisons, speculative acceptance, and surviving product defects in the active epic and affected GitHub issues; write the exact issue outcomes to `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md`.
- [ ] T007 [US1] Keep the independent generic relay defect open with a controlled completion test and narrow the HTTP-head diagnostic issue to this slice's count contract; verify both against `specs/212-evidence-diagnostic-reconciliation/spec.md`.
- [ ] T008 [US1] Read back each GitHub edit and verify all affected issue metadata and completion criteria against `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md` without publishing private target information.

## Phase 4: User Story 2 - See truthful partial-head evidence (P1)

**Goal**: Preserve the count observed by a bounded HTTP/1.1 head reader and project it consistently without retaining content.

**Independent Test**: Run the controlled cases in [the quickstart](quickstart.md) and compare native `proxy.jsonl` with the debug report for the same connection ID.

- [ ] T009 [US2] Add failing clean zero-byte EOF, zero-byte read-error, partial EOF, partial read-error, size-limit, and complete-head tests beside `read_head` in `crates/fragcap-proxy/src/http1.rs`.
- [ ] T010 [US2] Add optional observed-count ownership to `ProtocolError` and set it only at read-error, partial EOF, and size-refusal branches in `crates/fragcap-proxy/src/http1.rs`.
- [ ] T011 [US2] Propagate that optional count through the existing trace and typed connection record in `crates/fragcap-proxy/src/runtime.rs` and `crates/fragcap-proxy/src/model.rs` without changing terminal codes or timeout behavior.
- [ ] T012 [US2] Copy the optional count to the facade record in `crates/fragcap/src/deep_capture/diagnostics.rs` and conditionally emit it without `null` in `crates/fragcap/src/deep_capture/native.rs`.
- [ ] T013 [US2] Conditionally emit the same count in terminal diagnostics JSON at `crates/fragcap-cli/src/commands/assessment.rs`; add only valid counted head-read failures as sanitized `proxy.head_read_failures[]` rows in `crates/fragcap/src/deep_capture/route_report.rs`, with no empty array, raw detail, endpoint, head, or payload.
- [ ] T014 [US2] Add downstream count-propagation, no-content, native/report agreement, and older-record compatibility assertions in `crates/fragcap-proxy/src/runtime.rs`, `crates/fragcap/src/deep_capture/native.rs`, `crates/fragcap-cli/src/commands/assessment.rs`, and `crates/fragcap-cli/tests/cli_bundle.rs`; the bundle fixture must derive its report from the same native `proxy.jsonl` record.
- [ ] T015 [US2] Run focused proxy, facade, terminal diagnostics, and bundle debug-report tests for `crates/fragcap-proxy/src/http1.rs`, `crates/fragcap/src/deep_capture/native.rs`, `crates/fragcap/src/deep_capture/route_report.rs`, `crates/fragcap-cli/src/commands/assessment.rs`, and `crates/fragcap-cli/tests/cli_bundle.rs`; inspect terminal categories, exact counts, field and row absence, and content exclusion.

## Phase 5: User Story 3 - Finish the disposable code inventory (P2)

**Goal**: Make every original hunk's public or private outcome reviewable while preserving unfinished work.

**Independent Test**: Compare all 20 original paths and substantive hunks with the ledger and final clean-branch diff; verify excluded experiments and dependencies are absent.

- [ ] T016 [US3] Review every substantive original hunk against `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md`, recording extract, private retain, or retire decisions and any changed decision.
- [ ] T017 [US3] Compare the final public diff with `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md` and verify only six original extract paths plus newly authored `crates/fragcap/src/deep_capture/route_report.rs` carry the diagnostic, with no experiment-only `Cargo.toml` or `Cargo.lock` changes.
- [ ] T018 [US3] Preserve the original dirty worktree until the extraction is tested and reviewed; record its recoverable handoff and any unresolved private work in `specs/212-evidence-diagnostic-reconciliation/contracts/disposition-ledger.md` without private research details.

## Phase 6: User Story 4 - Keep the security audit operational (P2)

**Goal**: Repair a pre-existing CI audit failure with a narrow, expiring exception while preserving a separate replacement obligation.

**Independent Test**: Locked all-features and six isolated audits pass; direct affected-parent and expired-waiver canaries fail. Read back [#640](https://github.com/h8rt3rmin8r/fragcap/issues/640) as open.

- [x] T019 [US4] Pin `cargo-deny 0.21.0` in `.github/workflows/audit.yml` and `supply-chain/policy-v1.json`, add only the nine exact `heapless 0.8.0` advisory exceptions for `smoltcp@0.12.0` in `deny.toml` and governed policy through 2026-12-15 UTC, and retain `unsound = "all"`. Renew the due `rustls 0.23.46` maintenance review against unchanged checksum, features, three governed graphs, and Rust floor.
- [x] T020 [US4] Update only `spikes/http-mitm-proxy/Cargo.lock` and `spikes/native-proxy/Cargo.lock` to resolve `url 2.5.9` and `form_urlencoded 1.2.3`; verify the product workspace lockfile is unchanged by this repair.
- [x] T021 [US4] In `xtask/src/supply_chain.rs`, validate exact policy-to-denial mirror and governed workflow pin; document the finite exception in `docs/maintainers/supply-chain.md`. Run locked full all-features and six isolated graph audits, `cargo xtask supply-chain`, focused tests, and direct-parent, expiry, broad/missing/extra, and spoofed-mirror negative canaries; record results and bounded source-review basis in [#640](https://github.com/h8rt3rmin8r/fragcap/issues/640) without marking the dependency fixed.

## Phase 7: Polish and cross-cutting gates

- [x] T022 [P] Check glossary coverage, update shipped-state wording and the additive diagnostic contract in `docs/fragcap-specification.md`, then add S212 chronologically to `docs/plans/README.md` without promoting unverified S211 physical tests to passed.
- [x] T023 [P] Add a narrow user-visible fragment at `changelog.d/S212-http-head-failure-count.added.md` with the matching `spec-impact` field; keep unrelated public guidance unchanged.
- [x] T024 [P] Add a dated decision at `changelog.d/S212-audit-exception.decisions.md` for the audit workflow pin, source-review limit, exact exception, negative controls, and separate #640 replacement; align sections 24.3 and 27.3 of `docs/fragcap-specification.md` and the S212 planning entry with this independent CI repair.
- [x] T025 Run `specs/212-evidence-diagnostic-reconciliation/quickstart.md`, all applicable repository gates, privacy/diff inspection, and Spec Kit re-analysis; correct any failures before PR publication.
- [ ] T026 Commit, push, and publish the S212 PR, resolve every applicable review within the two-round Codex cap, wait for green final-head CI, and hand the merge decision to the owner; retain the final evidence in `specs/212-evidence-diagnostic-reconciliation/`.

## Dependencies and execution order

T001-T005 block Rust implementation. After T005, US1 and US2 can proceed in parallel. The focused HTTP-head reader test T009 was observed red before T010-T013 implemented the field; downstream T014 assertions follow the field because they require its type to compile. T015 proves the story. US3 inventory T016-T018 follows extraction but can audit unchanged private hunks while US2 progresses. The independent CI repair T019-T021 can proceed in parallel with the diagnostic extraction but must finish before final gate T025 and PR task T026. Documentation and decisions T022-T024 must also finish before those final gates.

**Parallel opportunities**: T004 can run alongside T003. US1 tracker editing, US2 source work, and US4 CI repair can run independently after analyze. T022-T024 target different documentation and decision files.

**MVP**: US1 produces a truthful public queue independently. The full S212 delivery additionally requires US2's exact diagnostic, US3's complete source disposition, and US4's exception-qualified CI repair; none alone is claimed as full slice completion.
