# Feature Specification: Stage Matching and Session Lifecycle

**Feature Branch**: `feat/stage-matching-lifecycle`

**Created**: 2026-08-10

**Status**: Draft

**Slice**: S12 (specification sections 10.3, 10.4, 10.5, and 10.6; constitution
P-2, P-3, P-4, P-6, and P-9)

**Input**: Implement specification sections 10.3 (stage matching), 10.4
(lifecycle classes), 10.5 (session lifecycle), and 10.6 (stop conditions).
Evaluate each process start event against the active profile's stages, bind the
matching process-tree node to that stage and assign its role, and drive the
five-state capture session (Arming, Watching, Capturing, Draining, Complete)
with its stop conditions. Stage matching is a pure decision over the synthetic
process tree and the profile, testable offline against the scripted watcher with
no capture driver, no elevation, and no game. Packets discarded before a target
is acquired are counted and surfaced. Capture acquisition, the process tree, and
the profile schema are S09, S11, and S05 respectively and are consumed, not
rebuilt.

## Overview

Eleven slices have built a capture tool that watches processes (S11), attributes
flows to them (S10), and runs a buffered pipeline (S08), but nothing yet decides
which processes matter. A profile declares stages and roles; the process watcher
emits a stream of starts and exits; the tree folds that stream into ancestry. S12
is the join between them: it reads the profile's stages against the tree and
decides which node is the launcher, which is the client, and which is background
platform noise, then it drives a capture session that arms before the target
exists, keeps nothing until a target is matched, and stops cleanly on any of six
conditions.

Two properties shape the slice.

**Matching is a pure decision over a value.** The process tree is a fold that
opens nothing (S11 established this). Stage matching reads that tree and the
profile and produces bindings; it too opens nothing and touches no platform
interface, so the whole of section 10.3 is tested against the scripted watcher on
any machine. A slice that had to run a game to test `descends_from` would be a
slice that could not be tested, which is why the predicate resolves over the
synthetic tree rather than the operating system parent chain.

**Missing the acquisition boundary is invisible.** A session that arms too late
misses the launcher authentication exchange, and a session that discards a packet
without counting it reports a clean capture that is silently short. The lifecycle
therefore opens the capture handle in `Arming`, before any target exists, and
every packet dropped in `Watching` is counted in a named counter and surfaced,
exactly as P-4 requires of every discard path.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the architecture of record (sections 10.2 through
10.6, 8.3) and the constitution; no operator escalation was required.

- Q: How does matching handle `descends_from` when the named ancestor role is
  not yet bound at the descendant's start event? → A: Matching is evaluated once
  per start event against the current bindings. Correctness relies on the causal
  creation order S11 guarantees (a parent's start event precedes its child's), so
  an ancestor stage that matches binds before its descendant is evaluated. No
  deferred re-evaluation queue is introduced.
- Q: Where is a stage binding stored? → A: On the process node in `fragcap-core`.
  The core process tree gains a binding method that sets the node's stage (the
  field S11 reserved for this slice); the `fragcap-profile` matcher decides the
  binding by evaluating predicates and applies it through that method. Predicate
  evaluation stays in `fragcap-profile`, the mutation stays in `fragcap-core`,
  and the only new edge is profile depending on core, which already holds.
- Q: Is the acquisition timeout mandatory? → A: It is an optional bound on the
  capture configuration. When set, Watching transitions to Complete on expiry;
  when unset, the session waits for a target and ends by another stop condition
  (the duration bound or an operator interrupt). Tests for the timeout transition
  set the bound explicitly.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Bind processes to roles by predicate (Priority: P1)

A profile author declares stages, each naming a role and a set of match
predicates. As processes start, fragcap evaluates each start against every stage
and binds the process to the first stage all of whose specified predicates hold,
assigning that stage's role. The hard case is a chain where several processes
share one image name and only the last holds sockets: matching on the image name
alone binds the wrong one, so `descends_from` resolves a role to an
already-bound ancestor and tests ancestry over the synthetic tree.

**Why this priority**: Without stage matching nothing in a capture carries a
role, and every downstream feature (filtering by role, terminal-stage stop,
managed launch) depends on the bindings this story produces. It is the core of
the slice.

**Independent Test**: Drive the scripted watcher with a start-event stream and a
profile, and assert the resulting bindings, including the three-process
shared-image chain where ancestry finds the client and the image name alone finds
the shim. No capture driver, elevation, or game.

**Acceptance Scenarios**:

1. **Given** a stage whose only predicate is `exe: eso64.exe`, **When** a process
   with image name `ESO64.exe` starts, **Then** it binds to that stage and takes
   its role (case-insensitive).
2. **Given** a stage with `exe` and `cmdline_contains`, **When** a process
   matches the name but not the command-line substring, **Then** it does not
   bind, because all specified predicates must hold.
3. **Given** a process whose command line was not observed, **When** a stage
   declares `cmdline_contains`, **Then** the stage does not match, because an
   unobserved command line cannot contain a substring.
4. **Given** a client stage with `descends_from: launcher` and a chain of three
   processes sharing one image name below a bound launcher, **When** the chain
   starts, **Then** the stage binds the descendant of the launcher, not the first
   process to carry the image name.
5. **Given** a `path_regex` predicate, **When** a process starts, **Then** the
   stage reuses the profile's pre-compiled expression rather than recompiling it.

---

### User Story 2 - Arm before the target and lose no traffic at acquisition (Priority: P1)

A capture session opens its capture handle and attaches the watcher before any
target process exists (`Arming`), then holds an armed capture discarding packets
while no stage has matched (`Watching`), then begins retaining packets the instant
the first stage matches (`Capturing`). Because the handle is already open, the
Watching-to-Capturing transition costs no setup and loses no traffic at the
boundary. Packets discarded during Watching are counted and surfaced.

**Why this priority**: The launcher authentication exchange is frequently the
most information-dense traffic of the session and happens before the client
exists. A session that arms after the target appears has already missed it. This
story is what makes the capture worth having.

**Independent Test**: Drive a scripted packet stream and a scripted watcher
through the session; assert that packets before the first match advance the
watching-discard counter, that the first match transitions to Capturing, and that
no packet at the boundary is lost or double-counted.

**Acceptance Scenarios**:

1. **Given** a session in Arming, **When** the watcher is attached and the capture
   handle is open, **Then** it transitions to Watching before any process matches.
2. **Given** a session in Watching, **When** packets arrive before any stage
   matches, **Then** they are discarded and counted in the watching-discard
   counter.
3. **Given** a session in Watching, **When** the first stage matches, **Then** it
   transitions to Capturing and retains that and every subsequent packet.
4. **Given** a session in Watching, **When** the acquisition timeout elapses with
   no match, **Then** it transitions to Complete and reports that no target was
   acquired.

---

### User Story 3 - Stop cleanly on any condition, always a valid file (Priority: P2)

Capture ends on the first of six conditions: the duration bound, the byte or
packet bound, the terminal stage exiting, all matched processes having exited with
no stage still awaited, an operator interrupt, or an unrecoverable sink error.
Every one produces the same orderly shutdown: capture halts, the buffer drains,
sinks flush and finish, and statistics are reported. An interrupt is a normal
stop, not an abort.

**Why this priority**: A capture tool that ends differently depending on why it
ended produces files that sometimes cannot be trusted. Uniform shutdown is what
lets the operator read any capture the same way, including one they interrupted.

**Independent Test**: Drive the session to each stop condition in turn and assert
that all six reach Draining then Complete, flush the sinks, and report a
conservation-closed statistics record.

**Acceptance Scenarios**:

1. **Given** a session in Capturing, **When** the configured duration bound is
   reached, **Then** it drains and completes with a valid file.
2. **Given** a terminal stage, **When** its bound process exits, **Then** capture
   stops.
3. **Given** all matched processes exited and no stage still awaited, **When** the
   last exit is observed, **Then** capture stops.
4. **Given** a session in Capturing, **When** an operator interrupt arrives,
   **Then** it drains and completes normally, not as an abort.
5. **Given** a `service` stage, **When** target acquisition is under way, **Then**
   the service process is never awaited (waiting on something already running
   would deadlock).

---

### Edge Cases

- A process that matches more than one stage binds to the first stage in profile
  declaration order, deterministically. (Profile validation in S05 makes role
  ambiguity within a chain a validation error; this rule makes the residual case
  total rather than order-dependent.)
- A `descends_from` predicate is evaluated once, on the descendant's start event,
  against the bindings that exist at that instant. Causal creation order (a
  parent's start precedes its child's, which S11 guarantees) means the ancestor
  is already bound, so no deferred re-evaluation is needed; a descendant that
  genuinely has no bound ancestor at its start does not match.
- An exit for a process never bound (background noise) advances no binding and no
  stop condition.
- A terminal stage that never binds cannot fire its stop condition; the session
  ends by another condition (timeout, all-exited, duration, interrupt).
- The Watching-to-Capturing transition must not drop or duplicate the packet that
  coincides with the first match.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST evaluate each process start event against every
  stage declared in the active profile, and a stage MUST match only when all of
  its specified predicates hold.
- **FR-002**: The system MUST support five match predicates with these semantics:
  `exe` (glob against the executable file name, case-insensitive);
  `path_contains` (case-insensitive substring of the full image path);
  `path_regex` (regular expression against the full image path, reusing the
  profile's pre-compiled expression); `cmdline_contains` (substring of the
  command line, never matching when the command line was not observed);
  `descends_from` (ancestry, over the synthetic tree, below a node already bound
  to the named role).
- **FR-003**: A match MUST bind the process node to the stage and assign the
  stage's role to that node.
- **FR-004**: `descends_from` MUST resolve over the synthetic process tree, never
  the operating system parent chain.
- **FR-005**: When more than one stage matches a single process, the system MUST
  bind the first stage in profile declaration order, deterministically.
- **FR-006**: Lifecycle classes MUST govern exit handling: a `transient` exit is
  normal and does not affect capture; a `session` exit is significant and, when
  the stage is terminal, ends capture; a `service` process MUST never be awaited
  during target acquisition.
- **FR-007**: A capture session MUST move through Arming, Watching, Capturing,
  Draining, and Complete, with an additional Watching-to-Complete transition on
  acquisition timeout.
- **FR-008**: Arming MUST open the capture handle and attach the process watcher
  before any target process exists.
- **FR-009**: Watching MUST discard packets, and every discarded packet MUST be
  counted in a named counter and surfaced in statistics (P-4).
- **FR-010**: The Watching-to-Capturing transition MUST occur on the first stage
  match, MUST retain that and every subsequent packet, and MUST lose no packet at
  the boundary.
- **FR-011**: Acquisition timeout MUST transition Watching to Complete and report
  that no target was acquired.
- **FR-012**: Capture MUST end on the first of these to occur: the elapsed
  duration bound, the byte or packet bound, the terminal stage exiting, all
  matched processes having exited with no stage still awaited, an operator
  interrupt, or an unrecoverable sink error.
- **FR-013**: Every stop condition MUST produce the same orderly shutdown: halt
  capture, drain the buffer, flush and finish the sinks, and report statistics;
  the result MUST be a complete and valid capture, including on interrupt.
- **FR-014**: Stage matching and the session lifecycle MUST be testable at tier 1
  against the scripted watcher with no capture driver, no elevation, and no game.
- **FR-015**: Every term this slice introduces MUST have a glossary entry in the
  same change (P-6).
- **FR-016**: Stage matching MUST live where it can read both the profile and the
  process tree without creating a dependency edge between the capture and
  attribution siblings; the session lifecycle MUST live where it can see the
  watcher, the pipeline, the profile, and the sinks together (P-2, P-3).

### Key Entities

- **Stage binding**: the association of a process-tree node with a stage and the
  role that stage assigns.
- **Stage matcher**: the pure decision that evaluates a stage's predicates against
  a process node and the tree.
- **Capture session**: the state machine that arms, watches, captures, drains, and
  completes.
- **Session state**: one of Arming, Watching, Capturing, Draining, Complete.
- **Stop condition**: one of the six triggers that ends capture.
- **Acquisition timeout**: the bound after which Watching gives up with no target
  acquired.
- **Watching-discard counter**: the named counter for packets discarded before a
  target is acquired.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every match predicate resolves correctly, including the
  three-process shared-image chain, verified by a test per predicate and a test
  that binds the client by ancestry where the image name alone would bind the
  shim.
- **SC-002**: A capture armed before the target starts loses no packet between the
  target's first traffic and the Capturing transition, verified by a boundary
  conservation test.
- **SC-003**: Packets discarded during Watching are counted, and the statistics
  reconcile: observed equals retained plus watching-discards plus every other
  named discard (conservation, extending the pipeline invariant).
- **SC-004**: Each of the six stop conditions ends the session and yields a
  complete, valid capture, verified by one test per condition.
- **SC-005**: The whole slice is exercised at tier 1 against the scripted watcher,
  with no capture driver, no elevation, and no game.
- **SC-006**: `cargo xtask ci` passes (format, clippy, tests, conventions lint,
  dependency direction, and license), and `cargo xtask neutral` and `msrv` exit 0.

## Assumptions

- The scripted watcher (`proc_script::ScriptedWatcher`) and a scripted packet
  source are the tier-1 substrate; the ETW watcher and live capture are not
  required to test this slice.
- Profile validation (S05) already guarantees non-empty stages, at most one
  terminal stage (always `session` lifecycle), unique roles, `descends_from`
  resolving acyclically within the stage set, and no ambiguous image match; the
  matcher relies on these invariants and does not re-check them.
- The pipeline (S08) provides the retain-versus-discard mechanism and the
  conservation-closed statistics record; the session lifecycle drives it rather
  than reimplementing it.
- The capture handle is modeled through the existing `PacketSource` abstraction;
  wiring the live handle and filter installation is S13 and S14, out of scope
  here.
- The acquisition timeout is an optional bound carried on the capture
  configuration (see Clarifications); when unset the session ends by another stop
  condition.
- The stage binding is written onto the process node through a new binding method
  in the core tree (see Clarifications); the matcher in `fragcap-profile` decides
  the binding and applies it, so the only new dependency edge is profile on core.
