# Tasks: Direct Route Acceptance

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [acceptance contract](contracts/acceptance.md), [quickstart.md](quickstart.md)

**Tests**: Failing regressions precede application fixes. Security, unrelated-traffic, access, and loss tests are required.

## Phase 1: Setup and coverage inventory

- [x] T001 Confirm clean S204 branch, active feature directory, issue states, source baseline, and existing gate commands.
- [x] T002 Map every #593, #594, #595, and #598 criterion to existing S203 code/tests or a new exact S204 task; distinguish already demonstrated results from unverified claims.
- [x] T003 Run the read-only spec-kit analyze gate; resolve every critical or high artifact finding before implementation.

## Phase 2: External selected TCP (#593)

**Goal**: Prove a client-visible non-loopback route and precise negative outcomes.

**Independent Test**: Exact two-way IPv4/IPv6 bytes, active unrelated sentinels, diagnostic on/off, refusal, cancellation, and cleanup.

- [x] T004 [US1] Add or strengthen failing external IPv4/IPv6 selected-route regressions that assert client-visible ACK and exact two-way bytes, plus upstream-connected zero-byte distinction.
- [x] T005 [US1] Fix any reproduced bridge, interface, checksum, admission, transfer, or terminal-report defect and preserve the exact regression; leave unproven candidates unchanged.
- [x] T006 [US1] Run controlled non-loopback selected TCP with active sentinels and diagnostics enabled and disabled; record actual packet/loss/latency results and verify remote listener exit.
- [x] T007 [US1] Run refusal, cancellation, repeat, and clean teardown cases through the signed Windows route and reconcile terminal outcomes.

## Phase 3: Direct calibration (#594)

**Goal**: Exact route facts survive passive original-packet absence without accepting incomplete proof.

**Independent Test**: Original-present/absent TCP/UDP, packet-history truncation, and all specified negative route-proof cases reconcile across four decision surfaces.

- [ ] T008 [US2] Add failing original-present/absent, TCP/UDP, competing-owner, reuse, passive-history-gap, incomplete-route-proof, helper, refusal, and no-payload tests where S203 coverage is incomplete.
- [ ] T009 [US2] Fix demonstrated correlation, compatibility, verdict, store, or ordinary-admission disagreement; keep packet availability independent of route reachability.
- [ ] T010 [US2] Qualify exact positive and negative decisions under the Windows driver matrix, including both available passive-packet order cases or explicitly record the unexercised driver order.

## Phase 4: Native diagnostics and help (#595, #598)

**Goal**: Complete report and wording acceptance without changing native source authority.

**Independent Test**: Complete/partial fixture corpus, access, Doctor, sharing, summary parity, and packaged help.

- [ ] T011 [US3] Add failing route-stream and report fixtures for normal, zero-byte, refused, queue gap, writer failure, crash prefix, incomplete cleanup, and optional packet correlation; assert source immutability and no secret receipt or unrelated payload.
- [ ] T012 [US3] Add failing manifest, access, Doctor/recovery, retention/share, and human/JSON parity checks for complete and partial bundles; fix any demonstrated gaps.
- [ ] T013 [US4] Check built and packaged CLI help, guided plan, resolver-linked auto/game tests for direct and launcher-mediated topologies, and active docs; fix wording drift only where found.

## Phase 5: Qualification and publication

- [ ] T014 Add missing required signed-driver and MSI/ZIP acceptance rows to the existing versioned matrix or package tier and fail on skipped rows; keep harness evidence distinct from a default installed journey.
- [ ] T015 Reconcile master specification, chronological plan index, unreleased changelog, S204 acceptance checklist, and issue traceability with observed behavior and limits.
- [ ] T016 Run foreground formatting, lint, tests, repository checks, controlled Windows and package gates, and UTF-8/LF/mojibake checks on the final code.
- [ ] T017 Commit the coherent S204 slice, push the authorized branch, and open an official PR tracing each issue separately.
- [ ] T018 Resolve every received CI, Codex, and security review comment; request at most one explicit second @Codex round and recheck the exact pushed head.
- [ ] T019 Close each included issue whose scoped acceptance is met, or state the specific unmet criterion; hand the reviewed green PR to the owner for final review and merge without merging it.

## Dependencies and Execution Order

The analyze gate precedes application implementation. Failing route and correlation tests precede fixes. Live route and package results precede acceptance and issue closure. Documentation records observed results, not expected results. Review closeout follows the official PR.
