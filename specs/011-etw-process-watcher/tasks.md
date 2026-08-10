# Tasks: ETW Process Watcher and Tree

**Slice**: S11

**Branch**: `claude/s11-s12-parallel-dev-086100`

**Created**: 2026-08-09

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/process-api.md](contracts/process-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. The half of this slice that touches the
platform cannot run in the ordinary check set at all, so the half that can must
be exhaustive, and the half that cannot must say so rather than being quietly
untested.

Four notes on the shape, because the phase order does not follow the priority
order.

**Phase 2 changes the S02 types before anything new is built.** The command line
on `ProcessEvent::Started`, the settlement of `image` as a path, and the new
`CommandLine` enum all ripple through `fragcap-core` and its trait doubles.
Doing them in one sweep means the workspace compiles again before any new
behavior is added, so a later failure is attributable to the behavior rather
than to the churn.

**Phase 3 is the tree, and it is the bulk of the slice.** It is a pure fold, it
is where every ancestry claim the project will ever make is decided, and it is
the only part testable everywhere. Building it first means the watcher is
written against something already proven.

**Phase 4 delivers user story 3, which is the offline half.** The scripted
watcher and the two Appendix D chains exercise the whole of section 10.2 through
the same `ProcessTree::apply` the ETW watcher will use, on any machine.

**Phase 5 is the part that needs Windows and elevation.** Last, because it is
the least verifiable, and putting it last means it integrates against a consumer
already known to be correct rather than being debugged alongside one.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel with other `[P]` tasks in the same phase
  (different files, no dependency on an incomplete task)
- **[Story]**: The user story from [spec.md](spec.md) this serves
- Every task names the file it changes

## Phase 1: Setup

- [x] T001 Add `windows-sys` to `[workspace.dependencies]` in the root
      `Cargo.toml`, default features off, with the five feature groups from plan
      D-1 and a comment recording why it is taken over `ferrisetw` and why
      `Win32_System_Time` is in the list.
- [x] T002 Declare `windows-sys` as an optional dependency of
      `crates/fragcap-attr/Cargo.toml` and declare the `etw` feature with
      `default = []`. Pass the feature through from `crates/fragcap/Cargo.toml`.
      Do not enable it anywhere by default (plan D-2).
- [x] T003 [P] Create `crates/fragcap-core/src/process/` and move the existing
      `process.rs` to `process/mod.rs`, unchanged, so that the tree can land
      beside it in Phase 3 without a second move.

## Phase 2: The S02 vocabulary, changed once

- [x] T004 Add `CommandLine` to `crates/fragcap-core/src/process/mod.rs` as an
      enum with `Observed(Arc<str>)` and `Unavailable`, with the doc comment
      recording why it is not an `Option` (plan D-6, FR-036).
- [x] T005 Add `command_line: CommandLine` to `ProcessEvent::Started` in
      `crates/fragcap-core/src/process/mod.rs`, and record in the module
      documentation that this is a breaking change to a variant of a
      `#[non_exhaustive]` enum and a recorded deviation.
- [x] T006 Add `command_line: CommandLine` to `ProcessRecord` in the same file,
      defaulting to `Unavailable` in `ProcessRecord::new`, with the doc comment
      recording that the Windows snapshot cannot supply one without a right P-1
      forbids (research R-3).
- [x] T007 Settle `image` as the full image path in both types: doc comments,
      and the existing tests updated to use paths rather than bare file names
      (FR-038).
- [x] T008 Add `ProcessId` and `NodeId` newtypes to the same file, with the doc
      comment recording that one recycles and the other does not.
- [x] T009 Add `Ancestry` with `Observed`, `Snapshot`, and `Unresolved`, with
      the doc comment recording that it is stored rather than derived and citing
      the S06 fidelity lesson (plan D-8, FR-022).
- [x] T010 Add `WatcherReport` to the same file, with the doc comment recording
      why it is not part of `CaptureStats` (FR-015).
- [x] T011 Update the trait doubles in `crates/fragcap-core/src/traits.rs` for
      the changed `ProcessEvent::Started`, and re-export the new types from
      `crates/fragcap-core/src/lib.rs`.
- [x] T012 Run `cargo test -p fragcap-core` and confirm the workspace compiles
      and the existing suite passes before any new behavior is added.

## Phase 3: The tree (User Story 2, P1)

- [x] T013 Create `crates/fragcap-core/src/process/tree.rs` with `ProcessNode`
      carrying the nine fields from [data-model.md](data-model.md), and
      `image_name` deriving the file name from the path.
- [x] T014 Write the failing tier 1 tests first in
      `crates/fragcap-core/tests/process_tree.rs`, one per invariant in
      [contracts/process-api.md](contracts/process-api.md) section 3. They must
      fail for the right reason before `ProcessTree` exists.
- [x] T015 Implement `ProcessTree::new`, `apply` for `Started`, and `NodeId`
      issue, satisfying FR-019 and FR-020.
- [x] T016 Implement parent resolution by the pair of identifier and time, and
      `Ancestry::Unresolved` for a parent that resolves to nothing (FR-026,
      FR-030).
- [x] T017 Implement `apply` for `Exited`, including holding an exit whose start
      has not arrived and counting it unmatched only at the end of the session
      (FR-031).
- [x] T018 Implement `resolve(pid, at)` including the rule that a node with an
      unknown start time is selected only when no node with a known start time
      covers the time (FR-023, FR-024).
- [x] T019 Implement `ancestry` and `descends_from`, returning the path in
      creation order and including exited nodes (FR-032).
- [x] T020 Implement `apply_snapshot`, reconciling against nodes already present
      in either arrival order and preferring creation-time ancestry (FR-033).
- [x] T021 Implement `note_lost`, `is_complete`, `len`, and `unmatched_exits`
      (FR-029, FR-034).
- [x] T022 Add the retention test: a fold at session scale, ten times the larger
      reconnaissance session, asserting the retained node count equals the
      distinct processes observed and that nothing was discarded (SC-017).
- [x] T022a Add the identifier recycling test by name rather than leaving it to
      T014's blanket coverage: two processes sharing one `ProcessId` with
      non-overlapping lifetimes become two nodes, neither inherits the other's
      children, and `resolve` returns each for a time in its own lifetime
      (FR-025, SC-004). This is the highest-consequence case in the tree and the
      one whose failure is plausible rather than obvious.
- [x] T022b Add the verbatim command line test: a command line carrying
      characters outside ASCII, and one longer than any buffer an
      implementation would plausibly have chosen, each reaching the tree byte
      for byte through `ProcessTree::apply` (FR-035, SC-013). Assert on bytes,
      not on a display form, because a display form can normalize.
- [x] T023 Reserve `stage: Option<StageId>` on `ProcessNode`, always `None`,
      with the doc comment naming S12 as its only writer (FR-049).
- [x] T024 Run `cargo test -p fragcap-core` and confirm every invariant in the
      contract is asserted by a test that fails when the invariant is broken.

## Phase 4: The offline watcher (User Story 3, P1)

- [x] T025 Create `crates/fragcap-attr/src/proc_script.rs` with `ProcessScript`
      and its builder, per [contracts/process-api.md](contracts/process-api.md)
      section 6.
- [x] T026 Implement `ScriptedWatcher` and its `ProcessWatcher` implementation,
      with `subscribe` returning an independent receiver per call (FR-012,
      FR-039).
- [x] T027 Declare both modules in `crates/fragcap-attr/src/lib.rs` and
      re-export their types. Touch only the lines this slice needs, because S10
      is editing the same file in parallel.
- [x] T028 [P] Write `crates/fragcap-attr/tests/chains.rs` replaying the ESO
      chain from Appendix D, asserting five levels from the shell to the client
      (SC-002).
- [x] T029 [P] Extend the same file with the Division 2 chain, asserting seven
      nodes, three of them sharing the image name `TheDivision2.exe`, told apart
      by ancestry (SC-003).
- [x] T030 Add the test that a tree built from a script is identical to a tree
      built from the same events delivered in a different valid order (FR-041,
      contract invariant 7).
- [x] T031 Run `cargo test -p fragcap-attr` on a machine with no elevation and
      confirm the whole of section 10.2 executes (SC-011).

## Phase 5: The ETW watcher (User Story 1 and 4, P1 and P2)

- [x] T032 Create `crates/fragcap-attr/src/etw/mod.rs` behind the `etw` feature
      with `EtwWatcher` and `WatcherError`, per
      [contracts/process-api.md](contracts/process-api.md) section 5.
- [x] T033 Implement `etw/session.rs`: `StartTraceW` with
      `EVENT_TRACE_SYSTEM_LOGGER_MODE` and the client context set to system
      time, `EnableTraceEx2` for the process provider, and teardown on `Drop`
      (FR-005, plan D-4, research R-6).
- [x] T034 Implement the error mapping from
      [contracts/process-api.md](contracts/process-api.md) section 5, with
      `ERROR_ACCESS_DENIED` producing `NotElevated` and every other failure
      relaying the platform's own code (FR-011, FR-016).
- [x] T035 Implement `etw/consumer.rs`: `OpenTraceW`, `ProcessTrace` on its own
      thread, the callback, and the unbounded fan-out to subscribers (FR-013).
- [x] T036 Implement `etw/record.rs`: the process event layout, the `FILETIME`
      conversion through a named epoch constant, and the lost-event counters
      into `WatcherReport` (FR-003, FR-014, research R-6).
- [x] T037 Implement `etw/snapshot.rs`: `CreateToolhelp32Snapshot` enumeration,
      and no process handle at all: the start time that would have needed
      one is recorded as unknown instead, per S10's lint and the amendment
      to research R-3 (FR-008, FR-009).
- [x] T038 Implement `EtwWatcher::start` subscribing before snapshotting, in
      that order, with the doc comment recording why (FR-007, plan D-5).
- [x] T039 Write `crates/fragcap-attr/tests/etw_live.rs` as tier 2, marked
      `#[ignore]`, spawning a short-lived child and asserting its start event
      names the test process as parent and carries its image path and command
      line (SC-001).
- [x] T040 Add the unelevated test asserting `WatcherError::NotElevated` rather
      than a generic failure (SC-008).

## Phase 6: The mechanical checks

- [x] T041 Extend `xtask/src/lint.rs` with the memory-rights check, failing on
      `PROCESS_VM_READ`, `PROCESS_VM_WRITE`, `PROCESS_VM_OPERATION`, and
      `PROCESS_ALL_ACCESS` in any fragcap source (plan D-11, SC-012).
- [x] T042 [P] Extend the `neutral` arm in `xtask/src/main.rs` to build
      `fragcap-attr` alongside `fragcap-core` and `fragcap-capture` (SC-010).
      The check is a match arm there, not a module; there is no
      `xtask/src/neutral.rs`.
- [x] T043 Extend `.github/workflows/platform.yml` with the
      `crates/fragcap-attr/**` path trigger, a build step for
      `fragcap-attr --features etw`, and a runtime elevation check gating the
      tier 2 tests, reporting plainly which case it took (plan D-10).

## Phase 7: Documentation and the record

- [x] T044 [P] Add six glossary entries to `docs/glossary.md`: synthetic process
      identifier, process node, ancestry provenance, startup snapshot, trace
      session, lost event. Cross-link the four existing entries rather than
      duplicating them (FR-046, plan D-9). S10 is adding entries to the same
      file in parallel, so append within the existing category sections and do
      not reorder anything, which keeps the merge conflict to the lines this
      slice actually adds.
- [x] T045 [P] Write `changelog.d/S11-etw-process-watcher.added.md`.
- [x] T046 Write `changelog.d/S11-etw-process-watcher.decisions.md` carrying the
      five deviations, the `windows-sys` choice, the refusal of a polling
      fallback, the dated decision for the `platform` workflow change, and the
      slice narrative that would otherwise go into `AGENTS.md`.
- [x] T047 Work through `checklists/observation.md` and record the resolution of
      every item, striking any that turn out not to apply with a one-line
      reason.

## Phase 8: Verification

- [x] T048 Run `cargo xtask ci` in the foreground on a machine with no
      elevation, and read the output (SC-009).
- [x] T049 Run `cargo xtask neutral` and `cargo xtask msrv`, and record which
      ran and which exited 2.
- [x] T050 Confirm the memory-rights lint fails when `PROCESS_VM_READ` is added
      to a source file, then remove it. A check that has never failed has not
      been shown to work.
- [x] T051 Run the existing pipeline tests and confirm the conservation identity
      is unchanged by this slice, which is what SC-016 asserts: the watcher's
      lost-event count lives in `WatcherReport` and appears nowhere in
      `CaptureStats`.
- [x] T052 Verify by hand that `windows-sys` 0.61.2 and `windows-link` 0.2.1 are
      MIT OR Apache-2.0 across the whole graph, against the allowlist in
      `deny.toml`, and record the result in the decisions fragment (FR-047).
      `cargo xtask license` does not check this: it checks per-crate license
      text for registry publication. `cargo deny` owns dependency licensing,
      and it is weekly and dispatch-only and has still never run, so this is a
      by-hand verification recorded as such, exactly as S09 did.

## Dependencies

- Phase 2 blocks Phase 3: the tree folds the changed events.
- Phase 3 blocks Phase 4 and Phase 5: both watchers feed the same tree.
- Phase 4 does not block Phase 5, and Phase 5 does not block Phase 4. If Phase 5
  cannot be completed on the available machine, Phase 4 still delivers user
  story 3 in full and the slice says which half is unverified.
- Phase 6 depends on Phase 5 for the lint to have a subject, and on Phase 2 for
  `neutral` to have a crate that must build without a backend.
- Phase 7 depends on everything above it, because a changelog written before the
  work is a prediction rather than a record.

## Parallel opportunities

- T003 with T001 and T002: different files.
- T028 with T029: both add tests to one file and must be written in sequence if
  the same author writes them, but they are independent as work items.
- T042 with T041: different files in `xtask`.
- T044 with T045: different files.
