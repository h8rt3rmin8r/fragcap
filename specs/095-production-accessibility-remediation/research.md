# Research: Production Accessibility Remediation

## Decision 1: One shared skip control, layout-specific primary regions

**Decision**: Render one skip link as the first body child. Use the native primary landmark already emitted by the home layout as its home-route destination, replacing each nested page-level `main` with a neutral wrapper. Give the documentation page article a primary role and the same destination identity.

**Rationale**: The root layout is the only shared boundary across all routes, while the two layout families already own the correct route-specific content containers. A programmatically focusable destination makes fragment activation reliable without adding it to sequential keyboard order.

**Alternatives considered**:

- Wrap either layout in a new primary landmark. Rejected because the home layout already emits one and the documentation grid also contains persistent chrome.
- Add a skip link independently to both route-group layouts. Rejected because duplicated policy can drift.
- Replace the documentation article component. Rejected because its existing semantic and layout behavior can be preserved by passing standard element properties.

**Implementation evidence**: The first production-export test showed that Next's router intercepted the plain fragment anchor and requested a route-specific RSC payload that the static server correctly did not expose. The shared anchor therefore uses a minimal client handler to update the fragment, focus the existing destination, and scroll it into view without a network navigation. This deviates from the initial native-anchor assumption because observed production behavior disproved that assumption.

## Decision 2: Normalize generated headings by encountered hierarchy

**Decision**: Treat the removed canonical changelog category as the generated page's level-one title. Normalize every remaining Markdown heading relative to its nearest surviving ancestor, making an orphan first heading level two and permitting descendants to advance by at most one level.

**Rationale**: The current generator removes a canonical level-three category into frontmatter, rewrites noncanonical level-three headings to level four, and leaves source level-four headings unchanged. A hierarchy-aware line transform corrects current and future variable-depth input while preserving text, order, anchors, links, and release history.

**Alternatives considered**:

- Edit `CHANGELOG.md`. Rejected because it is the canonical release record and the defect is introduced only by the site transform.
- Rewrite every level-four heading as level two. Rejected because it flattens real descendants.
- Subtract a fixed number from every heading. Rejected because orphan and mixed-depth headings require relationship-aware normalization.
- Add a Markdown syntax-tree dependency. Rejected as disproportionate for the bounded line transform.

## Decision 3: Override only the failing light-theme colors

**Decision**: Override the light muted foreground from `#737373` to `#6e6e6e`, leaving the dark theme untouched. Add a dependency-free post-highlight syntax-tree transform that changes only Shiki's light `#D73A49` foreground to `#cc3346`.

**Rationale**: `#6e6e6e` is the lightest integer gray that clears both observed backgrounds, measuring 4.5143:1 on `#f1f1f1` and 4.6769:1 on `#f5f5f5`. `#cc3346` is the closest integer RGB correction to the existing red that clears the darker background, measuring 4.5024:1 on `#f1f1f1` and 4.6645:1 on `#f5f5f5`.

**Alternatives considered**:

- Override serialized inline styles in CSS. Rejected because it depends on generated attribute text and requires `!important`.
- Replace or clone the complete syntax theme. Rejected because one failing token does not justify a new production theme edge or broad visual change.
- Change dark-theme tokens too. Rejected because the audit found no dark-theme failure.

## Decision 4: Use Mermaid-native accessible titles

**Decision**: Add distinct `accTitle` directives to the two architecture Mermaid fences: `Capture packet attribution architecture` and `Deep Capture session architecture`.

**Rationale**: Mermaid 11.16.1 turns its native title directive into a child `title` and `aria-labelledby` on the hydrated SVG while preserving the existing graphics-document role and readable diagram descendants.

**Alternatives considered**:

- Put `role="img"` and a label on the wrapper. Rejected because that makes the SVG descendants presentational and discards diagram structure.
- Alter the returned SVG string after rendering. Rejected because it duplicates Mermaid's supported behavior.
- Add a long description directive. Rejected because the surrounding prose already supplies the detailed explanation.

## Decision 5: Gate the hydrated production export with Playwright

**Decision**: Add exact-pinned `@playwright/test` 1.62.1 as a development dependency, run Chromium against the loopback-served static export, and execute the test after the export build in the documentation workflow.

**Rationale**: Hydrated Mermaid SVGs, computed foreground/background pairs, skip-link focus visibility, and activation focus transfer exist only in a browser. The lockfile adds four package records: `@playwright/test`, `playwright`, and `playwright-core` under Apache-2.0, plus Playwright's optional macOS `fsevents` record under MIT. Playwright requires Node.js 20 or newer, never enters the production bundle, and runs under the workflow's pinned Node.js 24.

**Alternatives considered**:

- Scan static HTML only. Rejected because Mermaid SVGs have not hydrated and computed styles and focus behavior do not exist.
- Keep the S094 evaluator as manual evidence. Rejected because an evaluator without a runner is not a regression gate.
- Depend on a preinstalled browser executable. Rejected because runner images can change and would make the test environment implicit.

## Decision 6: Require nonempty test populations

**Decision**: The browser gate must fail if it finds no public routes, no affected muted text, no affected syntax token, or fewer than two architecture diagrams, in addition to checking the values of each observation.

**Rationale**: Selector drift can otherwise turn a removed or unobserved subject into a false pass, contrary to constitution P-9.

**Alternatives considered**:

- Treat an empty result as not applicable. Rejected because every population is known to exist in the S094 evidence.
- Pin only representative element markup. Rejected because route-wide shared behavior and generated output are the acceptance boundary.
