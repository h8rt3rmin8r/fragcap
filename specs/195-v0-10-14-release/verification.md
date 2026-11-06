# S195 Verification

**Date:** 2026-11-06

**Branch:** `release/0.10.14`

**Base:** `1e44519a8d34406fcd3178365c1dde68b689439d`

## Source and defect evidence

- GitHub Actions run 42174601003 attempt 1 failed on the exact base commit when the controlled Steam setup path consumed 1,522 ms of a one-second test-only launch allowance. Attempt 2 passed on identical source, establishing a runner-timing regression rather than deterministic product failure.
- `steam_client_setup_reports_completion_then_reaches_fresh_calibration` now uses the same finite seven-second launch allowance as adjacent controlled calibration cases. Its observation duration remains one second. Production defaults, parsing, operator-supplied bounds, and deadline behavior are unchanged.
- The corrected focused integration test passed three controlled Windows executions during S195: once before candidate assembly, once through the complete `cargo xtask ci` suite, and once after all candidate files and dependency authorities were assembled.
- `owned_platform_capture_inherits_the_finite_session_launch_deadline` passed separately, preserving the exact finite deadline contract.

## Candidate evidence

- `cargo release 0.10.14 --workspace --execute --no-confirm` completed with tag, push, and publication disabled by repository release policy.
- Workspace packages, both isolated current lockfiles, generated capture goldens, conformance records, specification applicability, changelog, notes, and candidate review template identify 0.10.14 where candidate identity is required.
- `cargo xtask changelog --release 0.10.14 2026-11-06` consumed every unreleased fragment once after source format corrections. `changelog.d/` retains only its README.
- Current public-release markers, frozen published-review authority, download references, and registry references remain on verified v0.10.13.
- Dependency snapshots retain the reviewed package and edge counts: linux-all 188/455, windows-all 187/431, and windows-release 166/372. The release changes first-party identity only; no third-party package, source, feature, provider, or exception inventory changed.

## Local automated results

The following commands completed successfully against the assembled candidate:

- `cargo xtask ci`
- `cargo xtask msrv` (workspace builds at Rust 1.88)
- `cargo xtask neutral`
- `cargo xtask docs build`
- `cargo xtask notes 0.10.14`
- `cargo xtask supply-chain`
- `cargo deny --locked --all-features check advisories licenses bans sources`
- isolated `cargo deny --locked --all-features --manifest-path ... --config deny.toml check advisories` for fuzz, native performance, both native-proxy spike graphs, and both HTTP MITM spike graphs
- `pnpm --dir site audit --audit-level low`
- `git diff --check 1e44519a8d34406fcd3178365c1dde68b689439d`

The product dependency check reported only the repository's governed unmatched-license and duplicate-version warnings, then reported advisories, bans, licenses, and sources successful. Every isolated graph reported advisories successful. The site graph reported no known vulnerabilities.

A strict scan of all 44 changed non-binary files found valid UTF-8, no BOM, LF-only endings, and no common mojibake markers. Repository documentation, specification, review-handoff readiness, controlled conformance, supply-chain, wrappers, shell, and generated-file checks also passed through `cargo xtask ci` or their named candidate validators.

## Spec-Kit convergence

Post-implementation convergence mapped all eleven functional requirements and five success criteria to the chronological task record and assembled diff. No missing requirement, contradictory deadline, stale issue lifecycle, duplicate implementation task, or constitution conflict remained. The implementation changes only the controlled test timing boundary plus release-candidate authorities; it does not widen production behavior.

## Hosted orchestration record

- PR #568 opened from `release/0.10.14` with #566 closing on merge and #567 referenced without closure. Both project items moved to PR review.
- The automatic first Codex code-review round completed against implementation head `7b03592dfc6147fc87fb096aa134273b63f77f9e` with no finding, no review thread, and a thumbs-up reaction. The only following source change before hosted qualification recorded the PR number in this Spec-Kit evidence.
- Every applicable PR check completed successfully against `b7c5fafe6ec792a3b3183d21e9a8553d7b2a3de1`, including Windows and Ubuntu CI, MSRV, platform-neutral build, TShark analyzer consumption, all six fuzz targets, dependency policy, static documentation export, CodeQL, Windows capture, native integration, native performance, and Windows package certification. The manual soak and PR Pages deployment were expected skips.
- Branch-head Windows package-certification run 42184030117 completed successfully from `b7c5fafe6ec792a3b3183d21e9a8553d7b2a3de1` through exact build, candidate assembly, package lifecycle certification, and distribution and summary uploads.
- This receipt-only update changes no product, test, release identity, package input, or generated output. Its own final hosted checks, exact-head package rerun, and authorized second code and security reviews remain authoritative in GitHub because writing those terminal outcomes back into the tree would create another unqualified head.

## Explicitly unverified and excluded

- The final receipt-only head still requires its terminal hosted rerun, exact-head package certification, and authorized second code and security review before owner handoff.
- No merge, tag, GitHub release, crate publication, protected-environment approval, public-file readback, registry readback, or current-public marker advance has occurred.
- No installed candidate or private title was exercised. Installed-product observations occur after publication, and concrete failures return as version-specific GitHub issues.
- The controlled conformance suite passed; its separate live TShark analyzer tier has not yet run for this candidate.
- Review-handoff structure passed, but no independent whole-product security acceptance is claimed.
- Issue #566 remains open until the official PR merges. Issue #567 remains open through later publication and public-byte reconciliation.
