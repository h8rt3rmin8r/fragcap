# Feature Specification: Evidence and Diagnostic Debt Reconciliation

**Feature Branch**: `codex/s212-evidence-diagnostic-reconciliation`
**Created**: 2026-11-24
**Status**: In progress
**Input**: Owner-approved S212 work slice to reconcile public progress tracking with observed behavior and finalize a disposable diagnostic worktree through a narrow, general-purpose pull request.

## Scope

**In**: Audit the active Deep Capture epic and related open issues against current evidence; correct stale claims, acceptance criteria, issue metadata, and dependencies; disposition every tracked change in the disposable diagnostic worktree; extract a bounded HTTP/1.1 partial-head byte-count diagnostic through native evidence, terminal diagnostics JSON, and the derived bundle debug report; repair the separately discovered pre-existing dependency-audit blocker with governed tool and exception records, isolated spike locks, and a finite, parent-scoped exception; align affected specification, documentation, and changelog records.

**Out**: Client-specific investigation details, private decoder or plugin work, new interception techniques, TLS handshake policy changes, route-event packet timelines, generic relay lifetime repair, experimental switches or product dependencies, an open-ended security exception, live game launches, and release publication.

**Done when**: The affected public tracker states only supported product behavior with target-independent completion tests, every dirty source file has a recorded disposition, the public branch contains only reviewed general changes and separately documented CI repair files, controlled partial-head tests and exception-qualified audit checks pass, and the resulting pull request is ready for owner merge review.

**Tracker inventory**: S212 reconciles #607, #594, #635, #636, #622, #596, #632, #633, and #634; it explicitly reviews and retains #613, #615, #617, #620, #638, #610, #621, #637, and #580 without reopening them. Earlier completed epic children remain represented through #607's index. These 18 issue IDs form the closed audit set for SC-001.

**Independent CI blocker**: #640 tracks the pre-existing all-features `heapless 0.8.0` audit failure and its required dependency replacement. It is outside the 18-issue tracker-reconciliation inventory and remains open after this finite S212 exception.

## User Scenarios & Testing

### User Story 1 - Trust the public work queue (Priority: P1)

A maintainer can select an issue without inheriting a diagnostic premise that later evidence superseded. Completed comparisons, independent product defects, and optional research are distinguishable.

**Independent Test**: Read the active epic and each affected issue after reconciliation and compare its stated remaining work, status, and metadata against a sanitized disposition ledger.

**Acceptance Scenarios**:

1. Given a completed comparison still listed as future work, when the issue is reconciled, then the comparison is marked complete and is not required again for issue closure.
2. Given an acceptance criterion that asks the product to explain a client decision it cannot observe, when the issue is reconciled, then the criterion is retired or replaced with a controllable product outcome.
3. Given a confirmed defect independent of one client, when the tracker is reconciled, then it remains separately actionable with its own observed behavior and controlled completion test.

### User Story 2 - See truthful partial-head evidence (Priority: P1)

An operator investigating an inspected HTTP/1.1 connection that ends before a complete message head can see how many head bytes the proxy actually read before the terminal event. The report does not invent a request, infer a peer's reason, or expose the partial bytes.

**Independent Test**: Controlled origins and clients exercise a partial-head EOF, read error with zero or more bytes, head-size refusal, clean zero-byte EOF, and complete head. Inspect terminal diagnostics JSON, then derive a bundle debug report from the same native `proxy.jsonl` and compare the counted failure for the same connection ID.

**Acceptance Scenarios**:

1. Given a connection that sends a partial HTTP head then closes, when the read ends, then the diagnostic reports the exact observed byte count and the EOF outcome without retaining the partial head.
2. Given a read error after a partial head, when the error is reported, then the diagnostic keeps the observed count and actual error category separate from EOF and from a completed request.
3. Given a head-size refusal, when the bound is reached, then the diagnostic reports the bounded count and refusal rather than implying the client closed.
4. Given a complete head with no read failure, when the diagnostic is written, then the failure-only byte count is absent rather than present with an empty value.
5. Given clean EOF before any head byte, when the diagnostic is written, then the failure-only byte count is absent; a read error before any byte instead reports an observed count of zero.

### User Story 3 - Finish the disposable code inventory (Priority: P2)

A maintainer can distinguish reusable product corrections from experimental source, and the pull request contains no accidental change to the ordinary inspection path.

**Independent Test**: Reconcile the original dirty worktree file-by-file and inspect the final pull-request diff against the disposition ledger and dependency inventory.

**Acceptance Scenarios**:

1. Given the 20 modified tracked files in the diagnostic worktree, when the inventory is complete, then each file and its substantive hunks are classified as extracted, retained privately, or retired with a reason.
2. Given an experimental TLS or routing change, when the public diff is reviewed, then it and any experiment-only dependency are absent.
3. Given verified extraction into a clean branch, when the original worktree is reconciled, then no untracked or uncommitted diagnostic change is silently lost or presented as shipped behavior.

### User Story 4 - Keep the security audit operational (Priority: P2)

A maintainer can distinguish a bounded, reviewed exception for a pre-existing vulnerable dependency path from a clean dependency graph. The audit still fails for a new direct affected parent or after the exception expires, and the permanent replacement remains tracked separately.

**Independent Test**: Run the pinned all-features audit and six isolated graph audits, then run a direct affected-parent canary, an expired-exception canary, and the offline governed-policy validation. Inspect the exact workflow, governed policy, validator, denial policy, two spike lockfile diffs, and issue #640.

**Acceptance Scenarios**:

1. Given only the reviewed indirect `smoltcp 0.12.0` parent, when the current all-features and isolated audits run, then the nine exact advisories may be exception-qualified until 2026-12-15 UTC while `unsound = "all"` remains blocking.
2. Given an affected dependency reached through a new direct parent, when the audit runs, then the exception does not apply and the audit fails.
3. Given an expired exception, when the audit runs, then the audit fails instead of silently extending the waiver.
4. Given the two isolated spike lockfiles, when they are audited, then `url 2.5.9` and its resolved `form_urlencoded 1.2.3` are locked without changing the product workspace lockfile.
5. Given the online denial exception and the offline governed policy, when their advisory, parent, or expiry fields diverge, then the offline gate fails; a reviewed critical-dependency record may be renewed without claiming its source has changed.

### Edge Cases

- An issue contains both completed research and an independent implementation defect; preserve its history and transfer only the still-actionable requirement.
- An older Spec Kit artifact states an unverified gate; retain that limit rather than marking the historical test passed.
- The HTTP head contains zero bytes, a complete head, or bytes up to the configured limit; the reported count must match observation without retaining content, and clean zero-byte EOF remains an ordinary no-head outcome.
- A successful head read produces no failure-only count in native or human projections, including no placeholder null value.
- A read error occurs after some bytes; a later parser error must not be represented as the event that first ended the read.
- A diagnostic field is unavailable at a downstream projection; state that limitation rather than fabricate a count.
- The original worktree and clean branch diverge; extraction uses a reviewed hunk, not a whole-file copy.
- A dependency audit fails on a pre-existing vulnerable indirect path; a finite exception must preserve the failure for other parents and expire on a stated date while a replacement stays open.

## Requirements

### Functional Requirements

- **FR-001**: The affected public epic and related issues MUST have a verified disposition for stale evidence, remaining work, acceptance criteria, status, labels, milestone, and dependencies.
- **FR-002**: Tracker edits MUST preserve historical observations and name completed comparisons while removing completion gates that depend only on future field testing or an unobservable client decision.
- **FR-003**: Public issue, Spec Kit, source, commit, and pull-request text MUST omit private target identity, endpoints, traffic, decoder methods, local research paths, and experiment-specific details.
- **FR-004**: Every modified tracked file and substantive hunk in the original diagnostic worktree MUST receive an auditable disposition before branch cleanup; the original worktree MUST remain recoverable until extracted changes are verified.
- **FR-005**: A bounded inspected HTTP/1.1 head read MUST retain the number of bytes actually observed when a partial head ends by EOF, when a read error occurs, or when a head-size refusal occurs, and MUST keep those outcomes distinct where the underlying I/O distinguishes them. Clean EOF before any head byte is not a failure and has no failure-only count; an actual read error before any head byte reports zero observed bytes.
- **FR-006**: The partial-head count and terminal kind MUST reach the existing native connection diagnostic and terminal diagnostics JSON. A bundle debug report derived from that native artifact MUST include only valid counted head-read failures with connection ID, allowlisted code and boundary, and exact count. None may retain partial head bytes or change forwarding decisions. The failure-only field and report row MUST be absent, not null, when no head-read failure occurred.
- **FR-007**: Controlled tests MUST cover clean zero-byte EOF, zero-byte read error, partial-head EOF, partial-head read error, size refusal, and complete-head success, including a same-connection native-to-bundle-report readback and the absence of partial content.
- **FR-008**: The public change MUST preserve ordinary Capture, existing inspection scope, bounded retention, cancellation, cleanup, and existing artifact compatibility; experimental TLS or routing controls and their added dependencies MUST be excluded.
- **FR-009**: Affected master-specification, slice-planning, public documentation, and changelog statements MUST distinguish shipped behavior, controlled evidence, observed limits, and deferred work.
- **FR-010**: The pre-existing nine-advisory `heapless 0.8.0` finding MUST be handled through exact advisory-ID exceptions restricted to the reviewed `smoltcp 0.12.0` parent, expiring 2026-12-15 UTC, without disabling general unsoundness blocking or claiming the dependency fixed. The separate replacement issue #640 MUST remain open.
- **FR-011**: The audit workflow MUST run locked `cargo-deny 0.21.0` against the all-features product graph and six isolated Rust graphs, and the offline governed policy MUST pin the same tool version and mirror each exact advisory, parent, and expiry in the denial policy. The two spike lockfiles MUST resolve patched `url 2.5.9` and `form_urlencoded 1.2.3` without changing the product lockfile. Positive online and offline audits and negative direct-parent, expiry, and mirror-drift controls MUST be recorded. A dated decisions fragment MUST document the pinned workflow, bounded source-review rationale, and any routine unchanged-source review renewal.

### Key Entities

- **Issue disposition**: A supported statement of completed evidence, remaining product behavior, acceptance, tracker metadata, and whether work remains open.
- **Partial-head diagnostic**: Observed byte count, terminal category, and connection association for one bounded HTTP/1.1 head read, without partial content.
- **Source disposition**: Per-file and substantive-hunk decision to extract a general fix, retain a private experiment, or retire obsolete work.
- **Finite audit exception**: Advisory ID, exact allowed parent, expiry date, bounded source-review rationale, and controlled positive and negative outcomes for one pre-existing dependency path.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Every affected issue in the closed S212 tracker inventory has a documented disposition, and none retains a completed comparison or unspecified later field run as its only completion condition.
- **SC-002**: All 20 modified tracked files from the original diagnostic worktree have a reviewed disposition, and the public pull request contains zero experimental controls, private identifiers, or experiment-only dependency packages.
- **SC-003**: Every required controlled head-read case reports its exact observed count when a failure-only count applies and its distinguishable terminal outcome, with no failure-only count on clean zero-byte EOF or success and zero retained partial-head content.
- **SC-004**: Terminal diagnostics JSON projects the applicable count, and a bundle debug report derived from native `proxy.jsonl` agrees with that artifact on each valid counted failure's connection ID, code, boundary, and count. A successful head has no failure-only count or null placeholder, and no completed request is fabricated for an incomplete head.
- **SC-005**: The full applicable repository checks and pull-request CI complete successfully at the final branch head.
- **SC-006**: The locked all-features audit, six isolated audits, and offline governed-policy gate pass under the nine narrow exceptions, while direct-parent, expired-waiver, and policy-mirror drift canaries fail; the governed CI repair files and #640 identify the temporary boundary and replacement work.

## Assumptions

- The original diagnostic worktree is preserved as source evidence until the clean-branch extraction and verification complete.
- Existing native connection and terminal diagnostics can carry one bounded derived count; the bundle report can add a sanitized derived failure summary without a new capture mode, routing strategy, or proxy dependency.
- Controlled fixtures can establish this general diagnostic behavior without a live target or elevated driver run.
- Public issue history remains visible; S212 adds corrections and status updates without claiming older comments did not occur.
