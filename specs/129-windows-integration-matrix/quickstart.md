# Quickstart: Native Windows Integration Matrix

## Static Authority

```text
cargo xtask windows-integration
```

Expected: schema, closed row identities, source references, completion-domain coverage, workflow contract, and committed physical evidence metadata pass.

## Build and Stage the Production Binary

Acquire the Npcap SDK only as a temporary build input, set `LIB` to its x64 import-library directory, then run:

```text
cargo build -p fragcap-cli --release --features live,socket-table,etw --locked
```

Copy `target/release/fragcap.exe` into an ignored scratch install layout. Do not place the SDK, Npcap files, raw reports, or capture bundles in that layout.

## Hosted Tier

```text
cargo xtask windows-integration --run hosted --binary target/windows-integration/stage/fragcap.exe --report target/windows-integration/hosted.jsonl
cargo xtask windows-integration --validate-report target/windows-integration/hosted.jsonl
```

Expected: every hosted row executes exactly once, no row skips, the staged binary identity matches the report, all scratch effects reconcile, and the derived summary passes publication hygiene.

## Physical Tier

Run only on an authorized Windows test host after reviewing the registry. The runner neither installs prerequisites nor elevates itself.

```text
cargo xtask windows-integration --run physical --binary target/windows-integration/stage/fragcap.exe --report target/windows-integration/physical.jsonl
cargo xtask windows-integration --validate-report target/windows-integration/physical.jsonl
```

Expected: the preflight matches the physical tier, Npcap-present and analyzer rows exercise the installed prerequisites, non-admin and denied-elevation behavior remains exact, current-user test trust is restored, and no undeclared residue remains.

## Release Authority

```text
cargo xtask windows-integration --release
```

Expected: static, hosted, and current physical evidence authorities match the registry and product revision. This does not certify final MSI or archive packaging, which remains issue #329.

## Full Repository Gate

```text
cargo xtask ci
```

Expected: format, Clippy, locked tests, conformance, threat model, fuzz, failure, performance, Windows integration static authority, documentation, wrappers, licensing, dependency direction, and specification checks pass.
