# Feature Specification: Truthful Capture First-Use Guidance

**Feature Branch**: `codex/s177-capture-first-use-truth`

**Created**: 2026-10-21

**Status**: Draft

**Input**: Owner-approved S177 bundle of GitHub issues #487, #494, and #495.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Know whether this session can capture (Priority: P1)

A user runs Doctor before Capture or Deep Capture. Doctor distinguishes installed machine prerequisites from whether the current process has the privilege that live capture requires. An unelevated user receives a non-ready result and an elevation action in human and structured output. An elevated user with all other prerequisites satisfied receives a ready result.

**Why this priority**: A false ready verdict sends the user into an immediate, predictable live-capture refusal.

**Independent Test**: Classify otherwise-identical elevated and unelevated Windows sessions, then compare Doctor's human, structured, and exit results with the live-capture precondition.

**Acceptance Scenarios**:

1. **Given** working machine prerequisites and an unelevated current process, **when** Doctor runs, **then** both live modes are not ready, the result fails, and the current-session elevation remedy is shown.
2. **Given** the same machine prerequisites in an elevated current process, **when** Doctor runs, **then** elevation does not block either live mode.
3. **Given** a Doctor repair that relaunches elevated or returns to the caller, **when** readiness is reported afterward, **then** it reflects the token of the process reporting that result.

---

### User Story 2 - Continue after Steam client setup (Priority: P1)

A user confirms the first-time client executable for a stored Steam target. The command persists that declaration without claiming reachability calibration, then reports setup complete with the known target identifier and the exact command to start a fresh cold calibration after closing the game and Steam.

**Why this priority**: The existing success followed by a blocked, no-action verdict contradicts the operation that just completed.

**Independent Test**: In an isolated target store, complete setup with an exact confirmed plan, inspect the terminal result and stored target, and invoke the offered next command to reach the fresh calibration decision without treating setup as reachability evidence.

**Acceptance Scenarios**:

1. **Given** an eligible Steam target lacking an authored client, **when** its exact setup plan is confirmed and persistence verifies, **then** the terminal result says setup complete, reachability not attempted, identifies the target, and offers the same-store rerun command.
2. **Given** setup completed, **when** the next command runs after cold-state preparation, **then** calibration evaluates a fresh reachability case rather than repeating client setup.
3. **Given** an incomplete, altered, or CRLF-terminated structured plan response, **when** confirmation is checked, **then** setup remains refused and the target remains unchanged.

---

### User Story 3 - Follow usable Deep Capture help (Priority: P2)

A first-time user sees `--launch` as required in Deep Capture usage and understands that fragcap must start the selected game to apply proxy settings only to that launch. Help supplies copyable PowerShell examples at ordinary terminal width and explains the practical result of sensitive outputs and client identity options when each first appears.

**Why this priority**: The current help advertises a command that the runtime refuses and wraps an example into an invalid PowerShell command.

**Independent Test**: Render and parse the generated Deep Capture help at 80 columns, check missing-launch refusal, and validate the examples and option descriptions without starting a game.

**Acceptance Scenarios**:

1. **Given** Deep Capture help, **when** it renders at 80 columns, **then** usage and the required-input group show `--launch` as mandatory.
2. **Given** an invocation without `--launch`, **when** parsing or preflight refuses it, **then** the user sees the missing requirement and plain-language launch purpose.
3. **Given** the output-recipient and exact-plan PowerShell examples, **when** copied as shown, **then** their line boundaries and LF input are syntactically usable without relaxing exact-plan confirmation.
4. **Given** an option for HAR, key logs, or mutual TLS, **when** its first-use description is read, **then** the user can tell what file or credential it supplies and the sensitive consequence.

### Edge Cases

- Offline replay is separate from live Capture readiness and is not newly refused by Doctor's verdict.
- A missing or broken machine prerequisite still blocks the appropriate mode independently of elevation.
- A declined or stale Steam setup plan must never be reported as completed.
- A setup-only run creates no calibration workflow, compatibility fact, proxy session, or reachability claim.
- Paths containing spaces remain quoted in the offered next command; a custom local store remains selected.
- PowerShell CRLF is not accepted as the exact structured authorization line.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Doctor MUST treat absent current-process elevation as a blocking live Capture and Deep Capture prerequisite while preserving the separate machine checks and their own reasons.
- **FR-002**: Doctor MUST make human readiness, structured readiness with blocking-check identity, and exit status agree with that current-process refusal. Rechecks after `doctor --fix` MUST use the reporting process's current token.
- **FR-003**: Doctor MUST give a concrete elevation action; it MUST NOT claim that an offline replay needs elevation.
- **FR-004**: A verified Steam client-only update MUST produce a distinct setup-complete terminal state with target identity, explicit reachability-not-attempted meaning, and a same-store cold rerun command.
- **FR-005**: Only successful persisted-and-verified client setup may produce that state. Decline, drift, invalid exact-plan input, and verification failure MUST retain their existing refusal and no-write guarantees.
- **FR-006**: Structured authorization MUST continue to require the exact plan identifier followed by one LF; help MUST show a working PowerShell way to send that line without accepting CRLF or partial input.
- **FR-007**: Generated Deep Capture usage MUST declare `--launch` mandatory, and the flag/help and missing-launch refusal MUST explain why fragcap starts the selected game.
- **FR-008**: Embedded Windows examples MUST be copyable at 80 columns as complete one-line commands or explicit PowerShell continuations.
- **FR-009**: First-use help for HAR, key logs, and mutual TLS MUST state their concrete artifact or credential consequence without removing advanced detail.
- **FR-010**: The slice MUST preserve plan-bound authorization, target-scoped routing, compatibility evidence rules, and passive Capture behavior.

### Key Entities

- **Current-session readiness**: A verdict over the process's actual privilege and the independent machine prerequisites for each live mode.
- **Steam setup completion**: A verified local target update that precedes, but does not constitute, reachability calibration.
- **First-use help**: Generated usage, flag descriptions, and embedded examples that describe the same command grammar and authorization line accepted by the CLI.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Controlled elevated and unelevated cases give matching human, structured, and exit readiness outcomes, with the unelevated case naming privilege as a blocker.
- **SC-002**: The complete setup-only and next-invocation test retains the authored client and reaches a fresh calibration decision with zero setup-only reachability claims.
- **SC-003**: All tested incorrect, incomplete, and CRLF exact-plan responses leave the target unchanged.
- **SC-004**: At 80 columns the Deep Capture help displays required `--launch`, a copyable output-recipient example, an exact-LF PowerShell example, and concrete sensitive-option consequences.
- **SC-005**: Repository CI parity and the existing controlled authorization/security tests pass at the final committed head.

## Assumptions

- The existing live-capture elevation refusal is intentional and remains the executable precondition.
- Steam client authoring intentionally ends before reachability; a later invocation needs a cold game and Steam state and a separate effect authorization.
- Structured exact-plan confirmation is a security boundary; S177 documents it and does not broaden accepted input.
- The slice does not change target storage schema, calibration fact eligibility, general JSON/silent flags (#488), unrelated bundle cleanup (#492), or release publication.
