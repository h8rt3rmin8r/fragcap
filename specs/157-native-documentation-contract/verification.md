# Verification: S157 complete native documentation contract

## Scope and safety

- Verified repository documentation, committed fixtures, controlled loopback tests, and static-site output only.
- Did not execute the installed product, launch a real game, perform live packet capture, mutate a certificate store, or publish repository state.
- Preserved open external completion authority for #333, #413, #331, #334, and #278.

## Test-first evidence

Before the version-two registry existed, the focused documentation-coverage run failed because `docs/audits/native-documentation-coverage.v2.json` was absent. The seven mutation and parser tests already present passed, while the committed-inventory test failed at the intended new authority boundary. After the version-two registry, current documentation, and review mutations were implemented, all nine focused tests passed.

## Focused verification

The following commands completed successfully on 2026-09-17:

```text
cargo test -p xtask docs_coverage -- --nocapture
cargo xtask docs check
cargo test -p fragcap-cli --test cli_reference
cargo test -p fragcap-cli --test cli_reference --features net
cargo test -p fragcap --test native_conformance --features deep-capture published_manifest_examples_use_the_versioned_product_reader
cargo test -p fragcap --test goldens
cargo test -p fragcap --test native_conformance --features deep-capture
cargo test -p fragcap --test public_api --features deep-capture
pnpm dlx pnpm@10.26.0 --dir site install --frozen-lockfile --force
cargo xtask docs build
pnpm --dir site test:unit
pnpm --dir site test:accessibility
git diff --check
cargo xtask spec
```

Results:

- Documentation coverage: 9 passed.
- Default CLI command corpus: 9 passed.
- Network-capable CLI command corpus: 9 passed.
- Manifest specimen reader: 1 focused public-reader test passed; the complete native conformance suite passed 4 tests.
- Packet goldens: 7 passed.
- Stable public API: 9 passed.
- Pinned pnpm install: 491 packages installed from the frozen lockfile; esbuild was the only dependency lifecycle script executed, followed by the site package's own postinstall.
- Static site: 75 pages exported successfully.
- Site unit suite: 4 passed.
- Production accessibility suite: 14 passed with zero skipped.
- Formatting, specification impact, and documentation checks passed.

An exploratory `--all-features` CLI build selected the optional live Npcap backend and could not link because this workstation has no `wpcap.lib` SDK. That feature combination is outside the documentation command-corpus contract. Verification was corrected to the repository-owned default and `net` configurations, both of which passed. No product requirement or gate was weakened.

## Full repository gate

```text
cargo xtask ci
```

Result: passed with `ci: all checks passed`. This included format, Clippy, workspace tests, lint, dependency policy, license, supply-chain, package certification, wrappers, vendored skill integrity, documentation coverage, specification impact, calibration acceptance, threat model, review handoff, immutable review candidate, fuzz inventory, failure matrix, native protocol conformance, performance authority, and Windows integration authority.

The first hosted pull-request documentation job rejected the new pnpm workspace policy because pnpm requires a nonempty `packages` field. Adding the site root as the sole workspace package made the policy structurally valid. Second-round review then found that pinned pnpm 9 ignored `allowBuilds`, which was added in pnpm 10.26.0. The site manifest and hosted workflow now pin 10.26.0 together, and that exact toolchain regenerated the frozen lockfile byte-identically while enforcing the esbuild-only policy.

The first Codex review identified that obsolete-backend language was checked only on topic-owned pages. The correction enumerates every nonhistorical site MDX page independently from topic ownership and adds a mutation check proving an otherwise unowned page cannot reintroduce an external-backend instruction. The second and final review also found that the heading parser recognized only backtick fences. The parser now recognizes both Markdown fence markers and closes only on the opening marker at an equal or greater width; mutations cover shorter and mismatched apparent closers.

## Architecture decision

The shared Cargo test discovery intentionally recognizes supported integration-test authorities. Rather than weakening it to infer nested library unit tests, S157 added a public integration test that reads all committed manifest examples through `ManifestDocument`. The site build now sets noninteractive pnpm environment values and permits only the exact pinned `esbuild` install script required by the static-site toolchain.

## Outcome

The version-two registry closes thirteen current documentation topics, exact ordered page sections, supported executable authorities, five example-authority classes, and the published-versus-current completion boundary. Current reader entry points now converge on the stable Native product contract while continuing to state that independent review and final product acceptance remain outstanding.
