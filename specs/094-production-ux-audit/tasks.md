# Tasks: Production UX And Accessibility Audit

**Input**: Design documents from `specs/094-production-ux-audit/`

**Prerequisites**: `plan.md`, `spec.md`, `research.md`, `data-model.md`, `contracts/audit-report-contract.md`, `quickstart.md`

**Tests**: S094 is itself a structured production audit. Tasks distinguish artifact checks, browser observations, issue triage, and repository gates.

**Organization**: Tasks are grouped by user story. The audit report is the shared result assembled incrementally and finalized only after all stories complete.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can be performed independently after its phase prerequisites
- **[Story]**: Maps the task to a user story in `spec.md`
- Every task names its durable evidence destination or inspected source

## Phase 1: Setup

**Purpose**: Establish clean branch state, active feature context, and the evidence contract.

- [x] T001 Confirm clean `main`, create `codex/094-production-ux-audit`, and point `.specify/feature.json` at `specs/094-production-ux-audit`
- [x] T002 Read issue #249, issue #255, the constitution, authorized-use context, master specification, plan ordering, conventions, contributor workflow, and `specs/094-production-ux-audit/plan.md`
- [x] T003 Create and validate the S094 specification set under `specs/094-production-ux-audit/`
- [x] T004 Validate every item in `specs/094-production-ux-audit/checklists/requirements.md` and `specs/094-production-ux-audit/checklists/ux.md`

---

## Phase 2: Foundational Audit Inventory

**Purpose**: Produce the immutable subject and complete population before any route claim.

**CRITICAL**: No route or accessibility result is final until this phase reconciles the production inventory.

- [x] T005 Record repository commit, operating system, browser, Node.js, pnpm, and Rust versions for `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T006 Run `pnpm install --frozen-lockfile` in `site/` and prove `site/pnpm-lock.yaml` did not change
- [x] T007 Run the production build in `site/`, confirm `site/out/.nojekyll` and `site/out/CNAME`, and record the result
- [x] T008 Derive source routes from `site/app/`, `site/content/docs/`, and generated metadata for `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T009 Derive exported routes from `site/out/`, normalize their public paths, and reconcile source, export, and navigation sets in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T010 Start a loopback-only static server rooted at `site/out/` and record the command in `docs/audits/2026-08-28-production-ux-accessibility.md`

**Checkpoint**: The exact production artifact and route population are known and reproducible.

---

## Phase 3: User Story 1 - Trust The Published Route Set (Priority: P1)

**Goal**: Give every public route and the not-found probe one explicit production result.

**Independent Test**: Reconcile expected, exported, and observed route counts with no silent omission.

- [x] T011 [US1] Open every public route at 1440 px from the served export and record status, title, navigation, heading, landmark, footer, and console outcomes in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T012 [US1] Follow shared navigation and representative in-content links from the built pages and reconcile reachable destinations in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T013 [US1] Open an unknown route, assess not-found status and recovery navigation, and record the result in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T014 [US1] Reconcile route arithmetic and document every missing, extra, redirected, or failed route as a finding in `docs/audits/2026-08-28-production-ux-accessibility.md`

**Checkpoint**: One hundred percent of the reconciled route inventory has a desktop production observation.

---

## Phase 4: User Story 2 - Use The Site Across Access Modes (Priority: P1)

**Goal**: Establish keyboard, semantic, responsive, theme, zoom, search, and complex-content behavior from the production artifact.

**Independent Test**: Reconcile the access-mode matrix for every documentation route and the representative shared journeys.

- [x] T015 [P] [US2] Inspect heading hierarchy, landmarks, language, link purpose, control names and states, image alternatives, and diagram alternatives on every documentation route, recording results in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T016 [P] [US2] Run available automated accessibility inspection on every documentation route and record rule-set scope, violations, and limitations in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T017 [US2] Complete the 1440 px keyboard journey through skip link, top navigation, sidebar, search, table of contents, theme control, content, and footer, recording focus order, visibility, operation, and traps in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T018 [US2] Complete the 320 px keyboard journey through mobile navigation, search, content, theme control, and footer in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T019 [US2] Open every documentation route at 768 px and 320 px and record viewport overflow, clipping, overlap, navigation, and footer outcomes in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T020 [US2] Inspect every long command, code block, table, matrix, Mermaid diagram, and content image at required narrow widths in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T021 [US2] Inspect representative routes at 200 percent zoom and record reflow, reachability, focus, and fixed-content outcomes in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T022 [US2] Inspect representative home and documentation routes in light and dark themes and record computed text, focus, control, and graphical contrast evidence in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T023 [US2] Query search for at least two current Capture terms, two current Deep Capture terms, and two retired command terms, recording ranking and stale-result outcomes in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T024 [US2] Check internal links and anchors in the production route set and run the repository external-link surface, recording failures and network limitations in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T025 [US2] Reconcile all required accessibility checks as pass, fail, not run, or not applicable and disclose native assistive-technology limitations in `docs/audits/2026-08-28-production-ux-accessibility.md`

**Checkpoint**: Every required access mode and complex-content class has an explicit result with no inferred passes.

---

## Phase 5: User Story 3 - Act On Audit Findings (Priority: P2)

**Goal**: Give each material finding a reproducible, non-duplicate issue disposition.

**Independent Test**: Every critical, high, or medium finding has exactly one linked existing or new issue, and low findings have reasoned dispositions.

- [x] T026 [US3] Normalize each observed defect to the finding fields and severity rules in `specs/094-production-ux-audit/contracts/audit-report-contract.md`
- [x] T027 [US3] Search open and closed GitHub issues for overlap with every material finding and record terms and candidate results in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T028 [US3] Link an existing owner or file one narrow issue with reproduction, acceptance criteria, labels, issue #249 linkage, and the `Post-v0.7.0 documentation` milestone for every material finding
- [x] T029 [US3] Reconcile every finding to exactly one disposition and keep issue #255 open in `docs/audits/2026-08-28-production-ux-accessibility.md`

**Checkpoint**: Maintainers can triage every finding without repeating the audit.

---

## Phase 6: Polish And Cross-Cutting Validation

**Purpose**: Finalize durable evidence and prove repository integrity.

- [x] T030 Finalize `docs/audits/2026-08-28-production-ux-accessibility.md` in the required contract order with route arithmetic, limitations, findings, dispositions, and conclusion
- [x] T031 Add `changelog.d/249-production-ux-audit.fixed.md` describing the audit evidence and follow-up boundary
- [x] T032 Run `cargo xtask docs check` and `cargo xtask docs build` from the repository root
- [x] T033 Run `cargo xtask ci` in the foreground and record the successful result in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T034 Run `git diff --check`, UTF-8/BOM, LF, mojibake, Unicode dash, lockfile, task completion, and scope audits across the final diff
- [x] T035 Mark every completed task in `specs/094-production-ux-audit/tasks.md`, stage only S094 files, commit locally with the repository co-author trailer, and halt before `git push`

---

## Dependencies And Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts from clean `main`.
- **Foundational inventory (Phase 2)**: Depends on Setup and blocks all observations.
- **User Story 1 (Phase 3)**: Depends on the served export and reconciled inventory.
- **User Story 2 (Phase 4)**: Depends on the same foundation and may proceed independently from route-link traversal after inventory reconciliation.
- **User Story 3 (Phase 5)**: Depends on all failed observations from User Stories 1 and 2.
- **Polish (Phase 6)**: Depends on all findings and dispositions.

### User Story Dependencies

- **User Story 1**: No dependency on User Story 2 after Phase 2.
- **User Story 2**: No dependency on User Story 1 after Phase 2, except final route arithmetic shares the same inventory.
- **User Story 3**: Depends on findings from both P1 stories.

### Parallel Opportunities

- T015 and T016 can collect semantic and automated results independently.
- Route observations at different widths are mechanically independent, though one browser session may execute them sequentially.
- Overlap searches for distinct findings are independent after T026.
- Repository docs checks can be prepared independently, but the final full gate runs after the report stabilizes.

---

## Implementation Strategy

### MVP First

1. Complete Setup and the production inventory.
2. Complete User Story 1 and prove exact route coverage.
3. Preserve the route matrix as independently reviewable evidence.

### Incremental Delivery

1. Establish the immutable artifact and route population.
2. Add desktop route evidence.
3. Add keyboard, semantic, responsive, search, and link evidence.
4. Convert material findings into narrow issue dispositions.
5. Finalize the report and repository gates.

## Notes

- `.specify/feature.json` is local state and must never be staged.
- S094 is intentionally audit-only. Finding a small defect does not authorize fixing it in this branch.
- A not-run check is an honest result when its reason and confidence impact are recorded.
- Halt after the local commit. Push and pull-request creation require the user's next instruction.
