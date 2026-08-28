# Research: Production UX And Accessibility Audit

## Decision 1: Audit the immutable production export

**Decision**: Install exactly from `site/pnpm-lock.yaml`, build `site/out`, and serve that directory through a local static HTTP server for browser inspection.

**Rationale**: Issue #249 concerns the production application. Development mode can hide export-only routing, asset, and script failures. A static HTTP origin preserves the built artifact while allowing routing, search data, scripts, anchors, and browser semantics to operate normally.

**Alternatives considered**:

- Inspect source or generated HTML without serving it. Rejected because it does not establish interactive navigation, search, focus, themes, or script-backed diagrams.
- Use the development server. Rejected because it is not the artifact deployed by GitHub Pages.
- Inspect `file://` URLs. Rejected because browser origin and absolute-path behavior differ from production hosting.

## Decision 2: Reconcile routes from source and output

**Decision**: Build the route inventory from static application routes, content-source routes, and exported HTML paths, then reconcile the sets before testing.

**Rationale**: Any one source can omit a defect. The output proves what was generated, content metadata proves what should be navigable, and static application routes cover pages outside the documentation tree.

**Alternatives considered**:

- Follow only visible navigation. Rejected because valid leaf routes can be absent from shared navigation.
- Enumerate only `site/out`. Rejected because missing expected output would disappear from the audit population.

## Decision 3: Use layered accessibility evidence

**Decision**: Combine keyboard traversal, browser accessibility and DOM inspection, computed styles, viewport screenshots, zoom, themes, and automated accessibility results where available. Mark native screen-reader announcements and operating-system high-contrast behavior not run unless actually exercised.

**Rationale**: No single automated rule set proves accessible use. Layered evidence covers structural and visual barriers while keeping the confidence boundary honest.

**Alternatives considered**:

- Treat automated accessibility output as the entire audit. Rejected because it misses focus order, clipping, search quality, and interaction behavior.
- Claim semantic inspection as a screen-reader pass. Rejected under P-9 because accessible markup evidence does not prove a specific assistive technology's announcements.

## Decision 4: Keep evidence compact and durable

**Decision**: Record commands, route matrices, representative observations, exact reproductions, and issue links in one Markdown report. Commit screenshots only when they add information the written route, viewport, selector, and measurement cannot preserve.

**Rationale**: A durable text record is diffable and accessible. A screenshot-only audit is hard to search and cannot establish semantics; a screenshot for every route and width is noisy and costly without adding evidence.

**Alternatives considered**:

- Commit the full browser screenshot corpus. Rejected because it bloats the repository and duplicates tabular evidence.
- Keep all evidence outside the repository. Rejected because the pull request would not contain reproducible acceptance evidence.

## Decision 5: File defects instead of fixing them here

**Decision**: Search open and closed GitHub issues for every material finding, link an existing owner when present, and otherwise file one narrow issue in the documentation milestone with reproduction and acceptance criteria.

**Rationale**: The issue explicitly defines S094 as an audit and warns against an omnibus correction PR. Separate issues preserve review boundaries and allow each defect to be prioritized honestly.

**Alternatives considered**:

- Correct small findings opportunistically. Rejected because size does not change the audit-versus-remediation boundary.
- File one umbrella issue for all findings. Rejected because unrelated causes and acceptance criteria would become coupled.

## Decision 6: Use WCAG 2.2 AA as classification guidance

**Decision**: Evaluate applicable Level A and AA structure, keyboard, focus, alternative text, reflow, contrast, target, and consistent-navigation expectations, with exact numeric thresholds recorded in the specification.

**Rationale**: The issue requests accessibility and contrast assessment but does not name a conformance version. WCAG 2.2 AA is current, measurable, and appropriate for a public documentation site.

**Alternatives considered**:

- Use WCAG 2.1 AA. Rejected because 2.2 preserves the relevant 2.1 criteria while adding current focus and target guidance.
- Make a formal conformance claim. Rejected because this bounded audit cannot substitute for every assistive technology, user setting, and content condition.
