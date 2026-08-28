# Quickstart: Production UX And Accessibility Audit

## 1. Establish the artifact

From `site/`, install without lockfile drift and build the static export:

```powershell
pnpm install --frozen-lockfile
pnpm build
```

Record tool versions, the commit, both command results, and a clean `git diff -- site/pnpm-lock.yaml`.

## 2. Reconcile routes

Enumerate static application routes, content routes, and exported HTML under `site/out/`. Normalize `/index.html` to `/` and other `*/index.html` paths to their public route. Resolve every discrepancy before claiming complete coverage.

## 3. Serve the immutable export

Use an available local static server rooted at `site/out/`. Record its command and bind it to loopback. Do not use the development server as audit evidence.

## 4. Execute the browser matrix

Open every public route at desktop width. Open every documentation route at 768 px and 320 px. For shared layouts and pages containing complex content, record navigation, heading and landmark structure, focus, overflow, alternatives, themes, and computed contrast evidence. Exercise a representative route at 200 percent zoom.

Complete the keyboard journey through the skip link, top or mobile navigation, sidebar, search, table of contents, content links, theme control, and footer. Record absent surfaces as not applicable, not as passes.

Run the search and link cases named in the specification. Probe one unknown route for not-found and recovery behavior.

## 5. Triage findings

For every material observation, search open and closed GitHub issues before creating anything. Link an existing owner or file one narrow issue following [the report contract](contracts/audit-report-contract.md). Do not fix the defect in S094.

## 6. Write and verify the report

Create `docs/audits/2026-08-28-production-ux-accessibility.md`, reconcile the route arithmetic and required checks, disclose not-run checks, then run:

```powershell
cargo xtask docs check
cargo xtask docs build
cargo xtask ci
git diff --check
```

Review the final diff for issue #249 scope, UTF-8 without BOM, LF endings, no dash-like Unicode punctuation, no lockfile drift, and exclusion of `.specify/feature.json` from staging.
