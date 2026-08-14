# Phase 0 Research: correct the schema $id host

Established against current source before editing.

## The four locations (whole-repo sweep)

`grep -rn "fragcap.dev"` returns exactly four hits, all the same URL:

- `docs/schema/target-schema.v1.json:3` - the published `$id`.
- `crates/fragcap-profile/assets/target-schema.v1.json:3` - the embedded asset,
  byte-identical to the published copy.
- `crates/fragcap-cli/tests/cli_schema.rs:79` - asserts
  `out.contains("\"$id\": \"https://fragcap.dev/schema/target/v1.json\"")`.
- `specs/025-master-json-schema/contracts/master-schema.contract.md:12` - the
  identity example, in a bullet stating the host is "fixed at authoring time and
  never changed for a published version."

No other reference exists (no site route serves the schema; docs prose links the
file on GitHub, not the `$id`).

## Byte-identity is enforced by tests

`crates/fragcap-profile/tests/schema_conformance.rs` carries two guards:

- `print_output_equals_the_embedded_asset`: `schema_document()` equals the asset
  file (`schema_document()` returns `SCHEMA_JSON = include_str!(assets/...)`).
- `the_embedded_schema_matches_the_published_repository_copy`: the embedded schema
  equals `docs/schema/target-schema.v1.json`.

So the asset and the published copy must be edited identically. `fragcap schema
print` emits `schema_document()` (the embedded asset), which is why
`cli_schema.rs` sees the new `$id` once the asset changes; its assertion string
must be updated in the same commit or the CLI test goes red.

**Decision**: Change `fragcap.dev` -> `fragcap.com` in the two JSON files
identically, update the `cli_schema.rs` assertion string, update the contract
example, and annotate the contract's immutability note with a dated line pointing
at the decision fragment. Record the change in
`changelog.d/047-schema-id-host.decisions.md`.

## Why fragcap.com, and why a recorded decision

`fragcap.com` is the project's real, owned domain (the docs site;
`site/app/layout.tsx` sets `const url = 'https://fragcap.com'`, and the site emits
a `CNAME` of `fragcap.com`). `fragcap.dev` is owned by no one here. The `$id` is
an opaque identifier; nothing dereferences it, so this is a string correction,
not a hosting change. Because the S025 contract declared the host immutable for a
published version, the change is recorded as a deliberate pre-1.0 correction
(v1 is embedded-only, not registry-published), so the governance record is honest.

## Pinned-artifact check

`docs/schema/**` is not on the pinned-artifact list
(`.github/workflows/**`, `rust-toolchain.toml`, `release.toml`, `scripts/**`,
release docs). No pinned-file gate applies; the decision fragment exists for the
contract's immutability statement, not because the file is mechanically pinned.
