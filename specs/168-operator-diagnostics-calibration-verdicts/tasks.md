# Tasks: S168 operator diagnostics and calibration verdicts

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md), [data-model.md](data-model.md), and [operator output contract](contracts/operator-output.md).

## Phase 1: Setup

- [x] T001 Confirm exact S168 issue/directory/branch scope and complete Spec-Kit specify, clarify and checklist in specs/168-operator-diagnostics-calibration-verdicts/ (FR-014).
- [x] T002 Complete research and design artifacts using the installed plan workflow in specs/168-operator-diagnostics-calibration-verdicts/ (FR-001 through FR-014).
- [x] T003 Run blocking read-only analysis across specs/168-operator-diagnostics-calibration-verdicts/spec.md, plan.md and tasks.md; record the passed result after resolving findings (FR-014).

## Phase 2: Foundational contracts

- [x] T004 [P] Establish failing actual-cell, ANSI, Unicode, optional/indexed three-column and exact-whitespace continuation regressions in crates/fragcap-cli/src/display.rs (FR-001).
- [x] T005 [P] Establish failing exact selected Steam/provenance and no-effect early Steam regressions in crates/fragcap-steam/src/library.rs and crates/fragcap-cli/src/commands/calibrate.rs (FR-004 through FR-006).
- [x] T006 [P] Establish failing genuine completed HTTP exchange, independent terminal cause and late-phase authorization regressions in crates/fragcap-proxy/src/http1.rs and crates/fragcap/src/deep_capture/ (FR-007, FR-012).

## Phase 3: US1 consistent terminal reports

**Independent test**: Actual display positions preserve four-space adjacent anchors, exact values and color/plain equivalence across the complete renderer inventory.

- [x] T007 [US1] Implement ColumnLayout and render_fields plus exact-whitespace/ANSI handling in crates/fragcap-cli/src/display.rs (FR-001).
- [x] T008 [US1] Audit and migrate all non-target/session human multicolumn families in crates/fragcap-cli/src/commands/steam.rs, technologies.rs, bundle.rs, fresh_start.rs, output.rs and live_status/mod.rs; record complete renderer inventory in specs/168-operator-diagnostics-calibration-verdicts/renderer-inventory.md (FR-001).
- [x] T009 [US1] Replace discovery table with source chapters and report-wide field anchors in crates/fragcap-cli/src/commands/targets.rs; cover all candidates/repeated findings/empty/counts/Unicode/color/long values (FR-002).
- [x] T010 [US1] Migrate target details/hero/ambiguity/reconciliation, calibration prior facts, Deep Capture plan fields and ambiguity guidance in crates/fragcap-cli/src/commands/targets.rs, target_resolve.rs, calibrate.rs, deep_capture.rs and workflow_help.rs; retain selector authority (FR-001, FR-011).

## Phase 4: US2 actionable Doctor history

**Independent test**: Constant healthy summary rows coexist with individually actionable/mixed findings and truthful limits; exact human/JSON detail remains available.

- [x] T011 [P] [US2] Add large healthy/mixed/active/actionable/scan-limit and mode regressions in crates/fragcap-cli/src/doctor/ and tests/cli_doctor.rs (FR-003).
- [x] T012 [US2] Add --history-details and pure healthy-history human projection without altering checks/actions/JSON in crates/fragcap-cli/src/cli.rs and doctor/mod.rs; preserve observed findings alongside inventory limitations in doctor/checks.rs (FR-003, FR-011).

## Phase 5: US3 selected prompt preflight

**Independent test**: Exact selected discovery avoids unrelated roots; warm-Steam refusal satisfies the controlled one-second/no-effects budget and non-Steam/restart/resume/race/error paths retain authority.

- [x] T013 [P] [US3] Add selected app discovery and exact install-root lookup in crates/fragcap-steam/src/library.rs/lib.rs preserving duplicate/malformed/access/appinfo diagnostics (FR-004).
- [x] T014 [US3] Add typed source/root/target/operation diagnostic provenance and scoped source APIs in crates/fragcap-targets/src/source.rs, sources/, and crates/fragcap/src/discovery.rs with relevant human/structured projections (FR-004, FR-011).
- [x] T015 [US3] Scope client refresh/revalidation and targeted command resolution in crates/fragcap-cli/src/commands/calibrate.rs, targets.rs and target_resolve.rs; retain broad warnings and no-match coverage (FR-004).
- [x] T016 [US3] Observe running Steam first and apply selected-case refusal/restart before expensive processing/new effects in crates/fragcap-cli/src/commands/calibrate.rs and calibrate/steam_client_observation.rs, retaining fresh effect-boundary recheck (FR-005, FR-006).
- [x] T017 [US3] Complete controlled missing-client/stored/discovered/resume/noninteractive/JSON/cold/race/inventory-error/restart regressions in crates/fragcap-cli/tests/cli_calibrate.rs and owning unit modules (FR-004 through FR-006).

## Phase 6: US4 evidence, verdict and narrative

**Independent test**: Six failed connection endings plus seven completed exchanges, missing target ownership, partial artifacts and successful cleanup remain independently represented; attempted/current stored verdicts and phase eligibility are exact.

- [x] T018 [P] [US4] Implement bounded typed proxy terminal causes/exchanges/ownership/limitations and EvidenceWindow through crates/fragcap-proxy/src/ and crates/fragcap/src/deep_capture/ API/adapters/native/session/policy; exclude late/unavailable records from authorization while retaining artifacts (FR-007, FR-012).
- [x] T019 [US4] Add attempted-case assessment and current stored-case readiness projection using existing proposal/applicability, including conflicts and missing evidence, in crates/fragcap-cli/src/commands/calibrate/assessment.rs and integration in calibrate.rs/deep_capture.rs (FR-008, FR-009, FR-011).
- [x] T020 [US4] Establish narrative/completeness/root-path/fact/authorization and every-terminal-path regressions in crates/fragcap-cli/src/session_ux.rs and tests/cli_calibrate.rs (FR-007 through FR-012).
- [x] T021 [US4] Implement readable chapters, independent counter fields, exact semantic completeness/access, one fact summary, one bundle root, no omitted file claims, and visible owner-release action/deadline in crates/fragcap-cli/src/session_ux.rs and commands/deep_capture.rs (FR-010, FR-012).
- [x] T022 [US4] Render one final attempted/current-case verdict and one supported next action with exact prerequisites; retain canonical plan review, quiet/silent modes and additive structured decisions in crates/fragcap-cli/src/commands/calibrate.rs, assessment.rs and events.rs (FR-008 through FR-012).
- [x] T023 [US4] Complete controlled first/second/resumed/current/stale/conflicting/interrupted/failed/late/success and six-failure/seven-exchange fixtures in crates/fragcap-cli/tests/cli_calibrate.rs and owning native/session tests (FR-007 through FR-012).

## Phase 7: Polish, verification and publication

- [x] T024 Update AGENTS.md, CONVENTIONS.md, CONTRIBUTING.md, docs/fragcap-specification.md, public help/guides/examples, glossary as needed and changelog.d/S168.*.md with operator output/diagnosis contracts (FR-013).
- [x] T025 Audit every original criterion and complete specs/168-operator-diagnostics-calibration-verdicts/issue-acceptance.md with source and controlled verification evidence; sanity-check UTF-8/no BOM/mojibake and final renderer inventory (FR-001 through FR-014).
- [x] T026 Run focused owning regressions and full cargo xtask ci plus applicable generated API/documentation gates, watched to completion; record exact results in specs/168-operator-diagnostics-calibration-verdicts/verification.md (FR-014).
- [x] T027 Commit only S168 files with changelog fragments, automatically push codex/s168-operator-diagnostics-calibration-verdicts and publish/attach one official PR closing the fully satisfied scoped issues (FR-014).
- [ ] T028 Resolve every Codex/security review comment/thread and hosted CI failure, trigger at most one additional Codex round, verify green checks and review satisfaction on the final exact head, then hand off for owner review/merge (FR-014).

## Dependencies and parallel execution

T003 blocks all implementation. T004 precedes T007; T005 precedes T013 through T017; T006 precedes T018. T007 establishes the shared API before dependent renderer integration. After T003, layout/Doctor, discovery/preflight, and facade evidence tasks marked [P] may run as separate agents in disjoint owning files. Root owns session_ux.rs, events.rs, deep_capture.rs integration and final calibrate guidance functions; discovery owns calibrate preflight/front-door functions and targets/source code. Editing a shared file requires explicit function-boundary coordination. Stories converge at T019 through T023; all implementation and documentation converge before T025/T026. T027 and T028 are sequential publication gates.

## Implementation strategy

Deliver the complete eight-issue slice; each story has independently verifiable increments but no partial issue closure or silent scope reduction. Controlled evidence is sufficient for implementation acceptance, and deferred platform/storage/acquisition defects remain named. Automatic initial reviews are round one; only one explicit additional trigger is allowed.
