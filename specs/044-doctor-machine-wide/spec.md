# Feature Specification: doctor recognizes machine-wide extcap registration

**Feature Branch**: `044-doctor-machine-wide`

**Created**: 2026-08-14

**Status**: Draft

**Input**: Follow-up to slice 043 (PR #114), a Codex finding. `fragcap doctor`'s
`analyzer extcap` integration check probes only the per-user Wireshark extcap
directory, so a machine-wide-only registration (which slice 043's MSI can create
in Wireshark's system extcap directory) is reported as an optional warning even
though Wireshark can see the source. This slice teaches the check to also
recognize the machine-wide scope and to name which scope registered fragcap.

## Clarifications

### Session 2026-08-14

Resolved under autopilot from the existing doctor design and the slice scope:

- Q: How is the system extcap directory located? -> A: A new
  `paths::system_extcap_dir()`, env-overridable (`FRAGCAP_SYSTEM_EXTCAP_DIR`)
  like `extcap_dir()`, resolving `%ProgramFiles%\Wireshark\extcap` on Windows and
  a conventional default elsewhere; the override keeps the classifier testable on
  any platform.
- Q: How does the check combine the two scopes? -> A: `ok` when either the
  per-user or the system directory holds the fragcap binary; the detail names the
  scope (current user, machine-wide, or both). The not-registered case stays an
  optional `Warn`, unchanged.
- Q: Does the dependency model change? -> A: No. extcap stays the optional tier
  from slice 042; only the detection is widened.
- Q: How is the machine-wide directory located on Windows? (added after PR review)
  -> A: The probe resolves Wireshark's install directory from the same
  `HKLM\SOFTWARE\Wireshark` registry value the MSI's machine-wide option reads,
  then appends `extcap`, so a non-default install location is recognized; the
  `%ProgramFiles%\Wireshark\extcap` default is only a fallback. The read uses the
  `windows-sys` binding `fragcap-cli` already links (no new crate) and stays
  read-only.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A second user on a machine-wide install sees the integration (Priority: P1)

On a machine where fragcap was registered machine-wide (into Wireshark's system
extcap directory), a different user runs `fragcap doctor`. The `analyzer extcap`
row reports `ok` and names the machine-wide scope, instead of the misleading
"not registered" optional warning.

**Why this priority**: It is the whole point of the slice and the gap Codex
flagged: doctor contradicting reality (P-9) for the machine-wide scope.

**Independent Test**: With the system extcap directory containing the fragcap
binary (via the override env var in tests, or a real machine-wide install), the
integration check classifies `ok` and the detail names the machine-wide scope.

**Acceptance Scenarios**:

1. **Given** the fragcap binary in the system extcap directory and not in the
   per-user one, **When** doctor classifies, **Then** the row is `ok` and the
   detail names the machine-wide scope and its directory.
2. **Given** the binary in the per-user directory only, **When** doctor
   classifies, **Then** the row is `ok` and the detail names the current-user
   scope (unchanged behavior, refined wording).
3. **Given** the binary in both directories, **When** doctor classifies,
   **Then** the row is `ok` and the detail names both scopes.
4. **Given** the binary in neither, **When** doctor classifies, **Then** the row
   is the optional `Warn` with the `fragcap extcap install` guidance, unchanged.

### Edge Cases

- The system extcap directory cannot be determined (`None`): treated as
  not-installed for that scope, never a failure.
- Non-Windows and no-feature builds: the change must compile and classify (the
  probe stays thin and platform-tolerant; the classifier is pure over `Inputs`).
- The tutorial doctor sample in `getting-started.mdx` shows the per-user detail
  wording; if the wording changes it is updated, dash-free.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A `paths::system_extcap_dir()` MUST return the machine-wide
  Wireshark extcap directory, overridable by an environment variable for tests,
  and `None` when it cannot be determined.
- **FR-002**: The doctor probe MUST report whether the fragcap binary is present
  in the system extcap directory, read-only (installs and copies nothing, P-1).
- **FR-003**: The `analyzer extcap` check MUST classify `ok` when either the
  per-user or the system directory holds the binary, and MUST name the scope
  (current user, machine-wide, or both) and its directory in the detail.
- **FR-004**: The not-registered case MUST remain an optional `Warn` with the
  `fragcap extcap install` guidance; the slice 042 dependency model is unchanged.
- **FR-005**: The classifier MUST be unit-tested for all four scope combinations
  (user only, system only, both, neither).
- **FR-006**: The doctor goldens and the duplicated `ready`/`ready_inputs`
  fixtures MUST be updated so the gate passes.
- **FR-007**: `fragcap-core` MUST take no platform dependency (the change is in
  `fragcap-cli`); the default no-feature build and the Linux neutrality build
  MUST still compile.
- **FR-008**: All added or edited text (code, goldens, docs) MUST be UTF-8, LF,
  and free of em and en dashes; a changelog fragment MUST be added.

### Key Entities *(include if data involved)*

- **Extcap registration scope**: current-user (`%APPDATA%\Wireshark\extcap`) or
  machine-wide (Wireshark system extcap directory); doctor reports which holds
  the fragcap binary.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A machine-wide-only registration makes the `analyzer extcap` row
  report `ok`, naming the machine-wide scope (unit-tested via the override).
- **SC-002**: Per-user-only and both-scope cases report `ok` naming the correct
  scope; the neither case stays the optional warning.
- **SC-003**: `cargo xtask ci` is green (fmt, clippy, tests, lint, deps, license,
  docs), with the goldens regenerated and re-verified clean.
- **SC-004**: The default no-feature build and the Linux `fragcap-core`
  neutrality build compile.

## Assumptions

- The machine-wide path is exercised fully only on a real Windows plus Wireshark
  install; unit tests drive the classifier through the override env vars and
  `Inputs` fixtures, which is where the logic lives (the probe stays thin).
- No extcap CLI or MSI change; this is doctor-side detection only.
