# Tasks: Direct Packet and Process Correlation

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [correlation-evidence.md](contracts/correlation-evidence.md)

**Prerequisites**: S210 branch at merged S209 main; #594 and #618 open.

## Phase 1: Setup and analyze

- [x] T001 Confirm the direct-route receipt, flow registry, parser, socket attributor, process trace, artifact readers, and existing tests match the research boundary.
- [x] T002 Run the requirements and evidence-integrity checklists and resolve the Spec-Kit analyze gate before implementation.

## Phase 2: Red tests and packet facts

- [x] T003 Add parser-to-flow tests showing retained TCP flags and socket creation, including absent and truncated fields.
- [x] T004 Add failing process-tail tests for unique reset, continuous ACK/FIN, PID/tuple reuse, missing creation/image, event loss, and out-of-bound lag.
- [x] T005 Add failing direct-route tests for original packet present/absent/truncated, TCP/UDP transfer, no payload, competing owner and reused tuple.

## Phase 3: Process-tail implementation

- [x] T006 Carry optional socket creation from the exact IP Helper row and optional observed TCP flags through captured packet and finite flow history.
- [x] T007 Bind reset and continuity tails only to unique proved process generations within the finite bound and complete event history; retain true exit time and named limitations.
- [x] T008 Version and validate process trace records and trailer with tail basis and loss counts, preserving legacy readback.

## Phase 4: Direct-route reconciliation

- [x] T009 Reconcile one-use selected receipt, exact generation/tuple, packet disposition, transport-specific client transfer, proxy leg, and passive packet state as separate facts.
- [x] T010 Align compatibility, route journal, terminal/JSON, and manifest projections with versioned route/packet outcomes and explicit refusal/loss reasons.
- [ ] T011 Exercise controlled packaged WinDivert/Npcap original-present and original-absent cases and inspect complete artifact readback. This remains unverified on the current host: no elevated package binary, controlled external peer, or configured Npcap ordering fixture was available. Keep #594 open until this gate passes.

## Phase 5: Completion

- [x] T012 Update master specification, ordered slice map, glossary if new terms appear, and dated changelog fragments.
- [x] T013 Run focused and full foreground repository gates, package checks, encoding/lint, and exact fixture cleanup.
- [x] T014 Re-run Spec-Kit analyze, record the open physical gate, mark verified tasks complete, and commit only S210 changes.

## Dependencies and execution order

T001 and T002 precede red tests. T003 and T004 precede T006 through T008. T005 precedes T009 through T011. T012 through T014 close the source and evidence work.
