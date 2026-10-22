# Quickstart: Verify S179 CLI Result Output

## Prerequisites

Use disposable temporary target and catalog stores and the checked-in JSON fixtures. No Npcap driver, elevation, game, trust mutation, or owner bundle is needed for the controlled command matrix.

## Validation sequence

1. Run the targeted CLI output matrix test with the repository's default feature set. It exercises targets, catalog, schema, Doctor, Capture, and remaining top-level result families under default, quiet, silent, JSON, and JSON plus silent.
2. Run the matrix's target and catalog cases with the optional `net` feature where their feature-gated parser options exist; do not contact live external services.
3. Run `cargo xtask ci` with the repository's bounded local build settings and inspect its final result. Check format, lint, tests, spec, docs, and text hygiene reported by the gate.
4. On the official PR, verify the exact final head has green required checks, respond to every review thread, and stop before owner merge.

## Expected evidence

- JSON-mode stdout and stderr contain only parseable, newline-framed records on the channels declared in [the CLI result contract](contracts/cli-results.md), except authoritative JSON documents and extcap protocol output that retain their established grammar.
- Human silent suppresses ordinary successful result prose, while errors and required action text remain. JSON plus silent retains its selected machine records.
- At least four independent invalid schema facts produce four diagnostic records and one matching summary, with honest null source positions where none were observed.
