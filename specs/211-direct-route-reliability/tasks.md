# Tasks: Direct Route Reliability

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), [helper-and-packet-evidence.md](contracts/helper-and-packet-evidence.md)

## Phase 1: Spec and analysis

- [x] T001 Read #619, #594, S210 T011, existing route/bridge/journal code, and project constitution.
- [x] T002 Specify, clarify, checklist, plan, tasks, and analyze ownership, finite cleanup, and physical evidence boundary.

## Phase 2: Helper repair

**User stories**: Delayed valid connection and explain a real failure.

- [x] T003 Add failing 60-100 ms success, refusal, true timeout, and stage/error diagnostic tests.
- [x] T004 Move TCP helper SOCKS admission off packet worker with bounded exact pending state, packet accounting, and cancellation.
- [x] T005 Add protected typed helper outcome and verify no false refusal, watchdog release, sentinel loss, or duplicate route. Controlled source rows and the final exact-binary eight-case external-peer matrix passed; the worst active TCP sentinel was 18 ms against the 50 ms bound, route loss and watchdog release were zero, and the finite peer counted one selected exchange per permitted attempt.

## Phase 3: Physical correlation acceptance

**User story**: Trust original-packet evidence.

- [x] T006 Prepare exact portable package and controlled external client/peer runner, with no visible console children and fallible cleanup.
- [x] T007 Exercise original-present, original-absent, and truncated packet histories plus TCP/UDP negative cases on packaged WinDivert/Npcap. The final candidate retained a TCP refusal original with exact target ownership and incomplete history, while TCP transfers and UDP cases reported original unavailable. UDP refusal remained distinct from transfer. UDP original-present was not exercised; the acceptance matrix establishes both orderings across the supported transports.
- [x] T008 Reconcile route, compatibility, terminal/result, and manifest from the exact candidate bundle. All eight final-candidate hosted assertions and independent bundle readback agree on selected disposition, original/history state, route outcome, cleanup, and manifest correlation; required artifacts finalized with explicit process-trace partiality.

## Phase 4: Completion

- [x] T009 Update master specification, chronological slice map, decision/changelog records, and docs affected by changed helper behavior.
- [ ] T010 Run focused and full foreground gates, package and hosted checks, re-analyze, and inspect UTF-8/LF and cleanup.
- [ ] T011 Commit, push, publish PR, resolve no more than two total Codex review rounds and every security review, wait for green applicable CI, then hand off owner merge.

## Dependencies

T001-T002 precede implementation. T003 precedes T004-T005. T006 precedes T007-T008. T009-T011 close the shared slice only after both user-story tracks and their independent evidence gates are complete.
