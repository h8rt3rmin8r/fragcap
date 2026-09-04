<!-- spec-impact: 24.3, 25.5, 28.1 -->

- 2026-09-04: Kept cargo-fuzz and libfuzzer-sys in an isolated `fuzz/`
  workspace with an independent lockfile. Coverage campaigns require nightly,
  but the product graph, lockfile, and Rust 1.88 minimum remain unchanged.
- 2026-09-04: Counted only fragcap-owned parsing and state transitions. Rustls,
  h2, h3, Quinn, httparse, and serde_json retain ownership of their internal
  wire or syntax decoders, so S126 does not inflate its coverage claim.
- 2026-09-04: Made the committed synthetic corpus replay on stable Rust twice
  in deterministic order. A contributor can reproduce permanent evidence
  without installing the coverage engine, while the separate pinned CI matrix
  proves every libFuzzer target continuously builds and runs.
- 2026-09-04: Added finite byte and record limits to the application JSONL
  prefix reader after the fuzz audit found it was the only crash-readable
  artifact reader without an input or record cap. The limits are above existing
  writer bounds and convert excess input into an explicit invalid-data result.
- 2026-09-04: Kept `tinyvec` at 1.12.0 in the isolated fuzz lockfile because
  1.13.0 does not compile under the exact pinned nightly. This is harness-only
  lock resolution and does not affect the product dependency graph.
- 2026-09-04: Excluded only generated fuzz build, finding, and coverage
  directories from the repository text walk. Authored targets, dictionaries,
  manifests, and promoted corpus inputs continue through the ordinary linter.
