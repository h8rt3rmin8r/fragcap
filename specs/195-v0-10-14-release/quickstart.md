# Quickstart: Validate S195

## Prerequisites

- Rust 1.88 toolchain and the repository's locked dependencies.
- PowerShell 7 for Windows release tooling.
- No game, installed public package, capture driver, elevated trust mutation, or private title is required.

## Focused defect validation

```powershell
cargo test -p fragcap-cli --test cli_calibrate steam_client_setup_reports_completion_then_reaches_fresh_calibration -- --exact
cargo test -p fragcap-cli owned_platform_capture_inherits_the_finite_session_launch_deadline -- --exact
```

The setup-to-calibration test uses a finite seven-second controlled launch allowance and a one-second observation duration. The deadline test continues to preserve an exact shorter operator value and cap a longer value at the session deadline.

## Candidate validation

```powershell
cargo xtask ci
cargo xtask msrv
cargo xtask neutral
cargo xtask docs build
cargo xtask notes 0.10.14
cargo xtask supply-chain
cargo deny --locked --all-features check advisories licenses bans sources
```

Run the repository's isolated locked-graph audit for every non-workspace lockfile and verify `git diff --check`, UTF-8 without BOM, LF endings, and no mojibake.

## Hosted validation

The official PR must reach terminal success on all applicable checks. Because ordinary PR packaging may use GitHub's temporary merge commit, record a successful package-certification run whose source is the exact final branch head before handoff.

Installed-product and private-title observations occur only after publication. Any concrete failure becomes a version-specific GitHub issue.
