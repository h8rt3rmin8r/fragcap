# S151 Verification

## Scope and Authority

S151 implements bounded read-only Doctor diagnostics and reconciles current documentation with the published v0.10.0 baseline. The operator explicitly authorized push and an official PR, at most one manual second review request, and a final human-only merge handoff. No installed product, real game, production Doctor reproduction, real trust mutation, new release or registry-environment configuration was performed.

## Specification Cycle (2026-09-15)

The installed spec-kit sequence completed specify, clarify, requirements checklists, plan/research, tasks and blocking read-only analysis before implementation. Routine clarifications were recorded without operator interruption. Both checklists passed 16/16. Analysis mapped eight functional requirements and five success criteria to sixteen tasks with 100% coverage and zero findings or constitution conflicts. The checklist helper initially required a plan that did not yet exist; the constitutional checklist-before-plan order took precedence, and the helper passed after planning. Planning requested two read-only research agents. The ignored active-feature pointer and generated output remain uncommitted.

## TDD and Corrections (2026-09-15)

Before implementation, `cargo test -p fragcap-cli --lib slow_completion_is_timed_without_advance_opt_in --locked` and `cargo test -p fragcap-cli --lib doctor_timings_is_discoverable_in_normal_help --locked` each failed one regression (exit 101), demonstrating absent automatic slow durations and hidden timing help. `cargo test -p xtask actual_publication_has_separate_reviewed_source_authority --locked` failed before the publication record existed. After adding the checker but before reconciling documentation, that regression exposed twenty missing/stale/contradictory current-applicability problems across thirteen surfaces.

The first integrated Doctor run passed 93 tests and failed one delayed-work test: nested leaf completion could cause an immediate parent waiting update. The corrected shared cadence enforces a minimum one-second gap across nested phases. The subsequent `cargo test -p fragcap-cli --lib doctor --locked` passed 94 tests, zero failures or ignored tests, in 3.04 seconds. Controlled delayed work preserved positive, negative and indeterminate values; fixed-instant tests covered the threshold and nested parent resumption; worker/coordinator panic and broken diagnostic output exercised joined lifetime and unchanged report/exit behavior. These tests inject work and do not reproduce the operator's host stall.

## Completed Local Gates (2026-09-15)

- `cargo xtask ci`: exit 0, including workspace tests, format/Clippy, lint, dependency direction, licensing/supply chain, package contract, wrappers/skills, docs and default/net parser reference, specification currency, guided-calibration acceptance, threat-model/review-handoff readiness, fuzz/failure inventories and controlled conformance. Independent review readiness explicitly reported that independent review was not performed.
- `cargo test -p fragcap-cli --test cli_reference --locked`: 8 passed, zero failed or ignored, without command dispatch. The full CI gate also passed the net-feature variant.
- `cargo test -p xtask spec::tests --locked`: initially 11 passed after reconciliation, then 12 passed after adding the fixed inventory's uniqueness and repository-relative path regression. This test-only addition followed the whole CI run and was formatted and checked with `cargo clippy -p xtask --all-targets -- -D warnings` (exit 0).
- `cargo xtask msrv`: exit 0, complete default workspace builds on declared Rust 1.88.
- `scripts/lint-docs.sh fix`: exit 0, regenerated the existing glossary index for the two new diagnostics/lifetime entries; whole CI passed the subsequent documentation check.
- Production site build: the existing package script's three steps (`node scripts/prebuild.mjs`, `node node_modules/next/dist/bin/next build`, `node scripts/postbuild.mjs`) passed through the verified hidden launcher. Next.js compiled TypeScript and generated all 72 static pages.
- `node --test tests/changelog-headings.test.mjs`: 4 passed, zero failed or skipped.

- `node node_modules/@playwright/test/cli.js test --config=playwright.config.mjs`: all 13 production accessibility/browser contracts passed in 2.6 minutes, including all public routes at 320/768/1440px, current guidance searches and exported links. The existing conflicting `NO_COLOR`/`FORCE_COLOR` warning was non-fatal.
- `git diff --check`: exit 0. Strict UTF-8 decoding, absent BOM and mojibake sanity checks passed for all 38 scoped files. Build output and the feature pointer are ignored.

Hosted current-head checks and review disposition will be recorded on the PR and #414 after publication, rather than requiring an evidence-only source commit after every external transition.

## First-Round Review Correction (2026-09-15)

Automatic Codex review of `3ef67c9` raised one P2 comment: canonical version labels inside Markdown links bypassed the contradictory-claim scanner. The added linked stale/current-unpublished fixtures first failed (exit 101). The corrected normalization exposes canonical visible version labels for inline, reference and shortcut links, including balanced/escaped destination parentheses, without interpreting unrelated destination versions as claims. `cargo test -p xtask spec::tests --locked` then passed 13 tests with zero failures or ignored tests, targeted xtask Clippy passed, all 189 xtask tests passed and `cargo xtask spec` reconciled all thirteen actual surfaces. All comments/threads and the permitted second-round/current-head CI disposition are recorded on PR #415.

## Publication Evidence and Nonclaims

Read-only GitHub evidence confirms v0.10.0 was published at 2026-09-15T15:55:49Z as a non-draft, non-prerelease tag rooted at `787739edfa8d748e25cb4b5c4965f5c936850d16`. Its release run 34990387334 completed all four jobs. The reviewed source record and thirteen explicit current-baseline markers are independent of candidate workspace version. Actual downloaded package identities and publication results remain in the updated maintainer handoff; nothing was installed or executed to reconcile those facts.

S151 diagnostics are not in published v0.10.0. Actual elevated first-run/repeat measurement (#372), independent installed security review and performance retest (#333/#413), final completion (#334/#278), and the remaining final documentation criteria (#331) remain open. Existing unsigned distribution policy is unchanged. The `crates-io` environment was observed without protection rules; documentation now distinguishes that fact from the intended future approval gate, and S151 made no settings mutation.

## External Handoff

The official PR closes only bounded engineering issue #414. It references the parent criteria without claiming completion. Automatic first-round bot review and all hosted checks must settle on the final head; at most one manual second request is permitted. The agent does not merge its own PR, publish another release, or substitute its controlled tests for operator-owned installed evidence.
