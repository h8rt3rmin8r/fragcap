# Feature Specification: MSI extcap registration, both scopes

**Feature Branch**: `043-msi-extcap-registration`

**Created**: 2026-08-14

**Status**: Draft

**Input**: Resolves the installer part of #104 (the CLI part shipped in slice 041
/ PR #110). This is the MSI option spun out of slice 041 by its research decision
D-4, whose carried-forward design governs this slice. It adds an optional
Wireshark extcap registration to the Windows installer, offering both a per-user
default and a machine-wide option, by driving the already-shipped `fragcap extcap
install` command from a WiX custom action. No change to the extcap command
semantics.

## Clarifications

### Session 2026-08-14

Resolved under autopilot from decision D-4 (`specs/041-extcap-registration/research.md`),
the existing `main.wxs` patterns, and the slice scope:

- Q: Wizard control shape? -> A: A single optional checkbox on a wizard dialog,
  per-user by default and never pre-checked into a forced state, plus a
  machine-wide path an administrator selects. The checkbox drives the per-user
  action; the machine-wide action is gated on Wireshark being detected.
- Q: Per-user registration mechanism? -> A: A deferred, user-impersonated WiX
  custom action that runs the installed `fragcap.exe extcap install`, so the
  target resolves to the installing user's profile rather than SYSTEM (D-4).
- Q: Machine-wide mechanism? -> A: A WiX RegistrySearch for the Wireshark install
  location; when found, a deferred custom action runs `fragcap.exe extcap install
  --dir <WiresharkDir>\extcap`. The CLI already supports `--dir`, so no new Rust
  surface is added.
- Q: Failure handling? -> A: Mirror the existing Defender-exclusion pattern:
  `Return="ignore"` so a registration failure never fails the install, with a
  paired rollback that unregisters if a later install step fails.
- Q: Verification here? -> A: The MSI cannot be built or install-tested in this
  environment (no WiX toolchain), exactly as D-4 states. This slice verifies the
  well-formedness and consistency of the WiX and docs and the green `cargo xtask
  ci`, and enumerates the WiX build and the per-user and machine-wide install
  tests as manual verification for the operator at the pre-push halt.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Register from the installer, for me (Priority: P1)

A user installing fragcap from the MSI opts into "register fragcap with Wireshark"
in the wizard. When the install finishes, `fragcap doctor` shows the analyzer
extcap row as `ok` for that user, without a separate manual command. If the
registration cannot complete, the install still succeeds.

**Why this priority**: This is the whole point of the slice: make the optional
integration reachable from the installer that most users run, without forcing it
and without letting it break the install. It is the per-user path D-4 makes the
default.

**Independent Test**: Build the MSI (manual), run it with the checkbox selected as
a normal user, then run `fragcap doctor`: the analyzer extcap row is `ok` and
points at that user's Wireshark extcap directory.

**Acceptance Scenarios**:

1. **Given** the MSI wizard, **When** the user selects the register option and
   completes the install, **Then** `fragcap.exe extcap install` runs impersonated
   as that user and the extcap binary lands in the user's Wireshark extcap
   directory.
2. **Given** the register option is not selected, **When** the install completes,
   **Then** nothing is registered and `doctor` shows the row as an optional
   warning with the `fragcap extcap install` guidance.
3. **Given** the register option is selected but registration fails, **When** the
   install runs, **Then** the install still succeeds (the action returns ignore)
   and a paired rollback leaves no partial registration.

### User Story 2 - Register for every user on this machine (Priority: P2)

An administrator installing for a shared machine chooses the machine-wide option.
When Wireshark is installed, the installer registers fragcap into Wireshark's
system extcap directory so every user sees it.

**Why this priority**: The operator asked for both scopes. Machine-wide is the
administrator path; it depends on Wireshark being present, so it follows the
per-user default.

**Independent Test**: Build the MSI (manual), install machine-wide on a box with
Wireshark present, then run `doctor` as a second user: the extcap row is `ok`.

**Acceptance Scenarios**:

1. **Given** Wireshark is installed, **When** the administrator selects the
   machine-wide option, **Then** the installer resolves Wireshark's system extcap
   directory and runs `fragcap extcap install --dir <that>` so all users see the
   source.
2. **Given** Wireshark is not detected, **When** the machine-wide option would
   run, **Then** it is unavailable or a no-op, and the install still succeeds.

### User Story 3 - Skip it and still know what to do (Priority: P3)

A user who leaves the option unselected sees, in the installer and the docs, that
they can register later by running `fragcap extcap install`, and that the
per-user registration is for the current user only.

**Why this priority**: The integration is optional; a user who declines must not
be stranded. This is documentation and installer text, low risk.

**Independent Test**: Read the installer dialog text and the docs: both state the
per-user scope and the `fragcap extcap install` fallback.

**Acceptance Scenarios**:

1. **Given** the wizard dialog, **When** the register option is shown, **Then**
   its text states the registration is per user by default and names the
   `fragcap extcap install` fallback.
2. **Given** the documentation, **When** a reader looks up the installer,
   **Then** the optional extcap registration and its both-scope behavior are
   documented.

### Edge Cases

- A registration failure (Wireshark absent for per-user, a locked directory,
  Tamper-style refusal) must never fail the install; it returns ignore, like the
  Defender exclusion.
- An uninstall should not leave a dangling extcap registration for the per-user
  scope; unregister on uninstall where the scope allows, mirroring the rollback.
- The machine-wide option must degrade cleanly when Wireshark is not installed
  (the system extcap directory does not exist).
- main.wxs is release-adjacent and pinned: the change carries a dated changelog
  decision, and the release workflow and `release.toml` must still reference the
  same `main.wxs` consistently.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The installer MUST present an optional control to register fragcap
  as a Wireshark extcap source. It MUST NOT force registration; the default is
  per-user and opt-in, never a silent machine change.
- **FR-002**: Per-user registration MUST run as a deferred, user-impersonated
  custom action invoking the installed `fragcap.exe extcap install`, so the
  target is the installing user's Wireshark extcap directory, not SYSTEM.
- **FR-003**: The installer MUST offer a machine-wide option that, when Wireshark
  is detected via a registry search, runs `fragcap.exe extcap install --dir
  <WiresharkInstallDir>\extcap`.
- **FR-004**: A registration custom action MUST use `Return="ignore"` so a
  failure never fails the install, and MUST have a paired rollback that
  unregisters on a later failure, mirroring the Defender-exclusion pattern.
- **FR-005**: The installer text MUST state that per-user registration is for the
  current user only, and MUST name `fragcap extcap install` as the way to
  register later.
- **FR-006**: The change MUST NOT add or alter any `fragcap` CLI surface; it
  drives the existing `extcap install` and its `--dir` flag only.
- **FR-007**: The CLI/installer documentation and the getting-started guide MUST
  be updated to describe the installer's optional extcap registration and its two
  scopes, keeping the slice 042 single-sourced dependency model (extcap stays the
  optional tier; `doctor` still only warns on it).
- **FR-008**: The change MUST add a dated `changelog.d/<key>.decisions.md`
  fragment (main.wxs is release-adjacent and pinned) plus a `.added.md` feature
  fragment.
- **FR-009**: `main.wxs` MUST remain well-formed and consistent with the release
  workflow and `release.toml`, and all added or edited text MUST be UTF-8, LF,
  and free of em and en dashes.

### Key Entities *(include if feature involves data)*

- **Register-extcap wizard control**: the optional installer control; per-user
  default, machine-wide for administrators.
- **Per-user registration action**: deferred, impersonated, runs `extcap
  install`, ignore-on-failure, with rollback.
- **Machine-wide registration action**: gated on a Wireshark registry search,
  runs `extcap install --dir <detected>\extcap`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With the option selected during a normal-user MSI install, `fragcap
  doctor` reports the analyzer extcap row as `ok` for that user (manual, at the
  halt).
- **SC-002**: With the machine-wide option and Wireshark present, a second user
  on the machine sees the extcap row as `ok` (manual, at the halt).
- **SC-003**: A registration failure leaves the install successful and no partial
  registration behind.
- **SC-004**: With the option unselected, the installer registers nothing and
  both the installer and the docs state the `fragcap extcap install` fallback and
  the per-user scope.
- **SC-005**: `cargo xtask ci` is green; `main.wxs` parses as well-formed XML and
  the release workflow and `release.toml` still reference it consistently.
- **SC-006**: No `fragcap` CLI surface changed.

## Assumptions

- The MSI cannot be built or install-tested in this environment (no WiX
  toolchain), as decision D-4 states. SC-001, SC-002, and SC-003 are verified
  manually by the operator at the pre-push halt; this slice verifies the WiX and
  documentation consistency and `cargo xtask ci`.
- The already-shipped `fragcap extcap install` command and its `--dir` flag are
  the registration mechanism; the installer only invokes them, so no Rust CLI
  change is needed (a small internal helper or test seam is permitted if it adds
  no user-facing surface).
- Wireshark's install location is discoverable from the Windows registry for the
  machine-wide path; when it is absent, the machine-wide path is a clean no-op.
- Code signing (#79) is out of scope and remains a separate track; the installer
  stays unsigned.
