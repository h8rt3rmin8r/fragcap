# Tasks: Core Types and Traits

**Slice**: S02

**Branch**: `feat/core-types-traits`

**Created**: 2026-08-08

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/core-api.md](contracts/core-api.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional here. The spec's value is that the
constitution constraints are enforced structurally, and a constraint with no
test asserting it is a claim rather than an enforcement. Every task in Phase 6
maps to a validation rule V-1 through V-10 in `data-model.md`.

## Phase 1: Setup

- [x] T001 Add `bytes = "1.12"` to `[workspace.dependencies]` in `Cargo.toml`,
  per plan D-1
- [x] T002 Add `bytes.workspace = true` to `[dependencies]` in
  `crates/fragcap-core/Cargo.toml`
- [x] T003 Run `cargo build --workspace --locked` and commit the resulting
  `Cargo.lock` change, confirming exactly one crate entered the graph

## Phase 2: Foundational

These are leaf types with no dependency on each other. Every later phase needs
them.

- [x] T004 [P] Define `Timestamp` in `crates/fragcap-core/src/packet.rs` as an
  `i64` nanosecond count since the Unix epoch, per plan D-2, with constructors
  from and accessors to nanoseconds and no resolution field
- [x] T005 [P] Define `Proto` and `Direction` in
  `crates/fragcap-core/src/flow.rs`, deriving equality, hashing, copy, and debug
- [x] T006 [P] Define `LinkType` in `crates/fragcap-core/src/link.rs`, marked
  non-exhaustive, documenting that S09 discovers what the backend reports
- [x] T007 [P] Define `StageId` in `crates/fragcap-core/src/attribution.rs` as a
  shared immutable string, documenting that S05 profiles name stages
- [x] T008 [P] Define `FilterProgram` in `crates/fragcap-core/src/filter.rs` in
  its minimal shape, documenting that S13 owns filter management
- [x] T009 [P] Define `ProcessEvent` and `ProcessRecord` in
  `crates/fragcap-core/src/process.rs` in their minimal shapes, documenting that
  S11 owns the process watcher
- [x] T010 Declare the module set and crate documentation in
  `crates/fragcap-core/src/lib.rs`, replacing the S01 skeleton comment, and
  re-export the public surface

## Phase 3: User Story 1, a fixed vocabulary (Priority: P1)

**Goal**: A contributor can read one crate and know the shape of every seam.

**Independent test**: Write a stub implementation of each trait and express the
section 8.6 pipeline shape against them without adding a method.

- [x] T011 [US1] Define `Endpoint` in `crates/fragcap-core/src/flow.rs`, an
  address, port, and protocol
- [x] T012 [US1] Define `Attribution` in
  `crates/fragcap-core/src/attribution.rs` with `pid`, `process` and `role` as
  shared strings, and `stage`, per FR-007
- [x] T013 [US1] Define `RawPacket` in `crates/fragcap-core/src/packet.rs` with
  `ts`, `data` as the payload alias, and `orig_len` separate from payload
  length, per FR-008
- [x] T014 [US1] Define `CapturedPacket` in `crates/fragcap-core/src/packet.rs`
  with the raw fields plus optional `flow`, `direction`, and `attribution`, per
  FR-009
- [x] T015 [US1] Define the three error enums in
  `crates/fragcap-core/src/error.rs`, each non-exhaustive with named variants,
  and hand-write `Display` and `std::error::Error`, per plan D-3 and FR-022
- [x] T016 [US1] Declare `PacketSource`, `FlowAttributor`, `ProcessWatcher`,
  `Sink`, and `Dissector` in `crates/fragcap-core/src/traits.rs`, transcribed
  from specification section 8.5, per FR-014 through FR-018
- [x] T017 [US1] Document every public item, naming the owning slice for each
  seam a later slice fills, per FR-031

## Phase 4: User Story 2, the asymmetry cannot be papered over (Priority: P1)

**Goal**: A UDP attribution key carrying a remote endpoint is unrepresentable.

**Independent test**: Attempt to construct one and find the vocabulary offers no
way.

- [x] T018 [US2] Define `FlowKey` in `crates/fragcap-core/src/flow.rs` with
  `proto`, `local`, and `remote`, documenting that `local` is always the
  capturing host's endpoint, per FR-002
- [x] T019 [US2] Define `AttributionKey` in `crates/fragcap-core/src/flow.rs`
  with a `Pair` variant and a `Local` variant, and deliberately no variant
  carrying a remote for UDP, per FR-004
- [x] T020 [US2] Implement `FlowKey::attribution_key` returning `Pair` for TCP
  and `Local` for UDP, per FR-003
- [x] T021 [US2] Implement wildcard matching for a UDP local endpoint so both a
  wildcard bind address and a specific interface address match, per FR-005

## Phase 5: User Story 3, a discard path needs a counter (Priority: P1)

**Goal**: Every discard cause has its own named counter and no total is stored.

**Independent test**: Assert on one named counter rather than a total, and
confirm no stored total exists.

- [x] T022 [US3] Define `SourceStats` in `crates/fragcap-core/src/stats.rs` with
  `received`, `kernel_dropped`, and `interface_dropped`, using the names from
  specification section 12.4
- [x] T023 [US3] Define `CaptureStats` in `crates/fragcap-core/src/stats.rs`
  with `packets_captured`, `packets_attributed`, `packets_unattributed`,
  `buffer_dropped`, `sink_dropped`, `filter_gaps`, and a `source` field holding
  `SourceStats` by value, per plan D-4
- [x] T024 [US3] Implement totals as methods on `CaptureStats`, never as stored
  fields, per FR-025
- [x] T025 [US3] Implement `CapturedPacket::attribution_state` returning the
  three derived states, per FR-010 and plan D-5

## Phase 6: Tests asserting the validation rules

Each task names the rule it asserts. A rule with no failing-first test is not
enforced.

- [x] T026 [P] Test V-1 in `crates/fragcap-core/src/flow.rs`: `FlowKey` and
  `AttributionKey` work as map keys, and equal keys hash equally
- [x] T027 [P] Test V-3 in `crates/fragcap-core/src/flow.rs`: a TCP flow key
  derives `Pair`, a UDP flow key derives `Local`, asserted independently
- [x] T028 [P] Test V-10 in `crates/fragcap-core/src/flow.rs`: a UDP local
  endpoint matches both a wildcard bind address and a specific address
- [x] T029 [P] Test V-4 in `crates/fragcap-core/src/attribution.rs`: cloning an
  `Attribution` shares the process name rather than allocating
- [x] T030 [P] Test V-5 in `crates/fragcap-core/src/packet.rs`: a truncated
  packet reports its original on-wire length
- [x] T031 [P] Test V-9 in `crates/fragcap-core/src/packet.rs`: each of the
  three attribution states is constructed and read back correctly
- [x] T032 [P] Test V-6 in `crates/fragcap-core/src/stats.rs`: a total is
  computed from named counters, and an individual cause is assertable alone
- [x] T033 [P] Test the timestamp is lossless across construction and
  read-back, and that no rounding occurs, supporting FR-011
- [x] T034 Audit the public surface for P-9 compliance and record the result in
  `crates/fragcap-core/src/lib.rs` crate documentation: no public operation
  alters, masks, truncates, reorders, or withholds an observed field. Assert the
  observable part with a test that an observed field has no public setter and
  that `orig_len` cannot be lowered to match a truncated payload. Satisfies
  FR-029, which the analyze gate found had no task
- [x] T035 Test V-7 in `crates/fragcap-core/src/traits.rs`: construct each of
  the four behavioral traits behind a `Box<dyn _>`, which fails to compile if a
  trait stops being dyn-compatible
- [x] T036 Test V-8 and V-2 in `crates/fragcap-core/src/traits.rs`: wire stub
  implementations into the section 8.6 pipeline shape, proving the seams are
  expressible and that neither capture nor attribution references the other

## Phase 7: Documentation and record keeping

- [x] T037 Add a `docs/glossary.md` entry for every term this slice introduces,
  per FR-030 and P-6, following the entry template in specification section 4.3.
  Satisfies SC-008
- [x] T038 Queue the plan D-7 deviation for promotion to specification section
  29 by adding a `changelog.d/S02-core-types-traits.decisions.md` fragment
  naming the eight types that sections 8.4 and 8.5 reference without defining.
  The deviation is already recorded in `plan.md` D-7; what is outstanding is the
  promotion path, which the constitution requires and which a slice-local note
  does not satisfy
- [x] T039 Add a `changelog.d/S02-core-types-traits.added.md` fragment
- [x] T040 Update the `crates/fragcap-core/README.md` status section, since its
  published listing currently says the crate contains no functionality

## Phase 8: Verification

Run in the foreground and read the output. No step here may be inferred.

- [x] T041 Run `cargo xtask ci` and confirm `ci: all checks passed`. Satisfies
  SC-009
- [x] T042 Run `cargo xtask neutral` and confirm `fragcap-core` builds for a
  target with no capture backend, which is the P-2 proof and is now proving
  something about a real dependency. Satisfies SC-005
- [x] T043 Run `cargo xtask msrv` and read the output, confirming the declared
  1.82 minimum still holds with `bytes` in the graph, per research R-6.
  Satisfies SC-007
- [ ] T044 Run `cargo deny check licenses` if available locally and confirm the
  new dependency passes against the allowlist, per FR-028. Satisfies SC-006
- [x] T045 Verify every requirement FR-001 through FR-031 and every success
  criterion SC-001 through SC-011 traces to code, a test, or a check, and record
  any that do not. This is the task that catches a coverage gap of the kind the
  analyze gate found in FR-029. Satisfies SC-001, SC-002, SC-003, SC-004,
  SC-010, and SC-011

## Dependencies

```text
Phase 1 (setup) -> Phase 2 (leaf types) -> Phase 3 (US1 composites and traits)
Phase 3 -> Phase 4 (US2) and Phase 5 (US3), which are independent of each other
Phase 4 and Phase 5 -> Phase 6 (tests)
Phase 6 -> Phase 7 (docs) -> Phase 8 (verification)
```

Phase 4 and Phase 5 touch different files (`flow.rs` versus `stats.rs`) and can
proceed in either order.

## Parallel Opportunities

Phase 2 tasks T004 through T009 are all leaf types in separate files and carry
`[P]`. Phase 6 tasks T026 through T033 are separate test functions in separate
files and carry `[P]`; T034, T035, and T036 do not, because T034 edits crate
documentation and the other two both add to `traits.rs`.

## Implementation Strategy

The minimum viable increment is Phase 1 through Phase 3 plus T035: the
vocabulary exists, is documented, and is proven usable as trait objects. That
alone unblocks S03.

Phases 4 and 5 are what make the slice worth its spec, because they are where
the constitution constraints become structural rather than documented. Neither
is optional.

## Notes

`tasks.md` carries T043 explicitly because the constitution checklist item
CHK031 observed that the spec records the toolchain minimum as an expected
outcome rather than a requirement, so nothing would fail if it were forgotten.
Making it a task is the fix for that gap.

T040 exists because S02 changes what is true about a published crate. The
`fragcap-core` listing on the registry says the release is a skeleton with no
functionality, which stops being accurate the moment this slice ships. The
README change lands with the code, and the registry listing updates at the next
release rather than immediately.

T034 and T038 were added or rewritten by the analyze gate on 2026-08-08. T034
closes a coverage gap: FR-029 carries constitution P-9, which is
non-negotiable, and no task asserted it. T038 replaced a task that instructed
recording a deviation `plan.md` D-7 had already recorded, which would have let
the outstanding obligation be ticked off without action.

## Execution record

Completed 2026-08-08 on branch `feat/core-types-traits`.

**T044 did not run.** `cargo-deny` is not installed on this machine, so the
license audit was not executed and must not be reported as passing. What is
known instead: the one dependency added is `bytes` 1.12.1, licensed MIT,
confirmed against the registry, and MIT is on the allowlist in `deny.toml`. That
is evidence the check would pass, not evidence that it did. The `audit`
workflow owns the check, is weekly and dispatch-only, and has never completed a
run.

**Two checks were repaired rather than merely run.** `cargo xtask deps`
required `fragcap-core` to have zero dependencies, which is stricter than
constitution P-2 states; it now checks a named allowlist and still fails closed.
`cargo xtask msrv` built with the pinned toolchain and reported success, which
said nothing about the declared minimum; it now builds through `rustup run 1.82`
and exits 2 when that toolchain is absent. Both are recorded in
`changelog.d/S02-core-types-traits.decisions.md` and as plan decision D-8.

**T045 trace.** Every requirement FR-001 through FR-031 maps to a type, a test,
or a check. The four with no dedicated test are structural and are covered by
compilation: FR-014 through FR-018 are trait declarations that the stub
implementations in `traits.rs` exercise, and FR-031 documentation coverage is
enforced by review rather than mechanically until the documentation linter
arrives at S18.
