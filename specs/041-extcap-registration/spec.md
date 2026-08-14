# Feature Specification: extcap registration

**Feature Branch**: `041-extcap-registration`

**Created**: 2026-08-14

**Status**: Draft

**Input**: GitHub issue #104. Give operators a supported way to register fragcap
as a Wireshark extcap capture source, and to unregister it, without hand-copying
a binary; and offer the same registration from the Windows installer.

## Clarifications

### Session 2026-08-14

Resolved under autopilot from the constitution, existing code (the per-user
extcap directory the readiness check already probes), and the slice scope.

- Q: Does registration require elevation? -> A: No. The default target is the
  per-user Wireshark extcap directory, which is writable without an elevated
  session. A location that does require elevation (an explicitly overridden
  system directory) surfaces the write failure per FR-008 rather than silently
  succeeding.
- Q: When fragcap is already registered, does register skip or refresh the
  binary? -> A: Refresh. Register always writes the currently running binary,
  overwriting any existing registration, so the registered extcap always matches
  the running fragcap. "Idempotent" means it ends in the registered state and
  reports success whether or not a prior registration existed.
- Q: Is the override a directory or a file path? -> A: A directory. The binary
  name inside it is fixed to the name the readiness check probes; the override
  changes only which directory is targeted.

## Delivery note (2026-08-14)

On operator direction, this slice delivers the CLI command and its documentation
only. The Windows installer option (User Story 3, FR-007, SC-004) is split into a
dedicated follow-up slice so it gets a real WiX build and a multi-user install
test, and so it can add a proper installer checkbox, an at-install note that the
registration is per user, and the guidance to run `fragcap extcap install`
otherwise. The chosen model for that slice is to offer both scopes: per-user by
default, with a documented machine-wide option for administrators (register into
Wireshark's system extcap directory). The per-user default is already reachable
from the command, and the machine-wide path is reachable now via `--dir`; the CLI
reference documents both.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Register fragcap with one command (Priority: P1)

An operator who wants to drive fragcap from inside Wireshark runs a single
command to register it as an extcap capture source. They do not have to know
where Wireshark looks for extcaps, and they do not have to copy a binary out of
one program directory into another. After the command, `fragcap doctor` confirms
the integration is in place.

**Why this priority**: This is the whole point of the slice and the direct fix
for the issue. Today the only path is a manual, Defender-suspicious file copy
that a doctor message pointed at; this replaces it with a supported command.

**Independent Test**: Run the register command against a scratch directory, then
confirm the binary is present there and that `fragcap doctor`, pointed at the
same directory, reports the integration installed.

**Acceptance Scenarios**:

1. **Given** fragcap is installed but not registered, **When** the operator runs
   the register command, **Then** the running fragcap binary is placed in
   Wireshark's extcap directory (created if absent) and the command reports the
   destination path.
2. **Given** fragcap is already registered, **When** the operator runs the
   register command again, **Then** it succeeds without error and the
   registration is unchanged (idempotent).
3. **Given** fragcap has just been registered, **When** the operator runs
   `fragcap doctor`, **Then** the analyzer extcap check reports the integration
   installed rather than not registered.

---

### User Story 2 - Unregister fragcap (Priority: P2)

An operator who no longer wants the Wireshark integration runs a single command
to remove it, and it is gone. Running the command when nothing is registered is
not an error.

**Why this priority**: Registration without a clean removal is a half-feature;
uninstall is small and completes the pair, but it is secondary to being able to
register at all.

**Acceptance Scenarios**:

1. **Given** fragcap is registered, **When** the operator runs the unregister
   command, **Then** the registered binary is removed from the extcap directory
   and `fragcap doctor` reports the integration not registered.
2. **Given** fragcap is not registered, **When** the operator runs the
   unregister command, **Then** it succeeds and reports that nothing was
   registered (idempotent, not an error).

---

### User Story 3 - Register from the installer (Priority: P2) (Deferred to a dedicated installer slice)

An operator installing fragcap through the Windows installer can opt into
registering the Wireshark integration at install time, so they never touch the
command line for it.

**Why this priority**: It removes the last manual step for installer users, but
it depends on the same registration behavior as Story 1 and is a convenience on
top of it.

**Acceptance Scenarios**:

1. **Given** the Windows installer, **When** the operator selects the optional
   extcap registration component, **Then** the installed fragcap is registered as
   a Wireshark extcap source as part of installation.
2. **Given** the installer, **When** the operator does not select the optional
   component, **Then** installation proceeds and fragcap is not registered, with
   the command from Story 1 still available afterward.

---

### User Story 4 - The analyzer integration still works (Priority: P1)

A Wireshark analyzer that already drives fragcap as an extcap source keeps
working exactly as before. Adding the register and unregister commands does not
change how the analyzer discovers interfaces, reads the configuration, or
captures.

**Why this priority**: A regression here breaks live use of the tool inside
Wireshark, so it ranks with Story 1. The analyzer protocol is the load-bearing
existing behavior this slice must not disturb.

**Independent Test**: Exercise each of the four analyzer protocol invocations,
both as an explicit `extcap` subcommand and as the bare top-level form the
analyzer actually uses, and confirm each still produces its expected output.

**Acceptance Scenarios**:

1. **Given** the register and unregister commands exist, **When** the analyzer
   runs the interface, link-type, configuration, and capture invocations in the
   bare top-level form, **Then** each behaves exactly as before this slice.
2. **Given** the same, **When** those invocations are issued as explicit
   `extcap` subcommands, **Then** each behaves exactly as before this slice.

---

### Edge Cases

- The extcap directory does not exist yet: registration creates it.
- The target location cannot be determined on the platform, or the running
  binary's path cannot be determined: the command reports a clear error rather
  than registering nothing silently.
- Registration or removal is attempted without permission to write the target
  directory: the command reports the failure; it does not claim success.
- Unregister when nothing is registered: success, reported as a no-op.
- Register when already registered: success, reported as unchanged.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A register command MUST place the running fragcap binary into
  Wireshark's extcap directory, at the exact name and location the readiness
  check already probes, creating the directory if it does not exist, and MUST
  report the destination path.
- **FR-002**: The register command MUST be idempotent: run against an
  already-registered installation it succeeds and ends in the registered state.
  It refreshes the registration by writing the currently running binary, so the
  registered extcap always matches the running fragcap.
- **FR-003**: An unregister command MUST remove the registered binary from the
  extcap directory, and MUST be idempotent: run when nothing is registered it
  succeeds and reports a no-op.
- **FR-004**: Both commands MUST accept an option to override the target
  directory, and MUST fall back to the platform default when it is omitted.
- **FR-005**: After a successful register, the readiness check MUST report the
  analyzer extcap integration as installed; after unregister it MUST report it
  not registered.
- **FR-006**: The four existing analyzer protocol invocations (interface
  declaration, link-type declaration, configuration declaration, and capture)
  MUST continue to work unchanged, both as an explicit `extcap` subcommand and as
  the bare top-level form the analyzer uses. Adding the new subcommands MUST NOT
  regress any of them.
- **FR-007** (Deferred to the installer slice): The Windows installer MUST offer
  an optional component that registers fragcap as a Wireshark extcap source at
  install time; when the component is not selected, installation proceeds without
  registering, and the installer directs the user to run `fragcap extcap install`.
  The registration is per user; the installer states this. A machine-wide option
  is offered for administrators.
- **FR-008**: The register and unregister commands MUST report a clear error,
  and MUST NOT claim success, when the target location or the running binary
  cannot be determined, or when the target directory cannot be written.
- **FR-009**: The new subcommand MUST be documented in the command-line
  reference in the same change that introduces it.
- **FR-010**: The feature MUST register only fragcap's own binary; it MUST NOT
  download, install, or modify npcap or Wireshark.

### Key Entities *(include if feature involves data)*

- **Extcap directory**: the per-user location the analyzer searches for extcap
  executables; the target of registration and the location the readiness check
  probes.
- **Registered binary**: a copy of the fragcap executable placed in the extcap
  directory under the fixed name the analyzer and the readiness check both expect.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An operator registers the integration with a single command and no
  manual file handling, and the readiness check then reports it installed.
- **SC-002**: Running the register command twice, or the unregister command when
  nothing is registered, never produces an error.
- **SC-003**: All four analyzer protocol invocations produce their expected
  output after the new commands are added, in both invocation forms (a
  regression test proves it).
- **SC-004** (Deferred to the installer slice): An installer run with the
  optional component selected yields a fragcap that the readiness check reports as
  registered for the installing user, with no command-line step.
- **SC-005**: A failed registration (undetermined location, unwritable
  directory) reports an error and is never presented as a success.

## Assumptions

- The extcap directory and the registered binary name are exactly those the
  readiness check already probes, so the two agree without a second source of
  truth. This slice does not change how the readiness check locates them.
- The register command copies the currently running binary; it does not build or
  download one.
- The platform default extcap directory is the per-user Wireshark extcap
  location; a machine-wide or system location is only used when explicitly
  overridden.
- The installer component registers the just-installed fragcap using the same
  registration behavior the command provides, targeting the location the
  readiness check probes.
- The command-line reference is the CLI reference page; the narrative install
  walkthrough with screenshots is out of scope and belongs to a later slice.
