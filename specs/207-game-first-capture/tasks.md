# Tasks: Game-first Deep Capture

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [contracts/acceptance.md](contracts/acceptance.md)

## Phase 1: Setup

- [x] T001 Confirm the current S207 branch, five issue scopes, and existing test entry points in `specs/207-game-first-capture/spec.md` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T002 Record baseline behavior for route, scope, admission, and SOCKS classification in focused tests under `crates/fragcap/tests/`, `crates/fragcap-cli/src/commands/deep_capture.rs`, and `crates/fragcap-proxy/src/socks5.rs`.

## Phase 2: Foundational route authority

- [x] T003 Add failing tests for route-choice preservation and pre-effect mismatch in `crates/fragcap/src/deep_capture/session.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T004 Add typed launcher exposure and direct-game routing intent to `crates/fragcap/src/deep_capture/model.rs` and `crates/fragcap/src/deep_capture/routing.rs`.
- [x] T005 Carry one route identity through CLI authorization and `SessionPlan` preparation, refusing mismatches before effects in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap/src/deep_capture/session.rs`.
- [x] T006 Assert exact authorized and applied environment effects, including the empty ordinary launcher case, in `crates/fragcap/src/deep_capture/routing.rs` and managed-launch tests under `crates/fragcap-cli/src/commands/`.

## Phase 3: User Story 1, ordinary exact-game capture

**Goal**: First ordinary capture proceeds without calibration, routes only the final game, and retains only proven game evidence.

**Independent test**: Controlled zero-, one-, and multi-intermediary chains with no prior facts show one authorization, a direct final-game route, native updater traffic, game-only packet/application evidence, and current-session facts.

- [x] T007 [US1] Add failing first-run and stale-fact admission tests in `crates/fragcap/src/deep_capture/policy.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T008 [US1] Remove the prior-positive compatibility admission gate while preserving current feasibility and observations in `crates/fragcap/src/deep_capture/policy.rs`.
- [x] T009 [US1] Change `auto` inspection resolution to exact game in `crates/fragcap/src/deep_capture/inspection.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T010 [US1] Apply proxy-free launcher exposure and exact-game direct route by default in `crates/fragcap/src/deep_capture/routing.rs`, `crates/fragcap/src/deep_capture/session.rs`, and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T011 [US1] Bind packet and application admission to the exact final-game generation in `crates/fragcap/src/deep_capture/session.rs`, `crates/fragcap/src/deep_capture/native.rs`, and `crates/fragcap/src/deep_capture/application.rs`.
- [x] T012 [US1] Reconcile plan, manifest, terminal, structured output, and observed compatibility facts for ordinary scope in `crates/fragcap-cli/src/commands/deep_capture.rs` and `crates/fragcap/src/deep_capture/session.rs`.
- [x] T013 [US1] Run controlled cold chains and unrelated sentinel regression under `integration/`, asserting zero intermediary payload and no unaccounted observed loss.

## Phase 4: User Story 2, explicit advanced launcher route

**Goal**: An operator can authorize full-chain capture with SOCKS-only launcher exposure, and a failed HTTP CONNECT handoff names the observed stage.

**Independent test**: An advanced fixture sees SOCKS-only variables, authenticated two-way bytes, and full-chain evidence; separate CONNECT client close is named before TLS.

- [x] T014 [US2] Add failing explicit advanced route and environment tests in `crates/fragcap/src/deep_capture/routing.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T015 [US2] Add explicit full-chain/SOCKS-only CLI selection and canonical authorization effects in `crates/fragcap-cli/src/cli.rs` and `crates/fragcap-cli/src/commands/deep_capture.rs`.
- [x] T016 [US2] Apply the declared SOCKS-only child environment without HTTP proxy URL assignment in `crates/fragcap/src/deep_capture/routing.rs` and managed launch adapter tests.
- [x] T017 [US2] Diagnose HTTP CONNECT client close after success and before TLS in `crates/fragcap-proxy/src/` and its controlled proxy tests.
- [x] T018 [US2] Verify advanced route receipt, two-way transfer, and no automatic route escalation under `integration/`.

## Phase 5: User Story 3, truthful protocol and route evidence

**Goal**: Plan and effects agree, and missing SOCKS prefixes never become positive plaintext or decryption facts.

**Independent test**: Mismatch refuses before effects; delayed TLS, partial prefix, EOF, read error, binary, and positive protocol fixtures retain truthful classifications and bytes.

- [x] T019 [US3] Add failing missing/late/partial SOCKS prefix tests in `crates/fragcap-proxy/src/socks5.rs` and compatibility tests in `crates/fragcap/src/deep_capture/policy.rs`.
- [x] T020 [US3] Introduce unknown provenance and prevent positive plaintext promotion without evidence in `crates/fragcap-proxy/src/socks5.rs`, `crates/fragcap/src/deep_capture/application.rs`, and `crates/fragcap/src/deep_capture/policy.rs`.
- [x] T021 [US3] Preserve byte-transparent forwarding, loss counters, and historical fact readback in proxy and target-store tests under `crates/fragcap-proxy/` and `crates/fragcap/`.
- [x] T022 [US3] Assert mismatched authorization and late-port-only variation in `crates/fragcap-cli/src/commands/deep_capture.rs`.

## Phase 6: Polish and cross-cutting

- [x] T023 Remove public calibration command, guided/resume and embedded flags while preserving ordinary assessment and old store/bundle readers in `crates/fragcap-cli/src/cli.rs`, `crates/fragcap-cli/src/lib.rs`, and `crates/fragcap-cli/src/commands/`.
- [x] T024 Update directly affected README/help, master specification, chronological plan, and S207 changelog fragments in `README.md`, `crates/fragcap-cli/src/commands/workflow_help.rs`, `docs/fragcap-specification.md`, `docs/plans/README.md`, and `changelog.d/`.
- [ ] T025 Run focused tests, controlled Windows route matrix, `cargo xtask ci`, encoding check, and exact-head CI; record evidence in `specs/207-game-first-capture/quickstart.md` and PR.
- [ ] T026 Commit on `codex/s207-game-first-capture`, push, publish PR, resolve no more than two Codex review rounds, wait for terminal green checks, and hand off the final merge ritual.

## Dependencies and delivery order

Foundational tasks T003-T006 precede US1 and US2. US1 establishes ordinary route and scope, then US2 adds the explicit advanced route. US3 can begin its SOCKS tests after the route model is fixed and must finish before compatibility claims are accepted. T023-T026 follow all user stories.

## Parallel opportunities

Protocol prefix tests T019 and CLI calibration inventory T023 touch separate files, but implementation remains sequenced through the shared plan and compatibility contract. Windows matrix T013 and docs inventory T024 can be prepared without changing shared code.

## Implementation strategy

Complete the route authority and ordinary first-run path as the MVP, then the deliberate advanced route and provenance, then retire calibration controls. Every story keeps its independent controlled acceptance result; do not claim physical installed-title success from fixtures.
