# Feature Specification: S152 registry approval and patch release preparation

**Feature Branch**: `release/0.10.1`

**Created**: 2026-09-15

**Status**: Specified

**Input**: Explicit S152 autopilot kickoff, including automatic push and official pull request, following the approved #416 registry-protection and v0.10.1 preparation recommendation.

## User Scenarios & Testing

### User Story 1 - Deliberate registry publication (Priority: P1)

The operator wants registry publication to normally require separate explicit owner approval after a release tag, while retaining administrator bypass. Actual protection must be checked rather than assumed from an environment name.

**Why this priority**: The existing named publication environment has no required reviewers or release-ref restriction.

**Independent Test**: Supplied protection records distinguish the approved policy from missing, weakened, incomplete and unverifiable protection without publishing anything.

**Acceptance Scenarios**:

1. **Given** the approved operator reviewer and tag-only release policy, **When** protection is checked, **Then** the configuration is recognized but no deployment is approved by the checker.
2. **Given** missing reviewer protection, unknown bypass state, an unapprovable sole-reviewer policy or a nonrelease ref allowance, **When** protection is checked, **Then** release publication is refused with actionable findings.
3. **Given** protected publication waiting for approval, **When** an agent finishes implementation, **Then** the approval remains operator-owned and is never automatically supplied.

### User Story 2 - Prepared Doctor patch (Priority: P2)

The operator wants a fully checked v0.10.1 candidate containing S151 diagnostics so a subsequent separately authorized release can deliver the improvement for field measurement.

**Why this priority**: Published v0.10.0 does not contain S151 diagnostics; testing requires newly published product bytes.

**Independent Test**: Candidate versions, embedded output, release highlights, complete changelog and certified package identities reconcile while the published baseline remains v0.10.0.

**Acceptance Scenarios**:

1. **Given** merged S151 source, **When** v0.10.1 is prepared, **Then** all ten product crates and candidate output identify 0.10.1 without claiming it is published.
2. **Given** official v0.10.0 bytes and release history, **When** the candidate is prepared, **Then** the tag, assets, publication record and historical evidence remain unchanged.
3. **Given** green candidate checks and satisfied bot reviews, **When** work is handed off, **Then** the operator receives an official PR for final review and merge, not a release tag or completed field-validation claim.

### Edge Cases

- Protection metadata is unavailable, malformed, incomplete, oversized or contains unknown protection kinds.
- A tag policy is omitted from a partial page, duplicated, widened, or confused with a same-named branch policy.
- The sole operator account also initiates the workflow; normal approval must remain possible through a separate manual action, with deliberate administrator bypass available.
- An administrator can alter settings later; fresh verification must occur before release creation and again immediately before registry execution.
- Other environment secrets, variables, timers or custom settings exist or change concurrently; configuration must preserve unrelated state and refuse unexplained drift.
- Prepared changelog dates describe preparation, not proof of publication; independent acceptance and operator field measurements remain pending.

## Requirements

### Functional Requirements

- **FR-001**: The existing `crates-io` environment MUST require operator `h8rt3rmin8r` review on its normal publication path, with administrator bypass explicitly retained at the operator's direction; the same account MUST remain able to approve an owner-initiated workflow through a separate deliberate action. An operator-selected bypass MUST be reported as bypass, not required-reviewer approval, and no agent may approve or bypass deployment during S152.
- **FR-002**: The publication environment MUST allow only tag refs matching the intended `v*` release family, not branch refs or a broader ref policy; the release workflow MUST additionally require the exact tag to equal the prepared product version.
- **FR-003**: Configuration MUST use the scoped S152 authorization, preserve environment identity, secrets and unrelated settings, and retain a scrubbed before/after verification record. No deployment approval is authorized.
- **FR-004**: Fresh authoritative protection verification MUST precede release creation and registry execution. Missing, incomplete, contradictory, unsupported, unverifiable or weakened records MUST fail closed rather than count as operator approval.
- **FR-005**: Automation, reproducible verification instructions and negative regression checks MUST agree with the approved policy and separately identify configuration verification versus actual deployment approval.
- **FR-006**: The v0.10.1 candidate MUST reconcile all ten product crate versions, embedded product strings, output specimens, candidate specification applicability, short release highlights, complete chronological release records and package certification.
- **FR-007**: Actual publication identity MUST remain v0.10.0 until separately authorized publication is verified; existing official tags, assets, history and unsigned policy MUST remain unchanged. S151 diagnostics MUST remain described as unreleased during preparation.
- **FR-008**: Implementation verification MUST use controlled tests and isolated hosted certification only, without installed sensitive product execution, production Doctor, real games or real trust mutation on the operator machine.
- **FR-009**: S152 MUST automatically push its branch and open an official PR; every review comment MUST receive a disposition, actionable changes MUST be verified and review threads resolved, with no more than one manually triggered second review round. Current-head required CI MUST be green before human review handoff.
- **FR-010**: Human merge and later tag/publication MUST remain separate operator actions. #372 measurements, independent #333/#413 review and retest, final #331/#334/#278 acceptance and deferred #155/#94 MUST NOT be represented as completed by S152.

### Key Entities

- **Approval policy**: Exact environment identity, operator reviewer identity, self-review choice, administrator bypass choice and tag-only allowance.
- **Verification record**: Scrubbed authoritative settings and complete allowance inventory, observed at a stated time; configuration evidence is not deployment approval.
- **Release candidate**: Prepared 0.10.1 source, output and documentation, distinct from immutable published 0.10.0.
- **Review handoff**: Official PR, current-head checks, complete comment dispositions and human-owned final merge.

## Success Criteria

### Measurable Outcomes

- **SC-001**: All approved protection attributes are confirmed from authoritative records, and every negative protection class is refused without approving or publishing a deployment.
- **SC-002**: All ten product crates, candidate outputs and package certification agree on 0.10.1; zero published-baseline records or official v0.10.0 assets are changed.
- **SC-003**: Every required current-head check is green and every received review comment is addressed before the official PR is handed to the operator; manual review requests do not exceed one.
- **SC-004**: Zero release tags, registry publications, agent deployment approvals, installed sensitive product runs or real-game runs occur in S152.

## Assumptions

- The existing public repository supports environment required reviewers and tag policies. Authoritative readback, not assumed feature availability, determines whether configuration succeeded.
- The owner account is both workflow initiator and sole operator reviewer; self-review prevention would make normal approval impossible. Separate manual approval remains the normal path, with deliberate owner bypass retained.
- The existing release branch policy requires `release/*`, so `release/0.10.1` intentionally overrides the default `codex/` prefix for version-only preparation.
- S152 authorizes the bounded environment policy change and local patch preparation, not release tag/publication or deployment approval.
- Existing native behavior, dependencies and output schemas are unchanged apart from product version identity.

## Clarifications

### Session 2026-09-15

- Q: Is self-review prevention compatible with the sole owner reviewer and owner-initiated workflow? A: No. Require the owner as sole reviewer, allow owner self-review, retain administrator bypass and retain a separate manual approval action. Rejected alternatives are an unapprovable owner-only self-review ban or inventing another reviewer.
- Q: Does release preparation update actual publication or close field and security acceptance? A: No. Preserve published v0.10.0 identity and all independent gates until their actual evidence exists.
- Q: Do the default pre-push halt and branch prefix apply? A: Explicit user push/PR authorization overrides the pre-push halt; pinned `release/*` policy governs this version-preparation branch. No release tag or deployment approval is authorized.
- Q: Should administrator bypass be disabled? A: The operator explicitly rejected disabling it during S152. Restore and verify `can_admins_bypass=true`, preserve owner review and tag-only policy, and revise all policy checks and current instructions accordingly. This supersedes the initial autopilot policy choice without changing unrelated administration or claiming independent approval.
