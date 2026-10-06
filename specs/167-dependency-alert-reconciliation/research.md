# Research: S167 dependency alert reconciliation

## Current advisory inventory

The GitHub Dependabot API on 2026-10-06 reports eight open alerts: rustls 0.23.43 in four spike lockfiles (fixed at 0.23.45), source-map-js 1.2.1 in the site (fixed at 1.2.2), KaTeX 0.16.47 in the site (fixed at 0.18.2), and DOMPurify 3.4.13 in the site under two advisories (fixed at 3.4.16). The product root lockfile already resolves rustls 0.23.45.

## Decision: Reuse the narrow Dependabot patch

PR #455 changes the four isolated Rustls locks. It also changes one unrelated tempfile edge in the HTTP MITM spike, so S167 should regenerate or selectively retain only package-manager-valid changes after checking the diff.

**Rationale**: The bot update provides an exact, reviewable fixed-version target, but its existing PR omits the site findings.

**Alternative considered**: Merge PR #455 separately, then open a second site PR. That would split one approved slice and require two owner merge decisions.

## Decision: Resolve the site dependency graph as a whole

The site currently locks both source-map-js 1.2.1 and 1.2.2. Mermaid 11.16.1 introduces DOMPurify and KaTeX; Tailwind and PostCSS introduce source-map-js. Registry metadata for Mermaid 11.17.2 and 12.0.0 still declares KaTeX `^0.16.47`, so a parent upgrade alone cannot reach the fixed 0.18.2 line. Exact pnpm workspace overrides for all three packages remove vulnerable duplicate instances and remain visible in both `pnpm-workspace.yaml` and the generated lockfile. The site's frozen install and build are the first compatibility checks; its browser diagram tests are the direct Mermaid rendering check.

**Rationale**: Lockfile text replacement without dependency resolution can leave a stale or invalid frozen install.

**Alternative considered**: Dismiss low-severity alerts. This leaves known vulnerable versions in tracked source and does not meet the owner's requested reconciliation.

## Decision: Keep research and release boundaries explicit

The spike projects are historical experiments, the site is public documentation, and the root workspace is the released product. Updating the isolated locks and site cannot be represented as a new product release or as proof of universal live game behavior.

**Rationale**: The repository's supply-chain and release authority distinguishes proposed source, merged source, and published bytes.
