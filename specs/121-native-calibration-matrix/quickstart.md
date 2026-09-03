# Quickstart: Complete Native Calibration Matrix

## Preconditions

- Work from `codex/121-native-calibration-matrix`.
- `.specify/feature.json` points to `specs/121-native-calibration-matrix` and remains unstaged.
- Use only the committed controlled target and loopback origins for automated calibration verification.

## Focused verification

```powershell
cargo test -p fragcap-targets compatibility
cargo test -p fragcap --test deep_capture_session
cargo test -p fragcap --test native_conformance
cargo test -p fragcap-cli cli_deep_capture
```

Expected results:

- Schema version 9 stores migrate to version 10 without changing existing rows.
- New rows round-trip the complete case dimensions.
- Single-dimension mismatches never authorize an exact prepared case.
- Selected protocol facts derive only from matching S120 classifications.
- IPv4 and IPv6 controlled cases remain separate.
- Plans, events, artifacts, target detail, and terminal output agree.

## Full verification

```powershell
cargo xtask ci
```

Expected result: all repository gates pass, including formatting, dependency policy, specification and documentation checks, shell compliance, unit and integration tests, and native conformance.

## Review boundary

Review the final diff for issue #317 only. Confirm append-only migration, no inferred legacy defaults, exact current-case eligibility, no uncounted loss, no target process access, no target key extraction, no pinning bypass, no global proxy mutation, no #318 bypass implementation, no dependency or lockfile package, no Unicode dash punctuation, no BOM, no mojibake, and no staged `.specify/feature.json`.
