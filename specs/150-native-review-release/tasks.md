# Tasks: Native Documentation and Reviewable Release Handoff

**Input**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), and [handoff contract](contracts/handoff.md).

**Tests**: Required controlled TDD and unchanged repository, security, packaging, site, and release gates. No installed sensitive product execution.

## Phase 1: Setup

- [x] T001 Verify merged S149 and intended remote, create S150 spec-kit artifacts under specs/150-native-review-release/.
- [x] T002 Search existing issues and track bounded delivery under #411, retaining parent #331/#333/#334/#278 and #372 acceptance.

## Phase 2: Foundational

- [x] T003 Run the blocking cross-artifact analyze gate over specs/150-native-review-release/spec.md, plan.md, and tasks.md before implementation.
- [x] T004 Record release branch, hidden orchestration, version, and incomplete-review decisions in changelog.d/S150-native-review-release.decisions.md.

## Phase 3: User Story 1 - Current Native Contract

**Independent Test**: Parser-only examples and actual artifact readers, plus site build/check and accessibility.

- [x] T005 [US1] Expand no-dispatch documentation corpus tests in crates/fragcap-cli/tests/cli_reference.rs and demonstrate failure against the stale README command before correcting guidance.
- [x] T006 [US1] Correct current native architecture, first-session journey, CLI semantics, compatibility, and output authority in site/content/docs/ and README.md (FR-001 through FR-004).
- [x] T007 [US1] Add discoverable Doctor/troubleshooting, security/privacy, packaging/migration, and stable library guidance under site/content/docs/ with navigation in meta.json.
- [x] T008 [US1] Link actual manifest examples and prove product-reader specimen validation; correct docs/schema/README.md so target schema validation is not advertised for artifacts.

## Phase 4: User Story 2 - Independent Review Readiness

**Independent Test**: Static twelve-area scope mapping and negative fixtures reject missing/ignored evidence and invented completed-review state without running the product.

- [x] T009 [US2] Add failing handoff-scope and not-started-template tests in xtask/src/review_handoff.rs, then implement bounded validation and wire ordinary CI through xtask/src/main.rs (FR-005 through FR-007).
- [x] T010 [US2] Publish twelve-area scope and reproducible independent-review instructions plus explicit not-started record under docs/security/; link immutable identity, installed-build methods, finding disposition, independent retest, and public-safe disclosure.
- [x] T011 [US2] Update #411 and related parent planning references to preserve actual independent installed-build audit and final acceptance as open work (FR-009, FR-010).

## Phase 5: User Story 3 - Fresh Release Preparation

**Independent Test**: Version-bound controlled gates, changelog and notes validation, and operator publication instructions without tag or publish.

- [x] T012 [US3] Write release/0.10.0 handoff under docs/maintainers/ and prepare short release-notes/v0.10.0.md from accumulated user-visible changes (FR-008, FR-011).
- [x] T013 [US3] Preview and perform configured local version-only preparation, update exact release-bound source/evidence through reviewed mechanical changes and regenerate existing goldens; preserve historical physical measurements.
- [x] T014 [US3] Assemble accumulated fragments into CHANGELOG.md with existing release task only on release/0.10.0; reconcile specification Applies-To and prepared-versus-published guidance.

## Phase 6: Polish and Handoff

- [x] T015 Run full cargo xtask ci, declared MSRV build, docs build/check, site unit/accessibility checks, release-note validation, and encoding sanity checks; record read evidence in specs/150-native-review-release/acceptance.md.
- [x] T016 Commit only S150/release files with conventional messages and repository co-author policy; automatically push release/0.10.0 and open official PR against main.
- [ ] T017 Wait for first reviews, respond to every comment and resolve each actionable thread, request at most one second @Codex round, and finish only with green current-head CI and satisfied reviews; notify operator for merge.

## Dependencies and Parallel Opportunities

Setup precedes analysis and all implementation. T005 precedes T006/T007. T009 precedes T010. Documentation and static review mapping can be investigated independently in distinct files, but source corrections are integrated serially by the main agent. Release operations T013/T014 follow a clean committed preparation and require a full final gate. No task uses a sensitive installed binary. The read-only planning documentation audit already ran alongside local release research.

## Implementation Strategy

Deliver the parser-protected native guidance first, then static reproducible review readiness, then the version-ready release. Maintain separate issue traceability: #411 covers this bounded handoff; parent #331/#333 retain unperformed independent acceptance, #372 retains operator reproduction, and #334/#278 retain final completion. The PR does not tag or publish.
