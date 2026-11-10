# Tasks: Signed Direct Route Foundation

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data model](data-model.md), and [package component contract](contracts/package-component.md).

**Tests**: Required by issues #577 and #579 and the autopilot protocol. Negative identity and Doctor cases precede corresponding implementation.

## Phase 1: Spec-Kit gate

- [x] T001 Confirm clean S199 branch, open issue contracts, official upstream identity and absent extension hooks.
- [x] T002 Specify, clarify and complete the requirements checklist for #577 and #579.
- [x] T003 Plan the exact source, package, licensing, Doctor and CI contracts.
- [x] T004 Analyze constitution, spec, plan and tasks; resolve blockers before implementation.

## Phase 2: Governing decision

- [x] T005 Amend P-1 and licensing policy with a versioned sync report, then reconcile master specification, AGENTS, AI context, contributor guidance, template, glossary, public claims and audit policy.
- [x] T006 Record a dated decision and commit the governing gate before component implementation.

## Phase 3: Component contract and distribution

- [x] T007 Write failing static and Windows negative checks for missing, modified, wrong-architecture, unsigned and mismatched component entries.
- [x] T008 Pin official 2.2.2-A binary and source archive hashes and implement verified staging in the package workflow.
- [x] T009 Add four exact shared entries to ZIP and MSI, component notice/SBOM evidence, and bounded contract validation.
- [x] T010 Extend the Windows package certification report and installer lifecycle checks for signed-driver identity and exact installed/portable equality.

## Phase 4: Readiness and documentation

- [x] T011 Add failing Doctor readiness cases and implement read-only component inspection with honest static-versus-load distinctions.
- [x] T012 Verify ordinary Capture and offline operations do not load or require WinDivert; preserve Npcap and installation-CA paths.
- [x] T013 Reconcile operator and maintainer package guidance, specification, chronological plan and changelog.
- [x] T014 Re-run requirements checklist and Spec-Kit analysis; resolve every finding.

## Phase 5: Verification and review

- [ ] T015 Run focused tests, `cargo xtask ci`, package contract check, Windows package certification, encoding/mojibake and `git diff --check`.
- [x] T016 Commit, push the authorized branch and open an official PR with `Closes #577` and `Closes #579`.
- [ ] T017 Wait for exact-head CI and all reviews, answer every actionable comment, request at most one second `@Codex` round, and leave a green PR for owner final review and merge.

## Phase 6: Hosted gate repair (discovered after PR publication)

- [x] T018 Repair RUSTSEC-2026-0459 with patched bundled SQLite, reconcile MSRV, supply-chain policy, specification and changelog, then rerun local gates.
- [ ] T019 Make pinned package-tool installation check native exit status and retry transient CycloneDX download failure; rerun hosted Windows package certification.
- [ ] T020 Refresh the two isolated lockfiles carrying SQLite and rerun their hosted audit and short campaign.

## Dependencies

T004 gates all implementation. T005-006 precede T007-013. Negative checks precede their corresponding code. The packaging workflow uses the same exact archive identity as the Windows package validator. Documentation follows observed package behavior. Hosted review follows the pushed candidate.
