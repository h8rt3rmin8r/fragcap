# Feature Specification: S163 Steam Calibration First Run

**Feature Branch**: `codex/s163-steam-calibration-first-run`
**Created**: 2026-10-03
**Status**: Draft
**Input**: Owner-approved S163 outline covering issues #443, #444, #445, #446, and #447.

## Clarifications

### Session 2026-10-03

- Q: How does a basename-only platform start establish the owned root? A: Require the managed launch receipt and matching PID, parent, start time, and expected image; compare a reported path when available. Never bind a prelaunch snapshot as that owned root.
- Q: How is a first-run client established without a socket-ownership guess? A: Prefer bounded, read-only observation of the owned launch chain and operating-system socket ownership; require a separate exact client-authoring confirmation. A socket observation is not captured traffic or proxy reachability. Permit an explicitly labeled manual choice when observation is unavailable or ambiguous.
- Q: What should a failed run report? A: Name the earliest stage proven to have failed, exact observed counters and fact writes, workflow state, and actionable next command. Unknown evidence stays unknown.

## User Scenarios & Testing

### User Story 1 - Launch an owned Steam target (Priority: P1)

An operator authorizes reachability calibration for a cold Steam target. Fragcap recognizes the exact platform process it created even when process telemetry reports only the executable name, dispatches the selected title once, and then separately determines whether the final client and its proxy traffic were observed.

**Why this priority**: The active v0.10.3 release times out before title dispatch because the owned root cannot satisfy its generated stage identity.

**Independent Test**: A controlled Steam-style launch reports a basename-only platform start for the launch receipt, binds exactly that process, dispatches once, and advances to final-client acquisition. Same-named foreign processes and changed creation identity do not dispatch.

**Acceptance Scenarios**:

1. **Given** a cold authorized launch and a creation event for the exact created platform process containing only its name, **when** calibration observes the event, **then** it binds the platform stage and dispatches the title once.
2. **Given** a different process with the same executable name, **when** it starts before or during the session, **then** it cannot acquire the owned platform stage or cause dispatch.
3. **Given** the platform binds but the title or final client fails later, **when** the session ends, **then** the later failure is reported accurately without claiming final-client reachability.

---

### User Story 2 - Establish the actual client during first setup (Priority: P1)

An operator starts with a stored Steam target and a Steam launch hint. The CLI explains that the hint may be an intermediate launcher, guides the operator through observable evidence or an explicit client choice, and records only a supported final-client declaration. The operator does not need to know what a network socket is.

**Why this priority**: The current technical yes-or-no question either invites a false assertion about a heuristic hint or ends calibration without a usable next step.

**Independent Test**: A controlled chain with a platform, a socketless launcher, and a traffic-owning client reaches a correctly authored client from a target selection and ordinary operator actions. Ambiguous or absent evidence remains unresolved and provides a next action without changing target authority.

**Acceptance Scenarios**:

1. **Given** a heuristic Steam launch hint that is only a launcher, **when** the operator enters guided setup, **then** the CLI labels it as a hint and does not request a blind socket-holder assertion.
2. **Given** directly observed evidence for one final client, **when** the operator reviews the proposed stored change, **then** only that client can be confirmed under a separate exact setup plan.
3. **Given** no usable or ambiguous client evidence, **when** setup stops, **then** the target remains unchanged and the CLI gives a concrete next action and retry command.
4. **Given** a declined, interrupted, stale, or invalid setup response, **when** the command exits, **then** no client identity is silently promoted and no Deep Capture session effect begins.

---

### User Story 3 - Understand target authority and session outcome (Priority: P2)

The operator can inspect the stored launch roles separately from Steam metadata and can tell whether calibration failed at platform binding, title dispatch, final-client acquisition, proxy reachability, or protocol observation. Written files and an inconclusive compatibility fact do not appear to be a successful capture.

**Why this priority**: The human target view displays a hint as `executable`, and the failed session repeats generic cleanup prose while burying the first failed stage.

**Independent Test**: A synthetic target with distinct hint and authored client shows both with accurate labels. Controlled outcomes for each launch and observation stage produce one prioritized human summary consistent with structured records.

**Acceptance Scenarios**:

1. **Given** distinct Steam hint and authored client values, **when** the operator runs `targets show`, **then** the active launch entry and role are visible and the hint is labeled as a hint.
2. **Given** an owned root that never binds, **when** calibration ends, **then** the report names that stage, zero retained target traffic and zero proxy accepts when observed, the exact compatibility fact disposition, and the workflow's next state.
3. **Given** several cleanup events, **when** the terminal report renders, **then** each exact resource outcome remains available without repeating the same unqualified progress sentence.
4. **Given** a completed artifact but no final-client traffic, **when** the report renders, **then** it does not claim calibration or Deep Capture success.

---

### User Story 4 - Follow the published first-run guide (Priority: P2)

A newcomer can follow documentation from a target name through client setup, cold launch, separate authorizations, reachability, possible protocol work, and ordinary Deep Capture eligibility, with clear recovery guidance for declines and failures.

**Why this priority**: The published guide names commands but leaves the socket-holder decision and paused-failure states unexplained.

**Independent Test**: The documented examples parse with the product command tree, and each branch in the guide corresponds to a controlled workflow outcome.

**Acceptance Scenarios**:

1. **Given** no stored client, **when** a newcomer follows the guide, **then** they can distinguish a launch hint from an observed final client and understand each authorization before responding.
2. **Given** a paused or failed attempt, **when** they consult the guide, **then** they can tell whether resume, a new attempt, or a product fix is needed without treating artifact creation as positive routing evidence.

### Edge Cases

- An event carries a basename, a full DOS path, or a Windows extended path spelling for the owned root.
- A foreign process shares the platform executable name, or a PID is reused after an earlier instance exits.
- The platform exits before dispatch, dispatch fails, the child escapes the owned ancestry, or a final client never starts.
- The Steam hint is a launcher; observation yields no traffic-owning client or several candidates.
- Setup input is declined, closed, invalid, interrupted, or arrives after target or Steam metadata changes.
- The target has no launch entry, one authored entry, or an ordered existing chain; target detail must not conflate any of these with a hint.
- Capture and proxy start but retain zero target packets or accept zero connections, while cleanup still succeeds or reports an unresolved obligation.
- Quiet, silent, JSON, and structured authorization modes preserve their required output and effect boundaries.

## Requirements

### Functional Requirements

- **FR-001**: The exact owned Steam root MUST be able to bind to its platform role from a basename-only creation event without requiring a full path in that event.
- **FR-002**: A same-named unowned process, prelaunch snapshot, stale receipt, or reused process identity MUST NOT authorize title dispatch; binding requires the owned launch receipt, matching PID and parent, matching creation instance within the launch interval, and the expected image. A reported full path MUST also match the prepared path. Successful dispatch remains one-shot.
- **FR-003**: Final-client ownership and final-client proxy reachability MUST remain separate observations and MUST NOT be inferred from platform binding or launcher traffic.
- **FR-004**: Guided Steam client setup MUST identify app metadata as a heuristic launch hint and MUST NOT require a novice to attest to network socket ownership.
- **FR-005**: Guided setup MUST offer bounded, read-only observation of the owned launch chain and operating-system socket ownership as the preferred client-identification evidence. It MUST distinguish one observed candidate, no evidence, and ambiguous candidates, and offer an explicitly labeled manual choice without treating that choice as observed proof or as traffic or proxy reachability. Unresolved setup MUST leave the stored target unchanged and provide a concrete next action.
- **FR-006**: Any client-authoring change MUST remain a separately reviewed, exact, drift-checked plan, with no effect from decline, invalid input, interruption, or changed authority.
- **FR-007**: `targets show` MUST display active launch entries and roles distinctly from metadata hints, while preserving existing stored values and machine-readable export semantics.
- **FR-008**: Failed calibration MUST identify the earliest directly known failed stage and report observed target retention, proxy acceptance, exact compatibility fact writes, workflow state, and exact cleanup outcomes. Missing evidence MUST be labeled unavailable rather than replaced by an inferred zero or later-stage claim.
- **FR-009**: Human lifecycle output MUST avoid repeated identical unqualified cleanup lines while preserving the authoritative structured event stream and complete terminal cleanup detail.
- **FR-010**: First-run documentation MUST distinguish guided from advanced calibration, setup from reachability, positive from inconclusive facts, and session artifacts from observed final-client traffic.
- **FR-011**: No part of this slice may weaken exact plan authorization, target-scoped routing, cold-launch requirements, process-ownership checks, loss accounting, or no-handle process observation.
- **FR-012**: Public examples and controlled fixtures MUST use synthetic target identities; operator-specific title names, app IDs, executable names, paths, endpoints, and raw captures MUST remain unpublished.

### Key Entities

- **Launch hint**: Heuristic Steam metadata used to help find a target, never direct proof of the traffic-owning client.
- **Owned platform root**: The exact process instance created by the authorized managed Steam launch and identified by its retained launch receipt.
- **Final client declaration**: The reviewed stored launch entry that identifies the terminal process for target traffic.
- **Calibration observation**: Directly recorded launch, process, packet, or proxy evidence for one exact session and case.
- **Compatibility fact**: An append-only exact-case result whose value can be positive, negative, or inconclusive.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A controlled basename-only owned-root case binds the platform and dispatches exactly once; every same-named foreign-root and stale-instance case dispatches zero times.
- **SC-002**: A controlled first-run chain with a socketless launcher and one observed socket-owning final client completes target setup with one separately authorized client change, then reaches a final-client proxy observation under a fresh calibration plan.
- **SC-003**: Every no-evidence, ambiguous, declined, interrupted, and stale-setup case changes zero target rows and starts zero unauthorized session effects.
- **SC-004**: Controlled target-detail cases with different hint and client values show both values under distinct labels and preserve exported bytes and stored authority.
- **SC-005**: Controlled failures at platform binding, title dispatch, final-client acquisition, and proxy reachability each report the correct earliest known stage and fact disposition; a written artifact with zero target traffic never reports success.
- **SC-006**: Every new first-run command example parses, all required project checks pass, and the issue acceptance criteria for #443 through #447 are covered by repository-controlled evidence.

## Assumptions

- This slice repairs the observed v0.10.3 first-run Steam workflow. It does not claim universal Steam or real-title proxy compatibility.
- The existing passive Capture observation is a source of components, but its current Steam resolution and automatic dominant-image promotion are unsuitable for guided setup. The new path must keep observation read-only until the separate authoring plan is confirmed.
- Publisher-launcher stage matching shares part of the path-versus-basename defect. S163 fixes the owned Steam root and records publisher behavior as a remaining limitation; it does not claim publisher-chain completion.
- Native protocol engines, compatibility storage schema, system proxy policy, certificate trust policy, package publication, and release version are outside this slice unless a demonstrated defect makes a narrow change necessary and that deviation is recorded.
- A corrected future release may be tested privately against operator-owned titles, but unspecified future field measurements are not an implementation issue completion gate.
- The controlled first-run scenario uses a reserved synthetic owned-chain socket fixture for client setup and the existing synthetic direct-launch proxy harness for the subsequent reachability plan. It proves the setup-to-plan authority path and proxy observation without claiming real Steam child-environment propagation.
