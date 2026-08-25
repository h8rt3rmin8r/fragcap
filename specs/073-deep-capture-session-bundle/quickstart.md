# Quickstart: Deep Capture session bundle

This is a design-only slice. Validate the contract by reviewing the example bundle and running repository documentation gates.

## Automated validation

1. Run `cargo fmt --check`.
2. Run `git diff --check`.
3. Run `cargo xtask lint`.
4. Run `cargo xtask deps`.
5. Run `cargo xtask spec`.
6. Run `cargo xtask changelog --check`.

## Manual validation

1. Read `contracts/example-bundle.md`.
2. Confirm every artifact named in issue #216 appears either as a produced artifact or an omission with a reason.
3. Confirm the manifest can answer what `doctor` needs for #218: trust state, proxy process/port state, key-log state, sensitive artifact paths, and cleanup result.
4. Confirm HAR production is tied to HTTP observability rather than Deep Capture mode alone.
