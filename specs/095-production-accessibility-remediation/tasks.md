# Tasks: Production Accessibility Remediation

**Input**: Design documents from `specs/095-production-accessibility-remediation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Browser regression tasks are mandatory and precede each correction under the autopilot TDD discipline.

**Organization**: Tasks are grouped by independently testable user story while sharing one production-export browser harness.

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the development-only browser gate used by every story.

- [x] T001 Add exact-pinned `@playwright/test` 1.62.1 and the `test:accessibility` command in `site/package.json` and `site/pnpm-lock.yaml`
- [x] T002 Create the loopback export test configuration, route inventory, isolated activation request capture, and nonempty-population guards in `site/playwright.config.mjs` and `site/tests/production-accessibility.spec.mjs`

---

## Phase 2: User Story 1 - Reach The Primary Content Directly (Priority: P1) MVP

**Goal**: Give every public route one primary landmark and a visible-on-focus bypass that transfers focus to it.

**Independent Test**: Build and serve all 54 routes, then prove at 320, 768, and 1440 pixels that each has one `main-content` landmark, the first focusable control is the skip link, and activation focuses its destination.

### Tests for User Story 1

- [x] T003 [US1] Add route-wide landmark count, skip order, focus visibility, viewport containment, fragment, and destination-focus assertions in `site/tests/production-accessibility.spec.mjs`, then run them against the current export and record the expected failure
- [x] T004 [US1] Extend the serializable route observation with primary and skip-link evidence in `site/scripts/audit-export-dom.mjs`

### Implementation for User Story 1

- [x] T005 [US1] Render the shared skip link before persistent chrome, handle static-export focus transfer without a router request, and add its idle and focused styles in `site/app/layout.tsx`, `site/components/skip-link.tsx`, and `site/app/global.css`
- [x] T006 [US1] Make the home layout's existing primary element the destination and replace nested page landmarks with neutral wrappers in `site/app/(home)/layout.tsx`, `site/app/(home)/page.tsx`, `site/app/(home)/brand/page.tsx`, `site/app/(home)/disclaimer/page.tsx`, and `site/app/(home)/license/page.tsx`
- [x] T007 [US1] Make the documentation article the focusable primary destination in `site/app/docs/[[...slug]]/page.tsx`, then run the US1 browser regression to green

---

## Phase 3: User Story 2 - Read Normal Text In The Light Theme (Priority: P1)

**Goal**: Raise only the failed shared muted and red syntax text colors above 4.5:1 on every shipped light background.

**Independent Test**: Load representative light-theme documentation routes, require nonzero muted and syntax populations, and measure every affected computed foreground/background pair at or above 4.5:1.

### Tests for User Story 2

- [x] T008 [US2] Add computed-color, nearest-opaque-background, contrast-ratio, exact-token, and nonempty-population assertions in `site/tests/production-accessibility.spec.mjs`, then run them against the current export and record the expected failure

### Implementation for User Story 2

- [x] T009 [P] [US2] Override only the light muted foreground with `#6e6e6e` in `site/app/global.css`
- [x] T010 [P] [US2] Add a dependency-free post-highlight transform that changes only light Shiki `#D73A49` to `#cc3346` in `site/source.config.ts`, then run the US2 browser regression to green

---

## Phase 4: User Story 3 - Navigate A Truthful Content Outline (Priority: P2)

**Goal**: Preserve release content and anchors while preventing heading-level descents greater than one on generated changelog pages.

**Independent Test**: Generate every changelog route, compare preserved anchor identities, and assert that no rendered article heading descends by more than one level.

### Tests for User Story 3

- [x] T011 [US3] Add complete changelog-route heading-sequence and known-anchor preservation assertions in `site/tests/production-accessibility.spec.mjs`, then run them against the current export and record the expected failure

### Implementation for User Story 3

- [x] T012 [US3] Add failing synthetic orphan, sibling, nested, ascent, and fenced-code heading cases in `site/tests/changelog-headings.test.mjs`
- [x] T013 [US3] Implement an import-safe hierarchy-aware normalizer in `site/scripts/changelog-headings.mjs`, use it from `site/scripts/prebuild.mjs` without changing heading text, then run the Node and US3 browser regressions to green

---

## Phase 5: User Story 4 - Identify Architecture Diagrams (Priority: P2)

**Goal**: Give both architecture diagrams distinct Mermaid-native accessible names without hiding their internal semantics.

**Independent Test**: Hydrate `/docs/architecture` at every required viewport and assert exactly two graphics-document SVGs with the expected distinct resolved names.

### Tests for User Story 4

- [x] T014 [US4] Add hydrated diagram count, graphics-document role, expected-name, and distinctness assertions in `site/tests/production-accessibility.spec.mjs`, then run them against the current export and record the expected failure

### Implementation for User Story 4

- [x] T015 [US4] Add the two purpose-specific Mermaid `accTitle` directives in `site/content/docs/architecture.mdx`, then run the US4 browser regression to green

---

## Phase 6: Polish And Cross-Cutting Concerns

**Purpose**: Make the regression blocking, reconcile durable evidence, and complete repository verification.

- [x] T016 Update findings F01, F02, F03, and F06 with corrected evidence while leaving F04 and F05 open in `docs/audits/2026-08-28-production-ux-accessibility.md`
- [x] T017 Add Chromium installation plus the Node and accessibility regressions after the export build in `.github/workflows/docs.yml`, and record the pinned workflow and dependency rationale in `changelog.d/S095-browser-accessibility-gate.decisions.md`
- [x] T018 Add the user-visible correction summary in `changelog.d/263-268-production-accessibility.fixed.md`
- [x] T019 Run every command in `specs/095-production-accessibility-remediation/quickstart.md`, the full `cargo xtask ci` gate, encoding and staged-scope hygiene, and complete all task checkboxes in `specs/095-production-accessibility-remediation/tasks.md`

---

## Dependencies And Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately and blocks the browser-first story tests.
- **User Story 1 (Phase 2)**: Depends on Setup and establishes the shared route contract.
- **User Story 2 (Phase 3)**: Depends on Setup; its two implementation changes are parallel after the failing test exists.
- **User Story 3 (Phase 4)**: Depends on Setup and the generated changelog route inventory.
- **User Story 4 (Phase 5)**: Depends on Setup and client-side Mermaid hydration.
- **Polish (Phase 6)**: Depends on all four stories passing independently.

### User Story Dependencies

- **User Story 1 (P1)**: No dependency on another story.
- **User Story 2 (P1)**: No dependency on another story.
- **User Story 3 (P2)**: No dependency on another story.
- **User Story 4 (P2)**: No dependency on another story.

### Within Each User Story

- Add the browser assertion first and observe its failure against the current export.
- Make the narrow production correction.
- Rebuild and run the focused story assertion to green before proceeding.
- Require nonempty populations before accepting any aggregate pass.

### Parallel Opportunities

- After T008 fails as expected, T009 and T010 touch different files and can proceed in parallel.
- The four user stories own distinct production surfaces after the shared harness exists.
- Audit and changelog prose can be drafted separately after all browser results are known.

---

## Parallel Example: User Story 2

```text
Task: "Override the light muted foreground in site/app/global.css"
Task: "Correct the one failing light Shiki foreground in site/source.config.ts"
```

---

## Implementation Strategy

### MVP First

1. Establish the browser harness.
2. Write and observe the route-wide landmark and skip failure.
3. Complete User Story 1 and prove it independently.

### Incremental Delivery

1. Setup -> production-export browser gate available.
2. User Story 1 -> route structure and keyboard bypass corrected.
3. User Story 2 -> light-theme normal text meets the numeric threshold.
4. User Story 3 -> generated heading hierarchy corrected without content drift.
5. User Story 4 -> hydrated diagrams expose distinct names.
6. Polish -> workflow gate, durable evidence, changelog fragments, and full verification.

## Notes

- Every task uses an exact repository path and all 19 tasks follow the required checklist format.
- The four issue contracts remain separately demonstrable even though they ship in one pull request.
- No task changes search ranking or not-found recovery; those remain S096.
