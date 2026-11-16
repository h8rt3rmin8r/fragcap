# Quickstart: S207 Verification

1. Run focused library and proxy tests for route identity, packet/application scope, admission, and protocol provenance.
2. Run the controlled managed launcher fixture with no prior compatibility facts, then repeat with stale and failed rows. Compare the authorization, applied route, facts, bundle, and structured terminal output.
3. Run explicit advanced SOCKS-only fixture and HTTP CONNECT close fixture. Assert no HTTP proxy variables in the SOCKS plan and exact pre-TLS diagnosis for close.
4. Run the applicable Windows direct-route/sentinel matrix, then `cargo xtask ci` and exact-head CI after pushing the branch.
5. Inspect changelog fragments, current spec and help, file encoding, line endings, and historical store/bundle readback before PR handoff.

No installed private-title run, administrator prompt, or publication is required to claim controlled implementation closure.

## Local source evidence (2026-11-15)

- `cargo test -p fragcap-targets --lib --quiet`: 154 passed, including a version 15 fact-row migration to version 16 and new route readback.
- `cargo test -p fragcap-cli --lib --quiet`: 412 passed, 2 existing ignored, including direct-route fact receipts and the SOCKS-only launch-chain default.
- `cargo test -p fragcap-cli --test cli_reference --quiet`: 9 passed against the current public command tree.
- `cargo test -p fragcap-cli --test cli_deep_capture --quiet -- --test-threads=1`: 24 passed with controlled session artifacts and authorization checks.
- `cargo test -p fragcap --features deep-capture --lib game_first_gate_excludes_zero_to_three_intermediaries_and_same_name_bystanders --quiet`: 1 passed.
- Focused native SOCKS and HTTPS tests: 17 SOCKS tests and 2 CONNECT handoff tests passed.
- `cargo xtask ci`: all checks passed on the implementation source, including formatting, all-feature lint and tests, docs, specification, conformance, performance, and Windows integration authority.
- `git diff --check`: passed. Strict UTF-8 decoding, no BOM, LF, and mojibake check passed for 66 modified or new text files.

These controlled results do not establish installed ESO success.

## PR and hosted evidence (2026-11-16)

- [PR #623](https://github.com/h8rt3rmin8r/fragcap/pull/623) is published for S207. The two authorized Codex review rounds were completed; all findings were fixed, replied to, and resolved. No third round was requested.
- `cargo xtask ci` passed again on implementation head `319e2788ae47f26be5f21656265c5c69c2aaaea1`; the checked-in package-certification validator tests also passed all 11 cases, including canonical-plan session identity, a longer ordinary event stream, and report schema 3.
- The [PR Windows native integration](https://github.com/h8rt3rmin8r/fragcap/actions/runs/43812578152), [site export](https://github.com/h8rt3rmin8r/fragcap/actions/runs/43812578318), [advisory audit](https://github.com/h8rt3rmin8r/fragcap/actions/runs/43812578227), and remaining applicable checks finished green on the PR merge candidate. Expected soak and Pages deployment jobs were skipped.
- The [Windows package certification run](https://github.com/h8rt3rmin8r/fragcap/actions/runs/43812578309) passed on the PR merge candidate. Its sanitized report has schema 4, `complete=true`, zero findings, and complete portable and installed controlled smoke sessions. Each smoke reports five application events, session-evidence schema 3, and reconciled cleanup. Its embedded build revision is GitHub's temporary PR merge commit, so branch-head package identity is qualified separately at handoff.
- No private ESO installed-title capture was run for this slice. Controlled packaged smoke proves the shipped executable and installer workflow, not ESO gameplay or complete traffic capture.
