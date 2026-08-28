<!-- spec-impact: none -->

## 2026-08-28 - Gate hydrated production accessibility in Chromium

The documentation workflow now exact-pins `@playwright/test` 1.62.1, installs
Chromium after building the static export, and blocks on the generated-heading
unit tests and production accessibility browser suite. Static inspection cannot
establish Mermaid's hydrated SVG semantics, computed contrast, or keyboard
focus transfer.

The development-only lockfile delta is four package records:
`@playwright/test`, `playwright`, and `playwright-core` under Apache-2.0, plus
Playwright's optional macOS `fsevents` record under MIT. None enters the
production bundle. The existing workflow pins Node.js 24, above Playwright's
Node.js 20 minimum, and the Chromium binary remains a CI test artifact rather
than a repository dependency.
