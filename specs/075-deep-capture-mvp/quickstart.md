# Quickstart: Deep Capture MVP

This quickstart describes the intended maintainer verification flow for #219. Commands use placeholder targets only.

## Controlled target

Run the deterministic controlled target path first:

```powershell
cargo test -p fragcap-cli deep_capture_controlled_target -- --nocapture
```

Expected result:

- the test starts a synthetic target;
- the proxy path receives HTTP and HTTPS traffic;
- a bundle is written under a scratch directory;
- the manifest validates;
- compatibility facts are written to an in-memory or scratch local store;
- cleanup reports every session resource.

## CLI refusal checks

Run command contract tests:

```powershell
cargo test -p fragcap-cli cli_deep_capture --quiet
```

Expected result:

- raw process input is refused;
- missing `--launch` is refused;
- missing proxy backend is refused;
- unknown scoped proxy compatibility is refused;
- missing trust confirmation is refused.

## Optional local backend demonstration

When `mitmdump` is installed and available on PATH, run the ignored backend demonstration:

```powershell
cargo test -p fragcap-cli deep_capture_mitmdump_demo -- --ignored --nocapture
```

The test must skip with a clear message if `mitmdump` is unavailable. It must not use real game titles, real accounts, remote services, or local install paths.

## Full gate

Before opening the PR, run:

```powershell
cargo fmt --check
git diff --check
cargo test -p fragcap-cli --quiet
cargo test -p fragcap-targets --quiet
cargo test --workspace --quiet
cargo xtask lint
cargo xtask deps
cargo xtask spec
cargo xtask changelog --check
```

Then scan committed changes for local data:

```powershell
rg -n "C:\\\\|A:\\\\|Users\\\\|steamapps|token|account|password|email|@|https?://" specs crates docs changelog.d
```

Any match must be reviewed. Placeholder loopback URLs in controlled fixtures are allowed; real local paths, real title names from fact-finding, account identifiers, tokens, and third-party endpoints are not.
