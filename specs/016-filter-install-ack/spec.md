# Feature Specification: Filter Manager Install Acknowledgement

**Feature Branch**: `feat/filter-install-ack`

**Created**: 2026-08-10

**Status**: Draft

**Slice**: Follow-up to S13 (filter management), resolving GitHub issue #20;
specification sections 12.2 and 12.3; constitution P-4, P-9.

**Numbering note**: Spec directory `016-filter-install-ack`, a follow-up to S13,
not a roadmap slice. See `docs/plans/README.md`.

**Input**: Resolve the half of the S13 review finding P2 that was deferred (the
retired-handle half was fixed in PR #17). `FilterManager::poll` marks a handle's
desired endpoint set as installed and advances its per-handle timing and gap
accounting before the capture thread has actually applied it. The control thread
sends the program over an mpsc channel and the capture thread calls
`PacketSource::set_filter`, but no success or failure acknowledgement flows back.
If a maintenance `set_filter` is rejected by the backend, the real handle keeps
its prior program while the manager treats the new one as current; because the
manager sees installed equal to wanted, it never retries, and the handle may keep
excluding wanted traffic. Add a per-handle acknowledgement so the manager commits
installed state and gap accounting only on a confirmed install and retries on a
rejection, keeping a rejecting handle on its prior program (capture continuity)
rather than retiring it.

## Overview

S13 built the phase-two and phase-three filter lifecycle: the control thread reads
the wanted endpoint set, `FilterManager::poll` decides what to install under a
debounce and a per-handle rate limit, and each capture thread installs the program
on its own handle (the only thread that may touch it). The manager commits the
install optimistically inside `poll`: it sets the handle's `installed` program,
advances `last_install`, and clears the handle's gap set at the moment it decides
to install, before the capture thread has confirmed the backend accepted the
program.

The generated program is valid by construction, so a rejection cannot occur in
normal operation, which is why this is assessed low urgency and defensive. But the
optimistic commit means the manager's model can silently diverge from the real
handle: a rejected maintenance install leaves the old program on the handle while
the manager believes the new one is current, so it never retries and the handle
keeps excluding endpoints the operator wanted, with the divergence recorded
nowhere.

This slice closes the loop. The capture thread reports whether `set_filter`
succeeded back to the control thread over a per-handle channel, and the manager
commits the installed program, the timing, and the gap-set clear only when the
install is confirmed. A rejected install leaves the handle on its prior program
and is retried on a later poll rather than treated as done.

Two properties shape the slice.

**The manager's model must match the handle, or the divergence must be visible.**
An optimistic commit that a rejection silently invalidates is exactly the kind of
loss constitution P-4 forbids: the manager reports a narrowed filter it does not
actually have. Committing on acknowledgement keeps the model honest, and the gap
accounting, which is a set difference against `installed`, then measures against
the program the handle actually holds.

**A rejecting handle keeps capturing.** Correctness never depends on the kernel
filter being fresh (section 12.3): userspace attribution runs on every packet
regardless. So a handle whose maintenance install is rejected keeps its prior
program and keeps capturing; it is retried, not retired. Retiring it to spare a
failed optimization would lose all its later traffic, the worse outcome, and the
existing retire path is for a handle whose capture thread has ended, a different
condition.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the architecture of record (sections 8.6, 11.6,
12.2, 12.3) and the S13 decisions; no operator escalation required.

- Q (F-a): How does the acknowledgement travel from the capture thread to the
  control thread, given the handle is touched only by its owning thread? -> A: A
  reverse `std::sync::mpsc` channel carrying `(handle_index, installed_ok)`,
  mirroring the forward per-source `mpsc<FilterProgram>` channel S13 chose (its
  decision D-c). The capture thread, after calling `set_filter`, sends the result
  tagged with its handle index; the control thread drains the channel each poll
  and applies each acknowledgement to the manager. No new dependency, and
  `PacketSource` gains no bound (P-3): the ack channel is between threads, not on
  the trait.
- Q (F-b): How is an acknowledgement correlated with the install it answers, when
  the message carries no program? -> A: The manager issues at most one install per
  handle at a time. While a handle has an install in flight (a `pending` program
  not yet acknowledged) `poll` issues no new install for it, so a bare
  `(handle, ok)` acknowledgement unambiguously refers to the single in-flight
  program. A wanted-set change during an in-flight install is absorbed on the next
  poll after the acknowledgement, which the two-second debounce makes rare.
- Q (F-c): What does the manager commit on success versus failure? -> A: On a
  success acknowledgement it commits the pending program as `installed` and clears
  the handle's gap set (the new program admits every wanted endpoint). On a failure
  acknowledgement it drops the pending program and leaves `installed`,
  `last_install`, and the gap set unchanged, so the handle keeps its prior program
  and the next poll retries. The per-handle rate limit (`last_install`, set when
  the install is issued) spaces the retries at one per `min_reinstall_interval`,
  so a persistently rejecting handle retries steadily rather than in a tight loop.
- Q (F-d): Should a repeatedly-rejecting handle eventually be retired? -> A: No.
  Retirement is for a handle whose capture thread has ended and can install no
  more; a rejecting handle is still capturing on its prior program, and retiring
  it would advance no drop counter but lose its subsequent traffic. It is kept and
  retried, which is the S13 stance ("a maintenance reinstall failure is
  non-fatal") made mechanical rather than assumed.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A rejected maintenance install is not treated as installed (Priority: P1)

When a maintenance `set_filter` is rejected by the backend, the capture thread
reports the rejection, and the filter manager does not record the rejected program
as installed. The handle keeps its prior program, the manager retries the install
on a later poll, and its model of what is installed matches the program the handle
actually holds.

**Why this priority**: This is the divergence the slice exists to close. Without
it, a rejected install leaves the manager reporting a narrowed filter it does not
have, so it never retries and the handle keeps excluding wanted traffic, with the
loss recorded nowhere. It is the whole of the slice.

**Independent Test**: Drive the filter manager with a rejection acknowledgement for
an install it issued; assert the handle is not considered installed (a later poll
re-issues the same program) and that the prior program is what the manager still
models as installed. No capture driver.

**Acceptance Scenarios**:

1. **Given** a handle the manager has issued a maintenance install for, **When**
   the acknowledgement reports the `set_filter` was rejected, **Then** the manager
   does not record the rejected program as installed and keeps the prior program.
2. **Given** a rejected install, **When** the manager next polls with the same
   wanted set (after the rate-limit interval), **Then** it re-issues the install
   rather than treating it as already done.
3. **Given** a handle whose maintenance install is rejected, **When** the run
   continues, **Then** the handle is not retired and keeps capturing on its prior
   program.

---

### User Story 2 - A confirmed install commits state exactly once (Priority: P1)

When a `set_filter` succeeds, the capture thread confirms it, and the manager
commits the installed program, advances the per-handle timing, and clears the
handle's gap set. A handle already narrowed to the wanted set is not reinstalled,
and the gap accounting measures against the program the handle actually holds.

**Why this priority**: The acknowledgement must not change the observable
narrowing behavior on the ordinary success path: the same programs are installed,
the debounce and rate limit still hold, and the gap counting is unchanged. This is
what keeps the whole S13 behavior intact for the normal case while the P1 fix
protects the rejection case.

**Independent Test**: Drive the manager through an install and a success
acknowledgement; assert the handle is now considered installed (a later poll with
the same set installs nothing), the rate limit is measured from the install, and
the gap set is clear. Assert every existing filter-manager test still holds with
the acknowledgement applied.

**Acceptance Scenarios**:

1. **Given** an issued install, **When** the acknowledgement reports success,
   **Then** the manager records the program as installed and clears the handle's
   gap set.
2. **Given** a confirmed install, **When** the manager polls again with the same
   wanted set, **Then** it installs nothing (idempotence preserved).
3. **Given** a confirmed install, **When** the wanted set later changes, **Then**
   the reinstall is still governed by the debounce and the per-handle rate limit.

---

### User Story 3 - The acknowledgement flows through the pipeline (Priority: P1)

The capture thread, which is the only thread that may touch its handle, reports the
result of `set_filter` to the control thread over a per-handle channel, and the
control thread applies it to the manager. On the offline replay path, where the
source accepts every filter, this is a confirmed install every time and nothing
observable changes.

**Why this priority**: The manager's acknowledgement is only correct if it is
actually wired from the thread that installs to the thread that decides. Without
the pipeline plumbing the manager has no acknowledgement to act on, and the fix is
inert.

**Independent Test**: Run the pipeline with a source double that rejects
maintenance `set_filter` calls; assert the control thread retries the install
(the source records more than one attempt of the same program) and the run
completes on its own. Run the corpus through the pipeline and confirm the goldens
are unchanged.

**Acceptance Scenarios**:

1. **Given** a live pipeline, **When** a capture thread installs a filter, **Then**
   it reports the result to the control thread, which acknowledges it to the
   manager.
2. **Given** a source that rejects a maintenance install, **When** the pipeline
   runs, **Then** the control thread re-issues the install rather than considering
   it done, and the run still ends cleanly.
3. **Given** the offline corpus, **When** it runs through the pipeline, **Then**
   every install is a confirmed success and the goldens are byte-identical.

---

### Edge Cases

- A handle with an install in flight (a pending, unacknowledged program) is issued
  no new install until it is acknowledged, so a bare acknowledgement is
  unambiguous.
- An acknowledgement for a handle that has no pending install (a stale or
  duplicate ack) is ignored.
- A capture thread that ends after receiving an install but before acknowledging it
  leaves the manager's pending state for that handle unresolved; the run ends on
  its own (the control thread exits on `control_stop`), so no hang results, and the
  handle's last confirmed program is what the gap accounting used.
- A retired handle (its capture thread ended) drops any pending install and
  acknowledges nothing further, unchanged from S13 except that pending is cleared.
- The rate limit spaces retries: a persistently rejecting handle retries at one
  attempt per `min_reinstall_interval`, not once per poll.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The capture thread MUST report the result of each `set_filter` call
  (success or failure) to the control thread over a per-handle channel, tagged with
  its handle index. `PacketSource` MUST gain no new trait bound (P-3).
- **FR-002**: `FilterManager::poll` MUST NOT commit a handle's `installed` program
  or clear its gap set until the install is acknowledged; it MUST mark the handle
  as having a pending install and issue no further install for that handle until
  the acknowledgement arrives (one install in flight per handle).
- **FR-003**: The manager MUST expose an acknowledgement operation that, on
  success, commits the pending program as installed and clears the handle's gap
  set, and, on failure, drops the pending program and leaves the installed program,
  the per-handle timing, and the gap set unchanged.
- **FR-004**: A rejected maintenance install MUST NOT be treated as installed; the
  handle MUST keep its prior program and the install MUST be retried on a later
  poll, subject to the per-handle rate limit.
- **FR-005**: A rejecting handle MUST NOT be retired; retirement remains reserved
  for a handle whose capture thread has ended.
- **FR-006**: The per-handle rate limit MUST space retries at one attempt per
  `min_reinstall_interval`, so a persistently rejecting handle does not retry once
  per poll.
- **FR-007**: The gap accounting MUST measure against the program the handle
  actually holds (the acknowledged `installed`), so a gap is not cleared by an
  install that was rejected. The `filter_gaps` counter keeps its meaning and stays
  outside the pipeline conservation identity (P-4, P-9); no fabricated packet count
  is introduced.
- **FR-008**: The acknowledgement machinery MUST be pure over core types and
  testable at tier 1 against a source double, with no capture driver, no elevation,
  and no game. `fragcap-core` MUST take no new dependency and no platform dependency
  (P-2).
- **FR-009**: On the offline replay path, where every `set_filter` succeeds, the
  observable narrowing behavior and the goldens MUST be unchanged.

### Key Entities

- **Install acknowledgement**: a `(handle_index, installed_ok)` message from the
  capture thread to the control thread reporting the result of `set_filter`.
- **Pending install**: a program the manager has issued to a handle and not yet
  seen acknowledged; while set, no new install is issued to that handle.
- **Acknowledgement channel**: the reverse `std::sync::mpsc` channel carrying
  acknowledgements, mirroring the forward per-source filter-program channel.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A rejection acknowledgement leaves the handle not considered
  installed, verified by a manager test in which a later poll re-issues the same
  program.
- **SC-002**: A success acknowledgement commits the install and preserves
  idempotence, the debounce, the rate limit, and the gap accounting, verified by
  the existing filter-manager tests updated to acknowledge and by a new success
  test.
- **SC-003**: The pipeline retries a rejected maintenance install, verified at
  tier 1 with a source double that records more than one attempt of the same
  program and a run that ends on its own.
- **SC-004**: The offline corpus goldens are byte-identical.
- **SC-005**: `cargo xtask ci` passes, and `cargo xtask neutral` and `msrv` exit 0.

## Assumptions

- The forward per-source `mpsc<FilterProgram>` channel (S13 decision D-c) and the
  control-thread poll loop are the seams extended here; the acknowledgement is the
  reverse of that channel.
- The generated filter program is valid by construction, so a rejection is a
  defensive path that does not occur in normal operation; the slice makes the
  divergence impossible rather than fixing an observed failure.
- The offline replay source accepts every filter, so the offline path exercises
  only the success acknowledgement and its goldens do not move.
- `filter_gaps` keeps its S13 meaning (gap occurrences, not packets) and its place
  outside the conservation identity.
