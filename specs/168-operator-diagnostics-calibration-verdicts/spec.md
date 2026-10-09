# Feature Specification: S168 operator diagnostics and calibration verdicts

**Feature Branch**: `codex/s168-operator-diagnostics-calibration-verdicts`
**Created**: 2026-10-09
**Status**: Draft
**Input**: Owner-authorized eight-issue slice covering #459, #460, #461, #462, #463, #465, #466, and #467.

## Scope

Deliver one coherent operator journey from Doctor and target discovery through early calibration preflight, plan authorization, observation, evidence diagnosis, and final readiness guidance. Every acceptance criterion of the eight scoped issues is binding; [issue-acceptance.md](issue-acceptance.md) preserves the exact acceptance inventory. Trace to master specification sections 15, 17.6, 17.7, 25, and 26.

Windows artifact ACL repair (#464), storage retention and collection (#458), missing packet-flow acquisition/correlation investigation (#468), IGDB enrichment (#155), and community sync (#94) remain outside this slice. Missing/inaccessible evidence must still be reported truthfully with its supported blocker and related issue. This slice does not claim those defects repaired or infer positive calibration from missing evidence.

## Clarifications

### Session 2026-10-09

- Q: Does narrower terminal width permit a smaller inter-column gap? A: No; preserve four-space anchors and exact tokens, with hanging continuations. No exception is requested.
- Q: How are retained healthy Doctor records inspected? A: Add an explicit human history-detail option while retaining all existing exact structured records and cleanup decisions.
- Q: May missing final-client correlation be treated as a gameplay pause? A: No; report the observed evidence blocker and #468. Preserve prior applicable stored facts independently.
- Q: What preflight responsiveness is measurable without a real game? A: A ready synthetic inventory/identity must refuse within one second and invoke zero modeled slow downstream stages or new workflow/session effects.
- Q: Is automated publication permitted? A: Yes, explicitly authorized for this slice, with at most one additional Codex review trigger and owner-only final merge.

All clarification coverage categories are clear; platform implementation choices are resolved during design research. No human clarification is required under the autopilot decision policy.

## User Scenarios & Testing

### User Story 1 - Read consistent terminal reports (Priority: P1)

An operator reads any human command report with aligned columns and exact values, including discovery organized by source and target.

**Why this priority**: #461 is an owner-mandated terminal-wide contract on which the other renderers depend.

**Independent Test**: Controlled output containing optional long labels, multiple columns, wide/combining characters, styling, wrapping, and redirected output has the same visible anchors and no lost facts.

**Acceptance Scenarios**:

1. **Given** actual emitted cells, **When** a report is rendered, **Then** each later column begins exactly four spaces beyond the maximum preceding column width, including headings and delimiters.
2. **Given** discovery candidates from interleaved sources, **When** discovery is shown, **Then** each source has one flush-left chapter preceded by an 80-character underscore rule, each target has four-space indentation, and its fields have eight-space indentation with one report-wide value anchor.
3. **Given** a narrow or plain terminal, **When** output wraps, **Then** continuation anchors, identities, and values remain exact without reducing the four-space gap or requiring color for hierarchy.

### User Story 2 - Find actionable Doctor results (Priority: P1)

An operator with many retained completed sessions sees concise healthy-history counts and prominent actionable findings with a documented detail path.

**Why this priority**: Repeated healthy resource paragraphs obscure readiness failures while retention is deliberately independent.

**Independent Test**: A synthetic large inventory containing healthy, mixed, active, failed, unknown, unsupported, and bounded-scan results produces compact default history output while retaining exact structured identities and decisions.

**Acceptance Scenarios**:

1. **Given** many healthy resource records, **When** ordinary Doctor runs, **Then** healthy history uses bounded summary rows with separate session/resource counts and coverage limitations.
2. **Given** mixed or active records, **When** Doctor summarizes history, **Then** no summary infers whole-session inactivity and every actionable finding remains individually diagnosable.
3. **Given** a request for detailed human or structured output, **When** Doctor runs, **Then** exact identities, readiness, exit codes, and cleanup selection remain available and ordinary Doctor stays read-only.

### User Story 3 - Prepare only the selected calibration case promptly (Priority: P1)

An operator calibrates a selected target without unrelated discovery work or warnings and learns a cold-Steam blocker before expensive setup.

**Why this priority**: Existing work before refusal delays every affected attempt and creates confusing unrelated progress.

**Independent Test**: Injected process and discovery adapters prove ordered refusal without invoking slow downstream stages or creating session/workflow effects; relevant scoped diagnostics remain visible.

**Acceptance Scenarios**:

1. **Given** running Steam and a Steam-dependent case, **When** preflight begins, **Then** the first substantive observation identifies the blocker before broad discovery, recursive scans, prompts, tracing, launch preparation, or session effects; only minimum exact identity lookup precedes applying the result.
2. **Given** explicit warm restart, **When** Steam is running, **Then** its existing operator-owned shutdown/action workflow is entered promptly, with no automatic process control.
3. **Given** an independent direct/publisher target, **When** Steam is running, **Then** Steam does not become a new eligibility blocker.
4. **Given** an exact stored/platform identity and an unrelated truncated root, **When** setup resolves the target, **Then** it does not scan that root or emit its warning; broad discovery still reports it.
5. **Given** unavailable inventory or Steam becoming warm later, **When** preparation proceeds, **Then** observation failure remains explicit and a fresh pre-effect check prevents invalid cold-launch assumptions.

### User Story 4 - Understand calibration evidence and readiness (Priority: P1)

An operator receives one supported verdict for the attempted case, a separate current applicable stored-case assessment, and one useful next action after every terminal path.

**Why this priority**: Completed artifacts and resource cleanup currently look like calibration success despite absent final-client evidence.

**Independent Test**: Synthetic first, second, resumed, interrupted, and failed attempts reconcile exchange, connection, phase, ownership, artifact, and cleanup authorities with the stored applicability result.

**Acceptance Scenarios**:

1. **Given** six connections ending in five protocol errors and one idle timeout, seven completed HTTP 200 exchanges, missing target ownership, partial trace/manifest, and successful cleanup, **When** the attempt ends, **Then** calibration remains inconclusive, all populations are separately labeled, and absent correlation names #468 rather than a speculative gameplay remedy.
2. **Given** current valid, stale, conflicting, or inapplicable facts, **When** readiness is assessed, **Then** attempted outcome and current requested-case readiness remain distinct and follow exact target/launch/route/family/backend/version/protocol identity.
3. **Given** later owner-release traffic, **When** calibration evidence is projected, **Then** it cannot silently authorize the earlier observation phase.
4. **Given** any success, refusal, pause, interruption, or failure, **When** final guidance is emitted, **Then** it states what is established, missing, or inapplicable, whether the requested Deep Capture case may proceed, and one supported command with purpose/prerequisites or the explicit unresolved blocker when no command exists.
5. **Given** plan authorization, **When** human presentation becomes concise, **Then** the complete reviewed canonical plan and exact binding remain available with unchanged confirmation and effects semantics.

### Edge Cases

Empty values, optional/indexed labels, three-or-more-column reports, control characters, ANSI styling, combining/wide Unicode, exact long tokens, narrow terminals, redirected stdout/stderr, NO_COLOR, quiet/silent/JSON modes, duplicate target names, repeated technology categories, zero candidates, bounded/incomplete inventories, unavailable ownership, stale/conflicting facts, failed fact writes, access denied, partial evidence, interrupted cleanup, resumes, and late phase evidence all have explicit regression coverage. No invocation count determines readiness.

## Requirements

### Functional Requirements

- **FR-001**: All human multi-column output MUST share the actual-visible-width plus four-space anchor rule, including headings, optional/indexed fields, every adjacent column pair, stdout/stderr, styling, and continuations. No legacy compact layout self-authorizes an exception.
- **FR-002**: Discovery MUST provide deterministic source chapters and target blocks, emit every candidate once and every distinct fact, preserve source/store/accounting/warnings, and retain unchanged discovery/registration data decisions and JSON fields.
- **FR-003**: Doctor MUST summarize healthy historical resources with separate session/resource counts and explicit bounded coverage, preserve all actionable and mixed/active findings, retain exact detail access and structured identities, and remain read-only.
- **FR-004**: Targeted resolution MUST avoid unrelated source scans when exact identity exists and scope diagnostics by source/root/target/operation provenance rather than message text. Relevant ambiguity, access, malformed metadata, coverage, and lookup limitations MUST remain visible in human and structured output.
- **FR-005**: Running-Steam observation MUST be the first substantive calibration preflight; apply it immediately after minimum target/workflow applicability lookup and before expensive setup or incidental effects. Warm restart, resume/status history, non-Steam targets, observation failures, and fresh pre-effect revalidation MUST retain their existing authority.
- **FR-006**: Controlled warm-Steam refusal MUST finish within one second with ready injected inventory/identity while downstream stages each model at least five seconds or fail if invoked; downstream calls and new session/workflow writes MUST equal zero. This is a controlled responsiveness budget, not a claim about arbitrary Windows scheduling or storage.
- **FR-007**: Diagnosis MUST use typed independent populations for proxy admission/authentication/terminal causes, completed exchanges, classification records, ownership/correlation, packet conservation, observation versus owner-release windows, evidence completeness, and cleanup. Unavailable/bounded causes MUST remain explicit and totals MUST reconcile.
- **FR-008**: Every terminal calibration path MUST emit an attempted-case verdict and current applicable requested-case assessment. Routing and protocol inspectability, finalization, artifact access/completeness, and cleanup MUST remain separate authorities. Prior valid facts MUST not be silently erased by an unrelated or inconclusive attempt.
- **FR-009**: Every incomplete/paused path MUST name its observed blocker and supported next action. Missing correlation MUST not become generic gameplay advice; no executable remedy MUST be stated plainly with the unresolved issue. Success MUST provide the requested case's appropriate capture command.
- **FR-010**: Human calibration MUST use phase/result chapters, one transition per lifecycle change, one fact-write summary with distinct writes/failures, exact attempt/resume context, and one bundle root plus exact artifact statuses/external paths. Omitted artifacts MUST not imply a written file. Live/final populations and owner-release deadlines MUST be distinguished.
- **FR-011**: Human and structured decision/reason output MUST agree; quiet/silent/NO_COLOR/redirected contracts MUST remain explicit. Existing schemas and fields MUST remain compatible; necessary diagnostic additions MUST be additive and documented.
- **FR-012**: Complete canonical plan review, exact consent binding, target-scoped effects, finite bounds, loss accounting, and cleanup MUST retain their existing authorization semantics. Diagnosis MUST omit capabilities, credentials, raw payloads, and unnecessary private identifiers.
- **FR-013**: Repository directives, master specification, public guides/help, and examples MUST describe the resulting alignment, discovery, Doctor detail, and calibration verdict contracts. Text MUST be UTF-8 without BOM and contain no mojibake.
- **FR-014**: Completion MUST cover all scoped issue acceptance criteria with repository-controlled regressions and required gates. No real game, actual trust mutation, future field observation, new release, ACL repair, retention deletion, or inferred acquisition fix is required or claimed.

### Key Entities

- **Aligned report block**: Actual emitted cells, indentation, visible widths, anchors, and exact continuation values.
- **Scoped discovery diagnostic**: Provenance and relevance to the selected command/target, with the original limitation preserved.
- **Healthy history summary**: Distinct session and resource counts, inventory coverage, and retained exact detail.
- **Calibration assessment**: Attempted identity/outcome, current exact applicable facts, independent evidence populations, omissions/limits, and supported next action.
- **Evidence window**: Observation phase and subsequent owner-release interval with independently eligible observations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every inventoried human multi-column renderer follows the four-space rule in controlled representative output, with identical color/plain visible anchors and no truncated values.
- **SC-002**: Default Doctor healthy-history row count remains constant as healthy resources grow within the inventory bound, while every actionable finding and coverage limitation remains visible.
- **SC-003**: All exact-target/Steam setup fixtures exclude unrelated root enumeration/warnings; broad-discovery fixtures retain those warnings and full accounting.
- **SC-004**: Warm-Steam refusal meets FR-006 and invokes zero slow setup stages or new workflow/session writes; cold, inventory-error, warm-race, restart, resumed, and non-Steam cases preserve their expected outcomes.
- **SC-005**: Every controlled terminal calibration case produces exactly one final verdict and supported next action, reconciles all evidence populations, and preserves valid current stored readiness independently from attempt count.
- **SC-006**: The six-failure/seven-exchange fixture remains explicitly inconclusive with missing ownership, visible partial evidence, and successful cleanup, in both human and structured decisions.
- **SC-007**: All eight issue acceptance inventories have implementation and verification evidence; full repository checks pass before publication, and no release or real-game result is claimed.

## Assumptions

The existing target, compatibility, workflow, and native evidence authorities remain the source of decisions. Additional summaries project their facts rather than introducing a second readiness authority. A one-second controlled preflight budget and an always-80-character discovery divider are slice decisions. Discovery values use the existing default terminal color; headings/keys use existing color policy. Exact long tokens may exceed width rather than being modified. Publishing the branch/official PR is explicitly authorized; the owner remains the final reviewer and merger. At most one additional Codex review trigger is authorized after the automatic initial round.
