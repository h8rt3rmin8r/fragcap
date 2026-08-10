# Research: Filter Manager Install Acknowledgement

**Slice**: 016 (S13 follow-up; issue #20) | **Date**: 2026-08-10

Design decisions, grounded in a full read of `crates/fragcap-core/src/filter.rs`
(the `FilterManager`) and `crates/fragcap-core/src/pipeline/mod.rs` (the control
thread and `acquire`).

## D-1: Acknowledge-then-commit, one install in flight per handle

**Decision**: `FilterManager::poll` stops committing a handle's `installed`
program, `last_install`-based state, and gap-set clear optimistically. Instead it
records a `pending` program on the handle and issues no further install to that
handle while `pending` is set. A new `FilterManager::acknowledge(handle, ok)`
commits: on `ok`, `installed = Narrowed(pending)` and the gap set clears; on
failure, `pending` is dropped and `installed` / `last_install` / gap set are left
unchanged. `last_install` is set when the install is issued (in `poll`), so it
gates both reinstalls and retries.

**Rationale**: The bug is that `poll` commits before the capture thread confirms.
Committing on acknowledgement makes the manager's model match the handle. "One
install in flight per handle" is what makes a bare `(handle, ok)` acknowledgement
unambiguous: there is only ever one pending program to which it can refer, so the
message needs to carry no program identity. Setting `last_install` at issue spaces
retries at one per `min_reinstall_interval` (a persistently rejecting handle
retries steadily, not once per poll) without a second timer.

**Alternatives considered**:
- Keep `poll` optimistic and revert on a failure acknowledgement. Rejected: it
  requires the manager to remember the prior program to revert to and to correlate
  the ack with the specific install, which the one-in-flight model avoids
  entirely.
- Carry the `FilterProgram` (or a generation counter) in the acknowledgement to
  correlate. Rejected: unnecessary once at most one install is in flight per
  handle; a plain `(handle, bool)` suffices and is smaller.
- Supersede a pending install when the wanted set changes mid-flight. Rejected:
  the two-second debounce makes mid-flight changes rare, and waiting for the ack
  before re-evaluating keeps the correlation trivially sound; convergence is
  delayed by at most one acknowledgement cycle.

## D-2: The acknowledgement is the reverse of the S13 forward channel

**Decision**: A single `std::sync::mpsc::channel::<(usize, bool)>` carries
acknowledgements from every capture thread to the control thread. Each capture
thread holds a clone of the sender and its own handle index; after `set_filter` it
sends `(handle_index, result.is_ok())`. The control thread drains the channel each
poll iteration (`try_recv`) and applies each acknowledgement before polling.

**Rationale**: This mirrors S13's decision D-c (the forward per-source
`mpsc<FilterProgram>` channel): only the owning thread touches the handle and the
result travels by message, so `PacketSource` gains no bound (P-3) and
`fragcap-core` takes no new dependency (P-2). One shared reverse channel tagged
with the handle index is simpler than one channel per handle and is sufficient
because the control thread is the single consumer.

**Alternatives considered**:
- One reverse channel per handle (symmetric with the forward channels). Rejected:
  the control thread would have to poll several receivers; a single tagged channel
  is simpler and the sender clones cost nothing.
- Return the result through the existing forward channel path. Rejected: the
  forward channel is control -> capture; the acknowledgement is the opposite
  direction and needs its own channel.

## D-3: Gap accounting measures against the acknowledged program

**Decision**: The gap-accounting loop in `poll` continues to read `handle.installed`,
which is now updated only on a success acknowledgement. So during an in-flight or
rejected install the gap set is computed against the program the handle actually
holds (the prior one), and the gap set is cleared only when the new program is
confirmed to admit the wanted endpoints.

**Rationale**: This keeps `filter_gaps` honest (P-4, P-9): a gap opened by an
endpoint the installed program excludes is not cleared by an install that was
rejected and never took effect. On the success path the behavior is identical to
S13 (the commit just moves from `poll` to `acknowledge`, one poll cycle later at
most).

## D-4: A rejecting handle is retried, not retired

**Decision**: A failure acknowledgement does not retire the handle. Retirement
stays reserved for a capture thread that has ended (the existing `retire`, which
also now clears any `pending`). A rejecting handle keeps its prior program and is
retried on a later poll, rate-limited.

**Rationale**: Section 12.3 makes correctness independent of filter freshness, so a
handle on a slightly stale program still captures correctly; retiring it would lose
its later traffic to spare a failed optimization. This is the S13 stance ("a
maintenance reinstall failure is non-fatal") made mechanical.

## D-5: Existing filter-manager tests acknowledge their installs

**Decision**: The existing `FilterManager` unit tests, which assumed `poll`
commits `installed` synchronously, are updated to call `acknowledge(handle, true)`
after each successful install. Their assertions are otherwise unchanged.

**Rationale**: The commit moving from `poll` to `acknowledge` is the whole point;
the tests must model the acknowledgement to observe the committed state. This is
honest test maintenance, not a weakening: each test now exercises the confirmed
install path it always meant to. The pipeline tests, whose sources accept every
filter, acknowledge success automatically through the wiring and need no change
beyond continuing to pass.

## Tier boundary

Everything is tier 1. The `FilterManager` is pure over core types; the pipeline
plumbing is exercised by the replay source and a rejecting source double. No
capture driver, no elevation. The live path is unchanged in shape (the capture
thread already called `set_filter`; it now also sends the result) and stays
compiled-only where it was.
