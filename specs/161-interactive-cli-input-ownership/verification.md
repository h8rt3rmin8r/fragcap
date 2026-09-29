# Verification: S161 Interactive CLI Input Ownership

## Regression and prompt behavior

- Before the production input-owner change, `cargo test -p fragcap-cli authorization_owner_does_not_hold_stdin_lock_between_reads --lib --locked` failed with `dispatch must not retain the process-global stdin lock`.
- After the change, `cargo test -p fragcap-cli --lib --locked` passed 300 tests, including the input-owner, Doctor, target registration, and warm-restart regressions.
- A controlled `cargo run -p fragcap-cli --bin fragcap -- doctor --fix` through the integrated headless terminal offered an elevation action. Sending `n` and Enter echoed `n`, reported `skipped`, and proceeded to recheck. Its exit code was 1 because readiness checks remained unsatisfied. No action was confirmed.

## Repository gate

- The first `cargo xtask ci` attempt encountered host memory exhaustion during parallel Rust test compilation after formatting and Clippy passed. The resulting missing-crate diagnostics were compilation fallout.
- Repeating the complete foreground gate with `CARGO_BUILD_JOBS=1` passed (`ci: all checks passed`).

## Field limit

The focus-dependent Print Screen symptom was not reproduced or resolved in this headless run. No claim is made that S161 changes Windows screenshot hotkey behavior.
