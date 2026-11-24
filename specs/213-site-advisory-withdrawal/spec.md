# Feature Specification: Withdrawn Site Advisory Cleanup

**Feature Branch**: `codex/s213-site-advisory-withdrawal`
**Created**: 2026-11-24
**Status**: Draft
**Input**: Owner-approved S213 work slice for [#630](https://github.com/h8rt3rmin8r/fragcap/issues/630), following the [withdrawal of GHSA-fc5q-39xc-vjpf](https://github.com/advisories/GHSA-fc5q-39xc-vjpf) on 2026-11-18.

## Scope

**In**: Remove the site-only exception for the withdrawn advisory from pnpm configuration, the finite policy record, the hosted audit workflow, and repository validation. Guard the site workspace, root package manifest, repository `.npmrc` files, and the hosted audit step's environment against audit bypass settings accepted by pinned pnpm 10.26.0. Correct current maintainer and planning guidance while preserving the historical account of the v0.11.2 release. Prove the unfiltered site audit and site build on the changed revision.

**Out**: Changing the locked `picomatch` version solely because the advisory was withdrawn; modifying unrelated Rust advisory exceptions or #640; altering site features, pinned pnpm or Node versions, Windows product code, published v0.11.2 bytes, or historical release evidence; live or installed-product testing.

**Done when**: The site audit runs with no exception for this GHSA, the full audit and frozen-lockfile site build pass, controlled bypass canaries for every repository-controlled pnpm audit configuration surface are refused, current guidance records the withdrawal, and historical release claims remain accurate. The S213 pull request is ready for owner merge review with applicable CI and review results resolved. No later field run is an issue-completion gate.

## User Scenarios & Testing

### User Story 1 - Trust the site audit result (Priority: P1)

A maintainer sees a plain site dependency audit result rather than a result qualified by an exception for an advisory that GitHub withdrew. An accidental reintroduction of the known bypass configuration is rejected before it can make a green audit misleading.

**Independent Test**: Run the repository supply-chain validation and the full site audit on the exact changed tree, then inject the known site ignore configuration in a controlled fixture and confirm the validator refuses it.

**Acceptance Scenarios**:

1. Given the withdrawn GHSA's exception in the site audit path, when S213 is applied, then the exception record and CVE ignore are absent and the original `pnpm audit --audit-level low` audits the complete locked site graph.
2. Given a future site workspace `auditConfig` subtree that ignores or downgrades an advisory, when repository validation runs, then it fails with a specific site-audit-bypass diagnostic.
3. Given a future top-level site workspace `audit` key, including an ignore list or disabled audit, when repository validation runs, then it fails rather than accepting a potentially filtered result.
4. Given a root flow mapping, explicit key, merge or alias, tag, or quoted or escaped key that the small guard cannot classify reliably, when repository validation or the hosted pre-audit guard runs, then it refuses that noncanonical workspace syntax before auditing.
5. Given `site/package.json` contains `pnpm.auditConfig` with CVE or GHSA ignores, when repository validation or the hosted audit guard runs, then it fails before the audit can report a filtered result.
6. Given a repository `.npmrc` at the checkout root or under `site/`, when repository validation or the hosted audit guard runs, then it refuses that extra configuration source before the audit.
7. Given an audit-affecting pnpm environment setting supplied only to the hosted audit step, when that step runs, then its guard sees the same environment and refuses the bypass before invoking pnpm.
8. Given the audit step's HOME or XDG_CONFIG_HOME points at a repository-controlled global configuration file, when that step runs, then it replaces both paths with a fresh temporary home before invoking pnpm.
9. Given an independent advisory in the locked site graph, when the hosted audit runs, then that finding remains blocking.

### User Story 2 - Read an accurate current record (Priority: P2)

A contributor can distinguish the current withdrawn advisory from the valid historical fact that v0.11.2 shipped while the advisory was active and disclosed an exact, finite exception.

**Independent Test**: Read the current maintainer and planning guidance, issue #630, and historical release record against the dated GitHub advisory; verify that current claims and historical claims are each qualified by their time.

**Acceptance Scenarios**:

1. Given current guidance that still calls the withdrawn GHSA active or unpatched, when S213 lands, then that guidance records the withdrawal and the now-unfiltered audit boundary.
2. Given the v0.11.2 release handoff and release notes, when S213 lands, then their account of what shipped remains intact and current maintainer guidance supplies the dated later status.
3. Given the cleanup has merged and its unfiltered audit passes, when issue #630 is reviewed, then it can close as completed without waiting for a patched `picomatch` release or a future field test.

### Edge Cases

- The advisory database withdraws the GHSA but `pnpm audit` still reports it due to stale registry data. Record that exact failure; do not hide it with a replacement ignore entry or claim a clean audit.
- Another advisory is reported during the unfiltered audit. Keep it blocking and handle it on its own evidence; do not extend this issue into an open-ended waiver.
- A comment or ordinary value mentions `auditConfig` or `audit`. The guard distinguishes that text from an active top-level workspace key.
- A noncanonical root YAML form or bare carriage-return line break could hide an audit key from a line-based guard. Both gates accept LF and CRLF lines, refuse bare CR and unsupported root syntax instead of assuming the full YAML grammar was parsed. An indentless `- ` sequence item is accepted only after a preceding root mapping key, as with `packages`; a standalone root sequence is refused. Nested override values and comments remain accepted.
- Pinned pnpm 10.26.0 also reads `pnpm.auditConfig` from the root package manifest. The guard parses JSON so escaped property names cannot evade the check; benign manifest fields remain permitted.
- pnpm reads `.npmrc` and environment settings that can narrow dependency classes, suppress registry errors, or select fix or ignore behavior. The hosted environment check runs in the same step as `pnpm audit`, so audit-step-only direct settings cannot evade a preceding separate step. A fresh HOME and XDG_CONFIG_HOME prevent indirect redirection to a committed global config file.
- The site still depends on `picomatch@4.0.5`. Withdrawal alone does not require a lockfile churn or prove that every pathological pattern has bounded runtime.
- Historical release notes describe an exception that was correct at publication time. Do not rewrite those notes as though the release never used it.

## Requirements

### Functional Requirements

- **FR-001**: The hosted site job MUST run the original full-graph `pnpm audit --audit-level low` without the withdrawn GHSA's ignore configuration or pre-audit exception validator.
- **FR-002**: The site workspace and repository policy MUST contain no active exception record or ignored CVE for GHSA-fc5q-39xc-vjpf.
- **FR-003**: The offline repository supply-chain gate and the hosted site job MUST both reject top-level site workspace `auditConfig` or `audit` keys before the full audit runs. Both MUST accept LF or CRLF line breaks, refuse bare CR, accept only canonical root block-mapping keys with ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*` (optionally followed by whitespace before the colon), allow root-indent `- ` sequence items only after a preceding root mapping key, and refuse unsupported root YAML forms or a standalone root sequence rather than silently miss a bypass. Nested overrides and comments remain accepted; unrelated Rust advisory handling is unchanged.
- **FR-004**: Controlled negative tests MUST demonstrate that each prohibited workspace configuration and unsupported root YAML form fails the gate and that the hosted full-audit command is not conditional or filtered.
- **FR-005**: Frozen-lockfile site installation, the unfiltered full site audit, static export, and applicable repository gates MUST pass on the exact changed revision before completion is claimed.
- **FR-006**: Current project guidance MUST date the GitHub advisory's withdrawal, state the site audit's present qualification accurately, and link the authoritative advisory record.
- **FR-007**: Historical v0.11.2 release evidence MUST remain historically accurate; any correction of a current claim MUST not imply that the published release used a different audit policy.
- **FR-008**: The slice MUST leave the independent Rust `smoltcp` and `heapless` exceptions under #640 untouched.
- **FR-009**: The offline repository gate and hosted audit guard MUST parse `site/package.json` and reject a root `pnpm.auditConfig` property, including CVE and GHSA ignore lists, while allowing unrelated package metadata.
- **FR-010**: Both gates MUST reject an additional `.npmrc` at the repository root or `site/.npmrc`; the current audit requires neither file.
- **FR-011**: The hosted audit MUST check its own step environment for direct pnpm/npm configuration overrides capable of altering audit coverage or outcome. It MUST then run pinned pnpm with fresh temporary HOME and XDG_CONFIG_HOME paths in that same step, so a repository-controlled step environment cannot redirect pnpm to a committed global `rc` or user `.npmrc`. A separate preceding step alone does not satisfy this requirement.
- **FR-012**: Negative tests MUST cover the manifest CVE and GHSA forms, each `.npmrc` location, and an audit-step-only direct configuration override. Workflow-wiring mutations MUST fail if the fresh temporary directory creation, HOME or XDG_CONFIG_HOME export, or required ordering is removed; final-head hosted CI MUST execute the complete audit step. Positive tests MUST retain the current manifest and no-`.npmrc` baseline. Review results remain required after this change.

### Key Entities

- **Site audit configuration**: The workspace, root package manifest, repository `.npmrc`, and hosted audit-step environment inputs that pinned pnpm can use to narrow advisory reporting. After S213, none may supply an active bypass.
- **Site advisory disposition**: The current repository and issue record of the GHSA's withdrawn status, separate from the historical release record.
- **Historical release record**: The immutable account of the v0.11.2 audit policy at publication time.

## Success Criteria

### Measurable Outcomes

- **SC-001**: The full locked site dependency audit exits successfully with zero advisory ignore entries or exception records for this GHSA in either the workspace or root manifest, and no repository `.npmrc` source.
- **SC-002**: The four workspace bypass fixtures, plus flow-map, explicit-key, merge or alias, tag, escaped-key, bare-CR, and standalone-root-sequence canaries fail the repository gate with a named reason. Manifest CVE and GHSA ignores and both `.npmrc` locations also fail. A harmless comment, nested override, benign manifest sibling, CRLF baseline, and an indentless `packages` sequence remain accepted.
- **SC-003**: Frozen-lockfile site installation and static export pass on the changed revision; the hosted site advisory job checks direct configuration environment variables, isolates HOME and XDG_CONFIG_HOME to a fresh temporary directory, and runs its full-graph audit without a special-case exception step.
- **SC-004**: Current guidance and #630 state the 2026-11-18 withdrawal while the v0.11.2 record still accurately states that its release was exception-qualified.
- **SC-005**: The final source and policy diff contains zero changes to #640's Rust advisory exceptions, product runtime, or the site lockfile absent an independently demonstrated need.

## Assumptions

- GitHub's current advisory record is the authority for the withdrawn status; it states that the report was withdrawn because it is not a security issue.
- The locked site graph still contains `picomatch@4.0.5`, but a new version is not a requirement solely for withdrawing this obsolete exception.
- The existing full-graph site audit remains the acceptance authority. If it fails for another finding, that failure must be resolved on its own merits before S213 claims success.
