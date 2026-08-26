# Quickstart: Targets Finding Fidelity

## Prerequisites

- Rust toolchain from `rust-toolchain.toml`
- Working checkout on `codex/083-targets-finding-fidelity`

## Focused Validation

Run the new and directly affected tests:

```powershell
cargo test -p fragcap-targets readiness::tests::technology_summary_marks_unverified_findings
cargo test -p fragcap-targets targets_export::tests::the_machine_surface_preserves_finding_fidelity
cargo test -p fragcap-cli --test cli_targets targets_listing_marks_unverified_technology_findings
```

Expected result: all commands pass. The CLI test proves the human table marks below-verified findings and keeps verified-or-stronger findings unmarked. The export test proves the machine surface preserves finding fidelity.

## Full Validation

Run the repository gate:

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

Expected result: both commands pass, including lint, docs, spec, wrappers, and tests.
