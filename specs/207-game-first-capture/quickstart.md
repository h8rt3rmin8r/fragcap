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
- `cargo xtask ci`: all checks passed on the final source, including formatting, all-feature lint and tests, docs, specification, conformance, performance, and Windows integration authority.
- `git diff --check`: passed. Strict UTF-8 decoding, no BOM, LF, and mojibake check passed for 66 modified or new text files.

The hosted Windows direct-route matrix, staged executable, documentation site build, exact-head CI and PR review remain pending until the branch is published. These controlled results do not establish installed ESO success.
