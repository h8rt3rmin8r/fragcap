# Tasks: Site Discovery And Recovery

**Input**: Design documents from `specs/096-site-discovery-recovery/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/site-discovery-recovery-contract.md`, `quickstart.md`

**Tests**: Production-equivalent browser tests are required by FR-006, FR-012, and the autopilot test discipline. Test tasks precede implementation tasks and must reproduce the current failures first.

## Phase 1: Setup

**Purpose**: Establish the S096 artifact and dependency baseline.

- [x] T001 Validate the complete S096 artifact set and both requirements checklists in `specs/096-site-discovery-recovery/`
- [x] T002 Confirm the current 54-route export, failed retired-command ordering, generic not-found body, and clean dependency baseline in `site/out/`, `site/package.json`, and `site/pnpm-lock.yaml`

---

## Phase 2: Foundational Browser Contracts

**Purpose**: Add production-export regressions that fail against the current site before either correction is implemented.

**Critical**: No implementation task begins until both failure modes are reproduced.

- [x] T003 Add shared production-search helpers with nonempty result assertions to `site/tests/production-accessibility.spec.mjs`
- [x] T004 Add failing current-first, preserved-history, case, whitespace, and current-query baseline search cases to `site/tests/production-accessibility.spec.mjs`
- [x] T005 Add failing shallow and nested HTTP 404, landmark, recovery-link, overflow, skip, and keyboard-navigation cases at 320 and 1440 pixels to `site/tests/production-accessibility.spec.mjs`
- [x] T006 Run the focused production accessibility suite and record both expected pre-implementation failures in `specs/096-site-discovery-recovery/research.md`

**Checkpoint**: The existing production export fails for the two audited defects while earlier S095 coverage remains green.

---

## Phase 3: User Story 1 - Find Current Command Guidance First (Priority: P1)

**Goal**: Make exact retired-command searches lead to actionable current guidance while preserving history.

**Independent Test**: Search the hydrated production export for both retired names and verify the current command reference activates first, states the replacement, and leaves v0.5.0 history later in the results.

- [x] T007 [US1] Add concise `fragcap run` and `fragcap tap` migration guidance with stable destinations to `site/content/docs/reference/cli.mdx`
- [x] T008 [US1] Promote `zbsearch` 3.3.4 to an explicit direct dependency in `site/package.json` and update `site/pnpm-lock.yaml` without adding a package version
- [x] T009 [US1] Add exact-query promotion rules for the stable command-reference page id in `site/app/static.json/route.ts`
- [x] T010 [US1] Build the production export and make all retired-command and current-query search cases pass through `site/tests/production-accessibility.spec.mjs`

**Checkpoint**: Current command recovery is first, historical results remain searchable, and unrelated baseline leaders are unchanged.

---

## Phase 4: User Story 2 - Recover From A Missing Page (Priority: P2)

**Goal**: Turn every static-host not-found response into a branded, accessible recovery surface without changing its HTTP status.

**Independent Test**: Request shallow and deeply nested absent paths at both required widths, verify HTTP 404 and one `main-content` landmark, then reach the homepage and getting-started guide through keyboard-reachable recovery links.

- [x] T011 [US2] Implement the server-rendered branded recovery surface in `site/app/not-found.tsx`
- [x] T012 [US2] Narrow the expected main-document 404 console exception while keeping every other browser and console error fatal in `site/tests/production-accessibility.spec.mjs`
- [x] T013 [US2] Build the production export and make all not-found semantics, responsive, skip, and navigation cases pass through `site/tests/production-accessibility.spec.mjs`

**Checkpoint**: Missing paths remain true 404 responses and provide a complete recovery journey at narrow and desktop widths.

---

## Phase 5: Polish And Cross-Cutting Concerns

**Purpose**: Reconcile durable evidence and run the complete merge gate.

- [x] T014 Append S096 correction evidence for F04 and F05 without rewriting prior audit observations in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T015 Add the user-visible correction fragment in `changelog.d/266-267-site-discovery-recovery.fixed.md`
- [x] T016 Add the exact-query pinning and direct-dependency rationale in `changelog.d/S096-search-pinning.decisions.md`
- [x] T017 Run every command in `specs/096-site-discovery-recovery/quickstart.md`, the full `cargo xtask ci` gate, encoding and staged-scope hygiene, and complete all task checkboxes in `specs/096-site-discovery-recovery/tasks.md`
- [x] T018 Commit the complete verified S096 slice locally with the repository's conventional commit format

---

## Dependencies And Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational Browser Contracts (Phase 2)**: Depends on Setup and blocks both user stories.
- **User Story 1 (Phase 3)**: Depends on Phase 2.
- **User Story 2 (Phase 4)**: Depends on Phase 2 and may follow User Story 1 without sharing implementation files beyond the browser suite.
- **Polish (Phase 5)**: Depends on both user stories.

### User Story Dependencies

- **User Story 1**: Independently corrects issue #266 after the shared regression harness exists.
- **User Story 2**: Independently corrects issue #267 after the shared regression harness exists.
- The stories share only browser-test orchestration and can be reviewed as separate contract sections within the bundled slice.

### Within Each User Story

- Browser failure is reproduced before production changes.
- Authored current guidance exists before its search result is promoted.
- The direct dependency exists before first-party source imports its API.
- Production export is rebuilt before hydrated behavior is evaluated.
- Durable audit and changelog evidence follows observed passing behavior.

## Parallel Opportunities

- After T006, T007 and T011 affect separate production surfaces and can proceed independently.
- T014, T015, and T016 affect separate evidence files after both stories pass.
- Final verification remains sequential and foregrounded.

## Implementation Strategy

### Current-First Search Increment

1. Reproduce both retired-query failures and preserve the four baseline leaders.
2. Add actionable migration prose to the checked current command reference.
3. Make the already-installed search engine a direct dependency.
4. Pin only the two exact retired queries to the stable current page id.
5. Prove current-first activation and preserved historical groups in the real dialog.

### Not-Found Recovery Increment

1. Reproduce the true-404 dead end at shallow and nested missing paths.
2. Add one root-level server-rendered recovery page inside the existing provider.
3. Keep only the expected main-document 404 diagnostic from failing the error fixture.
4. Prove response status, landmarks, links, overflow, focus transfer, and destinations.

### Delivery

Finish both independently testable corrections, append durable audit evidence, run the full quickstart and repository gates, commit locally, and halt before push.
