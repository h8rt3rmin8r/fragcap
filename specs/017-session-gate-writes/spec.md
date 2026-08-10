# Feature Specification: The Session Gates Sink Writes (Watch From Arm, Hard Bounds)

**Feature Branch**: `feat/session-gate-writes`

**Created**: 2026-08-10

**Status**: Draft

**Slice**: Follow-up to S14 (CLI command surface), resolving GitHub issue #22;
specification sections 10.5, 10.6, 12.4, 17.5; constitution P-4, P-9.

**Numbering note**: Spec directory `017-session-gate-writes`, a follow-up to S14,
not a roadmap slice. See `docs/plans/README.md`.

**Input**: Resolve the deferred half of the S14 review (findings C2 and C3,
deferred from PR #21 with a dated commit message). The CLI capture engine composes
`CaptureSession` and `Pipeline` so the session observes packets through a counting
tee sink but does not gate what the user sinks write. Two consequences follow.
First, the pipeline is started only after acquisition, so no packet reaches
`CaptureSession::on_packet` while `Watching`; on a live capture the handle is open
from arm, so pre-acquisition frames should be read, discarded, and counted, and the
two-phase driver does not do that. Second, the tee only observes: the pipeline
writes every captured packet to every sink and the driver requests a stop only
afterward, so `--max-packets` and `--max-bytes` do not bound the produced file, and
the completion summary can count a packet as discarded that is in the capture. Make
the session decision gate the user-sink writes synchronously and run the live packet
path from arm, so watch-time discards are counted and the bounds produce
exactly-bounded files whose accounting matches what is on disk.

## Overview

S14 built the CLI capture engine as two components composed side by side (its
decision D-c): the `Pipeline` owns the packet threads and the shared attributor,
and a session driver owns the `CaptureSession`. They connect through a `StopHandle`
and a published binding snapshot rather than by routing packets through the session.
The session's retained counters and the volume bound are fed by a `TeeCountingSink`
(decision D-e), first in the sink list, that forwards each captured packet's length
and instant to the driver over an unbounded channel. The tee observes; it does not
decide.

That composition is correct for what S14 built and wrong for two things it deferred.

**Watch-time discards are not modeled.** The offline driver is two-phase:
acquisition folds process events until a non-service stage matches, and only then
does the pipeline start. So offline, the target is acquired before any packet flows,
`watching_discarded` is always zero, and that is defensible because nothing was
observed and then discarded. A live capture is different: the capture handle is open
from arm, so frames arrive while the session is still `Watching`, and those frames
are read, must be discarded, and must be counted (constitution P-4). The live
streaming driver does not run the packet path from arm, so it never counts them.

**The bounds do not bound the produced file.** The tee is an observer downstream of
the sinks: the pipeline writes each captured packet to every sink, the tee forwards
a receipt to the driver, and only then does the driver observe `--max-packets` or
`--max-bytes` and request a stop. Packets the output thread wrote before the stop
propagated are already in the file, yet the session counts them as
`discarded_out_of_window`. The bound is soft, and worse, the completion summary
reports a discard for a packet that is in the capture. That is the
configuration-side form of the loss P-4 forbids and a P-9-adjacent fidelity problem:
the count and the file disagree.

This slice makes the session's decision gate the writes. A gating layer on the write
path admits a packet to the user sinks only while the session is capturing and the
bound has not been reached; it discards and counts everything else, synchronously, at
the moment of the write. The bound then bounds the file exactly, the summary matches
what is on disk, and the live path reads and counts watch-time frames from arm.

Three properties shape the slice.

**The gate is synchronous, so the file and the accounting cannot disagree.** The
soft bound exists because the write and the stop decision are on different threads
with a channel between them. The fix is to move the admit-or-discard decision onto
the write path itself, where it is made before the packet reaches any user sink. A
packet the gate discards is never written, and a packet the gate admits is counted
as retained; the produced file and the retained count are then the same set by
construction, which is what the completion summary reports.

**The offline goldens stay byte-identical.** Offline acquires before any packet
flows and the committed goldens are unbounded runs, so for them the gate is always
open and never at its bound: a pure pass-through. The goldens do not move. The
gate's observable effect offline is confined to bounded runs, which have no committed
golden and are asserted by produced-packet count instead.

**Core learns nothing about sessions.** The gate seam added to the pipeline is
generic: the output loop consults a `WriteGate` that answers admit-or-discard for a
packet, and the session-aware implementation lives in the facade where the session
already does (constitution P-3). The discard the gate makes is a named counter folded
into the pipeline's conservation identity (P-4), so nothing the gate withholds
escapes the accounting.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the architecture of record (sections 8.6, 10.5, 10.6,
12.4, 17.5), the S14 decisions (D-c, D-e), and the issue #22 proposed fix; no
operator escalation required.

- Q (G-a): Where does the gate live, given constitution P-3 keeps
  `fragcap-core` free of session and profile knowledge? -> A: The generic seam is a
  `WriteGate` trait in `fragcap-core`, `Send + Sync`, answering
  `admit(len, ts) -> bool` (or an equivalent decision) with interior mutability. The
  output loop consults it before fanning a packet out to the sinks. The
  session-aware implementation, `SessionGate`, lives in the facade `session` module
  beside `CaptureSession` and `RoleStampingAttributor`, both of which are already the
  bridge between the session and the pipeline. Core defines the seam and provides no
  implementation, exactly as it does for `FlowAttributor` and `Sink`.
- Q (G-b): How does a discard the gate makes stay inside the accounting P-4
  requires? -> A: The output loop counts each gate discard in a new capture-wide
  counter, `gate_dropped`, on `CaptureStats`. Because the gate sits before the
  per-sink fan-out, a gate discard is withheld from every sink uniformly, so the
  pipeline's conservation identity gains one term and stays exact: for every sink,
  `received + buffer_dropped + gate_dropped + refusals == packets_captured`. The
  counter is distinct from `buffer_dropped` (a slow sink) and `sink_dropped` (a sink
  that could not accept), because the cause and the remedy differ: a gate drop is a
  packet outside the capture window or beyond the bound, which is intended, not loss
  to be remedied.
- Q (G-c): The gate runs on the output thread and the session runs on the driver
  thread; how does the gate know whether the session is capturing without a per-packet
  cross-thread call? -> A: The `SessionGate` holds a lock-free published view of the
  window (an atomic state: closed while `Arming`, `Watching`, or `Draining`, open
  while `Capturing`) that the driver updates as the session transitions. The gate
  reads it without locking, the same lock-free-read discipline section 11.6 already
  requires of the attribution snapshot. The gate owns the bound counting itself, so
  the admit-or-discard decision for the bound is made where the write is, with no
  cross-thread hop.
- Q (G-d): Which authority fires `VolumeReached` and stops the run, now that the
  gate closes at the bound synchronously? -> A: The gate closes its own window the
  moment its bound is reached, so no further packet is written; that is what makes
  the bound hard. The session remains the single owner of the six stop conditions
  (section 10.6): the gate reports its outcome per packet to the driver over the same
  channel the tee used, tagged as admitted or discarded and why, and the driver folds
  each outcome into the session so `SessionStats` matches the file and the session
  fires `VolumeReached` on the admitted packet that reached the bound. The gate's
  synchronous close and the session's `VolumeReached` use the same bound, so they
  agree; the gate prevents the extra write, the session names the stop.
- Q (G-e): Does the offline driver also run from arm? -> A: No. Offline the whole
  event timeline is pre-collected and every packet is available at once; running the
  pipeline from arm would flow packets while `Watching` and discard them, changing
  the offline behavior and moving no golden toward correctness. The offline driver
  keeps its two-phase shape (acquire, then start the pipeline), where the gate is a
  pass-through for an unbounded run and a hard bound for a bounded one. Running from
  arm is a live-path property, because only there is the handle open before
  acquisition. The gate's watch-time discard counting is nonetheless tested at tier 1
  by driving the gate directly with a closed (watching) window, so the counting is
  covered without the live feature.
- Q (G-f): For `--max-bytes`, is the packet that crosses the byte bound written or
  discarded? -> A: Written. The session's existing rule fires `VolumeReached` when
  `retained_bytes >= byte_bound`, which includes the packet that reaches the bound,
  and the gate matches that rule so the two authorities never disagree: the gate
  admits the crossing packet, records its bytes, and then closes. For `--max-packets`
  the gate admits exactly `packet_bound` packets and closes, since the bound is a
  count. Both are the produced-file meaning of the session's pre-existing bound
  semantics, now made hard.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A packet bound produces an exactly-bounded file (Priority: P1)

When an operator sets `--max-packets N`, the produced capture contains exactly N
packets, the completion summary reports N retained equal to the packets on disk, no
packet is both written and counted as discarded, and the stop reason is
`volume-reached`. Packets the source delivered beyond the bound before the stop
propagated are counted as discarded (out of window) and are not in the file, which is
the hard bound working rather than a leak.

**Why this priority**: This is the fidelity defect the slice exists to close. Today
the bound is soft: the file can contain more than N packets while the summary counts
the overflow as `discarded_out_of_window`, so the count and the file disagree, which
is the P-9-adjacent problem. Making the bound hard and the accounting honest is the
core of the slice.

**Independent Test**: Run the offline substrate with `--max-packets N` for an N well
below the fixture's packet count; read the produced pcapng and JSON Lines and assert
each contains exactly N packet records, the summary's retained count equals N and
equals the packets on disk, no packet in the file is counted as discarded, and the
stop reason is `volume-reached`. No capture driver.

**Acceptance Scenarios**:

1. **Given** an offline run over a fixture of more than N packets, **When**
   `--max-packets N` is set, **Then** the produced pcapng and the produced JSON Lines
   each contain exactly N packet records.
2. **Given** the same run, **When** it completes, **Then** the summary's retained
   count is N and equals the packets on disk, and the stop reason is `volume-reached`;
   packets captured beyond the bound are counted as discarded (out of window) and are
   not in the file.
3. **Given** a byte bound `--max-bytes B`, **When** the run completes, **Then** the
   produced file contains exactly the packets whose cumulative captured length first
   reaches or exceeds B and no more, and the summary's retained byte count equals the
   bytes on disk.

---

### User Story 2 - Watch-time frames are read, discarded, and counted (Priority: P1)

While the session is watching for a target and no stage has matched, frames the
capture handle delivers are read, discarded (nothing is written), and counted in
`watching_discarded`. Nothing observed is silently dropped (constitution P-4).

**Why this priority**: On a live capture the handle is open from arm, so the most
information-dense pre-acquisition traffic (section 5.2) arrives while watching and
must be accounted for. The current live driver does not run the packet path from arm,
so these frames are neither read nor counted, which is exactly the unobserved loss
P-4 forbids.

**Independent Test**: Drive the `SessionGate` directly with its window closed in the
watching state and feed it packets; assert none is admitted and each is counted as a
watch-time discard. This covers the gate's counting without the live feature. The
end-to-end run-from-arm wiring is exercised on the live path, which is compiled and
linked in CI but not executed there (tier 2).

**Acceptance Scenarios**:

1. **Given** a gate whose window is closed because the session is watching, **When**
   a packet is offered, **Then** the gate does not admit it and counts it as a
   watch-time discard.
2. **Given** the live driver, **When** the pipeline runs from arm and frames arrive
   before a stage matches, **Then** those frames are read and counted as
   `watching_discarded` rather than never observed.
3. **Given** a run that acquires a target, **When** the window opens, **Then** the
   gate admits subsequent packets and the watch-time count stops advancing.

---

### User Story 3 - The gate discard is inside the conservation identity (Priority: P1)

Every packet the gate withholds from the sinks is counted in a named counter, and the
pipeline's conservation identity holds with that counter as a term, so nothing the
gate discards escapes the accounting and the summary matches what is on disk.

**Why this priority**: The gate introduces a new discard path, and constitution P-4
makes an uncounted discard a defect. Folding `gate_dropped` into the conservation
identity is what proves, mechanically, that a packet is written, buffer-dropped,
sink-dropped, or gate-dropped and never lost silently.

**Independent Test**: In the pipeline tests, attach a gate that admits a scripted
subset and assert, for every sink, that `received + buffer_dropped + gate_dropped +
refusals == packets_captured`. Assert a run with no gate is unchanged (the identity
reduces to its prior three-term form).

**Acceptance Scenarios**:

1. **Given** a pipeline with a gate that discards some packets, **When** the run
   completes, **Then** for every sink the received count plus the three drop counters
   equals the captured count.
2. **Given** a pipeline with no gate attached, **When** the run completes, **Then**
   `gate_dropped` is zero and the identity is exactly the prior one.
3. **Given** the completion summary, **When** a gate discarded packets, **Then** the
   summary surfaces the gate discards distinctly from buffer and sink drops.

---

### User Story 4 - The offline goldens do not move (Priority: P1)

An unbounded offline run produces byte-identical pcapng and JSON Lines to what S14
produced, because the gate is open and unbounded for it and therefore a pass-through.

**Why this priority**: The slice reworks the output path, and the committed goldens
are the contract that unmodified analyzers still read fragcap output. A change that
moved them would be a compatibility regression, so their invariance is the guardrail
the rework is measured against.

**Independent Test**: Run the offline substrate with no bound and the standard sinks;
assert the produced pcapng and JSON Lines match the committed goldens byte for byte.
Run the corpus pipeline test and confirm every golden is unchanged.

**Acceptance Scenarios**:

1. **Given** an unbounded offline run, **When** it completes, **Then** the produced
   pcapng and JSON Lines are byte-identical to the committed goldens.
2. **Given** the corpus pipeline, **When** it runs through both writers, **Then**
   every committed golden is reproduced.

---

### Edge Cases

- A packet offered while the window is closed because the session is draining or has
  not yet armed is discarded and counted as out of window, distinct from a watch-time
  discard, so the two pre-existing session counters keep their meaning.
- The packet that reaches a `--max-packets` bound is the last one admitted; the next
  is the first discard, so the file contains exactly the bound.
- The packet that first reaches or crosses a `--max-bytes` bound is admitted (the
  session's `retained_bytes >= byte_bound` rule includes it), then the window closes;
  this keeps the gate and the session in agreement.
- An unbounded run never closes the window on a bound; the gate closes only when the
  session leaves the capturing state, which the driver publishes.
- A gate that is never attached (a caller that wants no gating) leaves the output
  loop's behavior and the conservation identity exactly as before, with `gate_dropped`
  zero.
- A zero bound (`--max-packets 0`) admits nothing and stops immediately with
  `volume-reached`, producing an empty but well-formed capture; this is the operator's
  explicit request, not a defect.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `fragcap-core` MUST define a generic `WriteGate` seam
  (`Send + Sync`, interior mutability) that answers whether a captured packet is
  admitted to the sinks, and the pipeline's output loop MUST consult it before
  fanning a packet out to any sink. `fragcap-core` MUST gain no session or profile
  knowledge and no new dependency (P-2, P-3).
- **FR-002**: A packet the gate does not admit MUST NOT be written to any sink and
  MUST be counted in a new capture-wide `gate_dropped` counter on `CaptureStats`,
  distinct from `buffer_dropped` and `sink_dropped`.
- **FR-003**: The pipeline's conservation identity MUST hold with the new counter as
  a term: for every sink, `received + buffer_dropped + gate_dropped + refusals ==
  packets_captured`. The existing pipeline tests' conservation check MUST be extended
  to include it, so a later discard path without a counter fails there.
- **FR-004**: A run with no gate attached MUST behave exactly as before, with
  `gate_dropped` zero and the conservation identity reduced to its prior three-term
  form; no existing pipeline caller or golden may change for this reason.
- **FR-005**: The facade MUST provide a `SessionGate` implementing `WriteGate` that
  admits a packet only while its published window is open (the session is capturing)
  and the configured bound has not been reached, and discards and counts every other
  packet, distinguishing watch-time discards from out-of-window discards.
- **FR-006**: With `--max-packets N` set, the produced capture MUST contain exactly
  N packet records, and with `--max-bytes B` set, the produced capture MUST contain
  exactly the packets whose cumulative captured length first reaches or exceeds B and
  no more. The stop reason MUST be `volume-reached`.
- **FR-007**: The completion summary MUST match the produced file: a packet written
  to the capture MUST be counted as retained, and no packet in the capture may be
  counted as discarded. `SessionStats.retained`, `retained_bytes`,
  `watching_discarded`, and `discarded_out_of_window` MUST reflect the gate's actual
  decisions.
- **FR-008**: The live driver MUST run the packet path from arm, so frames delivered
  while the session is watching are read, discarded by the gate, and counted in
  `watching_discarded`. The offline driver MUST keep its two-phase shape (acquire,
  then start the pipeline), so its behavior and goldens are unchanged (G-e).
- **FR-009**: The session MUST remain the single owner of the six stop conditions
  (section 10.6). The gate MUST close its window synchronously at its bound so no
  further packet is written, and the session MUST fire `VolumeReached` on the same
  bound, so the two never disagree.
- **FR-010**: The gate machinery MUST be testable at tier 1 with no capture driver,
  no elevation, and no game: the bound behavior through the offline substrate by
  produced-packet count, the watch-time discard counting by driving the gate directly,
  and the conservation identity in the pipeline tests.
- **FR-011**: An unbounded offline run's produced pcapng and JSON Lines MUST be
  byte-identical to the committed goldens, and the corpus goldens MUST be unchanged.

### Key Entities

- **Write gate**: the generic `fragcap-core` seam the output loop consults to decide
  whether a captured packet reaches the sinks. Carries no session knowledge.
- **Session gate**: the facade implementation of the write gate, admitting a packet
  only while the published window is open and the bound is not reached, and counting
  every discard by cause.
- **Capture window**: the lock-free published state (open while capturing, closed
  otherwise) the driver updates as the session transitions and the gate reads without
  locking.
- **Gate outcome**: the per-packet result (admitted, or discarded and why) the gate
  reports to the driver so the session's counters and stop reason match the file.
- **`gate_dropped`**: the new capture-wide counter for packets the gate withheld from
  the sinks, a term in the conservation identity.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `--max-packets N` produces a file with exactly N packet records in both
  writers, verified by counting records in the produced pcapng and JSON Lines over the
  offline substrate.
- **SC-002**: The completion summary of a bounded run reports the retained count equal
  to the packets on disk, and no packet is both written to the file and counted as
  discarded, verified over the offline substrate.
- **SC-003**: A watch-time discard is counted, verified by driving the `SessionGate`
  directly with a closed watching window and asserting the count advances and nothing
  is admitted.
- **SC-004**: The pipeline conservation identity holds with `gate_dropped` as a term,
  verified in the pipeline tests for a gate that discards a scripted subset, and a
  no-gate run leaves the identity in its prior form.
- **SC-005**: The unbounded offline goldens and the corpus goldens are byte-identical.
- **SC-006**: `cargo xtask ci` passes, and `cargo xtask neutral` and `msrv` exit 0.

## Assumptions

- The `TeeCountingSink` and the driver-side channel S14 built (decision D-e) are the
  seams reworked here: the tee becomes the gate, and the channel carries a per-packet
  gate outcome rather than a bare receipt.
- The offline replay path acquires before any packet flows and its committed goldens
  are unbounded runs, so the gate is a pass-through for them and the goldens do not
  move.
- The live path is compiled and linked in CI but not executed there (tier 2), so the
  run-from-arm wiring is verified by compilation and by the gate's tier-1 unit tests
  rather than by an executed live capture.
- The session's pre-existing bound semantics (`retained >= packet_bound`,
  `retained_bytes >= byte_bound`) are the produced-file meaning the gate makes hard;
  the slice makes them exact rather than changing them.
- `gate_dropped` is a discard of an intended kind (outside the window or beyond the
  bound), counted for honesty rather than remedied, and it is distinct from the two
  loss counters an operator acts on.
