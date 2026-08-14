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

## Zero-fragcap.dev check

```bash
grep -rn "fragcap.dev" .
```

Expected: no matches anywhere in the repository.

## Byte-identity spot check

```bash
diff docs/schema/target-schema.v1.json crates/fragcap-profile/assets/target-schema.v1.json
```

Expected: no differences (the two copies are identical).

## Done signal

`cargo xtask ci` green, the grep returns nothing, and the two JSON files diff
clean.
