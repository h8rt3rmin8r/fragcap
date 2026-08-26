# Feature Specification: Marker Cap Warning Subject

**Feature Branch**: `codex/084-marker-cap-warning`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Spec out S084 and run it end-to-end like usual"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Locate The Incomplete Scan (Priority: P1)

An operator who sees a detection coverage warning can tell which install root or discovered target produced it, without matching the warning to surrounding rows by guesswork.

**Why this priority**: The current marker-cap warning says only that some executable candidates were not read. It does not name the scan subject, so the loss is counted but not recoverable to its cause, which violates P-4.

**Independent Test**: Can be fully tested by scanning a directory whose binary-marker candidate set exceeds the cap and confirming the emitted warning names that scanned root and the skipped count.

**Acceptance Scenarios**:

1. **Given** a standalone directory scan with more executable candidates than the binary-marker cap, **When** warnings are rendered, **Then** the marker-cap warning names the scanned root and the number of candidates not examined.
2. **Given** a target discovery run that scans multiple roots, **When** more than one root exceeds the binary-marker cap, **Then** each warning names its own root so identical warnings cannot collapse into one indistinguishable message.
3. **Given** a scan that has both an unreadable subtree and a marker-cap truncation, **When** warnings are rendered, **Then** both warnings use subject-naming wording and neither replaces the other.

---

### User Story 2 - Explain The Operator Consequence (Priority: P2)

An operator reading the warning understands what was not examined and what that means for technology columns, without seeing an internal constant name or needing source code.

**Why this priority**: The warning appears beside `incomplete` target rows and in `fragcap technologies`; it should explain why the reported ENGINE and SENSITIVITIES evidence may be incomplete.

**Independent Test**: Can be fully tested by asserting the warning text says executable candidates were skipped for binary marker detection and that technology results may be incomplete.

**Acceptance Scenarios**:

1. **Given** a capped marker scan, **When** the warning is printed, **Then** it states that binary-marker detection skipped executable candidates.
2. **Given** a capped marker scan, **When** the warning is printed, **Then** it states that technology detection may be incomplete for that scanned root.
3. **Given** a capped marker scan, **When** the warning is printed, **Then** it does not require color, terminal width, or machine-readable mode to preserve the subject and consequence.

### Edge Cases

- A relative scan root must still render a usable subject rather than an empty path.
- A path containing spaces must remain readable in the warning.
- The warning must remain a single line so existing warning emitters do not change their framing.
- The skipped count must remain exact.
- The warning contract must not expose a new configuration knob or imply that operators can raise the cap in this slice.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The marker-cap coverage warning MUST name the scanned root whose candidate set was truncated.
- **FR-002**: The marker-cap coverage warning MUST preserve the exact count of candidate executables not examined.
- **FR-003**: The marker-cap coverage warning MUST state that executable candidates were skipped for binary marker detection.
- **FR-004**: The marker-cap coverage warning MUST state that technology detection for the named root may be incomplete.
- **FR-005**: The warning helper MUST continue to emit unreadable-subtree warnings and marker-cap warnings from the same scan outcome so callers cannot forget one cause.
- **FR-006**: Existing human warning callers (`fragcap technologies`, `fragcap targets`, and `fragcap targets discover`) MUST receive the improved text without each reimplementing warning wording.
- **FR-007**: The change MUST NOT add a new CLI flag, storage migration, runtime dependency, capture behavior, proxy behavior, process access, or network access.
- **FR-008**: The master specification MUST describe the subject-naming and consequence contract for capped binary-marker scans.

### Slice-Local Data Values

This slice does not introduce durable product entities. It uses these local data values to make the warning contract testable:

- **scanned root**: The directory passed to technology detection for one scan outcome.
- **marker-cap warning**: The human diagnostic emitted when binary-marker detection skipped candidate executables because the bounded candidate count was reached.
- **coverage warning list**: The complete list of human diagnostics for all reduced-coverage causes observed in one scan outcome.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A capped standalone directory scan emits a warning containing the scanned root and the exact skipped count.
- **SC-002**: Two capped scan outcomes for different roots produce warning strings that differ by subject.
- **SC-003**: Existing tests that depend on incomplete scan accounting continue to pass with no caller-specific warning duplication.
- **SC-004**: `cargo xtask ci` passes after implementation.

## Assumptions

- The warning should state that there is no configurable remedy in this slice by avoiding wording such as "increase the cap".
- The scanned root belongs on `ScanOutcome`, because every caller receives only the outcome when it asks for `coverage_warnings()`.
- Existing warning prefixes such as `warning:` and indentation remain owned by each command's emitter.
