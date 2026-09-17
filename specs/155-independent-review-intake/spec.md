# Feature Specification: S155 independent review execution and findings intake

**Feature Branch**: `codex/s155-independent-review-intake`\
**Created**: 2026-09-16 UTC\
**Status**: Draft\
**Input**: Approved S155, execute an immutable published-build review campaign on disposable hosted Windows infrastructure and enforce reviewer-owned findings intake under #333, without self-certifying independent acceptance or running installed sensitive software on the owner's host.

## User Scenarios & Testing

### User Story 1 - Validate an independent review record (Priority: P1)

An independent reviewer needs a bounded record contract that rejects incomplete identity, missing scope, unsupported approval claims and unresolved security findings before maintainers use the review as acceptance evidence.

**Why this priority**: #333 cannot close from a bot reaction, green CI or an unstructured narrative. Its acceptance criteria require reproducible identity, actual methods, finding dispositions and independent retests.

**Independent Test**: Exercise positive and negative synthetic review records through a repository-owned validator without committing a fabricated completed review or running the product.

**Acceptance Scenarios**:

1. **Given** a completed reviewer-owned record, **When** validation runs, **Then** the exact candidate, independent reviewer declaration, all twelve review areas, installed Windows methods, findings, retests and public-safe summary reconcile.
2. **Given** an unresolved critical or high finding, a medium finding without an owner and disposition, or an indeterminate required check, **When** validation runs, **Then** acceptance is refused with a specific diagnostic.
3. **Given** the existing not-started template, **When** readiness CI runs, **Then** it remains not-started and cannot be mistaken for completed review evidence.

### User Story 2 - Re-execute published installed-build evidence (Priority: P1)

A reviewer needs a clean Windows campaign that downloads the immutable published v0.10.1 bytes, verifies their recorded identity, installs them, exercises the controlled native product path and reconciles cleanup without touching the owner's machine.

**Why this priority**: Publication and source-built CI are prerequisites, but they do not establish a fresh installed-build observation against the exact public downloads.

**Independent Test**: Run the published package campaign on a disposable hosted Windows runner, using exact release digests, synthetic data roots, a controlled target and loopback-contained traffic.

**Acceptance Scenarios**:

1. **Given** the v0.10.1 release, **When** the campaign starts, **Then** all primary assets and checksum sidecars match the immutable recorded release identity before execution.
2. **Given** a clean installation, **When** the installed executable runs the controlled Deep Capture smoke, **Then** native backend identity, process ownership, loopback containment, complete observation and cleanup are recorded separately from portable-package smoke.
3. **Given** download drift, identity mismatch, unexpected process or network activity, incomplete cleanup or unavailable required evidence, **When** validation runs, **Then** the campaign fails rather than reporting a skip as success.

### User Story 3 - Preserve the independent acceptance boundary (Priority: P2)

A maintainer needs hosted execution and review intake to improve #333 evidence without claiming that implementation-authored automation is the independent whole-product verdict.

**Why this priority**: Honest state is a product requirement under P-9 and avoids closing #333, #413, #331, #334 or #278 on evidence those issues explicitly reject.

**Independent Test**: Check documentation, task records, issue references and command output for exact readiness versus acceptance language.

**Acceptance Scenarios**:

1. **Given** a green published-build campaign, **When** status is reported, **Then** it is identified as candidate evidence awaiting reviewer attribution and disposition.
2. **Given** no reviewer-owned completed record, **When** S155 closes, **Then** #333 and #413 remain open and the final documentation and product gates remain incomplete.
3. **Given** third-party PR review, **When** comments arrive, **Then** each comment is reconciled within the authorized two-round maximum without treating approval as the installed-product audit.

### Edge Cases

- A declared reviewer is an implementation author or does not provide an independence statement: refuse acceptance.
- A required review area is duplicated, absent, skipped or indeterminate: refuse acceptance.
- A finding is marked remediated without a separate retest identity and outcome: refuse acceptance.
- A medium finding names no owner or final disposition: refuse acceptance.
- The published asset is reachable but differs in size or digest: stop before install or execution.
- The hosted runner cannot establish loopback containment or cleanup: fail the campaign and retain only bounded sanitized evidence.
- A completed record contains private paths, credentials, host identifiers or raw sensitive payloads: reject the public-safe record.
- A green campaign has no independently authored analysis: keep the review state incomplete.

## Requirements

### Functional Requirements

- **FR-001**: The repository MUST define one bounded completed-review record contract for exact v0.10.1 candidate identity, reviewer independence, environment, methods, twelve-area coverage, installed-build checks, findings, retests and public-safe summary.
- **FR-002**: Validation MUST reject unknown keys, duplicates, missing required fields, invalid enumerations, unsafe public text, oversized collections or strings, and candidate identity that differs from the immutable published release.
- **FR-003**: Acceptance validation MUST require all twelve S150 review areas exactly once and explicit passed results for P-1 boundary and no-open-proxy review. Skipped, failed or indeterminate required areas MUST remain incomplete.
- **FR-004**: Critical and high findings MUST have a remediation identity and separate passed independent retest before acceptance. Medium findings MUST have an owner and explicit disposition. No record may self-accept residual risk implicitly.
- **FR-005**: The original not-started template and readiness validator MUST remain unchanged in meaning. A synthetic positive fixture MAY test the completed-record validator but MUST NOT be presented as an actual review.
- **FR-006**: Hosted Windows automation MUST download exact published v0.10.1 ZIP, MSI, catalog and checksum sidecars, verify recorded sizes and SHA-256 digests before effects, and use no locally rebuilt product bytes.
- **FR-007**: The hosted campaign MUST install the exact MSI in a disposable runner, exercise the installed executable through the existing controlled native Deep Capture path under loopback containment, and record installed smoke separately from portable smoke.
- **FR-008**: Every installed campaign child process MUST be hidden, non-interactive and time-bounded. Cleanup MUST attempt every acquired firewall, installer, data and owned security effect, and any incomplete cleanup MUST fail the campaign.
- **FR-009**: Public artifacts MUST be sanitized and bounded. Raw credentials, CA private material, payloads, private paths, host identifiers and unbounded logs MUST NOT be uploaded.
- **FR-010**: S155 MUST NOT run installed product, real games, live sensitive capture or real host trust mutation on the owner workstation. Optional #372 field measurement remains owner-only.
- **FR-011**: S155 MUST NOT close #333, #413, #331, #334 or #278 without an actual reviewer-owned record satisfying their independent acceptance criteria. Green automation and PR bot approval remain inputs, not the verdict.
- **FR-012**: Pinned workflow and script changes MUST receive a dated decision record. All local and hosted required checks and every received review MUST pass before owner handoff, with at most one requested second review round.

### Key Entities

- **Review record**: Exact published candidate, independent reviewer declaration, environment, area outcomes, installed checks, findings, retests and public-safe summary.
- **Area result**: One of the twelve closed S150 areas, its method, evidence identity and terminal outcome.
- **Finding**: Stable identifier, area, severity, impact, owner, disposition, remediation and independent retest facts.
- **Published campaign evidence**: Immutable release asset identities plus portable and installed controlled-smoke outcomes from one disposable hosted runner.
- **Acceptance state**: Incomplete until every required result and finding disposition satisfies the record contract.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Positive and negative validator tests cover every required top-level record section, all twelve areas, every finding severity and every acceptance-blocking terminal state.
- **SC-002**: A completed synthetic record validates, while at least twelve independently mutated invalid records each fail with a specific diagnostic.
- **SC-003**: The hosted campaign verifies all six public release files, runs both portable and installed controlled native smoke, records zero unexpected process and non-loopback observations, and reconciles every acquired effect.
- **SC-004**: The hosted report contains no unbounded or sensitive field and remains below the existing package-report byte ceiling.
- **SC-005**: All final-head required checks are green and every actual review thread is answered and resolved, with no more than one requested second review round.
- **SC-006**: Without an externally authored completed record, issue and documentation state explicitly preserve #333/#413 and downstream gates as open.

## Assumptions and Scope

The immutable review candidate is published v0.10.1 at `a7d24962999d38d7ff130722859d473543864862`. Existing package certification, controlled target, firewall observation and exact cleanup authorities are reused rather than replaced.

The hosted campaign is reproducible candidate evidence and supplies the installed-execution half of the handoff. Independent source analysis, reviewer identity, finding judgment and retest acceptance remain external facts supplied through the new contract. No requirement permits the implementation agent to assert those facts.

Specification, clarification, requirements checklist, plan, tasks and blocking analysis precede implementation. Explicit push and PR authorization satisfies the autopilot pre-push pause, but human merge remains the final boundary.
