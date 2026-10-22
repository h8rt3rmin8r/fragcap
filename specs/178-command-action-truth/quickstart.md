# Validation Guide: Truthful CLI Action Boundaries

Run S178 validation against disposable stores and synthetic bundles. Do not point collection tests at an owner bundle.

## Local checks

```text
cargo test -p fragcap-cli --test cli_bundle
cargo test -p fragcap-cli --test cli_catalog
cargo test -p fragcap-cli --test cli_targets
cargo test -p fragcap-cli --test cli_help
cargo xtask ci
```

The focused tests must demonstrate the cases in [the CLI contract](contracts/cli-action-results.md): incomplete and zero-work collection previews have no Apply advice; export rejects conflicting selectors before JSON; invalid catalog input creates no store; and discover/scan help agrees with first-run effects. `cargo xtask ci` is the repository-wide gate and must run to completion in the foreground.

## Review boundaries

Compare human and JSON collection reasons on synthetic missing/unowned paths. Inspect the final diff for untouched exact authorization, retained selection, and store identity behavior. Verify UTF-8 without BOM, LF, no mojibake, and the chronological S178 entry in `docs/plans/README.md`.
