# Quickstart: S212 Validation

## Preconditions

Use the clean S212 branch and preserve the original diagnostic worktree. Read [the field contract](contracts/http-head-failure-count.md) and [the disposition ledger](contracts/disposition-ledger.md). No live game, driver run, elevation, or custom installed-product path is needed for this slice.

## Controlled read cases

Run the focused proxy tests after implementation, including zero-byte clean EOF, zero-byte I/O error, partial EOF, partial read error, head-size refusal, complete-head success, and an unrelated failure. Compare the terminal code and optional count to the contract table. Assert that the count is bounded and no partial head content is retained in the diagnostic.

## End-to-end readback

Run the controlled native proxy/facade and CLI terminal diagnostics tests with new count assertions. Separately, write a bounded native `proxy.jsonl` fixture, invoke `bundle debug-report` on that exact bundle, and compare the `proxy.head_read_failures[]` row with the source connection ID, allowlisted code and boundary, and count. A complete head and clean zero-byte EOF omit `http_head_bytes_before_failure` from native and terminal diagnostics and have no bundle failure row; no projection uses a `null` placeholder. Prior records without the key remain readable. Check that the report omits raw detail, endpoints, head bytes, and payload.

## Tracker and diff audit

Read the active epic and all affected child issues after the tracker edit. Compare completed work, remaining behavior, metadata, and completion tests to [the specification](spec.md). Confirm all 20 original paths appear in the disposition ledger. The public diff has six narrowly extracted Rust diagnostic paths plus the newly authored bundle-report projection in `route_report.rs`, the separately governed CI repair files listed in the ledger, targeted tests, and documentation; no experiment-only dependency or private detail enters it.

## Finite audit repair

Run locked `cargo-deny 0.21.0` against the all-features workspace and each of the six isolated Rust graphs. Confirm that the nine `heapless 0.8.0` exceptions apply only through `smoltcp@0.12.0`, expire on 2026-12-15 UTC, and leave `unsound = "all"` intact. A controlled new-direct-parent canary and an expired-waiver canary must fail. Read both isolated spike locks for `url 2.5.9` and `form_urlencoded 1.2.3`. Read [#640](https://github.com/h8rt3rmin8r/fragcap/issues/640) as the still-open replacement obligation. This pass is exception-qualified, not a clean dependency graph.

Run `cargo xtask supply-chain` and its focused tests. The governed tool pin must match the hosted workflow, and the nine exception records must match `deny.toml` on advisory ID, exact package, parent, and expiry. Missing, extra, broad, expired, and spoofed mirror entries fail controlled negative tests. Confirm the routine `rustls 0.23.46` review renewal against unchanged checksum, selected features, three graph digests, Rust floor, and a fresh audit, without claiming an independent security assessment. Compare the final CI repair diff to all seven exact paths in the ledger.

## Repository gates

Run `cargo fmt --all -- --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all --locked` in the foreground. Run the repository conventions linter, documentation linter in check mode, and both shell wrapper compliance checkers according to `CONTRIBUTING.md`. Inspect final PR-head CI and review results separately. Passing source tests does not establish installed-product or live-target behavior.
