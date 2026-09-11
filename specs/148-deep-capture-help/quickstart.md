# Quickstart: Deep Capture Embedded Workflow Help

## Scope

This quickstart validates S148 without launching a real game, contacting a game service, mutating the real trust store, or capturing sensitive live traffic.

## Focused help contract

```powershell
cargo test -p fragcap-cli --test cli_help --locked
```

Expected result: every audited short and long help page renders, ordered workflow concepts remain present, narrow-width semantics remain readable, and every displayed command example parses.

## Focused refusal guidance

```powershell
cargo test -p fragcap-cli --test cli_deep_capture --locked
cargo test -p fragcap-cli --test cli_calibrate --locked
cargo test -p fragcap-cli --test cli_bundle --locked
cargo test -p fragcap-cli --test cli_doctor --locked
```

Expected result: the closed pre-session refusal categories retain truthful reasons and finish with registry-backed next commands without changing structured output or applying live effects.

## Complete repository gate

```powershell
cargo xtask ci
```

Expected result: formatting, linting, workspace tests, help contracts, documentation rules, architecture checks, and controlled native authorities all pass.

## Explicit non-result

These commands do not demonstrate compatibility with a real game and do not complete issue #334. Any future real-game validation remains operator-owned and restricted to published release bytes.
