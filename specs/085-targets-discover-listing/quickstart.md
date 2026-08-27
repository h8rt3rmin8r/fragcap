# Quickstart: Targets Discover Listing

Run the focused discovery listing tests:

```powershell
cargo test -p fragcap-cli --test cli_targets discover_lists_steam_titles_through_the_cli
cargo test -p fragcap-cli commands::targets::tests::discovery_account_groups_zero_outcomes
cargo test -p fragcap-cli commands::targets::tests::discovery_table_preserves_evidence_under_rows
```

Run the warning stream regression:

```powershell
cargo test -p fragcap-cli --test cli_targets targets_discovery_marker_cap_warning_names_the_scan_root
```

Run formatting and the full repository gate:

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

Expected outcome:

- Discovery stdout has no tab characters.
- Discovery stdout contains `Discovery stores:`, `SOURCE`, `IDENTITY`, `FIDELITY`, and `NAME`.
- Evidence lines remain indented under candidate rows.
- The account prints as labelled lines with zero outcomes grouped.
- Warnings remain on stderr.
