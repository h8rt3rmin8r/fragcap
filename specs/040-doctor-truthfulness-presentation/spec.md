# Feature Specification: doctor truthfulness and presentation

**Feature Branch**: `040-doctor-truthfulness-presentation`

**Created**: 2026-08-14

**Status**: Draft

**Input**: GitHub issues #102, #103, #105, #106, and the doctor-facing string
changes from #107 and #104. Make `fragcap doctor` report the truth about the
machine and present it legibly.

## Clarifications

### Session 2026-08-14

Resolved under autopilot from the approved plan, the constitution (P-4, P-9),
and existing doctor behavior; recorded here to pin the acceptance tests.

- Q: How is an undetermined loopback state classified? -> A: Non-blocking. It
  renders as a warning whose detail states the state could not be determined; it
  is never a blocking failure and never claims loopback is not installed.
- Q: Can any identity or paths row be a blocking failure? -> A: No. Identity and
  paths rows are always informational (ok); an unresolvable path is a neutral
  note on that row, not a failure, and never changes the command's exit status.
- Q: Are paths reported even when the target does not yet exist? -> A: Yes. The
  identity section reports the resolved location regardless of whether the file
  or directory exists yet (a fresh install still shows where its data will live).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Truthful capture-readiness diagnosis (Priority: P1)

A user who has installed fragcap and its capture prerequisites runs
`fragcap doctor` to confirm the machine is ready to capture. The report tells
them the truth: it lists the network interfaces that are actually present and
usable, and it reports whether loopback capture is actually available. A user on
a correctly configured machine sees a clean report, not a warning about missing
interfaces or missing loopback support that is not really missing.

**Why this priority**: This is the core defect. Today the report is wrong on
every machine (it always claims no interfaces were found, and it judges loopback
from an unrelated signal), so the whole command is untrustworthy. Fixing this is
the reason the slice exists, and it is what unblocks a truthful install tutorial
downstream. It maps directly onto the project principle that the instrument does
not lie.

**Independent Test**: On a machine with the live capture capability built in and
capture drivers present, run `fragcap doctor` and confirm the Interfaces section
lists the real adapters and the loopback line reflects the real loopback state.
On a machine with no adapters, confirm the "no interfaces were found" message
appears only then.

**Acceptance Scenarios**:

1. **Given** a machine with usable network adapters and the live capture
   capability built in, **When** the user runs `fragcap doctor`, **Then** the
   Interfaces section lists each real adapter with its name, a representative
   address, and its up or down state, and does not warn that no interfaces were
   found.
2. **Given** a machine where loopback capture support is installed, **When** the
   user runs `fragcap doctor`, **Then** the loopback line reports loopback as
   supported, regardless of which other optional capture components were
   installed.
3. **Given** a machine where the loopback state genuinely cannot be determined,
   **When** the user runs `fragcap doctor`, **Then** the loopback line says the
   state is undetermined rather than asserting it is not installed.
4. **Given** a build without the live capture capability, **When** the user runs
   `fragcap doctor`, **Then** the Interfaces section names the missing capability
   as the reason rather than implying an adapter or driver fault.

---

### User Story 2 - A report that identifies itself (Priority: P2)

A user preparing a bug report, or simply confirming which install is active,
runs `fragcap doctor` and can see, from the report alone, which fragcap version
produced it, where the running binary lives, and where fragcap keeps its
per-user data. They do not have to run a second command or guess.

**Why this priority**: The report is the natural thing to read or paste when
something is wrong, and today it omits the first facts a maintainer or user
needs. It is high value and low risk, but it depends on nothing from Story 1, so
it sits at P2.

**Independent Test**: Run `fragcap doctor` and confirm the report begins with an
identity section naming the version, the absolute path of the running binary,
the user profile directory, and the default hint-database path.

**Acceptance Scenarios**:

1. **Given** any machine, **When** the user runs `fragcap doctor`, **Then** the
   report's first section states the fragcap version, the absolute path of the
   running executable, the user profile directory, and the default
   hint-database path.
2. **Given** the machine-readable output, **When** the user runs
   `fragcap doctor --json`, **Then** the identity facts appear as ordinary
   records in the same one-record-per-check stream, not as a separate object or
   header.
3. **Given** a location that cannot be determined on this platform, **When** the
   user runs `fragcap doctor`, **Then** the corresponding line says so rather
   than printing a wrong or empty path.

---

### User Story 3 - A report a person can read at a glance (Priority: P3)

A first-time user runs `fragcap doctor` in their terminal and can scan the
result quickly: the pass or fail state of each check stands out by color, the
sections are visually separated, and no line runs off the edge of a normal
terminal window. When they redirect the output to a file or a pipe, or when they
have asked their environment to suppress color, the output is plain text with no
stray control codes.

**Why this priority**: Presentation does not change what the report says, only
how easily it is read, so it ranks below correctness and self-identification.
It matters because this command is a first-run experience and a screenshot
target.

**Independent Test**: Run `fragcap doctor` in a real terminal and confirm the
status words are colored, sections are separated, and lines fit a standard width.
Pipe the output to a file and confirm it is byte-for-byte plain with no color
codes. Set the no-color environment signal and confirm color is suppressed even
in a terminal.

**Acceptance Scenarios**:

1. **Given** output to an interactive terminal, **When** the user runs
   `fragcap doctor`, **Then** each status word (ok, warn, skip, fail) is shown in
   a color matching its severity and each section heading is visually separated
   from the one before it.
2. **Given** output redirected to a file or pipe, or the no-color environment
   signal is set, **When** the user runs `fragcap doctor`, **Then** the output
   contains no color control codes.
3. **Given** the default set of checks, **When** the user runs `fragcap doctor`
   in a standard-width terminal, **Then** no line overflows onto a wrapped
   second line; long guidance is presented as an indented continuation.
4. **Given** the machine-readable output, **When** the user runs
   `fragcap doctor --json`, **Then** the output is never colorized.

---

### User Story 4 - Guidance that points the right way (Priority: P3)

A user whose machine is missing a prerequisite reads the remediation guidance
and is pointed at a path that actually works. When a capture driver is absent,
the guidance mentions that the recommended analyzer's installer also provides the
driver. When the analyzer integration is not yet registered, the guidance points
at the supported registration step rather than telling the user to hand-copy a
binary, and it does not imply the analyzer is missing a feature it actually has.

**Why this priority**: These are wording corrections that improve first-run
success and stop sending users down wrong paths. They are low risk and
independent of the other stories.

**Acceptance Scenarios**:

1. **Given** a machine with no capture driver, **When** the user runs
   `fragcap doctor`, **Then** the driver-absent guidance names where to obtain
   the driver and notes that the recommended analyzer's installer also provides
   it.
2. **Given** a machine where the analyzer integration is not registered, **When**
   the user runs `fragcap doctor`, **Then** the integration line points at the
   supported registration step (a forthcoming install command) rather than a
   manual file copy, and does not suggest the analyzer lacks the integration
   framework.

---

### Edge Cases

- The machine genuinely has no usable interfaces: the report warns, truthfully,
  that none were found.
- The running build does not include the live capture capability: the report
  attributes the empty interface list to the missing capability, not to a
  driver or adapter fault.
- The loopback state cannot be determined: reported as undetermined, never
  collapsed into "not installed".
- The environment requests no color, or the stream is not an interactive
  terminal: no color is emitted.
- A per-user path cannot be resolved on the platform: the line says the location
  is undetermined rather than printing an empty or wrong path.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Interfaces section MUST list each capture-capable interface
  present on the machine, with its name, a representative address when it has
  one, and its up or down state, and MUST mark an interface judged virtual.
- **FR-002**: The "no interfaces were found" warning MUST appear only when
  interface enumeration genuinely returns an empty set. When the live capture
  capability is absent, the message MUST attribute the empty set to that missing
  capability.
- **FR-003**: The loopback check MUST reflect the real loopback adapter state as
  one of supported, not supported, or undetermined, and MUST NOT derive that
  state from an unrelated installed component. An undetermined state MUST render
  as a non-blocking warning that says the state could not be determined, never as
  a blocking failure and never as an assertion that loopback is not installed.
- **FR-004**: The report MUST begin with an identity section stating the fragcap
  version, the absolute path of the running binary, the user profile directory,
  and the default hint-database path. Each path MUST be reported regardless of
  whether the target already exists; a path that cannot be resolved on the
  platform MUST be reported as undetermined. Identity and paths rows MUST be
  informational only and MUST NOT change the command's exit status.
- **FR-005**: The machine-readable output MUST remain one record per check, with
  the identity facts represented as ordinary check records rather than a
  separate object.
- **FR-006**: When writing to an interactive terminal, the human output MUST
  color each status word by severity and MUST visually separate each section.
- **FR-007**: The human output MUST NOT emit color control codes when the output
  stream is not an interactive terminal, when the no-color environment signal is
  set, or for the machine-readable form.
- **FR-008**: On the default set of checks, no line of human output MUST overflow
  a standard-width terminal; guidance longer than one line MUST be presented as
  an indented continuation.
- **FR-009**: The driver-absent guidance MUST name the official driver source and
  MUST note that the recommended analyzer's installer also provides the driver.
- **FR-010**: The analyzer-integration guidance, when the integration is not
  registered, MUST point at the supported registration step rather than a manual
  binary copy, and MUST NOT imply the analyzer lacks the integration framework.
- **FR-011**: The command's exit status MUST continue to distinguish a
  capture-ready machine from one with a blocking problem, unchanged by the
  presentation and identity additions.

### Key Entities *(include if feature involves data)*

- **Readiness report**: the ordered set of checks the command produces, grouped
  into sections (now including a leading identity section), each check carrying a
  name, a human detail, a severity, and optional remediation.
- **Interface record**: one network interface as the machine describes it, with a
  name, a representative address, an up or down state, and a virtual verdict.
- **Loopback state**: a three-valued judgement (supported, not supported,
  undetermined) about whether loopback capture is available.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a correctly configured machine with adapters and the live
  capture capability, 100 percent of `fragcap doctor` runs list the real
  interfaces and none warn that no interfaces were found.
- **SC-002**: On a machine with loopback capture support installed, the loopback
  line reports it as supported regardless of which other optional capture
  components were selected at install time.
- **SC-003**: Every `fragcap doctor` report identifies the producing version and
  the running binary path without the user running any additional command.
- **SC-004**: In a standard-width terminal, no line of the default report wraps,
  and status words are visually distinguishable by color; piped or no-color
  output is byte-for-byte plain.
- **SC-005**: The machine-readable output continues to yield exactly one record
  per check, and no downstream consumer of the record stream breaks.

## Assumptions

- The existing interface enumeration and driver-detection capabilities are
  correct and are consumed as-is; this slice does not modify how interfaces or
  driver state are discovered, only whether the report uses them.
- Interface enumeration and driver detection are available only in builds that
  include the live capture capability; other builds present an empty interface
  set and an undetermined loopback state, and the report says so.
- The fragcap version shown in the identity section is the version compiled into
  the running binary; test fixtures supply a fixed version so golden comparisons
  do not churn on release bumps.
- The install command that the integration guidance points at is delivered by a
  later slice; this slice only updates the guidance wording to name it.
- A standard terminal width for the non-overflow criterion is 80 columns.
- Color is applied only in the presentation layer around the plain report text,
  so the machine-readable form and the fixture-compared plain form are unaffected.
