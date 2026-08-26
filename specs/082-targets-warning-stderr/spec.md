# Feature Specification: Targets Warning Stream Contract

**Feature Branch**: `codex/082-targets-warning-stderr`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "S082. Run issue #205 end-to-end: `targets` warnings must go to stderr through the existing emitter instead of contaminating stdout."

## User Scenarios & Testing

### User Story 1 - Pipe Targets Listings Without Diagnostics (Priority: P1)

An operator pipes `fragcap targets` or `fragcap targets list` into another command and receives only the listing bytes on standard output, even when discovery produces warnings.

**Why this priority**: The listing is the front door for target selection and the stream contract already promises that warnings and errors stay off command-result output.

**Independent Test**: Run a targets listing with an injected discovery warning and compare standard output against the same listing without the warning.

**Acceptance Scenarios**:

1. **Given** a targets listing that produces one or more warnings, **When** the command completes, **Then** standard output contains only the listing result and standard error contains each warning.
2. **Given** the same local store and catalog state without warnings, **When** the command completes, **Then** standard output is byte-identical to the warning-producing run.
3. **Given** `--silent`, **When** a targets command produces warnings, **Then** standard output remains only the command result and standard error contains no warning lines.

---

### User Story 2 - Keep Structured Diagnostics Structured (Priority: P2)

An automation runs a targets command with `--json` and can parse warnings from standard error as structured diagnostic records, without mixed human `warning:` lines on standard output.

**Why this priority**: The existing emitter defines the JSON diagnostic shape and the targets command should share it instead of creating a second warning format.

**Independent Test**: Run a warning-producing targets command with `--json` and assert standard error contains warning records while standard output remains parseable command output.

**Acceptance Scenarios**:

1. **Given** `--json` and a warning-producing targets command, **When** the command completes, **Then** each warning is emitted as a structured warning record on standard error.
2. **Given** `--json`, **When** the command result is a human table or command-result JSON, **Then** no human warning prefix is written to standard output.

## Edge Cases

- Discovery bootstrap warnings during the hero listing must not appear above the table.
- Discovery inspection warnings from `targets discover` and `targets scan` must be diagnostic output, not rows or account data.
- `targets add --steam` enumeration warnings must not precede or interleave with the registration result on standard output.
- Inline detection findings remain command results, while only warning diagnostics move to the emitter.
- `doctor --fix` discovery action continues to receive the target discovery result on its result stream; only warnings route through the emitter passed by the caller.

## Requirements

### Functional Requirements

- **FR-001**: Every warning produced by the targets command surface MUST be emitted through the shared diagnostic emitter.
- **FR-002**: Targets command results MUST remain on standard output and MUST NOT contain any line whose diagnostic prefix is `warning:`.
- **FR-003**: Warning-producing and warning-free listings over the same store MUST produce byte-identical standard output.
- **FR-004**: `--quiet` MUST retain targets warnings on standard error.
- **FR-005**: `--silent` MUST suppress targets warnings while preserving command results on standard output and errors on standard error.
- **FR-006**: `--json` MUST render targets warnings as structured warning diagnostics on standard error.
- **FR-007**: Target discovery accounts, technology findings, registration counts, table rows, empty-listing guidance, ambiguity lists, import/export documents, and detail views MUST remain command results on standard output.
- **FR-008**: Existing exit code behavior for targets subcommands MUST remain unchanged.
- **FR-009**: The implementation MUST use the existing emitter abstraction rather than introducing another warning routing mechanism.

## Success Criteria

### Measurable Outcomes

- **SC-001**: At least one integration test proves a warning-producing `targets list` leaves standard output byte-identical to a warning-free listing.
- **SC-002**: At least one integration test proves a targets warning is present on standard error in normal and quiet modes and absent in silent mode.
- **SC-003**: At least one integration test proves `--json` targets warnings are valid structured warning diagnostics on standard error.
- **SC-004**: The repository CI parity gate passes with no new dependency and no changed capture, attribution, or target-store behavior.

## Assumptions

- The existing `Emitter::warn` behavior is authoritative for human, quiet, silent, and JSON diagnostics.
- Warning routing is a CLI surface fix only; target discovery, registration, detection, and storage semantics are out of scope.
- No master specification edit is required because sections 17.5 and 17.6 already define the stream contract.
