# Feature Specification: Steam integration and managed launch

**Feature Branch**: `020-steam-integration`

**Created**: 2026-08-10

**Status**: Draft

**Roadmap slice**: S17 (spec section 16; depends on S05, S12; gated by Q-4, resolved)

**Input**: User description: "S17 - Steam integration and managed launch. Fill in the
fragcap-steam crate and wire its two command surfaces: library discovery, profile
scaffolding (`fragcap steam profile <app_id>`), and managed launch (`fragcap run
--launch`). Section 16.5 environment inheritance is deferred. The crate carries no
capture and no attribution logic."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Scaffold a profile for an installed Steam title (Priority: P1)

An operator wants to capture a game acquired through Steam but has no profile for it.
They run `fragcap steam profile <app_id>` and receive a profile skeleton pre-populated
with the title's platform, application identifier, and a first heuristic guess at its
launcher and client stages, ready to edit against a real observed session.

**Why this priority**: This is the slice's centre of gravity and the reason the crate
exists. Authoring a profile from scratch against a five- or six-level launch chain is
the hardest first step for a new title; scaffolding turns it into an edit. It is
independently valuable even if managed launch never ships, and it is fully testable
offline against a fixture Steam library.

**Independent Test**: Point the scaffolder at a fixture Steam library directory
containing a known application manifest and install directory, run the scaffold, and
assert the emitted profile (a) names the correct platform and app_id, (b) proposes a
plausible client stage, and (c) passes section 15.4 validation unedited.

**Acceptance Scenarios**:

1. **Given** a Steam library containing an installed title with app_id `900883`,
   **When** the operator scaffolds a profile for `900883`, **Then** the output is a
   profile declaring `game.platform = "steam"` and `game.app_id = "900883"` that passes
   validation, and carries a header comment stating the stage classification is
   heuristic and must be verified against an observed session.
2. **Given** an install directory containing one launcher-named executable and one
   larger non-launcher executable, **When** the operator scaffolds, **Then** the
   launcher-named image is proposed as a launcher stage and the largest non-launcher
   image is proposed as the client stage.
3. **Given** an app_id that is not installed in any discovered library, **When** the
   operator scaffolds, **Then** the command fails with an error naming the app_id and
   stating it was not found, and writes no profile.

---

### User Story 2 - Launch a title under capture without the acquisition race (Priority: P2)

An operator has a validated profile with a platform and app_id. They run `fragcap run
--profile <ref> --launch`. fragcap attaches the process watcher and opens the capture
handle first, then starts the title through Steam, so every process in the launch chain
- including a launcher whose whole lifetime is shorter than a poll interval - produces a
start event fragcap observes.

**Why this priority**: This is the payoff of ordering: it removes the acquisition race
the roadmap called out. It depends on the profile machinery of Story 1 and on the
session lifecycle already shipped in S12, so it is second. The wiring and its
configuration validation are testable offline; the physical launch is tier 2.

**Independent Test**: With a profile that declares platform and app_id, assert that the
assembled run configuration requests a managed launch of the correct app_id and that the
launch is sequenced after watcher attach and capture-handle open (asserted against the
run assembly / ordering, not a live Steam process).

**Acceptance Scenarios**:

1. **Given** a profile declaring `game.platform = "steam"` and `game.app_id`, **When**
   the operator runs with `--launch`, **Then** the run is configured to start that title
   through Steam after the watcher is attached and the capture handle is open.
2. **Given** a profile that declares no `game.platform` or no `game.app_id`, **When** the
   operator runs with `--launch`, **Then** the command is refused as a configuration
   error naming the missing key, before any capture starts.

---

### User Story 3 - Enumerate installed Steam titles (Priority: P3)

The scaffolder needs to resolve an app_id to an install directory, which requires
discovering every Steam library and every title installed across them. This discovery is
the shared substrate beneath Story 1 and is exercised as its own capability.

**Why this priority**: It is a dependency of Story 1 rather than a headline command, so
it ranks third, but it is the piece with the most parsing surface (registry lookup,
library-folders manifest, per-title application manifests in Valve key-value format) and
carries its own tests.

**Independent Test**: Given a fixture Steam root with a library-folders manifest pointing
at two libraries, each holding application manifests, assert discovery returns every
installed title with its app_id and resolved install path, across both libraries.

**Acceptance Scenarios**:

1. **Given** a Steam installation with two configured libraries, **When** discovery runs,
   **Then** it returns every installed title from both libraries with app_id and install
   path.
2. **Given** a library-folders manifest or an application manifest that is malformed,
   **When** discovery runs, **Then** the malformed entry is reported and skipped rather
   than aborting discovery of the well-formed entries.

---

### Edge Cases

- **Steam not installed / registry key absent**: discovery reports that no Steam
  installation was found and the dependent command fails cleanly, naming the cause; it
  does not panic and does not attempt to install anything.
- **Install directory holds no plausible client executable** (no non-launcher image):
  scaffolding still emits a profile, proposing the largest available executable as the
  client and stating in the header comment that the guess is weak, rather than emitting
  nothing.
- **Every executable in the install directory carries a launcher-suggestive token**: at
  least one stage is still proposed as the client (the largest), so the profile is never
  launcher-only.
- **Duplicate app_id across libraries**: the first discovered install wins and the
  collision is reported, so the result stays deterministic.
- **`--launch` on a non-Windows build**: refused as unsupported on this platform, named
  before capture starts, consistent with the crate being Windows-only.
- **VDF nested quoting / escapes / comments**: the parser handles the subset Valve
  actually emits in these manifests and reports, rather than silently mis-parsing, input
  it does not understand.

## Clarifications

### Session 2026-08-10

Resolved under autopilot from the specification (section 16), the constitution, the
architecture of record, and the resolved Q-4 findings. Recorded here; plan-level
implementation choices are listed as Deferred below.

- Q: Is section 16.5 (environment inheritance) implemented in this slice? → A: No -
  deferred. It requires a process handle with memory-read rights, forbidden by the
  constitution's technique denylist and the `OpenProcess` lint; it is a corroborating
  signal only, and section 10 ancestry already attributes reliably, so deferring it
  costs no capability. (See FR under "Deferred".)
- Q: Where does `fragcap steam profile <app_id>` write the scaffolded profile? → A: To
  standard output. A scaffold is a starting point the operator reviews and redirects;
  emitting to stdout keeps the command side-effect-free and leaves file placement to the
  operator. On the not-found error path it writes nothing (FR-009).
- Q: Does scaffolding emit process-ancestry (`descends_from`) predicates, or image-name
  matches? → A: Image-name (`exe`) matches, with `path_contains`/`path_regex` added only
  where two proposed stages would otherwise share a basename. Runtime process topology
  (the Div2 case where three processes share one image name) is not visible from a static
  install-directory scan, so ancestry cannot be inferred at scaffold time; the heuristic
  header comment and the existing section 15.4 runtime warning cover that case. The
  scaffolder MUST nonetheless guarantee the emitted profile passes the section 15.4
  ambiguous-image-match check.

**Deferred to planning** (implementation choices, not spec ambiguities): the
registry-access and protocol-handler mechanism and any dependency it implies (for
example `winreg` versus reusing `windows-sys` already in the graph); the platform-gating
strategy (a `cfg`-gated crate body versus a non-Windows stub) satisfying FR-014; and the
exact Steam protocol-handler URL form used for managed launch.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST locate the Steam installation through its Windows registry
  entry and, from it, read the library-folders manifest to enumerate every configured
  Steam library.
- **FR-002**: The system MUST read every application manifest across every discovered
  library and produce the set of installed titles, each with its application identifier
  and resolved installation directory.
- **FR-003**: The system MUST parse Valve's key-value (VDF) text format with a parser
  contained in the crate, covering the subset these manifests use, without taking a
  dependency for it.
- **FR-004**: A malformed library entry or application manifest MUST be reported and
  skipped, leaving discovery of the remaining well-formed entries intact.
- **FR-005**: The `fragcap steam profile <app_id>` command MUST generate a profile
  skeleton for an installed title, declaring the title's platform and application
  identifier.
- **FR-006**: Profile scaffolding MUST scan the title's installation directory for
  executable images and classify them heuristically: an image whose file name or path
  carries a launcher-suggestive token is proposed as a launcher stage; the largest
  executable image not classified as a launcher is proposed as the client stage. Stage
  match rules are emitted as image-name (`exe`) predicates and MUST NOT infer runtime
  ancestry (`descends_from`) from a static scan; where two proposed stages would share a
  basename, the scaffolder MUST add a `path_contains`/`path_regex` predicate so the
  output satisfies the section 15.4 ambiguous-image-match check (FR-008).
- **FR-007**: A scaffolded profile MUST carry a header comment stating that the stage
  classification is heuristic and requires verification against an observed session.
- **FR-008**: A scaffolded profile MUST pass section 15.4 validation unedited, so it is
  immediately usable and immediately correctable.
- **FR-009**: Scaffolding for an app_id not present in any discovered library MUST fail
  with an error naming the app_id, and MUST NOT write a profile.
- **FR-010**: The `fragcap run --profile <ref> --launch` command MUST start the title
  through Steam's protocol handler, and MUST do so only after the process watcher is
  attached and the capture handle is open.
- **FR-011**: Managed launch MUST require `game.platform` and `game.app_id` in the
  profile; if either is absent, `--launch` MUST be refused as a configuration error
  naming the missing key, before capture starts.
- **FR-012**: The crate MUST contain no capture logic and no attribution logic; core
  MUST retain no notion of Steam.
- **FR-013**: The two previously stubbed command surfaces - the `fragcap steam` stub and
  the `--launch` "deferred to S17" refusal - MUST be replaced by the real
  implementations, with no stub refusal path left reachable for the shipped capabilities.
- **FR-014**: The workspace MUST continue to build for the neutral non-Windows target;
  the Windows-only Steam integration MUST be platform-gated so it does not break that
  build.
- **FR-015**: fragcap MUST NOT bundle, download, or install Steam or any Steam component;
  it reads local Steam metadata and invokes the already-installed protocol handler only.

### Deferred (recorded, not implemented in this slice)

- **Section 16.5 environment inheritance** is deferred. Reading another process's
  environment block on Windows requires a process handle carrying memory-read rights,
  which the constitution's technique denylist and the `OpenProcess` lint forbid
  project-wide. It is explicitly a corroborating signal and never the primary mechanism,
  because section 10 ancestry already attributes reliably; deferring it costs no
  capability. See the clarification record and the plan's decision log.

### Key Entities

- **Steam library**: a filesystem location Steam installs titles into; discovered from
  the library-folders manifest. Zero or more per installation.
- **Installed title**: a game Steam has installed, identified by an application
  identifier and a resolved installation directory. The unit discovery yields and
  scaffolding consumes.
- **Application manifest**: Steam's per-title record (Valve key-value text) naming the
  app_id, install directory name, and state; the source of an installed title's
  identity.
- **Executable image**: a candidate stage found by scanning an install directory, carrying
  a file name, path, and size, classified as launcher or client by the scaffolding
  heuristic.
- **Scaffolded profile**: the profile skeleton emitted for an installed title, valid
  under section 15.4 and marked heuristic.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: For an installed title, `fragcap steam profile <app_id>` produces a profile
  that passes section 15.4 validation with zero manual edits.
- **SC-002**: Given a Steam installation with more than one library, discovery returns
  every installed title from every library, with no title dropped and no library missed.
- **SC-003**: For an install directory containing a launcher-named image and a larger
  non-launcher image, the scaffolder proposes the non-launcher image as the client stage
  in 100% of such cases.
- **SC-004**: `fragcap run --launch` against a profile missing platform or app_id is
  refused before any capture begins, with a message that names the missing key; it never
  starts a capture it cannot manage.
- **SC-005**: The VDF parser and the scaffolding classifier are covered by unit tests that
  run offline with no Steam installed, and `cargo xtask ci` passes.
- **SC-006**: The workspace builds for the neutral non-Windows target with the Steam
  integration present.

## Assumptions

- "Users" are operators running the fragcap CLI on Windows; there is no end-user GUI.
- The Steam installation is discoverable through its standard Windows registry entry; a
  non-standard or portable Steam layout is out of scope for this slice and would be
  reported as "not found".
- The VDF parser targets the subset of Valve key-value syntax these specific manifests
  use (nested quoted key-value blocks), not the whole binary/text VDF universe.
- Section 15.2 stage roles (launcher, client, and the rest) already exist and the
  scaffolder emits against that existing schema; S12 stage-ancestry matching is already
  shipped and is not re-implemented here.
- The physical act of launching a title through Steam and observing the process chain is
  a tier-2 / manual verification; this slice asserts the launch is correctly *configured
  and sequenced*, not that a live Steam process was spawned in CI.
- Platform-gating strategy (cfg-gated crate versus a stub on non-Windows) is an
  implementation choice deferred to the plan; either satisfies FR-014.
- The registry-access and protocol-handler mechanism (which specific API/dependency) is
  an implementation choice deferred to the plan; the spec constrains only that no new
  process handle with memory-read rights is opened and that nothing is bundled.
