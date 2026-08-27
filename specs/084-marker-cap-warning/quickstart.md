# Quickstart: Marker Cap Warning Subject

## Focused Validation

Run the library warning tests:

```powershell
cargo test -p fragcap-profile signature::tests::a_truncated_candidate_set_warning_names_the_scanned_root
cargo test -p fragcap-profile signature::tests::two_truncated_scans_for_different_roots_do_not_share_one_warning
```

Run the discovery forwarding test:

```powershell
cargo test -p fragcap-targets --test user_pointed a_truncated_candidate_set_names_the_scan_root_in_the_warning
```

Run the CLI warning tests:

```powershell
cargo test -p fragcap-cli --test cli_targets targets_discovery_marker_cap_warning_names_the_scan_root
```

## Full Gate

Run the repository gate:

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

Expected result: all commands pass, and the capped binary-marker warning names the scanned root plus the skipped executable count.
