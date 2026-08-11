# Feature Specification: Extcap analyzer integration

**Feature Branch**: `021-extcap-integration`

**Created**: 2026-08-11

**Status**: Draft

**Input**: Roadmap slice S18 sub-slice A (specification section 14.5, analyzer
integration). Deliver the extcap interface so an external analyzer (Wireshark and
compatible tools) can enumerate, configure, and start fragcap as a capture
source, presenting a full native configuration dialog without fragcap containing
any graphical code. Replace the current `fragcap extcap` stub that names this
slice. Extend `fragcap doctor` to report whether the binary is installed in the
analyzer's extcap directory and where that directory is.

## Clarifications

### Session 2026-08-11

Resolved under autopilot from the constitution, the architecture of record
(specification sections 14.1 to 14.5, 12.4, 13.3, and 17.2), and the existing
sink, pipeline, and command-surface contracts. No item required an operator call.

- Q: How many extcap interfaces does fragcap present, and what is the declared
  link type? -> A: One logical extcap interface named `fragcap`. fragcap is a
  single process-attributed capture source whose subject is chosen by the
  profile and role options, not by a host network adapter, so a single extcap
  interface keyed by those options matches the contract better than one entry per
  adapter. Its declared link type is Ethernet (the primary link type fragcap
  emits); a loopback conversation still carries its own Interface Description
  Block and link type inside the pcapng stream, exactly as the file writer
  already produces, so the top-level declaration is the default and the analyzer
  reads the per-packet interface from the stream.
- Q: What materializes the FIFO stream, and is it a new capture path? -> A: A new
  FIFO sink in `fragcap-sink` that opens the analyzer-supplied path for writing
  and builds a pcapng encoder over it through the existing `SinkFactory` seam
  (the same seam the file, rotating, and streaming sinks use). On Windows the
  path is a named pipe the analyzer created and fragcap connects to as a client;
  on other targets it is a FIFO opened for writing. It is a transport target, not
  a new capture path: the capture pipeline, the parser, and the attributor are
  unchanged, so P-1, P-3, and P-9 hold by construction.
- Q: What are the configurable options, and how are they applied? -> A: Exactly
  four, the ones specification 14.5 names: profile selection, role filter,
  direction filter, and loopback inclusion. `--extcap-config` declares them in
  the extcap argument grammar so the analyzer renders the dialog; at capture the
  analyzer passes the chosen values back and fragcap applies them through the
  same resolution the `run` command uses (an explicit value over the profile's
  `[capture]` default). No new capture option is introduced.
- Q: Is the extcap capture verifiable without a capture driver or an analyzer?
  -> A: Yes, at tier 1. In production the extcap capture is a live capture the
  analyzer starts, but the capture assembly is the same one `run` drives, so a
  test reaches it through the existing hidden offline substrate (a recorded
  capture replayed as the source) writing to a FIFO the test reads back with the
  same pcapng parser the writer tests use. Live execution stays tier 2 and
  unexecuted, consistent with S09.
- Q: How does doctor learn the extcap directory and installation state? -> A: The
  doctor probe reads the analyzer's personal extcap directory read-only and
  reports whether a fragcap binary is present there and the directory path. It
  installs nothing and copies nothing, which is the Licensing rule and P-1 made
  mechanical: detection only. Installation remains an operator action (copy the
  binary into the reported directory).
- Q: What happens when the analyzer stops reading the FIFO, or the path cannot be
  opened? -> A: A FIFO that cannot be opened is a run failure that names the
  path, before any capture is reported as started, so nothing is claimed written
  to a stream that never existed. An analyzer that closes the FIFO mid-capture
  breaks the write; that ends the capture as a clean stop (the single consumer is
  gone), and the pipeline's conservation accounting still balances, so no packet
  is silently lost (P-4, P-9).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An analyzer enumerates and starts fragcap (Priority: P1)

An analyst opens their capture tool, sees fragcap listed as a capture source,
picks it, and starts a capture. Behind that the analyzer runs `fragcap extcap`
four ways: it asks for the interface list, the link types, and the configurable
options, renders a native dialog from those declarations, and then starts the
capture by handing fragcap a FIFO path. fragcap streams pcapng to that FIFO and
the analyzer displays the live, process-attributed capture, with no graphical
code in fragcap.

**Why this priority**: This is the whole capability of the slice and the exact
contract specification 14.5 defines. Without the four invocations and the FIFO
stream, extcap does not exist.

**Independent Test**: Run each declaration invocation and assert its output is
accepted by an extcap control-grammar parser; drive `--capture --fifo` through
the hidden offline substrate replaying a fixture, read the FIFO back, and confirm
it is a valid pcapng reproducing the committed golden for that fixture, with no
analyzer and no capture driver present.

**Acceptance Scenarios**:

1. **Given** `fragcap extcap --extcap-interfaces`, **When** it is run, **Then**
   it prints the single `fragcap` extcap interface in the extcap control grammar,
   and exits 0.
2. **Given** `fragcap extcap --extcap-dlts --extcap-interface fragcap`, **When**
   it is run, **Then** it prints the declared link type for that interface in the
   extcap grammar, and exits 0.
3. **Given** `fragcap extcap --extcap-config --extcap-interface fragcap`,
   **When** it is run, **Then** it declares the four configurable options
   (profile, roles, direction, loopback) in the extcap argument grammar, and
   exits 0.
4. **Given** `fragcap extcap --capture --fifo <path>` with a source to capture,
   **When** it runs, **Then** it streams a single valid pcapng to `<path>`
   beginning with a Section Header Block and one Interface Description Block per
   declared interface, followed by the captured packets carrying the same
   attribution comments the file sink writes.

---

### User Story 2 - The dialog options select what is captured (Priority: P1)

The analyst fills in the native dialog: they pick a profile, narrow to a role,
scope to inbound traffic, and include loopback. The analyzer passes those choices
back to fragcap when it starts the capture, and fragcap applies them exactly as
the equivalent `run` flags: the chosen profile is resolved, the selected role
scopes which stages trigger, the direction is accepted and carried, and loopback
is included. The extcap dialog is parity with the `run` command line, whatever
each flag does there (output direction filtering itself is a later slice,
specification FR-011b, for `run` and extcap alike).

**Why this priority**: A dialog whose options do not actually change the capture
is the configuration-side form of the loss the project forbids: the capture runs,
exits zero, and does not contain what the operator asked for. The options
declared through extcap must be the same options `run` honors.

**Independent Test**: Assert `--extcap-config` declares exactly the four options
with call names that map to the capture options; then drive a capture carrying
each option value through the offline substrate and confirm the resulting stream
matches the equivalent `run` invocation (same profile resolution, role scope,
direction scope, loopback inclusion).

**Acceptance Scenarios**:

1. **Given** an extcap capture carrying a profile selection, **When** it runs,
   **Then** the profile resolves and validates by the same path `run --profile`
   uses, and a resolution failure is reported as a configuration error rather
   than a started-but-empty capture.
2. **Given** an extcap capture carrying a role filter, a direction filter, and
   loopback inclusion, **When** it runs, **Then** the captured stream is scoped
   exactly as the equivalent `run --roles`, `run --direction`, and
   `run --loopback` invocation, over the same input.

---

### User Story 3 - doctor reports extcap installation (Priority: P2)

An operator wants to make fragcap available in their analyzer. They run
`fragcap doctor`, which tells them whether the fragcap binary is installed in the
analyzer's extcap directory and, either way, names that directory so they know
where to copy it.

**Why this priority**: The installation is a manual copy, and the operator needs
to be told the target directory and whether the copy has been made. It is a
readiness report, subordinate to the capability itself, hence P2.

**Independent Test**: Classify a doctor input with a fragcap binary present in
the extcap directory and assert it reports installed with the path; classify one
with the binary absent and assert it reports not installed with the same path,
with the probe reading the filesystem read-only and installing nothing.

**Acceptance Scenarios**:

1. **Given** a fragcap binary present in the analyzer's extcap directory, **When**
   `fragcap doctor` runs, **Then** it reports the extcap integration as installed
   and names the directory.
2. **Given** no fragcap binary in the analyzer's extcap directory, **When**
   `fragcap doctor` runs, **Then** it reports the extcap integration as not
   installed and names the directory where the binary belongs.

---

### Edge Cases

- The FIFO path cannot be opened (the analyzer has already closed it, or the
  named pipe is not ready): the capture fails with an error naming the FIFO,
  before it reports a started capture, so nothing is claimed to have been written
  to a stream that never opened.
- The analyzer closes the FIFO mid-capture (the analyst quits the tool): the
  write end breaks, the capture ends as a clean stop because its single consumer
  is gone, and the pipeline conservation accounting still balances; no packet is
  counted as captured-then-lost without a counter.
- A declaration invocation names an interface fragcap does not present (a stale
  or malformed `--extcap-interface`): it is a usage error naming the unknown
  interface, not an empty declaration that the analyzer would render as a working
  but inert source.
- `--capture` without `--fifo`, or a declaration invocation missing its required
  `--extcap-interface` selector: a usage error (exit 2) naming the missing
  argument, before any capture starts.
- A newer analyzer passes the standard protocol version flag: fragcap accepts the
  extcap protocol flags it participates in (the version query and the interface
  selector) rather than rejecting the invocation as unknown.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: `fragcap extcap --extcap-interfaces` MUST print the available
  fragcap extcap interfaces in the extcap control grammar an analyzer parses, and
  exit 0.
- **FR-002**: `fragcap extcap --extcap-dlts` MUST print the link type(s) for the
  selected extcap interface in the extcap grammar, and exit 0.
- **FR-003**: `fragcap extcap --extcap-config` MUST declare the configurable
  options in the extcap argument grammar, so the analyzer renders a native
  configuration dialog with no graphical code in fragcap. The declared options
  MUST be exactly profile selection, role filter, direction filter, and loopback
  inclusion (specification 14.5).
- **FR-004**: `fragcap extcap --capture --fifo <path>` MUST stream the capture as
  pcapng to the named pipe or FIFO at `<path>`, reusing the pcapng writer, so an
  unmodified analyzer reads the live stream (constitution P-5).
- **FR-005**: The extcap pcapng stream MUST be the same bytes the file sink
  produces for the same capture, carrying the same header blocks and the same
  attribution comments, differing only in transport. A single-interface extcap
  stream MUST be record-comparable to a plain file capture of the same input.
- **FR-006**: The configurable options the analyzer passes back at capture
  (profile, roles, direction, loopback) MUST be applied through the same
  resolution the `run` command uses (an explicit value over the profile's
  `[capture]` default), so the extcap dialog and the `run` flags select capture
  identically.
- **FR-007**: `fragcap extcap` MUST accept the standard extcap protocol flags an
  analyzer sends, the version query and the `--extcap-interface <name>` selector,
  and use the named interface for the dlts, config, and capture invocations.
- **FR-008**: An extcap invocation missing a required argument (`--capture`
  without `--fifo`, or a declaration invocation without its required
  `--extcap-interface` selector) MUST be reported as a usage error (exit 2)
  naming the missing argument, before any capture starts.
- **FR-009**: `fragcap doctor` MUST report whether a fragcap binary is installed
  in the analyzer's extcap directory and MUST name that directory, reading the
  filesystem read-only and installing, downloading, or copying nothing
  (constitution P-1, the Licensing rule).
- **FR-010**: extcap MUST introduce no new capture or attribution technique; the
  capture is the existing pipeline with a FIFO sink attached (constitution P-1,
  P-3, P-9). The `extcap` command MUST replace the current stub that reports the
  command unimplemented and names this slice.
- **FR-011**: Every discarded packet on the extcap path MUST be counted and
  surfaced by the same pipeline conservation accounting a file or streaming
  capture uses; the extcap FIFO sink advances no uncounted discard (constitution
  P-4).
- **FR-012**: Any term this slice introduces (extcap, DLT and link type, named
  pipe and FIFO) MUST receive a glossary entry in the same change (constitution
  P-6).

### Key Entities *(include if data involved)*

- **Extcap interface**: The logical capture source fragcap presents to an
  analyzer, named `fragcap`, declared by `--extcap-interfaces`. One interface,
  keyed by the profile and role options rather than by a host adapter.
- **DLT (link type)**: The link type declared for the extcap interface by
  `--extcap-dlts`. The top-level default is Ethernet; per-packet link types are
  carried by the stream's Interface Description Blocks.
- **Extcap config option**: One of the four options declared by `--extcap-config`
  (profile, roles, direction, loopback) that the analyzer renders as a dialog
  field and passes back at capture.
- **FIFO sink**: The output sink that opens the analyzer-supplied FIFO or named
  pipe path for writing and streams pcapng to it through the existing
  `SinkFactory`, unchanged from the file writer above the transport seam.
- **Extcap directory**: The analyzer's directory into which the fragcap binary is
  copied to register it as a capture source; reported by `doctor`, read-only.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Each of the three declaration invocations (`--extcap-interfaces`,
  `--extcap-dlts`, `--extcap-config`) produces output accepted by an extcap
  control-grammar check, verified by tests with no analyzer installed.
- **SC-002**: A `--capture --fifo` run driven by the offline substrate produces a
  byte stream an unmodified pcapng parser reads in full, reproducing the
  committed pcapng golden for that fixture.
- **SC-003**: The four options declared by `--extcap-config` are exactly profile,
  roles, direction, and loopback, and each, when passed at capture, changes the
  capture as the equivalent `run` flag does over the same input.
- **SC-004**: `fragcap doctor` reports the extcap integration as installed with
  the directory path when a binary is present there, and as not installed with
  the same path when it is absent.
- **SC-005**: Every extcap misuse (missing `--fifo`, missing or unknown
  `--extcap-interface`) exits 2 with a message naming the cause, and no capture
  is started.
- **SC-006**: The pipeline conservation invariant (received plus buffer-dropped
  plus refusals equals captured) holds for an extcap capture exactly as for a
  file capture.
- **SC-007**: The full repository gate (`cargo xtask ci`) passes and the
  platform-neutral core build (`cargo xtask neutral`, which `ci` does not run)
  still builds, with the extcap path covered by tests that run with no capture
  driver, no elevation, and no analyzer.

## Assumptions

- The extcap capture in production is a live capture the analyzer starts. The
  slice is verified at tier 1 by driving the same capture assembly `run` uses
  through the existing hidden offline substrate, writing to a FIFO a test reads
  back. Live execution stays tier 2 and unexecuted, as it has since S09.
- The FIFO is opened for writing at the path the analyzer supplies. On Windows
  that is a named pipe the analyzer created and fragcap connects to as a client;
  on other targets it is a FIFO opened for writing. The sink reuses `SinkFactory`
  over the opened handle and adds no new format code.
- Only pcapng is streamed over extcap, because analyzers consume pcapng. JSON
  Lines and rotation are not extcap transports.
- The extcap directory doctor reports is the analyzer's personal extcap
  directory. Detection is best-effort and read-only; the operator performs the
  copy. doctor already carries an `extcap_installed` input classified by the
  environment report; this slice makes it real and adds the directory path.
- The pcapng writer, `SinkFactory`, and the capture pipeline from S06, S08, and
  S15 are reused unchanged; extcap adds a FIFO sink and a command, and does not
  modify the byte-level format output or the capture and attribution engines.
