# S213 Verification

**Date**: 2026-11-24

## Local gates

- `cargo test --locked -p xtask supply_chain::tests -- --nocapture`: 22 supply-chain tests passed. The audit guard fixtures cover the four bypass settings, noncanonical root forms, bare CR refusal, CRLF acceptance, comments, nested overrides, and indentless `packages` items.
- `cargo xtask supply-chain`: passed with zero findings on the changed repository policy.
- `cargo xtask ci`: passed on the final local source and test set. The run includes formatting, Clippy, the clean-root test suite, repository lint, dependency and license checks, supply-chain policy, package certification, wrappers, skills, documentation, conformance, performance, and native Windows evidence checks.
- `npm exec --yes --package=pnpm@10.26.0 -- pnpm install --frozen-lockfile` in `site/`: passed with the lockfile unchanged.
- `npm exec --yes --package=pnpm@10.26.0 -- pnpm audit --audit-level low` in `site/`: passed with `No known vulnerabilities found` and no site advisory ignore.
- `npm exec --yes --package=pnpm@10.26.0 -- pnpm build` in `site/`: passed and generated 120 static pages. The generated build outputs are ignored and produced no tracked source changes.

## Scope of proof

These checks qualify the repository policy and locked documentation site on this change. Hosted PR checks and external review remain separate authorities. S213 changes no product runtime, installed package, game behavior, site dependency lock, or Rust advisory exception under #640.
