# Quickstart: S167 dependency alert validation

## Preconditions

Use the repository's pinned Rust toolchain, Node.js, and pnpm 10.26.0. Run checks on the S167 branch. Do not run installed game software or mutate a live proxy session.

## Locked inventory checks

Inspect all four `spikes/**/Cargo.lock` files for rustls 0.23.45 or newer. Inspect every `site/pnpm-lock.yaml` occurrence of source-map-js, KaTeX, and DOMPurify, including duplicated versions and transitive references. Confirm the root `Cargo.lock` and released version files have no diff.

## Site and spike checks

From `site/`, perform a frozen pnpm install, then run the existing unit and production build commands. Run applicable locked checks for both spike manifests and their audit subprojects. Record unavailable platform-specific checks explicitly.

## Repository gate

Run `cargo xtask ci` and the documentation build. After push, require all applicable PR CI checks and third-party review findings to resolve against the final head before asking the owner to merge.

## Local evidence, 2026-10-06 UTC

- The affected lockfile scan found zero instances of rustls 0.23.43, source-map-js 1.2.1, KaTeX 0.16.47, or DOMPurify 3.4.13. All four spike locks carry rustls 0.23.45. The site lock carries only source-map-js 1.2.2, KaTeX 0.18.2, and DOMPurify 3.4.16.
- `pnpm install --frozen-lockfile`, `pnpm build`, and `pnpm test:unit` passed. The production build generated 86 static pages and the unit suite passed four tests.
- `pnpm test:accessibility` passed all 14 browser tests, including Mermaid diagram rendering at three viewport widths.
- `cargo test --locked` passed for the two spike projects and both audit subprojects. The main spike matrices passed seven and four tests respectively.
- The first `cargo xtask ci` run passed source tests but its source linter scanned newly generated ignored spike `target` output. After removing only the four build directories created by this run, `cargo xtask ci` completed with `ci: all checks passed`. No linter rule or security gate was weakened.
