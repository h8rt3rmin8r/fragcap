# Tasks: Direct Socket Route

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [internal contract](contracts/internal-route-v1.md)

**Tests**: Required by #578/#581/#582 and the constitution. Add failing controlled assertions before each implementation phase.

## Phase 1: Setup and governing dependency gate

- [x] T001 Review the exact `smoltcp` 0.12.0 and `managed` 0.8.0 source, features, transitive graph, 0BSD license, unsafe code, and MSRV against `supply-chain/policy-v1.json` and constitution P-8; obtain the required explicit owner approval before adding either dependency.
- [x] T002 Record the owner-approved broad pass-through, measured first-packet and unrelated-traffic probes, packet-to-stream choice, WFP comparison, and constitution 3.0.0 amendment in `specs/200-direct-socket-route/research.md` and the dated decisions changelog fragment.
- [x] T003 Build a closed, versioned scenario oracle for #578 under `integration/` with exact original flows, process generations, expected results, active sentinel metrics and zero skipped rows.

## Phase 2: Foundational ownership and admission

- [x] T004 [US1] Write failing driver-free cases in the owning `crates/fragcap/src/deep_capture/direct_route.rs` test module for exact final-client generation, chain ancestry, initial TCP SYN, delayed and missing socket events, PID/port reuse, ambiguous owner and unrelated same-destination sockets.
- [x] T005 [US1] Implement an immutable direct component in `crates/fragcap/src/deep_capture/routing.rs` with explicit pre-effect authorization, Administrator prerequisite, scoped final-client selection, destination admission policy, and independent child-environment route effects; bind each flow's exact original destination before diversion.
- [x] T006 [US1] Implement bounded original-flow, packet-disposition and one-use route-receipt types in `crates/fragcap/src/deep_capture/`, with session and tuple binding, expiry, replay refusal, and exact terminal accounting.
- [x] T007 [US2] Write failing security cases for cross-session receipt replay, helper PID substitution, unknown-owner promotion, duplicate flow admission, proxy recursion and unrelated artifact contamination.

## Phase 3: User Story 1, raw TCP and proxy bridge

**Independent test**: The synthetic final client exchanges an arbitrary non-proxy TCP payload exactly once through the native proxy while the original-flow oracle, receipt, and proxy result agree.

- [x] T008 [US1] Write failing controlled TCP tests for payload ordering, retransmission, half-close, original destination, trusted TLS and unknown protocol in the direct stack, bridge, runtime, and proxy test modules.
- [x] T009 [US1] Add Windows-only dynamic loading and exact signed-component identity checks, socket-event observation, preauthorized prearmed TCP/UDP packet handle, finite queue, progress watchdog, and finite handle lifetime to the direct adapter under `crates/fragcap/src/deep_capture/`.
- [x] T010 [US1] Reinject unrelated packets promptly after header-only candidacy, without application-payload inspection or retention; hold only finite candidate first packets, then join socket events, current managed process proof and exact packet tuple before selected admission. Keep WinDivert event/packet order in the QueryPerformanceCounter domain, bound wall-clock conversion, expiry, and ambiguity.
- [x] T011 [US1] Add a finite TCP packet-to-stream path with bidirectional packet accounting, ordered bytes, retransmission, half-close, cancellation and restart.
- [x] T012 [US1] Add a session-private authenticated bridge from raw TCP to existing native HTTP CONNECT or SOCKS5 CONNECT, with one-use receipt binding and destination policy.
- [x] T013 [US1] Feed trusted original owner into native proxy/application correlation while retaining helper-socket identity separately; classify inspected, opaque, unknown and refused from observed facts.

## Phase 4: User Story 2, unrelated traffic and cleanup

**Independent test**: Active same-destination and local-service sentinels complete through start, run, fault and stop within the declared loss and latency bound, without game artifact admission.

- [x] T014 [US2] Write failing sentinel and packet-conservation cases for shared destination, distinct destination, helper/proxy legs, loopback and IPv4/IPv6.
- [x] T015 [US2] Exclude unrelated tuples from selected consumption and application evidence, reinject them promptly, account every packet delivered to the adapter as reinjected, consumed, or explicitly lost, and report pre-receive driver loss as unmeasurable.
- [x] T016 [US2] Add bounded failure injection for driver open/read/send, stack/bridge/proxy failure, queue saturation, process exit and writer failure.
- [x] T017 [US2] Bind the route lease and its exact driver handles, tasks, and receipts to the pre-effect route resource obligation; close process-owned handles and tasks on shutdown, and name unconfirmed cleanup in the existing journal and Doctor recovery path.
- [x] T018 [US2] Run the disposable Windows sentinel phase matrix from `.github/workflows/windows-integration.yml`, including paired light/parallel load, candidate expiry, worker stall and watchdog release, and repair every isolation, latency or cleanup failure.

## Phase 5: User Story 3, selected UDP

**Independent test**: The synthetic client exchanges exact datagrams and replies through the authenticated association with original peer and process truth intact.

- [x] T019 [US3] Write failing UDP and QUIC cases for IPv4/IPv6, contacted-peer reply ownership, datagram boundaries, unknown payload, cancellation and saturation.
- [x] T020 [US3] Add finite UDP packet and association ownership to the direct adapter and bridge into authenticated SOCKS5 UDP and existing native QUIC paths where valid.
- [x] T021 [US3] Reconcile UDP packet counts, original-peer ledger, proxy connection ID and actual inspected or opaque classification.

## Phase 6: Full gate and handoff

- [ ] T022 Run all required driver-free and disposable Windows rows with zero skips and compare the exact oracle and sentinel metrics.
- [x] T023 Reconcile the master specification, glossary entries, plan index, security inventory, dependency review, operator truth and dated changelog; run a second Spec-Kit analysis and resolve every finding.
- [ ] T024 Run `cargo xtask ci`, applicable Windows, supply-chain and documentation gates and exact-head hosted CI; repair failures and verify UTF-8 without BOM or mojibake.
- [ ] T025 Commit the complete slice, push `codex/s200-direct-socket-route`, open the official PR with issue closures, respond to every CI/review finding, request at most one second `@Codex` review, then hand the green PR to the owner for final review and merge.

## Dependencies

T001 gates adding either 0BSD dependency. T003-T007 establish the oracle and admission boundary. T008-T013 complete TCP before T014-T018 isolate and harden it. T019-T021 add UDP over the same ownership and bridge contracts. T022-T025 are the final cross-cutting gate. No user story can be called complete from driver-free tests alone.

## Implementation Strategy

Keep the first useful controlled TCP path small, prove owner truth and packet conservation, then add isolation and UDP. The end state is one complete slice; intermediate checkpoints are development evidence only.
