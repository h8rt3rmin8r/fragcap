# Quickstart: Verify Public Entry Point Reconciliation

## Audit Retired Claims

Search only the in-scope public surfaces and current-status master-spec prose for retired claims:

```powershell
rg -n -i "planned Deep Capture|Deep Capture is planned|not present in released binaries|pre-implementation|There is no Cargo workspace|v0\.5\.0 is in progress|None of the following has shipped" README.md CONTRIBUTING.md site/content/docs/index.mdx site/content/docs/contributing.mdx .github/ISSUE_TEMPLATE docs/fragcap-specification.md
```

Historical revision rows may describe what was planned at that revision. No present-tense match may remain.

## Verify Current Command Examples

Compare the issue forms and README examples with current help:

```powershell
cargo run -q -p fragcap-cli -- --help
cargo run -q -p fragcap-cli -- capture --help
cargo run -q -p fragcap-cli -- doctor --help
```

The bug form must not contain `fragcap run --profile`, and the feature form must not point readers to work "beyond v0.2.0".

## Parse Issue Forms

Use the available YAML parser as a focused structural check:

```powershell
python -c "import pathlib, yaml; [yaml.safe_load(p.read_text(encoding='utf-8')) for p in pathlib.Path('.github/ISSUE_TEMPLATE').glob('*.yml')]"
```

## Verify Repository Metadata

```powershell
gh repo view --json description --jq '.description'
```

Expected:

```text
Passive process-attributed Capture and explicit, target-scoped Deep Capture for Windows game traffic.
```

## Run Documentation And Specification Gates

```powershell
cargo xtask docs check
cargo xtask docs build
cargo xtask spec
```

## Run The Full Repository Gate

```powershell
cargo fmt --all -- --check
cargo xtask ci
```

Review `git diff --check`, changed-file punctuation, UTF-8 decoding, and the final changed-file inventory. Confirm the diff contains no runtime source, dependency, workflow, toolchain, release-configuration, or issue #245 through #249 page changes.
