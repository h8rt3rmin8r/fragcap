# Quickstart: Deep Capture MVP

This quickstart describes the intended maintainer verification flow for #219. Commands use placeholder targets only.

## Controlled target

Run the deterministic controlled target path first:

```powershell
cargo test -p fragcap-cli --test cli_deep_capture controlled_deep_capture_writes_a_bundle_and_compatibility_facts -- --nocapture
```

Expected result:

- the test launches a placeholder child process;
- a live deterministic loopback adapter receives HTTP-like, CONNECT/HTTPS, metadata-only, and unsupported requests;
- a bundle is written under a scratch directory;
- the manifest validates;
- compatibility facts are written to an in-memory or scratch local store;
- cleanup reports every session resource.

Run the partial-session case as well:

```powershell
cargo test -p fragcap-cli --test cli_deep_capture partial_controlled_session_writes_observed_facts_and_manifest -- --nocapture
```

It proves that an observed prefix survives a target failure, the manifest and
application trailer report `partial`, packet truth remains declared, and only
observed compatibility facts are written.

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

The test must skip with a clear message if `mitmdump` is unavailable. With the backend running, it verifies that the requested empty key-log file already exists at its final bundle path before shutdown or finalization. It must not use real game titles, real accounts, remote services, or local install paths.

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
