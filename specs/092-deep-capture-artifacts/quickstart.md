# Quickstart: Validate the Deep Capture Bundle Reference

## Prerequisites

- Work from `codex/092-deep-capture-artifacts` with `.specify/feature.json` pointing to `specs/092-deep-capture-artifacts`.
- Use the repository-pinned Rust and documentation toolchains.
- Do not run a live capture, start a proxy, trust a certificate, or use local game data. This slice is validated from source contracts and synthetic examples.

## 1. Check the Corrected Output Boundary

Read `site/content/docs/reference/output-formats.mdx` from the top. Confirm that ordinary Capture pcapng and packet JSON Lines appear before a separate Deep Capture session-bundle section, and that no sentence claims all outputs carry the same facts.

## 2. Audit the Artifact Matrix

Confirm the matrix names all nine roles from [data-model.md](data-model.md). For each row, compare path, authority, sensitivity, required flag, lifetime, and omission condition with `manifest_json` and `artifact` in `crates/fragcap-cli/src/commands/deep_capture.rs`.

Pay special attention to two potentially misleading facts:

- `capture.fcapng` has the emitted sensitivity label `ordinary`, but packet payloads can still be sensitive and the file is not automatically safe to share.
- `compatibility.json` records scrubbed run context before local facts are written, so it does not prove that the later local-store write succeeded.

## 3. Audit States, Omissions, and Correlation

Confirm the page defines `complete`, `partial`, and `failed` from the current session-state branch. Confirm it lists only the omission tokens emitted by the current manifest writer: `writer-failed`, `no-http-semantics`, `not-requested`, and `not-produced`.

Confirm missing `flow_id`, process fields, or sidecar fields are described as unavailable joins rather than evidence that corresponding activity did not occur.

## 4. Audit Sensitive-Artifact Handling

Confirm the TLS key-log section describes final-path creation before proxy traffic, incremental live analyzer use, removal of an empty placeholder, and nonempty-only retention. Confirm it calls the material proxy-owned and never target-extracted.

Compare the later cleanup guidance with `deep_capture_cleanup_candidates` in `crates/fragcap-cli/src/doctor/fix.rs`. It must describe selective, confirmation-gated removal of known sensitive sidecars and unfinished manifests under fragcap-owned session storage, not automatic deletion of every completed bundle file.

## 5. Check Cross-Links and Synthetic Data

Confirm both the CLI and Deep Capture compatibility pages link to `/docs/reference/output-formats`. Inspect the manifest example for placeholders only, with relative paths and no usable key-log material.

## 6. Run the Documentation Gates

```powershell
cargo xtask docs check
cargo xtask docs build
```

Expected result: the documentation checker and production static export finish successfully.

## 7. Run Repository Parity

```powershell
cargo fmt --all -- --check
cargo xtask lint
cargo xtask ci
```

Expected result: formatting, lint, tests, dependency checks, license checks, wrapper checks, skill checks, documentation checks, and specification checks all pass.

## 8. Review Scope

Confirm the diff contains only the output reference, two narrow inbound-link edits, one changelog fragment, and S092 artifacts. `.specify/feature.json` remains local and unstaged. No runtime, dependency, workflow, toolchain, release, or master-specification file changes.
