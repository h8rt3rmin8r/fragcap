# Validation Guide: S172

Use hidden noninteractive Windows execution with streamed output and exact exit codes. Product checks use controlled fixtures.

1. Inspect candidate 0.10.6 identities and unchanged historical/public v0.10.5 identities in `git diff`.
2. Run `cargo test -p fragcap-proxy --test https_proxy --locked`; all seven cases must pass with zero generic fallback retained.
3. Run notes 0.10.6, specification, review-handoff, review-record, supply-chain and conformance xtask gates.
4. Run `cargo xtask ci`, `cargo xtask msrv`, `cargo xtask neutral`, `cargo xtask docs build`, site frozen installation and unit/accessibility tests.
5. Check diff, strict UTF-8/LF/mojibake and final committed handoff. Hosted and publication checks remain explicitly not run at the local boundary.
