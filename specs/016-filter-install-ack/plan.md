# Implementation Plan: Filter Manager Install Acknowledgement

**Branch**: `feat/filter-install-ack` (spec dir `016-`) | **Date**: 2026-08-10 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/016-filter-install-ack/spec.md`

## Summary

Close the deferred half of the S13 review finding P2 (issue #20). `FilterManager`
no longer commits a handle's installed program and gap-set clear inside `poll`;
it records a `pending` program, issues one install in flight per handle, and
commits only on a `(handle, ok)` acknowledgement the capture thread sends back
over a reverse `mpsc` channel after `set_filter`. A rejected install keeps the
prior program and is retried; a rejecting handle is not retired. The offline path,
where every filter is accepted, is unchanged. Decisions in [research.md](research.md).

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82.

**Primary Dependencies**: None added. A `std::sync::mpsc` reverse channel and a
`BTreeSet<Endpoint>` `pending` field.

**Testing**: `cargo test --workspace --locked`; `FilterManager` unit tests
(rejection retry, success commit, plus the existing tests updated to acknowledge)
and a pipeline integration test with a rejecting source double.

**Target Platform**: Platform-neutral tier 1; `cargo xtask neutral` still builds.

**Project Type**: Rust workspace (library + CLI).

**Performance Goals**: The control-thread poll gains a bounded `try_recv` drain of
the ack channel per iteration; no per-packet cost.

**Constraints**: P-2 (no new dep, no platform dep in core), P-3 (`PacketSource`
gains no bound; the ack is a thread channel), P-4/P-9 (`filter_gaps` unchanged,
outside the conservation identity). UTF-8 no BOM, LF, no em/en dashes.

**Scale/Scope**: One field and one method on `FilterManager`, a `poll` change, one
reverse channel plumbed through `Pipeline::run` and `acquire`, and test updates.

## Constitution Check

*GATE: passed at plan time; re-checked after design.*

- **P-1**: PASS. No process handle or denylisted technique.
- **P-2**: PASS. No new dependency; the ack channel is `std::sync::mpsc`. Core
  takes no platform dependency; `cargo xtask neutral` builds.
- **P-3**: PASS. `PacketSource` and `FlowAttributor` unchanged; the acknowledgement
  travels by thread channel, not on a trait.
- **P-4 / P-9**: PASS. `filter_gaps` keeps its meaning and stays outside the
  conservation identity; committing on acknowledgement makes the count measure
  against the truly-installed program, which is more honest, not less.
- No violations; Complexity Tracking empty.

## Project Structure

### Documentation (this feature)

```text
specs/016-filter-install-ack/
├── plan.md              # This file
├── spec.md
├── research.md          # Decisions D-1..D-5
├── data-model.md        # FilterManager state and the ack message
├── quickstart.md        # Tier-1 validation guide
└── checklists/requirements.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── filter.rs            # HandleState.pending; poll no longer commits optimistically; acknowledge(); retire() clears pending; updated + new tests
└── pipeline/mod.rs      # reverse ack channel: create it, clone the sender per capture thread with its handle index, drain + acknowledge in the control loop; acquire sends the set_filter result; new rejecting-source integration test

changelog.d/             # S016 added fragment (+ decision note if warranted)
```

**Structure Decision**: Confined to `fragcap-core`: the `FilterManager` policy and
the pipeline wiring. No other crate changes; the CLI and attribution are untouched.

## Implementation order (TDD, tier 1)

1. **`FilterManager`** (`filter.rs`): add `pending: Option<BTreeSet<Endpoint>>` to
   `HandleState`; change `poll` to set `pending` and `last_install` and issue one
   install in flight (skip while `pending` is set), removing the optimistic
   `installed`/gap-clear; add `acknowledge(&mut self, handle, ok)`; make `retire`
   clear `pending`.
2. **Update existing `FilterManager` tests** to `acknowledge(handle, true)` after
   each install; assertions unchanged.
3. **New `FilterManager` tests**: a rejection acknowledgement leaves the handle not
   installed and a later poll re-issues (SC-001); a success acknowledgement commits
   and preserves idempotence/rate-limit/gap accounting (SC-002); retries are spaced
   by the rate limit.
4. **Pipeline** (`pipeline/mod.rs`): create `mpsc::channel::<(usize, bool)>`; clone
   the sender into each capture thread with its handle index; in the control loop
   drain the ack channel and call `manager.acknowledge(...)` before `poll`; in
   `acquire` send `(handle, set_filter_result.is_ok())` after installing. Update the
   `acquire` signature and its callers/tests.
5. **New pipeline test**: a source double that rejects maintenance `set_filter`
   records more than one attempt of the same program (the control thread retried),
   and the run ends on its own (SC-003).
6. **Docs / changelog**: S016 added fragment; glossary entry for the install
   acknowledgement / pending install if a new term is introduced (P-6).
7. **Verify**: `cargo xtask ci`, `neutral`, `msrv`; corpus goldens byte-identical.

## Risks

- **Existing pipeline tests**: the `acquire` signature change and the ack send must
  not alter what the RecordingSource-based tests observe (they accept filters, so
  acknowledgement is success and the installed programs are unchanged). Guarded by
  running the full pipeline test module.
- **Liveness**: a capture thread that dies mid-install leaves a handle `pending`;
  the control thread still exits on `control_stop`, so no hang. Covered by the
  edge-case reasoning and the existing shutdown tests.
- **Goldens**: the corpus goldens must not move (offline accepts every filter).
  Guarded by `corpus_pipeline` and the CLI `cli_run` goldens.
