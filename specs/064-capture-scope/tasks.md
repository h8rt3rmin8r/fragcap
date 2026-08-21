---

description: "Task list for slice S064, capture scope and truthful narration"
---

# Tasks: Capture scope and truthful narration

**Input**: Design documents from `/specs/064-capture-scope/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md)

**Tests**: Required, and written before the predicate. The gate is a hot-path
admission decision whose failure mode is silent data loss, so the accounting
assertion exists before the thing it accounts for.

## Format: `[ID] [P?] [Story] Description`

- **[Story]**: US1 the scope decision, US2 the accounting, US3 the reporting,
  US4 the narration

---

## Phase 1: Foundational

- [x] T001 Re-confirm the three blast-radius measurements from Phase 0 against
  this branch, since every later decision rests on them: `--process` goldens
  carry `role`/`stage`; the CLI capture goldens are entirely `game.exe`; the
  corpus tests pass no gate. A figure that does not match halts the slice.

---

## Phase 2: User Story 2 - The accounting, first (P1)

The counters and their reconciliation come before the predicate, so the
predicate is added to a gate that already refuses to lose anything silently.

- [x] T002 [US2] Add `scope_discarded` and `scope_unresolved_discarded` atomics
  to `SessionGate`'s shared state, beside `watch_discarded` and
  `out_of_window_discarded`, with accessors. Satisfies FR-007 and FR-008 in
  part.
- [x] T003 [US2] Add the session-level reconciliation assertion to the gate unit
  tests in `crates/fragcap/src/session.rs`: the four discard reasons sum to the
  gate's own rejection count, which is what the pipeline counts as
  `gate_dropped`. Satisfies FR-009.
- [x] T004 [US2] Confirm the pipeline conservation identity in
  `crates/fragcap-core/src/pipeline/mod.rs` needs no edit, because `gate_dropped`
  is counted at the call site and the reasons have always lived on the gate.
  Assert by running the existing tests, not by reading. Satisfies FR-010.

---

## Phase 3: User Story 1 - The scope decision (P1)

- [x] T005 [US1] Add a scope value to the session config and a `--scope
  target|profile|all` flag defaulting to `target`, threaded through
  `assemble.rs`. Satisfies FR-002.
- [x] T006 [US1] Write the gate tests first, over synthetic packets: an
  attributed-and-bound packet is admitted; an attributed-but-unbound packet is
  rejected and counts `scope_discarded`; an unattributed packet is rejected and
  counts `scope_unresolved_discarded`; `--scope all` admits all three. Watch
  them fail.
- [x] T007 [US1] Add the fourth test to `SessionGate::admit`, after the window
  tests and **before** the volume bound, so an out-of-scope packet never
  consumes the operator's byte budget. Resolve the role set once at
  construction; no allocation on the hot path. Satisfies FR-001, FR-003, FR-004,
  FR-005, FR-006.
- [x] T008 [US1] Confirm T006 is green and the counters land in the right one.

- [x] T008a [US1] Handle the observe-mode interaction (FR-021). A target
  resolved in observe mode is not yet identified, so a run scoped to it writes
  nothing and promotes nothing: `holder_tally` counts only gate-admitted
  packets, so the gate would starve the mechanism that decides what the target
  is. Widen the scope to `all` for that run and warn, so the override is
  reported rather than silent. Found by running the S059 promotion test, which
  failed with `retained 0`.

---

## Phase 4: User Story 3 - The reporting (P2)

- [x] T009 [US3] Add both scope counters to `CompletionSummary` and to the
  `--json` completion event. Satisfies FR-011.
- [x] T010 [US3] Add the per-image breakdown from `stats.holder_tally`, and
  state at the render site that it counts gate-admitted packets only, so it is a
  breakdown of the file rather than of the wire. Satisfies FR-013.
- [x] T011 [US3] Put a target-scoped count beside `attributed` rather than
  redefining `attributed`, which other output already uses. Satisfies FR-012.
- [x] T012 [US3] Make the scope line report the effective `--scope` and drop
  `(enforced)` where it is not true of retention. Satisfies FR-014.

---

## Phase 5: User Story 4 - The narration (P1)

- [x] T013 [US4] Delete the two one-shot emits at `orchestrator.rs:277` and
  `:684`. They sample the endpoint count at acquisition, which on a `--launch`
  run is structurally zero, and never update.
- [x] T014 [US4] Emit a phase line before capture saying that capture is
  machine-wide while the target opens its first socket. Satisfies FR-016.
- [x] T015 [US4] Observe the narrowing transition in `drive`, which already runs
  on the CLI thread while the pipeline runs on its own, and emit a human line on
  the first narrowing naming the target and the count. Satisfies FR-015 and
  FR-017.
- [x] T016 [US4] Emit `Event::FilterNarrowed` per narrowing in the structured
  stream; debounce or suppress the human line for subsequent changes. Satisfies
  FR-018 and FR-020.
- [x] T017 [US4] Remove the bare "endpoint(s)" count from human output.
  Satisfies FR-019.

---

## Phase 6: Record and verify

- [x] T018 Add the "capture scope" glossary entry (P-6). The term is
  specification vocabulary in sections 11.5 and 12.3 and has no entry, because
  nothing implemented it until now.
- [x] T019 Write `changelog.d/S064-capture-scope.fixed.md` and
  `.decisions.md`. The decisions fragment records the user-visible default
  change (scoped output) and the two-counter split with its reasoning.
- [x] T020 Run `cargo xtask ci` in the foreground, watched to completion. The
  CLI capture goldens must not move (SC-005) and the conservation tests must
  pass unchanged (SC-003).
- [x] T021 Verify `--scope all` is byte-identical to `main`'s output over the
  same fixture, by running both and comparing bytes. Satisfies SC-004.
- [x] T022 Run a replay capture over traffic with attributed and unattributed
  packets and reconcile the counters by hand. Satisfies SC-002.
- [~] T023 **NOT PERFORMED.** A real `--launch` capture could not be run here.
  The `live` feature does not link on this machine: `wpcap.lib` comes from the
  npcap software development kit, which continuous integration downloads at
  build time and this repository never vendors, so it is absent from the
  checkout. Acquiring it is a third-party download, and a meaningful live run
  additionally means launching a game on the operator's desktop, which is a
  disruptive side effect not to be taken unprompted mid-slice.

  Per SC-001 as amended, this weakens the claim rather than blocking the slice,
  and the weakening is stated rather than hidden. What **was** demonstrated, by
  replay over the same gate: a capture whose traffic resolves to a process no
  profile stage binds now reports `packets captured 24, retained 0, out of scope
  24`, where before this slice it wrote all 24 to the file. That is the issue
  #184 signature exactly, at fixture scale. The remaining unconfirmed step is
  that a live socket-table attribution produces the same stamps a scripted one
  does, which the committed goldens already show for the `--process` path.
- [x] T024 Stage only this slice's files and commit. Never stage
  `.specify/feature.json`; never edit `CHANGELOG.md` from a feature branch.

---

## Dependencies

- T001 blocks everything.
- Phase 2 precedes Phase 3: the accounting exists before the discard path it
  accounts for.
- T006 precedes T007, watched failing first.
- Phase 4 depends on Phase 2 for the counters it renders.
- Phase 5 is independent of Phases 2 to 4 and may proceed in parallel; it shares
  only `orchestrator.rs` with T012.
- T023 follows everything and needs npcap plus a Steam title.

## Out of scope

Per `spec.md`: the live status display (#186, OOS-001), directional output
filtering (OOS-002), shortening the bootstrap window (OOS-003), and retroactive
filtering of written output (OOS-004).
