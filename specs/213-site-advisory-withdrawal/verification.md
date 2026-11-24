# S213 Verification

**Date**: 2026-11-24

## Initial PR candidate gates

The results below qualified the first PR #642 candidate before its security review identified the additional root-manifest configuration source. They do not qualify the corrected final head.

- `cargo test --locked -p xtask supply_chain::tests -- --nocapture`: 22 supply-chain tests passed. The audit guard fixtures cover the four bypass settings, noncanonical root forms, bare CR refusal, CRLF acceptance, comments, nested overrides, and indentless `packages` items.
- `cargo xtask supply-chain`: passed with zero findings on the changed repository policy.
- `cargo xtask ci`: passed on the initial local source and test set. The run includes formatting, Clippy, the clean-root test suite, repository lint, dependency and license checks, supply-chain policy, package certification, wrappers, skills, documentation, conformance, performance, and native Windows evidence checks.
- `npm exec --yes --package=pnpm@10.26.0 -- pnpm install --frozen-lockfile` in `site/`: passed with the lockfile unchanged.
- `npm exec --yes --package=pnpm@10.26.0 -- pnpm audit --audit-level low` in `site/`: passed with `No known vulnerabilities found` and no site advisory ignore.
- `npm exec --yes --package=pnpm@10.26.0 -- pnpm build` in `site/`: passed and generated 120 static pages. The generated build outputs are ignored and produced no tracked source changes.

## Review correction local gates

- [x] The expanded manifest, both `.npmrc` path, and audit-step environment/wiring fixtures were observed red before implementation, then green. The focused `supply_chain::tests` run passed 24 tests on the corrected local source. The hosted guard now checks static inputs and audit-step environment before the unchanged command in the same step.
- [x] The direct-source review correction passed `cargo xtask ci` before the fresh-home workflow change.
- [x] The hosted audit step now creates a fresh temporary directory and exports HOME and XDG_CONFIG_HOME after rejecting direct configuration variables and before invoking the unchanged audit command. Rust workflow-wiring mutation tests detect removal or reordering of that sequence; the focused tests passed 24/24, `cargo xtask supply-chain`, formatting, and diff checks passed after the change.
- [x] The exact hosted JavaScript guard passed its baseline and refused a direct `npm_config_only` value scoped only to the audit step in local execution. An isolated manifest-fixture command was rejected by the command-runner policy before it executed; the Rust manifest CVE and GHSA fixtures passed. This is a test-harness limit, not a passing hosted manifest-fixture result.
- [x] After the fresh-home workflow change, the final local `cargo xtask ci` passed on the then-current implementation, public documentation, and Spec Kit files. A read-only cross-artifact review found no remaining critical or high requirement gap. The later Spec Kit wording refinement changes no runtime or workflow code; its Markdown and diff checks are recorded separately below.
- [x] The pinned pnpm frozen install, unfiltered full audit (`No known vulnerabilities found`), and static build (120 pages) passed after the first review fix on the unchanged site manifest, workspace, and lockfile. The subsequent fresh-home change touched the hosted workflow, so those site commands do not prove the new hosted environment isolation; final-head hosted CI must execute that step.
- [x] After the final Spec Kit wording refinement, `cargo xtask docs check` passed, including both CLI reference test variants (9 tests each). `git diff --check` passed and the edited Spec Kit files contained no mojibake matches.
- [ ] Record the pushed final head SHA, terminal applicable hosted CI checks, and disposition of the security review finding and all review threads. Prior local green checks do not replace hosted final-head evidence.

## Scope of proof

The initial checks qualified the repository policy and locked documentation site before the expanded audit-source guard. Later focused checks and the full local gate qualify the expanded local source; hosted PR checks and external review remain separate authorities. S213 changes no product runtime, installed package, game behavior, site dependency lock, or Rust advisory exception under #640.
