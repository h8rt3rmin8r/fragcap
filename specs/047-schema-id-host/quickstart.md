# Quickstart / Verification Guide: schema $id host

## Mechanical gate (CI parity)

```bash
cargo xtask ci
```

Includes the two schema drift tests in
`crates/fragcap-profile/tests/schema_conformance.rs` (the embedded asset equals
both the emitted document and the published `docs/schema` copy) and the CLI
schema test `crates/fragcap-cli/tests/cli_schema.rs` (the printed `$id` string).
A missed location, a non-identical edit, or a stale assertion fails here.

## Targeted checks

```bash
cargo test -p fragcap-profile schema
cargo test -p fragcap-cli --test cli_schema
```

Expected: the drift and print tests pass with the `fragcap.com` `$id`.

## Live-identifier check (no fragcap.dev as an identifier)

```bash
grep -rn "fragcap\.dev" docs/schema crates/fragcap-profile/assets crates/fragcap-cli/tests/cli_schema.rs
```

Expected: no matches. The `$id`, the embedded asset, and the CLI assertion carry
only `fragcap.com`. The identity contract's canonical example line is also
`fragcap.com`; the repo still contains `fragcap.dev` in deliberate historical
references (this slice's decision fragment, the contract's dated correction note,
and the spec artifacts), which FR-005/SC-001 permit, so a whole-repo grep is
expected to match those and is not the check.

## Byte-identity spot check

```bash
diff docs/schema/target-schema.v1.json crates/fragcap-profile/assets/target-schema.v1.json
```

Expected: no differences (the two copies are identical).

## Done signal

`cargo xtask ci` green, the scoped live-identifier grep returns nothing, and the
two JSON files diff clean.
