# Feature Specification: Watch / Attach Mode (Launch-Agnostic Capture)

**Feature Branch**: `feat/watch-attach-mode`

**Created**: 2026-08-12

**Status**: Draft

**Slice**: S028 (GitHub issue #77, slice 2 of 4). Builds on the S027 target
resolution cascade (merged). Delivers the first-class launch-agnostic capture
surface: capture a game by a target identity, however it was started, including
when it is already running. Constitution principles in play: passive observation
only (P-1), no silent loss on the give-up path (P-4), the honesty posture that an
observed answer is stamped observed (P-9).

**Input**: Add a `watch` subcommand that captures by a target identity (an
executable image name plus an optional path anchor), launch-agnostic: fragcap
arms the watcher and sinks and captures the first process that matches, however
it was started, with no authored profile file and no managed launch. Wire the
S027 runtime-observation provider so a target already running when `watch` starts
is attached immediately over the startup snapshot, not only on a new start event.
Reuse the existing acquisition timeout and its named give-up outcome. Name watch
mode as the default launch-agnostic path in the spec and glossary, and reference
the launcher_mediated term (issue #78/#83) as the database-side label for the
runtime case watch mode handles.

## Overview

Launching a game and attributing its packets are independent problems (issue
#77). The one durable fact is that at runtime a process exists that is the game
and holds sockets. Watch mode is the capture surface built directly on that fact:
the operator names the target by identity, fragcap arms and listens, and captures
the process that matches, no matter how or where it started. This is the only
path that works for a modded Skyrim install launched from Mod Organizer 2 through
a script extender, a standalone GOG title, or any non-Steam game, because it
assumes nothing about origin.

Much of the runtime machinery already exists and is reused rather than rebuilt.
The capture session already arms into a Watching state, folds process events, and
acquires the target when a stage matches; `run` without a managed launch already
captures whatever appears; the acquisition timeout and its dedicated
`StopReason::AcquisitionTimeout` already give up loudly; watch-time frames are
already counted (P-4). What this slice adds is the parts that turn that machinery
into a launch-agnostic identity capture an operator can actually invoke:

- **A `watch` subcommand keyed on identity, not a profile.** `run` needs an
  authored profile and `tap` matches an executable name only. The modded case
  needs a **path anchor**: `SkyrimSE.exe` launched from a Defender-excluded
  directory outside `steamapps` is ambiguous by name alone, and the path anchor
  is what distinguishes the modded install from any other. `watch` takes an
  executable glob plus an optional path substring or path regular expression, and
  an acquisition timeout, which `tap` does not carry.
- **Attach to an already-running game.** The session's wait-for-start acquires a
  process that starts after fragcap arms. It does not, on its own, acquire a game
  that was already running when fragcap started. Wiring the S027
  runtime-observation provider over the startup snapshot the process watcher
  already takes closes that gap: at arm, if a process matching the identity is
  already present, it is acquired immediately, stamped observed. Both are runtime
  observation, at two moments: attach-to-running and wait-for-start.
- **The fidelity is honest, and the two axes stay separate.** The identity the
  operator types is a target definition they authored, so the synthesized
  identity is `authored` fidelity, exactly as `tap`'s is (S027 refuses `observed`
  on a profile precisely because `observed` is a runtime result, not a claim an
  author makes). The attach-to-running decision, "is a matching process already
  running?", is answered by the S027 runtime-observation provider, whose answer is
  `observed`; that is the resolver's answer about a live process, distinct from
  the definition's authored fidelity. Identity uses only the image name and path
  already in the process snapshot; no process handle is opened (P-1).

Three properties are load-bearing.

**Capture is launch-agnostic, and that is the whole point.** No `steam://`, no
authored profile, no launcher ancestry. The target is the process that matches
the identity, whatever started it. Managed launch stays a convenience on `run`,
never the spine.

**The give-up is loud, and it already has a name.** A watch that never sees its
target does not hang and does not exit silently: the acquisition timeout fires,
the run ends with `StopReason::AcquisitionTimeout`, the watch-time discard
accounting is surfaced (P-4), and the exit is a failure. A clean operator
interrupt during the watch is a cancellation, not a failure.

**Identity keys on the target process itself.** The watch identity is an
executable name plus a path anchor. Ancestry (`descends_from`) is not part of it,
because a modded launch has alien ancestry; `descends_from` stays reserved for
multi-stage runtime disambiguation (the Division 2 case), which is a profile
concern, not a watch identity.

The slice stops at the `watch` surface, the attach-to-running wiring, the reuse
of the acquisition timeout, and their tests, plus the spec and glossary. It does
not add engine detection (S029), Steam appinfo walking (S030), or the hint
database (#78).

## Clarifications

### Session 2026-08-12

Resolved under autopilot from the spec, the constitution, issue #77, and the code
on main, and confirmed with the operator where the approved plan's premise had
changed on contact with the code.

- Q: The approved plan framed watch mode as greenfield ("no such flag; Watching
  is only an internal state"), but the code already watches on `run` without a
  launch and already has an acquisition timeout and a named give-up reason. What
  is the surface? -> A: A new `watch` subcommand (operator decision). It names
  watch mode explicitly, keeps `tap` as the simplest exe-name form, and captures
  by an executable glob plus an optional path anchor with an acquisition timeout.
- Q: Does the slice also wire the S027 ObservationProvider, or only reuse the
  session's wait-for-start? -> A: Also wire it (operator decision), for the
  attach-to-already-running case. The provider resolves over the startup snapshot
  the process watcher already takes, so a game already running at arm is acquired
  immediately, stamped observed; a game that starts later is acquired by the
  existing wait-for-start. Both are runtime observation.
- Q: How is the identity expressed and matched? -> A: As the existing match
  predicates: an executable glob (`exe`) plus an optional `path_contains` or
  `path_regex`. `watch` synthesizes a validated one-stage identity profile from
  them (the same validated-construction path `tap` uses), so the session captures
  it with no new matching code. At least one predicate is required; an identity
  with no executable and no path anchor is refused at construction.
- Q: Is the acquisition timeout new? -> A: No. `--wait` and
  `StopReason::AcquisitionTimeout` already exist and are reused; `watch` exposes
  `--wait` (which `tap` lacks). The give-up path is already counted and surfaced;
  the slice confirms it on the `watch` surface rather than adding a counter.
- Q: What fidelity does a watch capture report? -> A: The operator-supplied
  identity is `authored` (they typed it, exactly as `tap`'s synthesized identity
  is), because S027 refuses `observed` on a profile: `observed` is a runtime
  result, not an author's claim. The attach-to-running decision uses the S027
  observation provider, whose answer carries the `observed` tier; that is the
  resolver's answer about a live process, a separate axis from the definition's
  authored fidelity. The two are not conflated.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Capture a game by identity, however it launches (Priority: P1)

An operator wants to capture a modded Skyrim install that launches from Mod
Organizer 2 through a script extender, from a directory outside `steamapps`, with
Steam never in the process tree. They run `watch` with the executable name and a
path anchor, launch the game however their setup demands, and fragcap captures
the socket-holding client.

**Why this priority**: This is the reason watch mode exists and the case no other
surface handles: `run` needs an authored profile, `tap` cannot express the path
anchor that disambiguates a modded install, and managed launch is useless when
Steam is not the launcher. Without it, the launch-agnostic core is not reachable
from the command line.

**Independent Test**: Drive the offline substrate with a process timeline in which
a process matching the executable and path anchor starts from an arbitrary parent
outside any storefront directory, with no managed launch; confirm `watch`
acquires and captures it, and that the same timeline with a non-matching path
anchor does not acquire.

**Acceptance Scenarios**:

1. **Given** a `watch` identity of an executable glob plus a path anchor, **When**
   a process matching both starts from an arbitrary parent with no `steam://`
   involvement, **Then** it is acquired and captured.
2. **Given** two processes sharing the executable name but only one under the
   path anchor, **When** `watch` runs, **Then** the one matching the path anchor
   is acquired and the other is not.
3. **Given** a `watch` capture, **When** the output is produced, **Then** it is
   byte-identical to an equivalent single-stage profile capture (the surface is
   new, the capture engine is the shared one).

---

### User Story 2 - Attach to a game that is already running (Priority: P2)

An operator starts fragcap after the game is already up. Watch mode attaches to
the already-running process immediately rather than waiting for a start event
that will never come.

**Why this priority**: Wait-for-start alone misses a game that was running before
fragcap armed, which is a common real case (the operator remembers to capture
after launching). It builds on US1's identity and is the attach half of
attach-or-wait.

**Independent Test**: Seed the process substrate with a startup snapshot
containing a process that matches the identity and produces no later start event;
confirm `watch` acquires it at arm and captures, stamped observed.

**Acceptance Scenarios**:

1. **Given** a process matching the identity already present at arm, **When**
   `watch` starts, **Then** it is acquired immediately from the startup snapshot,
   without waiting for a start event.
2. **Given** an attach-to-running acquisition, **When** the resolution is
   examined, **Then** the S027 runtime-observation provider produced the answer
   (its `observed` tier), distinct from the operator-authored identity.
3. **Given** no matching process present at arm and one that starts later,
   **When** `watch` runs, **Then** the later start is acquired by wait-for-start,
   proving attach and wait compose.

---

### User Story 3 - Give up loudly when the target never appears (Priority: P3)

An operator sets an acquisition timeout and the target never appears. Watch mode
gives up at the timeout with a named reason and the discard accounting, rather
than hanging or exiting silently.

**Why this priority**: A capture tool that waits forever, or exits zero having
captured nothing, is the silent-loss failure P-4 forbids. It reuses the existing
timeout machinery on the new surface.

**Independent Test**: Run `watch --wait` against a timeline in which no process
ever matches; confirm the run ends with `StopReason::AcquisitionTimeout`, surfaces
the watch-time discard accounting, and exits a failure; and that an operator
interrupt during the watch exits zero instead.

**Acceptance Scenarios**:

1. **Given** `--wait` set and no matching process, **When** the timeout elapses,
   **Then** the run ends with `StopReason::AcquisitionTimeout`, reports the
   watch-time discard accounting, and exits a failure.
2. **Given** a watch in progress, **When** the operator interrupts it, **Then**
   the run is a clean cancellation and exits zero, not a failure.
3. **Given** any watch that captured nothing, **When** it ends, **Then** the
   summary states it acquired no target rather than presenting an empty success.

---

### Edge Cases

- An identity with neither an executable nor a path anchor is refused at
  construction (an empty predicate set matches every process); the operator sees
  a configuration error, not a capture of the whole system.
- A process matching the executable but not the path anchor is not acquired; the
  path anchor is a required part of the identity when supplied.
- The target is already running and also restarts during the watch: it is
  acquired once (attach-to-running wins at arm), not double-counted.
- The acquisition timeout is not set and the target never appears: the watch runs
  until interrupted, and the interrupt is a clean stop (unbounded watch is a
  deliberate choice, and the operator can see it in their own invocation).
- A path anchor regular expression that does not compile is refused at
  construction with the profile's own diagnostic, exit 2.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A `watch` subcommand MUST capture by a target identity expressed as
  an executable glob plus an optional path anchor (`path_contains` or
  `path_regex`), synthesizing a validated one-stage identity through the same
  validated-construction path `tap` uses, with no authored profile file.
- **FR-002**: Watch mode MUST be launch-agnostic: it arms the watcher and sinks
  and captures the first process matching the identity, however it was started,
  with no `steam://` and no managed launch.
- **FR-003**: A process matching the identity that is already running when `watch`
  starts MUST be attached immediately via runtime observation over the startup
  snapshot, not only on a later start event.
- **FR-004**: The watch identity MUST key on the target process itself (executable
  plus path anchor); `descends_from` MUST NOT be part of a watch identity and
  stays reserved for multi-stage disambiguation.
- **FR-005**: `watch` MUST expose the acquisition timeout (`--wait`); a watch that
  never acquires MUST give up at the timeout with `StopReason::AcquisitionTimeout`,
  surface the watch-time discard accounting (P-4), and exit a failure; a clean
  operator interrupt during the watch MUST exit zero.
- **FR-006**: The operator-supplied watch identity MUST be `authored` fidelity
  (the operator authored it, exactly as `tap`'s synthesized identity is), never
  falsely `observed` (which S027 refuses on a profile); the attach-to-running
  decision MUST use the S027 runtime-observation provider (whose answer is the
  `observed` tier) over the startup snapshot; identity MUST use only the image
  name and path in the process snapshot and MUST open no process handle (P-1).
- **FR-007**: `watch` MUST reuse the shared capture engine so its output is
  byte-identical to an equivalent single-stage profile capture.
- **FR-008**: An identity with no predicate MUST be refused at construction, and a
  path-anchor regular expression that does not compile MUST be refused with the
  profile's own diagnostic.
- **FR-009**: The master specification MUST name watch mode as the default
  launch-agnostic capture path (sections 7.1 and 10.5), and the glossary MUST gain
  a `watch mode` entry near the process watcher and acquisition timeout, in the
  same change; the entry MUST reference `launcher_mediated` as the database-side
  label (issue #78/#83) for the runtime case watch mode handles.

### Key Entities *(include if feature involves data)*

- **Watch identity**: an executable glob plus an optional path anchor, the match
  predicates that recognize the target process. Synthesized into a one-stage
  identity profile for the shared capture engine.
- **Attach-to-running acquisition**: acquiring a target that was already present
  at arm, resolved by the S027 runtime-observation provider over the startup
  snapshot, stamped observed.
- **Acquisition timeout outcome**: the `StopReason::AcquisitionTimeout` give-up,
  with the watch-time discard accounting, surfaced rather than silent.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A modded-Skyrim-shaped target (arbitrary parent, directory outside
  `steamapps`, no `steam://`) is captured by `watch` in a test, keyed on an
  executable plus a path anchor.
- **SC-002**: A process already running at arm (present in the startup snapshot,
  producing no later start event) is attached and captured, with the
  attach-to-running answer produced by the runtime-observation provider, 100% of
  the time.
- **SC-003**: A watch that never sees its target ends with
  `StopReason::AcquisitionTimeout`, surfaces the watch-time discard accounting,
  and exits a failure; an interrupt during the watch exits zero.
- **SC-004**: A `watch` capture produces byte-identical output to an equivalent
  single-stage profile capture.
- **SC-005**: An identity with no predicate, or a non-compiling path regular
  expression, is refused at construction rather than capturing the whole system
  or deferring the fault to capture time.
- **SC-006**: The path anchor disambiguates two processes sharing an executable
  name: only the one under the anchor is acquired.

## Assumptions

- **Reuse over rebuild.** The Watching state, stage matching, the acquisition
  timeout, `StopReason::AcquisitionTimeout`, the watch-time discard counting, and
  the shared capture engine already exist and are reused. This slice adds the
  `watch` surface, the attach-to-running wiring, and the docs, not a new capture
  path.
- **Attach-to-running composes with the existing snapshot.** The process watcher
  already takes a P-1-safe toolhelp startup snapshot; the S027 observation
  provider resolves over it. The offline substrate models an already-running
  target with a snapshot or an early start event for testing.
- **Identity is exe plus path anchor.** No new identity vocabulary; the existing
  match predicates express it. `descends_from` is not a watch identity.
- **Scope boundary.** No engine detection (S029), no Steam appinfo (S030), no hint
  database (#78). `tap` and `run` are unchanged except where they share code.
- **Toolchain.** The workspace minimum supported toolchain stays 1.82 and MUST
  remain green; the slice adds no dependency.
- **Text hygiene.** All artifacts are UTF-8 without BOM, LF line endings, no
  em-dashes or en-dashes, including code comments and JSON string values.
