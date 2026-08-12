# Feature Specification: CLI readiness, help, and output-contract polish

**Feature Branch**: `024-cli-readiness-polish`

**Created**: 2026-08-12

**Status**: Draft

**Input**: Post-v0.2.0 correctness/polish slice for the `fragcap` command-line
surface, bundling backlog issues #56, #62, #63, #65, #66, #67, #68, #69, #70.
No new capture or attribution capability; the goal is to make the existing
surface honest and scriptable.

## Overview

v0.2.0 shipped and real use surfaced a cluster of command-line quality defects
that share one root confusion: the shipped binary reports itself "ready to
capture" while every capture fails, because it was built with none of its
capability features. Around that root sit a set of smaller contract defects in
the readiness report, the structured-output stream, the exit codes, and the
help text. This slice corrects them together so an operator (human or script)
can trust what the tool says about itself.

The affected principle is P-9 (the instrument does not lie): a readiness verdict
that says "ready" over a binary that cannot capture is exactly the class of
untruth P-9 forbids. Behavior changes here update master specification section
17 (command line, exit codes, structured stream) and section 26.3 (diagnostics)
in the same slice.

## Clarifications

### Session 2026-08-12

- Q: Exit code both `profile show` and `profile validate` should agree on for a
  reference that resolves to nothing? → A: Exit 1 (expected failure / not-found,
  per section 17). Verified against the resolver (research R1): "resolves to
  nothing" covers both an absent slug and an unresolvable path-shaped reference;
  both exit 1. A profile *file* that exists but fails validation stays exit 2.
- Q: Exit class for refusing live capture when the session is not elevated? → A:
  Exit 1 (an environment precondition failure, consistent with the existing "no
  live capture backend" and "driver absent" refusals).
- Q: JSON shape for `profile list --json` and `profile validate --json`? → A:
  The section 17.5 event stream (the existing `{"ts","event",...}` NDJSON), one
  `diagnostic` event per problem plus a terminal `summary` event, consistent
  with the `run`/`tap`/`extcap` capture surface.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The readiness report tells the truth about capability (Priority: P1)

An operator installs fragcap, runs `fragcap doctor`, and needs a verdict that
reflects whether *this* binary can actually capture and attribute traffic. Today
`doctor` can report "ready" while the binary has no capture backend at all, and
it presents the resulting empty interface list as an npcap/adapter problem,
sending the operator to chase a red herring.

**Why this priority**: This is the highest-value fix. A false "ready" verdict
destroys trust in every other report the tool makes, and it is the first thing
an operator sees when something does not work.

**Independent Test**: Run `doctor` against a binary built without the live
backend and confirm it reports the missing backend as a blocking problem and
does not present "no interfaces" as the cause; run it against a fully featured
binary and confirm it reports ready with the real npcap version shown.

**Acceptance Scenarios**:

1. **Given** a binary compiled without the live capture backend, **When** the
   operator runs `doctor`, **Then** a first-class readiness line reports the
   live backend as absent and blocking, and the overall verdict is "not ready"
   (exit 1).
2. **Given** a binary compiled without the socket-table attribution backend,
   **When** the operator runs `doctor`, **Then** a readiness line reports the
   socket-table backend as absent with a non-blocking severity.
3. **Given** the live backend is absent, **When** the interface section is
   rendered, **Then** the empty-interface message points at the missing backend
   rather than reporting "no interfaces were found" as an adapter fault.
4. **Given** npcap is installed without loopback support and no loopback capture
   is requested, **When** the operator runs `doctor`, **Then** loopback is
   reported as a non-blocking warning and the overall verdict is not forced to
   "not ready" by loopback alone.
5. **Given** npcap is installed, **When** `doctor` reports it, **Then** the line
   shows the detected npcap version string, falling back to a plain "installed"
   only when the version cannot be determined.

---

### User Story 2 - The released binary can actually capture (Priority: P1)

An operator downloads the official release archive, installs the binary, and
expects `run`, `tap`, and analyzer capture to work on a supported machine. Today
the release binary is built with no capability features, so all three fail
immediately with "no live capture backend".

**Why this priority**: This is the root cause behind Story 1's confusion and the
single defect that makes the primary purpose of the tool inert from the shipped
artifact.

**Independent Test**: Confirm the release build recipe compiles the binary with
the capture and attribution features enabled and that the resulting binary
enumerates interfaces and opens the capture path on a supported machine.

**Acceptance Scenarios**:

1. **Given** the release build recipe, **When** the release binary is built,
   **Then** it is compiled with the live, socket-table, and process-event
   tracing capabilities enabled.
2. **Given** the featured release binary on a supported machine, **When** the
   operator runs `doctor`, **Then** the capability readiness lines report the
   backends as present.

---

### User Story 3 - Structured output is machine-consumable everywhere (Priority: P2)

A script or the analyzer integration runs fragcap with `--json` to react to
results without scraping human text. Today `profile list` ignores `--json`
entirely, and `profile validate --json` collapses every diagnostic into one
newline-joined string, discarding the per-diagnostic structure the human output
exposes.

**Why this priority**: The structured stream is what makes the wrappers and the
analyzer integration thin (P-7); a consumer forced to re-parse human text
defeats the flag's purpose. Lower than P1 because the human surface is correct.

**Independent Test**: Run `profile list --json` and `profile validate --json`
and confirm each emits well-formed newline-delimited records, one record per
diagnostic for validation, that a consumer can read without re-parsing prose.

**Acceptance Scenarios**:

1. **Given** `--json`, **When** the operator runs `profile list`, **Then** the
   profile counts are emitted as structured records rather than human text.
2. **Given** a profile with N diagnostics, **When** the operator runs
   `profile validate --json`, **Then** N separate diagnostic records are
   emitted, each carrying its code, path, line, column, and message as distinct
   fields, followed by a terminal summary record.
3. **Given** any subcommand that does not emit structured records, **When** the
   operator consults the `--json` help, **Then** the scope of the flag is
   discoverable rather than silently inconsistent.

---

### User Story 4 - Exit codes are a consistent, documented contract (Priority: P2)

A script branches on fragcap's exit code. Today conceptually identical failures
disagree: a named-but-absent profile exits 1 under `profile show` but 2 under
`profile validate`.

**Why this priority**: Scriptability depends on a stable exit contract; the
inconsistency is small in surface but breaks any consumer that switches on it.

**Independent Test**: Run `profile show` and `profile validate` against the same
missing profile reference and confirm they agree, and that the agreed codes
follow the documented 0/1/2 contract.

**Acceptance Scenarios**:

1. **Given** a profile reference that resolves to nothing, **When** the operator
   runs `profile show` and `profile validate` on it, **Then** both exit with the
   same code.
2. **Given** the exit-code contract, **When** an operator reads the
   documentation, **Then** the classes (success / expected failure / usage or
   configuration error) and which conditions map to each are stated.

---

### User Story 5 - Help text is trustworthy and free of internals (Priority: P2)

An operator reads `fragcap run --help`. Today it exposes a developer
implementation note about the argument parser, internal roadmap slice
identifiers (S15/S16/S17), and a stale "deferred to slice S17" note for a
capability that has since shipped.

**Why this priority**: Help text is the operator's contract with the tool;
internal leakage and stale copy erode trust and confuse. Lower than the
capability and scriptability fixes because it misleads without breaking.

**Independent Test**: Inspect the help output for `run` and `extcap` and confirm
no argument-parser implementation note and no internal slice identifier appears,
and that any not-yet-available capability is described without an internal id.

**Acceptance Scenarios**:

1. **Given** `run --help` and `extcap --help`, **When** the operator reads the
   roles option, **Then** it describes only what the option does, with no
   implementation note about the argument parser.
2. **Given** any `--help` output, **When** the operator reads it, **Then** no
   internal slice identifier (of the form S-followed-by-digits) appears; a
   not-yet-available capability reads as "not yet implemented" without the id.
3. **Given** the managed-launch capability has shipped, **When** the operator
   reads the launch option help, **Then** it describes the real behavior rather
   than saying the capability is deferred.

---

### User Story 6 - Live capture refuses clearly without elevation (Priority: P3)

An operator starts a live capture from a non-elevated terminal. Today they may
hit a confusing lower-level driver error. Instead the tool should detect the
missing elevation, explain how to fix it, and refuse before touching the driver.

**Why this priority**: A clear guardrail improves the first-run experience, but
it is a refinement of an already-failing path rather than a correctness defect
in a working one.

**Independent Test**: Run a live-capture command from a non-elevated session and
confirm it refuses with an actionable elevation message and a defined exit code,
before any driver access; confirm offline and read-only commands still run
unelevated.

**Acceptance Scenarios**:

1. **Given** a non-elevated session, **When** the operator runs a live-capture
   command (`run`, `tap`, or analyzer capture), **Then** the tool refuses with a
   message that states elevation is required and how to obtain it, and it does
   so before opening the capture driver.
2. **Given** a non-elevated session, **When** the operator runs an offline or
   read-only command (`replay`, `profile`, `steam profile`, `doctor`, argument
   validation), **Then** it runs normally without requiring elevation.
3. **Given** the refusal path, **When** it triggers, **Then** the tool does not
   spawn a separate elevated process; it only detects, instructs, and refuses.

---

### User Story 7 - Success output is free of redundancy (Priority: P3)

An operator validates a profile by path and sees the same path printed twice in
the success line.

**Why this priority**: Purely cosmetic, but cheap and part of the same output
pass.

**Independent Test**: Run `profile validate` on a valid profile given by path
and confirm the path appears once.

**Acceptance Scenarios**:

1. **Given** a valid profile referenced by path, **When** the operator runs
   `profile validate`, **Then** the success line names the path once.

### Edge Cases

- npcap is present but its version cannot be read: the report shows "installed"
  rather than a wrong or empty version, and does not fail.
- Loopback capture *is* requested (`--loopback`) but loopback support is absent:
  its absence is a genuine blocker on that path, distinct from the standalone
  `doctor` downgrade in Story 1.
- A binary built with some but not all capability features: each capability line
  reports independently (live absent + socket-table present, and the reverse).
- `profile validate` finds zero diagnostics under `--json`: a terminal summary
  record still distinguishes success from "no output".
- A profile reference that is malformed (not a valid id or path) versus one that
  is well-formed but resolves to nothing: these may map to different exit
  classes; only the "resolves to nothing" case must agree across subcommands.
- Elevation state cannot be determined: the tool defaults to the safe reading
  and does not falsely refuse a legitimately elevated session.

## Requirements *(mandatory)*

### Functional Requirements

**Doctor readiness (Story 1, 2)**

- **FR-001**: `doctor` MUST report whether the live capture backend is compiled
  into the running binary, as a first-class readiness line.
- **FR-002**: `doctor` MUST report whether the socket-table attribution backend
  is compiled into the running binary, as a first-class readiness line.
- **FR-003**: An absent live capture backend MUST be a blocking problem (the
  verdict is "not ready", exit 1); an absent socket-table backend MUST be
  non-blocking (a warning).
- **FR-004**: When the live backend is absent, `doctor` MUST NOT present the
  empty interface list as an npcap/adapter fault; the message MUST point at the
  missing backend as the cause.
- **FR-005**: `doctor` MUST report a missing npcap loopback adapter as a
  non-blocking warning when loopback capture is not requested; loopback absence
  MUST NOT by itself force the verdict to "not ready".
- **FR-006**: `doctor` MUST surface the detected npcap version, falling back to a
  plain "installed" indication only when the version cannot be determined.
- **FR-007**: The release build recipe MUST compile the released binary with the
  live, socket-table, and process-event tracing capabilities enabled.

**Structured output (Story 3)**

- **FR-008**: `profile list` MUST honor `--json`, emitting its counts as
  section 17.5 structured events (the `{"ts","event",...}` NDJSON form) instead
  of human text.
- **FR-009**: `profile validate --json` MUST emit one section 17.5 `diagnostic`
  event per diagnostic, each preserving the diagnostic's code, configuration
  path, line, column, and message as distinct fields, followed by a terminal
  `summary` event. It MUST NOT collapse multiple diagnostics into one
  newline-joined string.
- **FR-010**: The scope of `--json` (which subcommands emit structured records)
  MUST be discoverable, either by every subcommand honoring it or by the flag's
  help stating where it applies.

**Exit codes (Story 4)**

- **FR-011**: A profile reference that resolves to no profile MUST produce exit 1
  (expected failure / not-found) from both `profile show` and `profile validate`,
  whether the reference is an absent id-slug or an unresolvable path-shaped
  string. A profile file that exists but fails validation MUST remain a usage or
  configuration error (exit 2).
- **FR-012**: Exit-code behavior MUST conform to the documented 0/1/2 contract
  (success / expected failure / usage or configuration error), and that contract
  MUST be documented in operator-facing material.

**Help text (Story 5)**

- **FR-013**: User-facing help for the roles option (on `run` and `extcap`) MUST
  describe only the option's behavior, with no argument-parser implementation
  note; any such note MUST live in a source comment instead.
- **FR-014**: User-facing help MUST NOT contain internal roadmap slice
  identifiers; a not-yet-available capability MUST be described without one.
- **FR-015**: Help for the managed-launch option MUST describe its real,
  shipped behavior rather than stating it is deferred.

**Elevation gate (Story 6)**

- **FR-016**: Live-capture commands (`run`, `tap`, analyzer capture) MUST detect
  a non-elevated session and refuse before opening the capture driver, with a
  message stating that elevation is required and how to obtain it. The refusal
  MUST exit 1 (an expected environment-precondition failure), consistent with the
  existing "no live capture backend" and "driver absent" refusals.
- **FR-017**: Offline and read-only commands (`replay`, `profile`,
  `steam profile`, `doctor`, argument validation) MUST run without requiring
  elevation.
- **FR-018**: The elevation gate MUST NOT auto-relaunch an elevated process; it
  only detects, instructs, and refuses. The detection MUST read only the current
  process's own token and MUST open no handle to any other process (P-1).

**Output polish (Story 7)**

- **FR-019**: The `profile validate` success line MUST name the profile path at
  most once.

**Governance**

- **FR-020**: Behavior changes MUST be reflected in master specification section
  17 (exit codes, structured stream) and section 26.3 (diagnostics) in this
  slice, and any newly introduced term MUST get a glossary entry in the same
  change (P-6).
- **FR-021**: The change to the pinned release workflow (FR-007) MUST be recorded
  as a dated decision fragment, per the repository's pinned-artifact rule.

### Key Entities

- **Readiness check**: one line in the `doctor` report, carrying a section, a
  name, a severity (ok / warn / skip / fail), a detail string, and an optional
  remediation. This slice adds capability-presence checks for the live and
  socket-table backends and adjusts the severity of the loopback check.
- **Diagnostic record**: one structured `--json` event describing a single
  profile-validation problem, carrying code, configuration path, line, column,
  and message.
- **Capability feature**: a compile-time backend (live capture, socket-table
  attribution, process-event tracing) whose presence or absence in the running
  binary is now a reported fact.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a binary that cannot capture, `doctor`'s verdict is "not ready"
  and names the missing backend; there is zero chance of a "ready" verdict over
  a binary with no live backend.
- **SC-002**: A machine consumer can obtain every profile-validation diagnostic
  field from `profile validate --json` without parsing any human-readable string
  (one record per diagnostic, distinct fields).
- **SC-003**: `profile show` and `profile validate` return identical exit codes
  for the same missing profile reference in 100% of cases.
- **SC-004**: No `--help` output contains an internal slice identifier or an
  argument-parser implementation note.
- **SC-005**: A live-capture command started without elevation refuses with an
  actionable message and never accesses the capture driver, while all offline
  and read-only commands still complete unelevated.
- **SC-006**: The released binary, built by the updated recipe, reports its
  capture and attribution backends as present under `doctor` on a supported
  machine.
- **SC-007**: The full repository gate set (`cargo xtask ci`) passes, including
  the doctor unit-test suite extended for the new checks.

## Assumptions

- The current-process elevation detection already present in the codebase is
  reused for the elevation gate; it reads only the process's own token and is
  therefore P-1 safe, so the gate introduces no new process-handle surface.
- "Supported machine" for capture means Windows with npcap installed; the
  capability checks and elevation gate are Windows-only paths and MUST NOT leak
  into platform-neutral core (P-2).
- The release feature set is live, socket-table, and process-event tracing; the
  process-event tracing capability has no build/link prerequisite, and the live
  capability requires the npcap SDK acquisition step to be present in the release
  build job.
- The 0/1/2 exit contract is authoritative as already stated in the master
  specification; this slice aligns the outlier subcommands to it rather than
  redefining it.

## Out of Scope

- Re-issuing, tagging, or publishing the v0.2.0 distribution archive (#54/#55):
  this slice fixes only the release build recipe. The re-issue is a separate,
  explicitly authorized release action.
- The accuracy of the loopback-adapter presence heuristic (its filesystem
  proxy): only its severity in the standalone `doctor` verdict changes here.
- Any new capture mode, sink, transport, or attribution source.
