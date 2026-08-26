# Feature Specification: Doctor Single Enumeration

**Feature Branch**: `codex/080-doctor-single-enumeration`

**Created**: 2026-08-26

**Status**: Draft

**Input**: User description: "Implement issue #203: `fragcap doctor` enumerates the npcap device list twice per run. Derive the loopback verdict from the interface inventory already gathered by doctor, preserve the existing report semantics, and run the slice end to end."

## User Scenarios & Testing

### User Story 1 - Avoid Duplicate Capture Driver Enumeration (Priority: P1)

An operator runs `fragcap doctor` and the capture driver and interface probe does not pay for two back-to-back npcap device-list enumerations to produce one readiness report.

**Why this priority**: `doctor` is the first troubleshooting command. S079 made slow probes visible; this slice removes a confirmed duplicate cost without weakening readiness truthfulness.

**Independent Test**: Run the doctor probe through an injected live-backend seam that counts device-list enumeration calls. A successful live probe obtains interfaces and loopback support from one enumeration.

**Acceptance Scenarios**:

1. **Given** the live backend is linked and `wpcap.dll` is loadable, **When** interface enumeration succeeds, **Then** doctor derives the interface list and loopback support from that single inventory.
2. **Given** the interface inventory contains any loopback device, either by explicit loopback flag or the existing loopback marker in its description, **When** doctor classifies npcap loopback support, **Then** it reports `Some(true)`.
3. **Given** the interface inventory contains no loopback device, **When** doctor classifies npcap loopback support, **Then** it reports `Some(false)` only because enumeration succeeded.

---

### User Story 2 - Preserve Honest Unknown States (Priority: P1)

An operator whose live backend cannot enumerate interfaces receives the same honest "not determined" loopback state as before, rather than a fabricated absence.

**Why this priority**: P-9 requires doctor to report what it observed. An enumeration failure means loopback support was not determined, not that loopback is absent.

**Independent Test**: Run the doctor probe through an injected enumeration failure and assert the loopback value remains `None` while the interface error remains present in the report inputs.

**Acceptance Scenarios**:

1. **Given** the live backend is linked and `wpcap.dll` is loadable, **When** interface enumeration fails, **Then** doctor reports no interfaces, carries the enumeration error, and leaves loopback support as `None`.
2. **Given** the live backend is not linked or `wpcap.dll` is not loadable, **When** doctor gathers readiness inputs, **Then** loopback support remains `None` as it does today.
3. **Given** enumeration succeeds with zero interfaces, **When** doctor classifies loopback support, **Then** `Some(false)` is permitted because the absence was observed.

---

### User Story 3 - Keep Report Contracts Stable (Priority: P2)

Existing consumers of `fragcap doctor`, including JSON automation and human golden tests, see unchanged report bodies except for faster collection on systems where duplicate enumeration was expensive.

**Why this priority**: The issue is a probe implementation defect, not a report-format change. S079 explicitly preserved the final report surfaces, and this slice continues that contract.

**Independent Test**: Run the focused doctor tests and existing doctor goldens. The final human and JSON outputs remain unchanged, while unit coverage proves the probe now uses one enumeration.

**Acceptance Scenarios**:

1. **Given** `fragcap doctor --json`, **When** doctor runs after this slice, **Then** the JSON record shape and values for equivalent inputs are unchanged.
2. **Given** the existing human doctor golden tests, **When** they run after this slice, **Then** no golden report body changes are required.
3. **Given** an interactive `doctor --timings` run on a live-capable machine, **When** the capture driver and interfaces probe completes, **Then** any measured timing belongs to the single enumeration path and no second loopback-only enumeration is performed.

### Edge Cases

- A loopback device detected by description marker rather than explicit loopback flag still counts as supported.
- A successful enumeration with no loopback device is the only path that may report `Some(false)`.
- An enumeration failure carries the error and leaves loopback unknown.
- Backend absence and `wpcap.dll` load failure leave loopback unknown without attempting enumeration.
- `detect_driver()` remains available for callers that ask only for driver presence and do not already hold an interface inventory.

## Requirements

### Functional Requirements

- **FR-001**: `fragcap doctor` MUST NOT call the live device-list enumeration twice in the capture driver and interfaces probe.
- **FR-002**: When live interface enumeration succeeds, doctor MUST derive loopback support from the returned interface inventory.
- **FR-003**: The loopback predicate used by doctor MUST preserve both existing evidence paths: explicit loopback flag and the existing npcap loopback description marker.
- **FR-004**: When live interface enumeration fails, doctor MUST leave `NpcapInfo::loopback_supported` as `None`, not `Some(false)`.
- **FR-005**: When the live backend is not linked or `wpcap.dll` is not loadable, doctor MUST leave loopback support as `None` and preserve the existing report behavior.
- **FR-006**: Existing doctor human and JSON report contracts MUST remain unchanged for equivalent probe inputs.
- **FR-007**: Tests MUST cover loopback `Some(true)`, `Some(false)`, and `None` outcomes, including an enumeration-failed case.
- **FR-008**: Tests MUST prove or mechanically constrain that the doctor live probe obtains interfaces and loopback support from one enumeration call.
- **FR-009**: The fix MUST NOT change capture behavior, interface selection behavior, packet parsing, attribution, output writing, or Deep Capture readiness semantics.
- **FR-010**: The completed slice MUST include a changelog fragment for issue #203 and state any measurement limitation rather than claiming an unobserved speedup.

### Key Entities

- **Live Interface Inventory**: The set of interface records returned by the live capture backend enumeration, including each interface's name, description, addresses, and loopback evidence.
- **Loopback Support Verdict**: The three-valued `NpcapInfo::loopback_supported` fact in doctor inputs: `Some(true)` observed present, `Some(false)` observed absent, `None` not determined.
- **Doctor Capture Driver Probe**: The probe phase that gathers npcap readiness, live backend availability, loopback support, and interface inventory for final report classification.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A focused unit or integration test demonstrates that one successful doctor live probe performs exactly one device-list enumeration through the injected seam.
- **SC-002**: Loopback classifier coverage proves `Some(true)`, `Some(false)`, and failed-enumeration `None` behavior.
- **SC-003**: Existing focused doctor tests pass without doctor golden output changes.
- **SC-004**: `cargo xtask ci` passes, or any unavailable platform/toolchain component is reported with the exact command and observed result.
- **SC-005**: The slice changelog records the user-visible performance fix and avoids unsupported timing claims when local live measurement is unavailable.

## Assumptions

- The current final doctor report classification is correct; this slice changes only how one probe gathers its inputs.
- Existing `InterfaceRecord` data is sufficient to reproduce the loopback predicate used by driver detection.
- `detect_driver()` keeps its current purpose for callers that do not already have an interface inventory.
- The master specification already defines the doctor report contract and does not need a revision unless implementation changes that contract.
