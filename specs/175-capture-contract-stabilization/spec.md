# Feature Specification: Capture Contract Stabilization

**Feature Branch**: `codex/s175-capture-contract-stabilization`

**Created**: 2026-10-21

**Status**: Implemented; pending published PR review

**Input**: Owner-approved S175 bundle of GitHub issues #484, #485, #486, #490, #496, and #497.

## Scope

**Scope in**: Capture duration, acquisition and timestamp-domain correctness; effective role and direction validation; metadata-only pcapng across ordinary, ring, and Deep Capture capture artifacts; comma-capable sink destination grammar; Capture mode and volume-bound help; exact regression evidence for every included issue.

**Scope out**: Doctor readiness and history, target discovery, bundle cleanup, generic CLI JSON semantics, IGDB, release publication, and unrequested process or proxy behavior.

**Done when**: Each included issue's repository-controlled acceptance criteria passes, the Spec-Kit analyze gate has no unresolved findings, and the full repository gate passes. Real-game calibration is optional later compatibility evidence and is reported separately from controlled verification.

## User Scenarios & Testing

### User Story 1 - Duration means elapsed capture time (Priority: P1)

An operator can request a duration-bound Capture or guided calibration and receive the requested observation window even when packets carry absolute epoch timestamps.

**Why this priority**: #484 can terminate a live observation on its first retained packet and invalidate calibration.

**Independent Test**: Run controlled live and replay sources with epoch-stamped packets, then compare the stop cause and elapsed observation against the requested bound under busy, sparse, and silent traffic.

**Acceptance Scenarios**:

1. **Given** a newly armed 45-second Capture, **when** an epoch-stamped packet arrives immediately, **then** it does not cause duration completion before 45 seconds of monotonic elapsed time.
2. **Given** no packets or sparse packets, **when** 45 seconds elapse, **then** duration completion occurs without needing another packet.
3. **Given** a terminal process event before duration expiry, **when** capture stops, **then** the terminal cause wins and packet/process evidence keeps its absolute timestamps.
4. **Given** an acquisition wait with a pending terminal stage, **when** its finite deadline elapses, **then** it stops for the correct deadline rather than a packet epoch comparison.

### User Story 2 - Capture scope and retained content match requested options (Priority: P1)

An operator can select roles, direction, and payload retention knowing the output and counters reflect those choices in every sink.

**Why this priority**: #485 can retain bytes the operator explicitly excluded, while #486 and #490 make successful captures misrepresent their scope.

**Independent Test**: Replay a bidirectional fixture with known payload markers through file, explicit sink, ring, and Deep Capture capture paths; inspect the emitted records and accounting.

**Acceptance Scenarios**:

1. **Given** a role list with empty members or a role absent from the resolved profile, **when** Capture or extcap capture is requested, **then** it exits as invalid input before a source or sink is opened and names the offending value.
2. **Given** `--direction in` or `--direction out`, **when** bidirectional packets arrive, **then** only packets of the selected direction enter retained output and excluded packets are counted as a declared scope decision.
3. **Given** metadata-only output, **when** a packet with a known application marker arrives, **then** no pcapng or JSON Lines artifact retains that marker, while ordinary payload-enabled output retains it.
4. **Given** a pcapng metadata-only packet, **when** an unmodified pcapng reader opens the artifact, **then** its captured length and original length truthfully describe retained bytes and observed wire length, with parseable headers retained where they can be identified without ambiguity.
5. **Given** global and sink-specific payload settings, **when** output formats, rotation, ring dump, or Deep Capture artifacts are selected, **then** each sink honors its effective setting or the invocation fails before creating output.

### User Story 3 - Sink paths and Capture guidance are accurate (Priority: P2)

An operator can name a normal Windows file containing commas and can rely on Capture help to describe available modes and the meaning of volume bounds.

**Why this priority**: #497 rejects valid destinations, and #496 describes implemented modes as unavailable while misnaming volume-limit counters.

**Independent Test**: Exercise comma and modifier grammar with file destinations, then compare help snapshots and fixture-based ring and volume output with the documented descriptions.

**Acceptance Scenarios**:

1. **Given** a sink filename containing a comma, spaces, or both, **when** the path is entered with the documented grammar, **then** it creates that exact destination with or without modifiers.
2. **Given** a malformed escape or an actual malformed modifier, **when** sink parsing fails, **then** the diagnostic identifies the relevant syntax without mislabeling a filename as an option.
3. **Given** Capture help for ring, stream, and volume bounds, **when** the operator reads it, **then** it names implemented modes and states that the bounds apply to retained target packets or bytes, separately from source packets captured and ring evictions.

### Edge Cases

- Empty, whitespace-only, interior-empty, case-mismatched, and custom profile role names must be decided before side effects.
- Unknown packet direction cannot silently pass an explicit `in` or `out` filter; its omission must be counted and visible.
- Malformed, unsupported, truncated, or non-IP frames must never leak payload in metadata-only output; unknown header boundaries are represented explicitly rather than guessed.
- A duration tick and a terminal event arriving at the same monotonic boundary must have a deterministic stop cause without altering absolute packet evidence.
- Backslash and comma escaping must remain unambiguous for ordinary Windows paths and existing `,key=value` modifiers.

## Requirements

### Functional Requirements

- **FR-001**: Capture duration and acquisition deadlines MUST use one arm-relative monotonic clock for packet and idle wakeups. Absolute driver and process timestamps MUST remain evidence and ordering facts, never deadline inputs.
- **FR-002**: Replay and live Capture MUST share deadline semantics and preserve distinct duration, acquisition-timeout, terminal-stage, source-exhaustion, and interrupt outcomes.
- **FR-003**: Role lists MUST reject empty or whitespace-only lists and members, and validate exact case-sensitive names against the resolved profile's declared roles before opening capture output or sources. Custom roles MUST remain valid.
- **FR-004**: Explicit inbound or outbound selection MUST filter retained packets after attribution and before every sink, using one shared decision. Unknown direction MUST be excluded under explicit direction selection and reported in scope accounting.
- **FR-005**: Global and sink-specific metadata-only selection MUST be honored in pcapng and JSON Lines for ordinary files, explicit sinks, rotation, ring dumps, and the Deep Capture packet artifact. Unhonorable configurations MUST be refused before creating output.
- **FR-006**: Metadata-only pcapng MUST be valid for unmodified readers, preserve truthful original and captured lengths, retain only confidently classified required headers, and never retain application payload or other uncertain packet bytes.
- **FR-007**: Sink destination grammar MUST allow literal commas in file paths with an explicit documented escape, while preserving existing comma-delimited modifiers and giving accurate syntax errors.
- **FR-008**: Capture help MUST describe implemented ring and stream modes, supported stream destinations, the relation between pre-ring retention and ring evictions, and the retained-target meaning of volume bounds.
- **FR-009**: Human, silent, and JSON invocation modes MUST never report a direction or payload policy as enforced when output contradicts it.
- **FR-010**: All included issue outcomes MUST be covered by deterministic, controlled regression tests; fixture and golden changes MUST be reviewed for content and declared omission.

### Key Entities

- **Capture clock**: An elapsed, monotonic time domain anchored when the session arms; used only for deadline decisions.
- **Packet evidence timestamp**: The source-observed absolute instant, retained for output, process ordering, and correlation.
- **Effective capture scope**: Validated role, direction, and payload choices applied to retention and reporting.
- **Metadata-only packet**: Original wire length plus a bounded captured header prefix (or no captured bytes when the boundary is uncertain), with an explicit omission outcome.
- **Sink specification**: Destination and independent format, payload, rotation, queue, and timeout modifiers.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every controlled epoch-stamped busy, sparse, silent, pending-terminal, and replay deadline case stops for its expected cause, with no early duration completion from an absolute timestamp.
- **SC-002**: Every metadata-only artifact path contains zero known payload-marker occurrences, while the matching full-payload paths retain the marker and all pcapng artifacts remain readable.
- **SC-003**: In a bidirectional fixture, explicit `in` and `out` runs retain zero opposite-direction packets, with captured, retained, filtered, and dropped counts reconciling.
- **SC-004**: Every invalid role case exits 2 before any sink or source side effect; every valid custom-role case proceeds.
- **SC-005**: The documented comma-path grammar accepts all named path and modifier combinations and rejects every malformed escape with a specific diagnostic.
- **SC-006**: Help snapshots and controlled ring and five-packet volume examples agree on modes, retained bounds, source capture count, and ring evictions.

## Assumptions

- The existing Capture and Deep Capture authorization and target-selection boundaries remain authoritative; this slice does not add a product-usage policy.
- Metadata-only is an explicit operator-selected retention policy. Omitted bytes and excluded packets are reported under P-4 and P-9, rather than represented as observed full payload.
- The controlled source and fixture corpus are sufficient implementation evidence. A later operator-run game calibration is compatibility evidence, not an implementation completion gate.
