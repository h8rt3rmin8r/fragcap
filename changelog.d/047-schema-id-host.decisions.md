**2026-08-14** Corrected the master schema's `$id` host from `fragcap.dev` to
`fragcap.com` (`https://fragcap.com/schema/target/v1.json`), in the published copy
`docs/schema/target-schema.v1.json`, the byte-identical embedded asset
`crates/fragcap-profile/assets/target-schema.v1.json`, the CLI test that asserts
the exact string (`crates/fragcap-cli/tests/cli_schema.rs`), and the identity
contract example (`specs/025-master-json-schema/contracts/master-schema.contract.md`).
Resolves issue #117.

This is recorded as a deliberate decision because the S025 identity contract states
the `$id` host is fixed at authoring time and never changed for a published version.
The change is made anyway, and it is safe: `fragcap.dev` was never a domain the
project owned or served, `fragcap.com` is the project's real registered domain (the
docs site), and the `$id` is an opaque stable identifier that nothing in the project
dereferences over the network, so this is an identifier correction rather than a
hosting change (no schema route is served at that URL, and none is added here).
Schema version 1 is embedded in the binary and not published to any schema registry,
so overwriting the v1 identity before 1.0 breaks no external consumer. The two schema
copies are edited identically and remain byte-identical, enforced by the drift test
in `crates/fragcap-profile/tests/schema_conformance.rs`.
