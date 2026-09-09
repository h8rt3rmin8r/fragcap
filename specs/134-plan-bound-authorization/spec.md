# Feature Specification: Plan-Bound Deep Capture Authorization

**Feature Branch**: `codex/134-plan-bound-authorization`

**Created**: 2026-09-09

**Status**: Draft

**Input**: S134 closes issue #382 by replacing Deep Capture `--trust-ca` and `--yes` with one authorization bound to a complete immutable plan.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review and authorize one complete session plan (Priority: P1)

An authorized operator starts ordinary Deep Capture or compatibility calibration, reviews every effect-bearing part of the exact session plan, and provides one affirmative decision for that plan before fragcap binds a listener, creates a bundle, changes trust, launches a target, or appends a compatibility fact.

**Why this priority**: The current flags authorize overlapping subsets of behavior before the operator can review the concrete session. A single complete decision is the security and usability boundary for every later effect.

**Independent Test**: Run a controlled session through the interactive authorization seam and prove that the displayed plan contains the exact target, launch, trust, artifact, routing, deadline, cleanup, and refusal information, that one affirmative response starts only that plan, and that decline or end-of-input leaves every effect counter at zero.

**Acceptance Scenarios**:

1. **Given** an interactive terminal and a valid Deep Capture request, **When** preparation completes, **Then** fragcap displays the complete immutable authorization plan and asks once whether to authorize its exact identifier.
2. **Given** a displayed plan, **When** the operator answers affirmatively, **Then** fragcap executes the exact authorized plan without asking for a second trust confirmation.
3. **Given** a displayed plan, **When** the operator declines, closes input, or interrupts authorization, **Then** fragcap reports the refusal and performs no session effect.
4. **Given** a target or launch authority that changes after display, **When** fragcap realizes the authorized plan, **Then** it refuses the drift before starting any session effect.

---

### User Story 2 - Authorize the exact plan from structured automation (Priority: P1)

An authorized automation client consumes the structured plan event and returns that plan's complete identifier over the dedicated authorization input without receiving or relying on a generic approve-everything switch.

**Why this priority**: JSON and redirected workflows must remain usable without prompts, but a reusable boolean cannot safely authorize a certificate, target, bundle, and launch that did not exist when the command started.

**Independent Test**: Drive the controlled command through redirected input and structured output, return the emitted identifier, and prove that an exact match proceeds while an absent, malformed, stale, replayed, or different identifier refuses with zero effects.

**Acceptance Scenarios**:

1. **Given** structured output with dedicated plan-authorization input selected, **When** fragcap emits the authorization plan, **Then** it emits no prose prompt and waits only for the exact plan identifier.
2. **Given** the exact current plan identifier, **When** automation returns it on the dedicated input, **Then** the current plan may execute.
3. **Given** a missing, malformed, stale, or different identifier, **When** automation returns it or closes input, **Then** fragcap refuses before every session effect.
4. **Given** structured output without dedicated plan authorization, **When** the request reaches authorization, **Then** fragcap refuses as a usage error rather than prompting or assuming consent.

---

### User Story 3 - Calibrate reachability without approving trust (Priority: P2)

An operator running reachability-only calibration reviews and authorizes its launch and routing plan while seeing that the plan contains no certificate trust action, HAR output, or TLS key log.

**Why this priority**: Reachability calibration still has launch and proxy effects, but presenting it as approval of a trust change would repeat the current confusing model and misstate what the phase does.

**Independent Test**: Run controlled reachability calibration and prove the plan says that trust mutation and TLS-sensitive artifacts are absent, the authorization language does not request trust approval, and no trust adapter is invoked.

**Acceptance Scenarios**:

1. **Given** reachability-only calibration, **When** the plan is displayed, **Then** it states that the trust action, HAR output, and TLS key log are absent.
2. **Given** an authorized reachability plan, **When** calibration runs, **Then** no current-user Root-store operation is attempted.
3. **Given** trust or TLS-sensitive output options on reachability calibration, **When** the request is validated, **Then** fragcap refuses before authorization and effects.

---

### User Story 4 - Migrate away from legacy Deep Capture flags (Priority: P3)

An operator or script that still supplies Deep Capture `--trust-ca` or `--yes` receives actionable migration guidance without either legacy flag authorizing a session.

**Why this priority**: Immediate silent removal would make existing commands confusing, while continuing to honor either boolean would preserve the security defect.

**Independent Test**: Invoke each legacy flag in human and structured modes and prove it is absent from normal help, returns a usage error naming the new flow, and invokes no target, endpoint, proxy, trust, artifact, launch, capture, or fact effect.

**Acceptance Scenarios**:

1. **Given** normal Deep Capture help, **When** it is rendered, **Then** neither legacy flag appears and the new interactive and automation authorization flows are described.
2. **Given** either legacy Deep Capture flag during the compatibility release, **When** arguments are validated, **Then** fragcap refuses with migration guidance and never treats the flag as consent.
3. **Given** an unrelated `--yes` owned by Doctor, bundle cleanup, or target reconciliation, **When** that command runs, **Then** its existing separately scoped confirmation contract is unchanged.

### Edge Cases

- The authorization identifier is compared exactly after removing only the input line ending; leading or trailing spaces, prefixes, abbreviations, and case changes do not match.
- A second identifier line cannot authorize a later plan, and one accepted identifier cannot be replayed after the owning prepared session is consumed.
- A plan whose target identity, launch case, certificate, bundle destination, routing policy, artifact selection, deadline, or product version changes is a different plan and requires a new authorization.
- The concrete loopback port may be selected only after authorization, but it must remain within the displayed address family and loopback-only scope or the session refuses.
- Generating in-memory session identity or certificate material for review is not a trust mutation; declining destroys that unpersisted material and leaves no cleanup obligation.
- A warm-to-cold transition may retain its separately explained operator-close interaction, but only the final freshly prepared cold plan can authorize session effects and generic preconfirmation cannot bypass either decision.
- An output, flush, or input error during authorization refuses the session rather than assuming approval.
- Narrow-terminal rendering may wrap values but cannot omit or truncate authorization-relevant fields.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: fragcap MUST construct one immutable authorization plan before every ordinary Deep Capture, reachability calibration, or TLS calibration session.
- **FR-002**: The plan MUST identify the session, exact stored target, fully resolved profile and managed launch authority, observed and requested launch case, target-scoped proxy routing and bypass policy, loopback address family, bundle destination, requested sensitive outputs, exact millisecond effective deadlines, product and backend versions, cleanup obligations, important refusal boundaries, and possible compatibility fact writes.
- **FR-003**: A trust-bearing plan MUST identify the exact current-user Root-store addition, certificate thumbprint, temporary lifetime, and removal obligation in plain language.
- **FR-004**: A reachability-only plan MUST state that it performs no trust mutation and produces no HAR or TLS key log.
- **FR-005**: The complete plan MUST be emitted in human or structured form before listener bind, bundle creation, proxy start, trust mutation, routing application, target launch, packet capture start, artifact persistence, or compatibility fact append.
- **FR-006**: Every plan MUST have one complete canonical identifier that changes when any authorization-relevant field changes, including target identity, launch authority, certificate, paths, routing policy, deadlines, artifact selection, backend version, or product version.
- **FR-007**: Interactive use MUST request exactly one affirmative authorization for the complete plan and MUST NOT request a second confirmation for the trust action already described by that plan.
- **FR-008**: A negative answer, unrecognized answer, incomplete input line, end-of-input, input error, plan or prompt write error, flush error, or interruption during authorization MUST refuse with no session effects.
- **FR-009**: Structured output MUST never emit an interactive prompt and MUST require the dedicated exact-plan authorization input.
- **FR-010**: Redirected automation MUST return the complete current plan identifier through the dedicated input; no constant, wildcard, abbreviation, reusable preference, installation state, or generic boolean may authorize a plan.
- **FR-011**: The plan identifier comparison MUST accept only the exact identifier after removing the input line ending and MUST use a comparison suitable for authorization-sensitive values.
- **FR-012**: Authorization MUST be single-use and bound to the in-memory prepared plan that emitted the identifier.
- **FR-013**: fragcap MUST revalidate stored target identity immediately after approval and MUST independently resolve and compare the full profile, client executable, platform root, dispatch, and managed launch authority inside the facade's final target resolver before endpoint selection, MUST revalidate trust identity, validity, paths, routing policy, deadlines, artifacts, and versions before the first session effect, and MUST refuse any drift or expired prepared authority.
- **FR-014**: Selecting and binding the concrete listener after authorization MUST remain inside the displayed loopback family and scope; failure to realize that scope MUST refuse without widening or fallback.
- **FR-015**: Declined or failed authorization MUST leave generated session and certificate material unpersisted and MUST create no cleanup obligation.
- **FR-016**: Deep Capture `--trust-ca` and Deep Capture `--yes` MUST be absent from normal help and MUST NOT authorize any behavior.
- **FR-017**: During one bounded tagged-release compatibility period, either legacy flag MUST produce an actionable usage error naming the interactive exact-plan flow and the dedicated automation input; the flags become eligible for parser removal after that period.
- **FR-018**: Unrelated command-specific `--yes` options for Doctor repair, bundle cleanup, and target reconciliation MUST remain unchanged.
- **FR-019**: The warm-to-cold workflow MUST preserve its operator-owned normal-shutdown boundary, MUST prepare a fresh cold plan after shutdown, and MUST use the same final exact-plan authorization as every other session.
- **FR-020**: Authorization outcomes MUST be represented in structured and human reporting without exposing private certificate material, proxy credentials, local target inventory beyond the selected target, or captured payloads.
- **FR-021**: Cleanup and crash recovery MUST retain the existing exact certificate ownership, journal, retained-evidence, and truthful terminal-report guarantees after authorization succeeds. Owner-registry records MUST become discoverable only through complete atomic publication. A new Deep Capture invocation MUST inspect prior-session recovery authority without mutation and MUST direct the operator to Doctor before emitting a new plan when pending recovery actions exist.
- **FR-022**: Tests MUST cover human, structured, redirected-input, narrow-terminal, decline, incomplete-line end-of-input, plan write failure with successful flush, mismatch, replay, plan drift at both resolver boundaries, prepared-authority expiry, pending prior recovery, warm restart, reachability, TLS, cleanup failure, and controlled automation without real trust mutation, game launch, capture driver, or elevated privilege.
- **FR-023**: S134 MUST NOT implement the guided one-game calibration flow owned by #380, the complete embedded-help rewrite owned by #379, broader first-run UX owned by #332, or final documentation and completion gates owned by #331 through #334.

### Key Entities

- **Authorization Plan**: The complete immutable review object containing every authorization-relevant session field and one canonical identifier.
- **Plan Identifier**: The complete single-use authorization value derived from the canonical plan and compared exactly against the operator or automation response.
- **Trust Action**: The exact certificate identity, current-user store scope, temporary lifetime, and cleanup obligation, or an explicit statement that no trust action exists.
- **Authorization Decision**: An exact approval, decline, interruption, invalid response, input or output failure, plan-drift refusal, or expired-plan refusal associated with one plan.
- **Plan Realization**: The post-authorization conversion of the approved scope into an exact listener, proxy route, launch, artifacts, and lifecycle resources without widening any displayed boundary.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every effect-bearing controlled test records exactly one complete plan before its first effect, and 100 percent of decline, end-of-input, mismatch, replay, and drift cases record zero effects.
- **SC-002**: An interactive first-time operator can reach the complete plan and authorize or decline it without supplying an authorization flag and without receiving duplicate trust questions.
- **SC-003**: A structured automation client can authorize the current plan with one exact identifier response, while every altered identifier or altered plan is refused.
- **SC-004**: Every trust-bearing plan displays the exact certificate thumbprint and current-user cleanup promise, and every reachability plan displays zero trust actions and zero TLS-sensitive outputs.
- **SC-005**: Normal Deep Capture help contains zero legacy authorization flags, and every legacy invocation exits with one actionable migration error during the bounded compatibility period.
- **SC-006**: All authorization and session lifecycle tests complete without a real trust-store mutation, target process, capture driver, elevation, or external network service.
- **SC-007**: Existing exact cleanup, recovery, artifact, compatibility-fact, and unrelated command confirmation tests remain green with no weakened assertion.

## Assumptions

- S134 uses the existing library-first prepared-session and exact-plan authorization boundary rather than creating a second lifecycle coordinator.
- A dedicated standard-input identifier handshake is the automation contract because it allows the same process that generated ephemeral certificate material to receive exact authorization without persisting private key material or accepting a replayable global switch.
- The concrete loopback port is a realization detail inside the authorized loopback family and target-scoped routing boundary; no wildcard or system-wide endpoint is permitted.
- The compatibility period begins with the first tagged release containing S134 and lasts for that release only; the legacy flags are accepted solely to return migration errors and never retain their former behavior.
- Existing command-specific confirmation flags outside Deep Capture are separate action models and remain out of scope.
