# Feature Specification: CLI Command Surface (run, tap, doctor, profile)

**Feature Branch**: `feat/cli-command-surface`

**Created**: 2026-08-10

**Status**: Draft

**Input**: Slice S14 of `docs/plans/README.md`: "CLI: run, tap, doctor,
profile" (release v0.2.0), realizing master specification section 17 (Command
Line Interface) and section 26.3 (the `doctor` diagnostic).

## Overview

fragcap has been a library with no operator-facing entry point: every capability
proven so far (parsing, attribution, profiles, pipeline, sinks) is reachable only
from Rust. This slice adds the command surface that turns the library into a
usable tool. It is the final slice of v0.2.0, the first functional release, after
which an operator can capture a game's traffic to a file, attributed to the
process that produced it, without writing code.

The command surface is `run`, `tap`, `doctor`, and `profile`. Three further
commands named in the specification (`replay`, `steam`, `extcap`) are registered
so the help text foreshadows the whole tool but are not yet functional; they
belong to later slices.

## Clarifications

### Session 2026-08-10

- Q: What is the size-literal grammar for `--max-bytes` (and the size form of
  `--ring`)? → A: Integer plus a required unit `b`/`kb`/`mb`/`gb`, binary
  (1024-based), zero rejected; a shared grammar mirroring the existing duration
  literal so a later ring slice reuses it.
- Q: When is a missing process-tracing (ETW) session a blocking `doctor` failure
  versus a non-blocking skip? → A: Blocking (exit 1) only when the session is
  elevated and the process-event session cannot open, because attribution is then
  degraded; a skip when the tracing capability is not built in.
- Q: What exit code does an unrecoverable sink failure that ends a run produce?
  → A: 1 (an expected failure; the output may be partial), not 2, since it is not
  a usage or configuration error.
- Q: Does this slice enforce `--roles` and `--direction` on output, or only carry
  them? → A: `--roles` scopes which stages trigger and are captured; `--direction`
  is recorded on the effective configuration and surfaced, with full directional
  output filtering deferred to a later slice. Both are accepted and validated now.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Diagnose environment readiness (Priority: P1)

An operator who has just installed fragcap, or whose capture failed, runs
`fragcap doctor` to learn whether the machine can capture and, if not, exactly
what to fix. This is the first command a confused user runs.

**Why this priority**: It is the lowest-risk, highest-value command, depends on
none of the capture machinery, and is the one an operator reaches for before
anything else works. It also carries the npcap gate obligation (detect, never
install), which is a release requirement independent of capture.

**Independent Test**: Run `fragcap doctor` and `fragcap doctor --json` against a
set of constructed environment states (npcap present/absent, each npcap option
present/absent, elevated/not, interfaces up/down); confirm the report classifies
each check, names the exact remediation for every failing check, and returns exit
0 when capture is possible and 1 when a blocking problem exists.

**Acceptance Scenarios**:

1. **Given** a ready machine (npcap installed with both non-default options,
   session elevated, at least one interface up), **When** the operator runs
   `fragcap doctor`, **Then** every check reports ok, the report ends "Ready to
   capture.", and the exit code is 0.
2. **Given** npcap is installed without loopback capture support, **When** the
   operator runs `fragcap doctor`, **Then** the loopback-adapter check fails and
   names its specific remediation (reinstall enabling "Support loopback traffic")
   distinctly from the WinPcap-API-mode check, and the exit code is 1.
3. **Given** npcap is not installed, **When** the operator runs `fragcap doctor`,
   **Then** the driver check fails with a remediation that fragcap does not
   install it and where to obtain it, and the exit code is 1.
4. **Given** only the optional analyzer integration is missing, **When** the
   operator runs `fragcap doctor`, **Then** that check warns (not fails), the
   report still ends ready, and the exit code is 0.
5. **Given** any environment state, **When** the operator adds `--json`, **Then**
   the same content is emitted as one structured record per check.

---

### User Story 2 - Capture a game with a profile (Priority: P1)

An operator captures a game's network traffic using a profile that names the
game's processes, writing an attributed capture file. The capture arms before the
target exists, waits for it, captures while it runs, and stops on a bound or an
interrupt, then reports what it captured and what it dropped.

**Why this priority**: This is the product. Everything else in the tool exists to
support or diagnose this command. It is the acceptance of the whole v0.2.0
release.

**Independent Test**: Drive `fragcap run` end to end over a recorded capture with
a scripted process timeline and scripted attribution (no driver, no elevation, no
game): confirm the produced file matches a golden, the completion summary's
counters satisfy the conservation identity, the `--json` event sequence matches a
golden, the written attribution carries the expected role and stage, and an
operator interrupt yields exit 0.

**Acceptance Scenarios**:

1. **Given** a valid profile reference and an output path, **When** the operator
   runs `fragcap run --profile <ref> --out <path>`, **Then** a capture file is
   written, its packets are attributed to the profile's stages with role and
   stage recorded, and the exit code is 0.
2. **Given** a profile whose `[capture]` table sets an option, **When** the
   operator passes the same option on the command line, **Then** the command-line
   value is used and the profile value is overridden.
3. **Given** a running capture, **When** the operator sends an interrupt, **Then**
   the capture stops normally, the output is valid, the completion summary is
   printed, and the exit code is 0.
4. **Given** a duration, packet-count, or byte bound, **When** the bound is
   reached, **Then** the capture stops for that reason and the summary names it.
5. **Given** an acquisition wait and a target that never appears, **When** the
   wait elapses, **Then** the capture ends having captured nothing, the summary
   says the target was never acquired, and the exit code is 1.
6. **Given** any completed run, **When** the summary is printed, **Then** it
   surfaces the existing discard counters (packets watched-and-discarded before
   acquisition, buffer drops, sink drops) rather than reporting a bare success.

---

### User Story 3 - Manage and validate profiles (Priority: P2)

A profile author validates a profile before capturing with it, lists the profiles
available to the tool, and inspects how a reference resolves.

**Why this priority**: A profile is the operator-authored input to `run`, and an
author working against a game update needs every mistake in one report before a
capture wastes a session. It depends only on the existing profile machinery.

**Independent Test**: Run `fragcap profile validate/list/show` against fixture
profiles and profile directories a test constructs; confirm a valid profile
reports valid, an invalid profile reports every diagnostic at once and exits 2,
and listing/showing reports the bundled and user profiles and which source
supplied a resolved reference.

**Acceptance Scenarios**:

1. **Given** a valid profile, **When** the operator runs `fragcap profile
   validate <ref>`, **Then** it reports the profile is valid and where it resolved
   from, and the exit code is 0.
2. **Given** a profile with several mistakes, **When** the operator runs
   `fragcap profile validate <ref>`, **Then** every diagnostic is reported in one
   pass (not just the first), and the exit code is 2.
3. **Given** the available profiles, **When** the operator runs `fragcap profile
   list`, **Then** the bundled and user profiles are listed with their counts, and
   the exit code is 0.
4. **Given** a reference that resolves, **When** the operator runs `fragcap
   profile show <ref>`, **Then** the resolved profile and the source that supplied
   it are shown; a reference that does not resolve reports every location searched
   and exits 1.

---

### User Story 4 - Capture a running process ad hoc (Priority: P2)

An operator captures a process that is already running, by image name, without
authoring a profile first.

**Why this priority**: It is the fastest path to a capture for a process that has
no profile, and it reuses the entire `run` capture engine, so it is low marginal
cost once `run` exists.

**Independent Test**: Drive `fragcap tap --process <name> --duration <dur>` over
the same offline substrate as `run`; confirm a single-stage capture of the named
process is produced and stops at the duration or when the process exits, with the
same completion summary and exit contract as `run`.

**Acceptance Scenarios**:

1. **Given** a process image name and a duration, **When** the operator runs
   `fragcap tap --process <name> --duration <dur>`, **Then** traffic attributed to
   that process is captured until the duration elapses or the process exits, and
   the exit code is 0.
2. **Given** an invalid process name or missing required argument, **When** the
   operator runs `fragcap tap`, **Then** a usage error is reported and the exit
   code is 2.

---

### User Story 5 - Discover the whole tool from its help (Priority: P3)

An operator reading `fragcap --help` sees the complete command surface, including
the commands that are planned but not yet functional, so the tool does not appear
to change shape between releases.

**Why this priority**: It is cheap honesty. Hiding the future commands would make
later releases look like breaking additions; stubbing them makes the roadmap
visible without implementing it.

**Independent Test**: Run each of `fragcap replay`, `fragcap steam`, `fragcap
extcap`; confirm each reports it is not yet implemented, names the slice that will
deliver it, and exits 2, while appearing in `fragcap --help`.

**Acceptance Scenarios**:

1. **Given** the tool, **When** the operator runs `fragcap --help`, **Then** all
   seven commands (run, tap, replay, profile, steam, doctor, extcap) are listed.
2. **Given** a not-yet-implemented command, **When** the operator runs it, **Then**
   it reports "not yet implemented" with the delivering slice and exits 2.

---

### Edge Cases

- **Interrupt during capture** is a normal stop, not a failure: the output is
  valid and the exit code is 0.
- **A sink that writes to standard output** forces all diagnostic and progress
  output to standard error, so the capture data stream stays uncontaminated.
- **`--quiet`** suppresses progress but keeps warnings and errors; **`--silent`**
  suppresses everything except errors; errors are never suppressed by either.
- **A capture mode not yet supported** (`stream`, `ring`) and a **transport sink
  not yet supported** (named pipe, TCP) are rejected as configuration errors (exit
  2) naming the slice that will deliver them, rather than silently ignored.
- **`--launch`** is accepted by the parser but managed launch is a later slice; it
  is rejected as not-yet-implemented rather than silently ignored.
- **No usable interface or capture driver absent** ends the run as an expected
  failure (exit 1), distinct from a usage error.
- **A profile that fails to resolve** (bad reference) is a usage error (exit 2); a
  reference that is well-formed but matches nothing searched is an expected failure
  for `show` (exit 1).

## Requirements *(mandatory)*

### Functional Requirements

#### Command surface and dispatch

- **FR-001**: The tool MUST expose the commands `run`, `tap`, `replay`, `profile`,
  `steam`, `doctor`, and `extcap`, all visible in the top-level help.
- **FR-002**: The tool MUST implement `run`, `tap`, `doctor`, and `profile`, and
  MUST register `replay`, `steam`, and `extcap` as stubs that report "not yet
  implemented", name the slice that will deliver them, and exit with a usage or
  configuration error.
- **FR-003**: The tool MUST print its version and a help listing for the tool and
  for each command.

#### Exit-code contract

- **FR-004**: The tool MUST follow the 0/1/2 exit-code contract: 0 for success
  (a completed capture, passed diagnostics, or an operator interrupt during
  capture); 1 for an expected failure (target never appeared, capture driver or
  interface absent, diagnostics found a blocking problem); 2 for a usage or
  configuration error (bad arguments, invalid profile, an unsupported mode or sink
  requested).
- **FR-005**: An operator interrupt during a capture MUST be treated as success
  (exit 0) because the capture completes normally and its output is valid.
- **FR-005a**: A run ended by an unrecoverable sink failure MUST exit 1 (an
  expected failure, with possibly partial output), not 2, because it is not a usage
  or configuration error.

#### run

- **FR-006**: `run` MUST capture using a profile resolved from a path, name, or
  game id, and MUST accept every capture option in specification section 17.2.
- **FR-007**: Any capture option present in a profile's capture table MUST be
  overridable on the command line, and the command-line value MUST win.
- **FR-008**: `run` MUST arm before the target exists, wait for it (optionally
  bounded by an acquisition timeout), capture while it runs, and stop on the first
  of its configured bounds, a terminal stage exit, all targets exiting, an
  operator interrupt, or an unrecoverable sink failure.
- **FR-009**: `run` MUST write captured packets to the configured sinks with each
  attributed packet carrying the role and stage of the process that owned its
  flow.
- **FR-010**: `run` MUST support writing a capture file for the default file mode;
  it MUST reject the not-yet-supported `stream` and `ring` modes as configuration
  errors that name the delivering slice.
- **FR-011**: `run` MUST support file-target sinks (capture file and JSON Lines
  metadata) and MUST reject not-yet-supported transport sinks (named pipe, TCP) as
  configuration errors that name the delivering slice.
- **FR-011a**: A size bound (`--max-bytes`, and the size form of `--ring`) MUST be
  written as an integer plus a required unit (`b`, `kb`, `mb`, `gb`), interpreted
  as binary (1024-based), with zero rejected; a missing or unrecognized unit is a
  usage error.
- **FR-011b**: `run` MUST accept and validate `--roles` and `--direction`.
  `--roles` scopes which stages trigger and are captured; `--direction` is recorded
  on the effective configuration and surfaced. Full directional filtering of output
  is deferred to a later slice.

#### tap

- **FR-012**: `tap` MUST capture a named running process without an authored
  profile, by constructing a single-stage profile through the same validation path
  an authored profile uses (no unvalidated construction), and MUST otherwise use
  the same capture engine, completion summary, and exit contract as `run`.

#### doctor

- **FR-013**: `doctor` MUST report environment readiness across platform (OS,
  subsystem, privilege), capture driver (npcap presence and version, loopback
  adapter, WinPcap API compatibility mode), tracing (process-event session),
  interfaces, integration (analyzer extcap), and profiles (bundled and user
  counts).
- **FR-014**: `doctor` MUST only detect the capture driver; it MUST NOT install,
  download, or modify it. When a non-default npcap option is absent, `doctor` MUST
  name that option individually with the exact remediation to enable it, treating
  loopback capture support and WinPcap API compatibility mode as separate checks.
- **FR-015**: `doctor` MUST exit 0 when capture is possible and 1 when a blocking
  problem exists; warnings about optional integration MUST NOT block. A missing
  process-tracing session MUST be treated as blocking (fail) only when the session
  is elevated and the process-event session cannot open; when the tracing
  capability is not built in, that check is a non-blocking skip.
- **FR-016**: Every failing `doctor` check MUST name a specific remediation.

#### profile

- **FR-017**: `profile` MUST validate a profile, reporting every diagnostic in one
  pass and exiting 2 when the profile is invalid; list the available bundled and
  user profiles with counts; and show how a reference resolves, including the
  source that supplied it.

#### Structured output and streams

- **FR-018**: With the structured-output option, the tool MUST emit
  newline-delimited structured events on standard error over the capture
  lifecycle (session armed, stage matched, stage exited, filter narrowed, session
  complete) while capture data goes to the configured sinks. `doctor` MUST emit its
  report as structured records under the same option.
- **FR-019**: Progress and diagnostic output MUST go to standard error and capture
  data MUST go to sinks; when a sink writes to standard output, all diagnostic and
  progress output MUST move to standard error.
- **FR-020**: The tool MUST support a quiet mode (suppress progress, keep warnings
  and errors) and a silent mode (suppress all but errors); errors MUST never be
  suppressed.

#### Loss accounting

- **FR-021**: The completion summary MUST surface the existing discard counters,
  packets discarded while watching before a target was acquired, packets discarded
  out of the capture window, buffer drops, and per-sink drops, and MUST NOT report
  a bare success that hides them. The tool MUST NOT invent new counters or fabricate
  counts it did not observe.

### Key Entities *(include if feature involves data)*

- **Effective capture configuration**: the capture options actually used, formed
  by overlaying command-line options onto a profile's capture defaults; command
  line wins, and an option absent from both remains absent.
- **Environment report**: the ordered set of readiness checks `doctor` produces,
  each with a section, a name, a detail, a status (ok, warn, skip, fail), and, when
  failing, a remediation; the report as a whole yields a single exit code.
- **Lifecycle event**: a structured record emitted over a capture's life (armed,
  matched, exited, narrowed, complete), carrying the fields a downstream wrapper
  needs to react without parsing human-readable output.
- **Completion summary**: the end-of-run accounting an operator reads, surfacing
  the captured and attributed counts and every discard counter.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator can produce an attributed capture file from a profile
  with a single command, verified by an end-to-end run over the offline substrate
  whose output matches a committed golden.
- **SC-002**: `fragcap doctor` correctly classifies every readiness state and
  returns the correct exit code (0 when capture is possible, 1 when blocked) across
  every constructed environment case, with each failing check naming a specific
  remediation.
- **SC-003**: An invalid profile passed to `profile validate` reports 100% of its
  diagnostics in a single invocation (not just the first) and exits 2.
- **SC-004**: For every completed run, the completion summary's counts satisfy the
  conservation identity (everything observed is either received by a sink, dropped
  with a named counter, or refused with a named counter), no packet is
  unaccounted.
- **SC-005**: An operator interrupt during a capture yields a valid output file and
  exit 0 in 100% of interrupted runs.
- **SC-006**: The complete seven-command surface is discoverable from `fragcap
  --help`, and every not-yet-implemented command exits 2 with a message naming the
  delivering slice.
- **SC-007**: Every capture behavior in this slice is demonstrated with no capture
  driver, no elevated privilege, and no game, so the whole slice is verifiable in
  continuous integration.

## Assumptions

- The offline verification substrate proven in earlier slices (a recorded capture
  replayed as a source, a scripted attributor, and a scripted process timeline)
  drives every `run`/`tap` acceptance test; live capture and the operating-system
  socket table are compiled but exercised only on a developer machine, consistent
  with the project's standing position that live capture has never executed in
  continuous integration.
- The bundled profile set is empty in this release; `profile list` and resolution
  operate over user profile directories the operator or a test supplies.
- Managed launch (`--launch`), streaming and ring capture modes, and transport
  sinks (named pipe, TCP) are out of scope for this slice and are surfaced as
  parser-accepted but not-yet-implemented, delivered by later slices.
- The user profile directory is located from the platform's standard per-user
  configuration location; no profile is created or written by this slice.
- Color output is a cosmetic concern deferred beyond this slice; the stream
  routing, quiet, and silent behaviors are in scope.
- This slice adds no new counters: it surfaces the discard counters the pipeline
  and session already maintain.
