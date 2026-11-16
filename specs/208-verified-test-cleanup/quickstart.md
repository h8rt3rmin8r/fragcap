# Quickstart: Validate Test Cleanup

## Prerequisites

- Rust toolchain from `rust-toolchain.toml` and workspace dependencies.
- A writable effective temporary root. Windows validation uses a controlled redirected root and records its canonical physical location.
- No game, Npcap driver, administrator token, or real trust-store effect is required for the scoped tests.

## Focused checks

1. Run the Deep Capture session test and skills runner tests under an isolated effective temp root. Confirm that assertions can read in-test journals and that their owned fixture roots are absent after return.
2. Run the controlled cleanup-failure and panic regressions. Confirm failure is reported with an exact residual path and the original assertion is not lost.
3. Run the aggregate residue gate with successful and deliberately leaking controlled children. Confirm the successful run leaves no new owned root and the leaking run fails without touching a same-prefix unrelated directory.
4. Run the complete repository checks, including the hosted Windows and Linux CI-equivalent test command. Record any unresolved native-session evidence separately.

## Expected result

All ordinary checks pass with zero new test-owned residue. A forced failure is a failing controlled test, never a clean claim. Existing historic temp directories and native bundles are untouched by the validation.
