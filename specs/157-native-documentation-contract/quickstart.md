# Quickstart: verify S157 documentation without product dispatch

## Prerequisites

- Repository checkout on the S157 branch.
- Pinned Rust and site toolchains already used by the repository.
- No installed fragcap execution, live capture, game launch, elevated session, or trust-store mutation.

## 1. Validate the specification artifacts

Run the Spec Kit prerequisite and analysis flow against `specs/157-native-documentation-contract`. Requirements and documentation checklists must have zero incomplete items before implementation.

## 2. Run the documentation contract tests

```powershell
cargo test -p xtask docs_coverage -- --nocapture
cargo test -p fragcap-cli --test cli_reference
cargo test -p fragcap-cli --test cli_reference --features net
cargo xtask docs check
```

Expected result: the version-two registry has all thirteen topics, required sections and example authorities; every current fragcap command parses without dispatch under applicable feature trees; documentation lint passes.

## 3. Validate artifact and API authorities

```powershell
cargo test -p fragcap --test native_conformance --features deep-capture published_manifest_examples_use_the_versioned_product_reader
cargo test -p fragcap --test goldens
cargo test -p fragcap --test native_conformance --features deep-capture
cargo test -p fragcap --test public_api --features deep-capture
```

Expected result: manifest specimens, packet goldens, complete controlled bundle authorities, and the stable API example remain owned by actual readers or exact controlled contracts.

## 4. Build and inspect the production documentation site

Use the repository's existing hidden non-interactive site launcher through the xtask documentation command. Run the site unit suite and production accessibility suite after static export.

Expected result: every route exports, internal links and anchors resolve, current contract search queries lead to the intended pages, accessibility checks pass at supported widths, and no test is unexpectedly skipped.

## 5. Run full CI parity

```powershell
cargo xtask ci
```

Expected result: formatting, lint, workspace tests, documentation, parser, site-independent gates, specification currency, security registries, examples, and repository conventions all pass. No step dispatches the installed sensitive product.

## 6. Inspect status truth

Search current README and nonhistorical site content for stale S154 candidate language and obsolete external-backend instructions. Confirm published v0.10.1 remains distinct from merged S154 through S157 documentation and verification work, and that #333, #413, #331, #334, and #278 remain outstanding.
