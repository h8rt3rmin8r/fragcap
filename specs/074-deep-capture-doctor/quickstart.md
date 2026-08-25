# Quickstart: Deep Capture doctor readiness and cleanup

## Manual Checks

1. Run `fragcap doctor`.
2. Confirm the report contains a `Deep Capture` section.
3. Confirm ordinary `doctor` does not create session storage or modify trust.
4. Place a test-only `tls-keylog.log` under the configured `FRAGCAP_SESSION_DIR`.
5. Run `fragcap doctor` and confirm the key log is reported as stale residue.
6. Run `fragcap doctor --fix` in an interactive terminal and confirm cleanup is offered before deletion.

## Automated Checks

```powershell
cargo fmt --check
git diff --check
cargo test -p fragcap-cli --quiet
cargo test --workspace --quiet
cargo xtask lint
cargo xtask deps
cargo xtask spec
cargo xtask changelog --check
```
