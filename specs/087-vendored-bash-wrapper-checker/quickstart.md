# Quickstart: Vendored Bash Wrapper Checker

## Baseline Measurements

Run the vendored checker directly against every gated Bash script:

```powershell
bash .agents/skills/shruggie-bash/scripts/test-script-compliance.sh scripts/fragcap.sh
bash .agents/skills/shruggie-bash/scripts/test-script-compliance.sh scripts/lint-docs.sh
bash .agents/skills/shruggie-bash/scripts/test-script-compliance.sh scripts/cut-release.sh
```

Expected outcome:

- Each script passes the structural checks.
- ShellCheck must run for the result to count as a complete S087 validation.

## Focused Validation

Run:

```powershell
cargo test -p xtask wrappers
cargo xtask wrappers
```

Expected outcome:

- xtask tests pass.
- `cargo xtask wrappers` reports vendored Bash checker results for `fragcap.sh`, `lint-docs.sh`, and `cut-release.sh`.
- The wrapper gate exits 2 if ShellCheck is not runnable from Bash.

## Repository Gate

Run the full repository gate:

```powershell
cargo xtask ci
```

Expected outcome:

- Formatting, clippy, tests, repository lint, dependency direction, licenses, wrappers, skills, docs, and spec checks pass.
