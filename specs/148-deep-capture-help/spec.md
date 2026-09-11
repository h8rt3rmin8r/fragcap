# Feature Specification: Deep Capture Embedded Workflow Help

**Feature Branch**: `codex/s148-deep-capture-help`

**Created**: 2026-09-11

**Status**: Draft

**Input**: User description: "Make the complete Deep Capture workflow discoverable from embedded CLI help and actionable first-run refusals, closing issue #379 without live game or sensitive execution."

## Clarifications

### Session 2026-09-11

- Q: Which refusals form the closed first-run inventory? A: Include every post-parse, pre-session stop on the documented journey: environment readiness, target resolution, launch authority, process state, current compatibility, prior recovery, bundle destination, authorization freshness or response, and guided-calibration choice, pause, limitation, or continuation. Ordinary argument grammar remains Clap's usage contract, and failures after a session starts remain typed runtime outcomes.

## User Scenarios & Testing

### User Story 1 - Discover the first successful path (Priority: P1)

A first-time operator who knows only an installed game name can use embedded help to move from environment readiness through target registration and calibration to the exact ordinary Deep Capture command, without consulting the website or source code.

**Why this priority**: The command is currently discoverable while the workflow needed to use it is not, which turns safe refusals into trial and error.

**Independent Test**: Invoke short and long help for the root command, Doctor, target discovery and registration, target detail, calibration, and Deep Capture, then verify that the complete ordered journey and pasteable commands can be reconstructed offline.

**Acceptance Scenarios**:

1. **Given** only an installed game name, **When** the operator follows the long-help journey, **Then** the help leads through Doctor, target discovery or registration, guided calibration, and the ordinary Deep Capture command in execution order.
2. **Given** a stored target that is not yet compatible, **When** the operator reads target detail or Deep Capture help, **Then** the help distinguishes registration from calibration and explains that compatibility must be observed before an ordinary session.
3. **Given** a supported Steam target or a supported direct or publisher launch, **When** the operator reads Deep Capture long help, **Then** at least one realistic pasteable example for each launch family is available.

---

### User Story 2 - Choose safe defaults without hiding advanced controls (Priority: P2)

An operator can identify the normal first-run inputs and understand which controls are advanced, sensitive, or troubleshooting-specific before authorizing any Deep Capture effect.

**Why this priority**: Deep Capture has a broad option surface with materially different security, privacy, and compatibility consequences.

**Independent Test**: Render the audited long help at supported narrow widths and with color enabled and disabled, then verify ordered prerequisites, stable section boundaries, truthful limitations, and visible advanced controls.

**Acceptance Scenarios**:

1. **Given** a first-time operator, **When** Deep Capture long help is displayed, **Then** required inputs and the safe default path precede common, advanced case and protocol, networking, mutual TLS, key-log, custom-storage, and troubleshooting controls.
2. **Given** an operator considering TLS inspection, **When** the operator reads help, **Then** the wording states that inspection depends on observed compatibility and never promises universal decryption or certificate-pinning bypass.
3. **Given** a narrow terminal or disabled color, **When** audited help is rendered, **Then** commands and security boundaries remain readable and no meaning depends on color.

---

### User Story 3 - Recover from every first-run stop (Priority: P3)

An operator who reaches a refusal, partial calibration result, interrupted-session recovery requirement, or evidence-bundle decision receives one exact next command or a finite set of exact alternatives.

**Why this priority**: A refusal is safe only if it preserves an understandable route forward; vague references to another feature strand the operator.

**Independent Test**: Exercise the closed inventory of pre-session first-run refusals and calibration terminal states with controlled inputs, then verify that every actionable state names its established facts, remaining unknowns, and parseable next command.

**Acceptance Scenarios**:

1. **Given** environment readiness but no usable stored target, **When** Doctor or Deep Capture guidance is rendered, **Then** the operator is directed to an exact target discovery or registration command.
2. **Given** a target with unknown or partial compatibility, **When** an ordinary session is refused or calibration stops, **Then** the output states what is known, what remains unknown, and the exact calibration or ordinary-session command that follows.
3. **Given** interrupted-session residue or a sensitive bundle, **When** recovery, cleanup, or export help is requested, **Then** the help identifies sensitive artifacts, destructive cleanup, required retained records after failed cleanup, and the exact Doctor or bundle command to use.

### Edge Cases

- Short help must remain concise even when long help carries the complete workflow.
- Help examples must remain correct when a target selector contains spaces or non-ASCII text.
- Structured output and non-interactive execution must not receive decorative prose that changes their machine contract.
- Warm, ambiguous, stale, unknown, interrupted, or partially observed states must not be described as compatible.
- A failed cleanup must not recommend deleting the records needed for exact recovery.
- A command example that needs a user-provided value must show an unmistakable placeholder and remain backed by a parse test with a concrete representative value.
- The journey must remain navigable at 40 display columns and when ANSI color is unavailable.

### Closed First-Run Refusal Inventory

- Environment readiness is incomplete and Doctor has an exact remediation or next diagnostic command.
- The target store is unavailable, empty, unmatched, ambiguous, stale, or requires one explicit discovery or registration choice.
- The selected target lacks exact managed-launch authority, has an unsupported or ambiguous topology, or is already warm.
- Current exact reachability or protocol compatibility evidence is absent, stale, conflicting, negative, partial, or still unknown.
- A prior interrupted session owns residue that must be reviewed or recovered before a new plan can proceed.
- The selected bundle destination is unsafe, conflicts with retained evidence, or cannot preserve required recovery records.
- The authorization plan expires or drifts, plan output fails, or the operator declines, interrupts, or supplies an invalid response.
- Guided calibration needs an explicit candidate, client, launch case, route, family, protocol, operator exercise pause, resume, or bounded alternative before another effect can be proposed.

## Requirements

### Functional Requirements

- **FR-001**: The root short help MUST keep a concise summary that exposes Doctor, targets, calibration, Deep Capture, and bundle operations without embedding the complete tutorial.
- **FR-002**: Long help across the audited command set MUST present one ordered first-run path from an installed game name to an ordinary Deep Capture invocation.
- **FR-003**: The audited command set MUST include the root command, Doctor, targets, targets discovery, target registration, target detail, guided calibration, Deep Capture, bundle cleanup, and bundle export.
- **FR-004**: Deep Capture long help MUST state prerequisites in execution order and include at least one Steam example and one direct or publisher-launch example.
- **FR-005**: Help MUST distinguish target registration or selection, compatibility calibration, ordinary Deep Capture, interrupted-session recovery, destructive cleanup, and evidence export.
- **FR-006**: Help MUST explain why ordinary Deep Capture requires a stored target, managed launch authority, and current observed compatibility.
- **FR-007**: Long help MUST identify the safe default path before visually distinct common, advanced case and protocol, networking, sensitive output and mutual TLS, custom-storage, and troubleshooting controls.
- **FR-008**: Every state in the closed pre-session first-run refusal inventory MUST end with one exact next command or a finite set of exact command alternatives.
- **FR-009**: Calibration completion and pause guidance MUST state what was established, what remains unknown, and either the exact ordinary Deep Capture command or the exact continuation command.
- **FR-010**: Doctor readiness guidance MUST direct an environment-ready operator toward target registration or calibration when environment readiness alone is insufficient.
- **FR-011**: Bundle and recovery help MUST identify sensitive artifacts, destructive operations, and the records that must be retained when cleanup is incomplete.
- **FR-012**: Human wording MUST NOT claim universal decryption, automatic certificate-pinning bypass, or compatibility that was not directly observed.
- **FR-013**: Short and long help MUST remain readable at supported widths from 40 through 80 display columns, and meaning MUST remain complete with color disabled.
- **FR-014**: Automated semantic or snapshot tests MUST cover short and long help for every audited command and actionable text for every closed first-run refusal category.
- **FR-015**: Every command example emitted by audited help or refusal guidance MUST be represented by a parseable command contract exercised in automated tests.
- **FR-016**: Embedded workflow terminology and command examples MUST have one maintainable synchronization boundary for the final public documentation owned by issue #331.
- **FR-017**: Help and refusal improvements MUST preserve structured-output schemas, effect authorization, cleanup authority, compatibility evidence rules, and existing command behavior.
- **FR-018**: Validation MUST use controlled or side-effect-free execution only and MUST NOT require a real game, real trust-store mutation, live sensitive capture, or a compatibility claim.

### Key Entities

- **Workflow Stage**: One operator-visible step in the ordered journey, with prerequisites, purpose, and the next exact command.
- **Command Example**: A displayed pasteable command paired with a representative argument sequence that automated parsing can validate.
- **Help Section**: A stable presentation group for required, common, advanced, sensitive, custom-storage, or troubleshooting controls.
- **First-Run Refusal**: A closed, named pre-session state that blocks effects or ordinary execution and carries exact recovery guidance.
- **Calibration Outcome**: A truthful statement of established evidence, unresolved evidence, and the next ordinary or continuation command.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Starting with only an installed game name, every command needed to reach the supported ordinary Deep Capture invocation is discoverable through embedded long help in one continuous ordered path.
- **SC-002**: Both required launch-family examples and every emitted next-command example parse successfully in automated tests.
- **SC-003**: Every audited command has passing short-help and long-help coverage, and every item in the closed first-run refusal inventory has actionable-guidance coverage.
- **SC-004**: At 40, 60, and 80 display columns, audited help retains every command token and security boundary without truncation or color dependence.
- **SC-005**: Controlled validation completes without a game account, remote game service, capture driver, elevated privilege, real trust mutation, or sensitive live traffic.
- **SC-006**: Existing machine-readable schemas, authorization decisions, compatibility records, and effect counts remain byte- or value-compatible except where human-only prose is intentionally improved.

## Assumptions

- The guided calibration workflow completed by S147 remains the sole supported path for producing first compatibility evidence.
- Target discovery and registration keep their existing one-store authority and precedence.
- The final public documentation under issue #331 will consume the same workflow vocabulary and command-example boundary established here, but authoring that complete documentation is outside S148.
- First-run refusal scope ends before a successfully authorized session begins; runtime protocol, transport, writer, and cleanup failures retain their existing typed behavior unless they are part of interrupted-session recovery guidance.
- The existing supported terminal-width policy is 40 through 80 display columns.

## Out of Scope

- Running a real game, mutating the real trust store, or capturing sensitive live traffic.
- Claiming title compatibility or completing general Deep Capture issue #334.
- Rewriting the full architecture specification inside terminal help.
- Completing the public documentation set owned by issue #331.
- Hiding advanced or security-sensitive controls that an operator explicitly selects.
