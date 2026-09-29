# Feature Specification: S161 Interactive CLI Input Ownership

**Feature Branch**: `codex/s161-interactive-cli-input-ownership`
**Created**: 2026-09-29
**Status**: Draft
**Input**: Issue #437 and the owner's S161 autopilot request.

## User Scenarios & Testing

### User Story 1 - Respond to Doctor actions (Priority: P1)

An operator can answer every `fragcap doctor --fix` action question in the active terminal. A negative answer leaves the named action unperformed. An affirmative answer performs only the action just displayed.

**Why this priority**: The shipped v0.10.2 command visibly asks a question but cannot read its answer.

**Independent Test**: In a controlled interactive command run with an offered harmless action, provide `n` and Enter, observe `skipped`, process completion, and zero performer calls. Provide `y` in a separate controlled run and observe exactly one performer call.

**Acceptance Scenarios**:

1. **Given** an offered Doctor action, **when** the operator enters `n` followed by Enter, **then** Doctor reports `skipped`, does not perform that action, and continues to the next named action or final report.
2. **Given** an offered Doctor action, **when** the operator enters `y` or `yes` followed by Enter, **then** Doctor performs only that action and reports its real outcome.
3. **Given** an empty or other completed response, **when** Doctor evaluates it, **then** the action is skipped.
4. **Given** a prompt write, flush, or input failure, **when** confirmation cannot be obtained, **then** Doctor reports an input failure and stops the action phase without representing that failure as an operator decline.

---

### User Story 2 - Answer other CLI questions (Priority: P2)

An operator can answer the interactive socket-holder question during target registration and the warm-restart confirmation during Deep Capture preparation.

**Why this priority**: Both questions are reached through the same command entry and currently attempt to acquire input a second time.

**Independent Test**: Controlled runs reach each question, provide a complete answer, and finish within a bounded deadline with the documented result. An input failure does not approve a restart or author a socket-holder assertion.

**Acceptance Scenarios**:

1. **Given** target registration with an executable and no explicit socket-holder flag, **when** the operator supplies a valid answer, **then** the recorded answer matches that completed response.
2. **Given** an eligible warm-restart question, **when** the operator declines, **then** no restart proceeds; an affirmative completed response follows the existing restart flow.
3. **Given** a Deep Capture or calibration authorization plan, **when** an exact permitted response is supplied, **then** it remains bounded and tied to that plan; incomplete, changed, or extra input retains its existing refusal behavior.

---

### User Story 3 - Interpret the screenshot symptom accurately (Priority: P3)

The operator and maintainers can distinguish the confirmed CLI input failure from the reported Print Screen behavior in an elevated window without an unsupported claim that one fixes the other.

**Why this priority**: The original report included both symptoms, but the confirmed stdin deadlock explains only the CLI response.

**Independent Test**: Review the issue and slice record for the exact observed focus comparison, the external elevated-window report, and an explicit statement of what was and was not demonstrated.

**Acceptance Scenarios**:

1. **Given** the confirmed stdin correction, **when** the slice is reported, **then** it claims only the scoped prompt behavior as fixed.
2. **Given** Print Screen observations, **when** evidence does not establish a fragcap-specific cause, **then** the slice records that limit without claiming screenshot behavior changed.

### Edge Cases

- More than one Doctor action is offered in a single run.
- A completed empty line differs from input EOF or an I/O error.
- Output fails before the question can be read.
- A single command requests multiple separate authorization responses, with an already buffered subsequent line.
- Machine-readable or noninteractive commands retain their existing refusal gates.
- The operator interrupts a blocked prompt; no unconfirmed action may run.

## Requirements

### Functional Requirements

- **FR-001**: Doctor MUST read a complete interactive response after presenting and flushing each actionable question.
- **FR-002**: Doctor MUST preserve its existing affirmative tokens and default-No treatment for completed nonaffirmative lines.
- **FR-003**: Doctor MUST never perform an action without its explicit confirmation, and MUST retain the report's offered-action ordering.
- **FR-004**: Doctor MUST distinguish prompt output or input failure from a completed negative answer, report the failure, and stop further actions.
- **FR-005**: Interactive target registration and warm-restart questions MUST accept completed responses without blocking on input ownership.
- **FR-006**: Deep Capture and calibration plan authorization MUST preserve exact identifiers, bounded input, complete-line requirements, separate per-plan responses, and refusal before effects.
- **FR-007**: Noninteractive and machine-readable command gates and `doctor --fix --yes` semantics MUST remain as specified.
- **FR-008**: The slice MUST document the Print Screen report separately and MUST not claim a fragcap hotkey fix without evidence of a fragcap-specific cause.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Controlled production-entry prompt coverage completes a Doctor negative response within a bounded timeout, reports `skipped`, and proves no action performed.
- **SC-002**: Controlled positive and multi-action Doctor cases preserve exactly one confirmation decision per offered action and no unconfirmed effect.
- **SC-003**: Controlled target-registration and warm-restart prompt cases complete within a bounded timeout with the expected recorded decisions.
- **SC-004**: Controlled prompt I/O failures report an error, do not report `skipped`, and perform zero subsequent actions.
- **SC-005**: Existing authorization refusal and successful multi-plan cases continue to pass with their exact input contract.
- **SC-006**: Required repository verification gates pass; any unavailable focused Print Screen comparison is reported as unverified, not as a fixed behavior.

## Assumptions

- Issue #437's confirmed code-level stdin deadlock is the scope authority for this slice.
- A completed empty Doctor line is a negative answer; EOF and I/O failure are input failures rather than proof of an operator choice.
- No new dependencies, capture behavior, proxy behavior, target storage schema, artifact format, or release claim is needed.
- The operator's Print Screen report remains real, but its relation to fragcap is unestablished.
## Clarifications

### Session 2026-09-29

- Q: Does EOF count as the default-No answer? A: No. Only a completed response is an operator choice. EOF or a read error stops the action phase without performing or reporting a decline.
- Q: Does this slice remediate the Print Screen hotkey? A: Only if evidence establishes a fragcap-specific cause. The confirmed stdin correction is the deliverable; the current focus observation remains separately documented.
- Q: May the correction relax exact plan authorization to make prompts work? A: No. Bounded, complete, separate plan responses and refusals remain required.
