# Feature Specification: S173 Bundle access stabilization

**Feature Branch**: `codex/s173-bundle-access-stabilization`

**Created**: 2026-10-19

**Status**: Specified

**Input**: Owner-authorized S173 combines #476 automatic same-account elevated output access and #477 historical access inspection/repair retirement, with automatic PR publication and at most two review rounds.

## Clarifications

### Session 2026-10-19

- Q: Does removing historical repair also remove different-account authorization? A: No. Retain the authenticated new-output helper and explicit recipient/destination contract (FR-005).
- Q: Can equivalent restricted-token tests substitute for the elevated regression? A: No. Required elevated production acquisition must run; equivalents remain supplemental (FR-009).
- Q: What happens to earlier shipped contracts? A: Preserve historical records and label the changed current-source contract separately (FR-008).

## User Scenarios & Testing

### User Story 1 - Use elevated output from the normal desktop (Priority: P1)

A signed-in Windows user runs Deep Capture or calibration once in a same-account elevated terminal and reads its completed output from the normal desktop without identifying their own account or running another terminal helper.

**Why this priority**: The published v0.10.6 default recipient lookup fails before output creation and prevents the intended workflow (#476).

**Independent Test**: A controlled genuinely elevated Windows producer creates synthetic output through production recipient acquisition and publication; the matching ordinary context enumerates its parents and reads every expected artifact after reconciliation.

**Acceptance Scenarios**:

1. **Given** a same-account split-token Windows desktop, **When** an elevated producer prepares default output for Deep Capture or calibration, **Then** the ordinary recipient is established automatically without a SID argument, explicit destination or second terminal.
2. **Given** successfully reconciled synthetic output, **When** the ordinary recipient enumerates and reads it, **Then** parent containers, independent journals, empty files and final/atomic artifacts are accessible; unrelated ordinary authority is denied private evidence.
3. **Given** recipient acquisition fails, **When** the command reports failure, **Then** it names the failed acquisition stage and preserves the platform error without claiming readable output or prescribing self-SID handoff as the normal remedy.

### User Story 2 - Use the maintained bundle commands (Priority: P1)

The operator sees only maintained bundle operations and truthful unresolved-access guidance, with the historical repair workflow retired (#477).

**Why this priority**: The owner recovered historical output outside fragcap and explicitly requests removal of the failed repair feature.

**Independent Test**: Both retired verbs are rejected by argument parsing, help and current guidance contain no instruction to invoke them, and collection, sensitive cleanup and Doctor still operate on controlled legacy fixtures.

**Acceptance Scenarios**:

1. **Given** current command help, **When** the operator lists bundle operations, **Then** historical access inspection and repair are absent and invoking either retired verb fails before effects.
2. **Given** legacy bundle fixtures, **When** supported cleanup, collection or recovery is requested, **Then** the established operation and retention rules remain usable without adopting historical permission repair.
3. **Given** inaccessible historical output, **When** fragcap reports it, **Then** concise guidance identifies the unresolved status and a supported manual access-administration next step without promising automatic repair.

### User Story 3 - Select a genuinely different recipient (Priority: P2)

Explicit different-account output keeps its authenticated recipient and exact destination semantics.

**Why this priority**: Removing historical repair must preserve the independently supported new-output recipient path.

**Independent Test**: Controlled authenticated handoffs retain exact recipient/path binding, ordinary-context checks, finite timeout and refusal of mismatched or elevated helpers.

**Acceptance Scenarios**:

1. **Given** explicit different-account output, **When** an ordinary helper authenticates the exact request and destination, **Then** the selected recipient contract is retained for new output.
2. **Given** a missing, expired, mismatched or elevated helper, **When** authentication is attempted, **Then** it refuses before output or session effects.

### Edge Cases

- Ordinary, limited and full elevated tokens; unavailable linked ordinary context; session/account mismatch; platform errors at acquisition, duplication, validation and restoration.
- Inherited parent permissions, independently protected sidecars, atomic replacement and post-reconciliation read failure.
- Both retired verbs with otherwise valid flags, JSON and legacy bundles.
- Legacy closed or crash-prefix artifacts, missing action journals, resource obligations and finite history retention.

## Requirements

### Functional Requirements

- **FR-001**: Same-account elevated Deep Capture and calibration MUST establish the initiating ordinary recipient automatically without self-SID input, an explicit destination or a separate helper.
- **FR-002**: Recipient authority MUST prove exact account/session association and ordinary elevation/integrity; matching identity alone MUST NOT establish ordinary access.
- **FR-003**: Acquisition failures MUST identify their operation stage and preserve platform error evidence; readable-output success requires post-reconciliation ordinary-context access verification.
- **FR-004**: New output MUST retain recipient access across parent creation, all writers, protected journals, empty files and final/atomic publication while denying unrelated ordinary authority private evidence.
- **FR-005**: Genuinely different-account elevation MUST preserve explicit authenticated recipient/destination semantics and finite refusal behavior through the retained authorization helper.
- **FR-006**: Public historical access inspection/repair verbs, facade API, implementation and exclusive tests MUST be removed. Current help, next-command hints, recovery instructions and maintained product docs MUST stop recommending them.
- **FR-007**: Collection, sensitive cleanup, Doctor recovery/retention, new-output protection and final access verification MUST remain usable including on supported legacy bundles. Retirement MUST NOT broaden permissions or delete evidence as a side effect.
- **FR-008**: Unresolved historical access guidance MUST state actual status and direct the owner to normal Windows access administration without promising automated historical repair. Historical release/spec records MUST retain what v0.10.6 shipped; maintained contracts MUST distinguish the source change from published behavior.
- **FR-009**: Controlled Windows evidence MUST exercise genuine elevated production recipient acquisition and ordinary enumeration/full reads of synthetic new output, plus unrelated-authority denial. Restricted equivalents supplement but do not replace elevated-path evidence. No owner bundle, game, real trust change or manual operator run is a completion gate.
- **FR-010**: The complete spec-kit analysis and repository-controlled gates MUST pass. Human reports retain shared display layout, structured output retains its existing contracts, and all child console tooling is hidden/noninteractive.

### Key Entities

- **Output recipient**: Exact individual account, session and retained proven ordinary context, distinct from the producer's elevated authority.
- **Access result**: Actual post-reconciliation enumeration/read outcome, independent of content completeness and resource cleanup.
- **Retired historical workflow**: Inspection/proposal/repair surface removed from the current product while its release records remain historical.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every required same-account elevated synthetic new-output case completes without self-identification or helper steps and passes full ordinary-context enumeration/read checks.
- **SC-002**: Both retired verbs are rejected; no maintained guidance instructs their use; all retained legacy operations pass their scoped regressions.
- **SC-003**: Every required authenticated handoff and access-denial case has the expected outcome with zero unreported verification failures.
- **SC-004**: Both issues' acceptance matrices and repository gates pass with exact evidence and explicit limits; neither issue depends on future owner testing.

## Assumptions

- S172/v0.10.6 and completed S170/S171 remain historical implementation records; #476 is a new defect, not reopening #464.
- No storage-schema migration, IGDB work, release/tag, owner-data permission changes or new product access policy is part of S173.
- The owner authorizes push/PR publication and one optional requested second review round; human merge remains the final boundary.
