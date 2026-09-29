# Quickstart: S161 Interactive CLI Input Ownership

## Preconditions

Use the S161 branch and the repository's pinned Rust toolchain. Tests must run headlessly; they must not launch a visible Windows console or use real game, trust, proxy, or capture effects.

## Focused verification

1. Run `cargo test -p fragcap-cli --lib --locked`, including `authorization_owner_does_not_hold_stdin_lock_between_reads`.
2. Confirm the former command-wide stdin lock is absent and the bounded authorization reader still rejects incomplete, extra, and wrong-plan responses.
3. Exercise controlled Doctor cases: negative line reports `skipped` with no performer call; positive line calls only its named performer; prompt I/O failure stops and reports an error with no later action.
4. Exercise controlled target registration and warm-restart answers, including no-effect failure paths.
5. Run `cargo xtask ci` in the foreground and read every gate result. Set `CARGO_BUILD_JOBS=1` if this host runs out of memory under parallel compilation.

## Operator-facing interpretation

The original `fragcap doctor --fix` prompt should accept a completed `n` line and continue without cleanup. The confirmed code correction covers the CLI prompt. Print Screen behavior remains a separate observation unless a focused elevated-window comparison identifies a fragcap-specific difference.

## Completion evidence

Record commands, exits, test counts, and any unavailable host comparison in the PR. Do not describe an unperformed installed-binary or Print Screen field check as passing.
