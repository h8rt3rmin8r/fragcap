# Tasks: Doctor and Published Release Stabilization

**Input**: S151 spec, plan, research, data model and contract artifacts.

**Tests**: TDD is mandatory. Write and run failing regressions before their corresponding correction.

## Phase 1: Setup

- [x] T001 Verify clean branch, installed spec-kit templates/configuration and ignored local feature pointer in `.specify/feature.json`.
- [x] T002 Complete specify/clarify/checklist/plan artifacts in `specs/151-doctor-release-stabilization/` and track bounded delivery #414 under #372/#331.

## Phase 2: Foundational

- [x] T003 Run blocking read-only analysis over `specs/151-doctor-release-stabilization/{spec,plan,tasks}.md`, resolve any inconsistency before source work.

## Phase 3: User Story 1 - Explain Slow Work (P1, MVP)

**Independent Test**: Injected delayed readiness and tracing show waiting before controlled release and preserve exact values with joined lifetimes; no host probe occurs.

- [x] T004 [US1] Add and run failing slow automatic timing and normal-help discoverability regressions in `crates/fragcap-cli/src/doctor/progress.rs` and `src/cli.rs`.
- [x] T005 [US1] Add fixed readiness labels, one-second threshold/cadence and two-level phase-state tests in `crates/fragcap-cli/src/doctor/progress.rs`.
- [x] T006 [US1] Implement scoped bounded event coordinator with controlled delayed value, nested phase and worker/coordinator failure tests in `crates/fragcap-cli/src/doctor/probe.rs`.
- [x] T007 [US1] Instrument existing readiness/CA-store/loopback calls without changed ordering, errors or effects in `crates/fragcap-cli/src/doctor/probe.rs`.
- [x] T008 [US1] Integrate injected read-only gather/render, suppression and broken-writer tests in `crates/fragcap-cli/src/commands/doctor.rs`, using `src/emit.rs` eligibility and visible `src/cli.rs` timings.
- [x] T009 [US1] Update current source CLI option contract in `site/content/docs/reference/cli.mdx` and unreleased diagnostics guidance in `site/content/docs/guides/doctor-and-troubleshooting.mdx`; run focused CLI/reference tests including existing final report goldens.

## Phase 4: User Story 2 - Published State (P2)

**Independent Test**: Reviewed publication identity and all current markers agree; stale/missing/contradictory current specimens fail while historical/candidate distinctions pass.

- [x] T010 [US2] Add and run a failing actual-publication baseline regression in `xtask/src/spec.rs` before documentation correction.
- [x] T011 [US2] Implement strict reviewed identity, contained surface inventory and pure marker/contradiction checks in `xtask/src/spec.rs`, with negative/malformed/history/candidate tests.
- [x] T012 [US2] Add `docs/published-release.json` and reconcile fourteen current repository/site applicability surfaces (including the Doctor guide added during round-two review), CLI version claims and master release history under `docs/fragcap-specification.md`.
- [x] T013 [US2] Reconcile actual publication/review instructions and unconfigured registry-approval claims in `docs/{maintainers/v0.10.0-release-handoff,security/native-product-review-handoff}.md` without settings mutations.

## Phase 5: Polish and Handoff

- [x] T014 Add chronological S151 records and dated fragment under `docs/plans/README.md`, `docs/fragcap-spec-outline.md` and `changelog.d/S151.*.md`; check glossary and encoding.
- [x] T015 Run whole CI/MSRV, documentation and production site unit/browser gates from `specs/151-doctor-release-stabilization/quickstart.md`; preserve failures and exact outcomes in verification record.
- [ ] T016 Commit scoped files, automatically push and publish official S151 PR; update #414 and project stage, process all reviews with at most one manual second request, verify current-head green CI and leave human-only merge handoff in `specs/151-doctor-release-stabilization/verification.md` and PR.

## Dependencies and Execution Order

T001/T002 precede T003; analysis must pass before any source implementation. US1 runs T004 through T009 in TDD order. US2 runs T010 through T013 in TDD order. Both share no runtime dependency; T014/T015/T016 require both complete. T016's external CI/review state is recorded in PR/tracker after the source verification record, avoiding an endless evidence-only commit/check cycle.

## Parallel Examples

US1 fixed-state review and CLI help review can be researched independently; their edits are sequential here. US2 current-site and repository/handoff applicability audits can be researched independently. The installed planning skill requested read-only research agents; no delegated implementation is required and no concurrent edits/builds are used.

## Implementation Strategy

Deliver controlled Doctor diagnostics first, then reconcile actual published state without implying those diagnostics shipped in v0.10.0. Preserve existing product truth and external acceptance. Run focused tests at each checkpoint, full gates before authorized PR and hosted checks afterward. Never merge, publish another release or mutate registry settings in S151.
