# Quickstart: Master JSON Schema validation

Runnable scenarios that prove the slice works end to end. Assumes a built
workspace (`cargo build`).

## Prerequisites

- The workspace builds: `cargo build`.
- The embedded schema asset exists at
  `crates/fragcap-profile/assets/target-schema.v1.json`.

## Scenario 1: a valid profile validates

```bash
cargo run -p fragcap-cli -- schema validate crates/fragcap-profile/tests/fixtures/schema/profile-valid.json
```

Expected: a single confirmation line, exit code 0.

## Scenario 2: every mistake reported at once

Given a profile with four independent faults (wrong type on `schema`, missing
`game.name`, an unknown key, a `fidelity` value outside the enum):

```bash
cargo run -p fragcap-cli -- schema validate crates/fragcap-profile/tests/fixtures/schema/profile-four-faults.json
```

Expected: four violations, each with a JSON-pointer location, in one run, exit
non-zero. Re-running yields byte-identical output (stable ordering).

## Scenario 3: a hint without fidelity is refused

```bash
cargo run -p fragcap-cli -- schema validate crates/fragcap-profile/tests/fixtures/schema/hint-no-fidelity.json
```

Expected: a refusal naming the missing `fidelity` (and `provenance` if also
absent), exit non-zero.

## Scenario 4: a database export round-trips

A `kind: export` document (a hint projection) validates with no manual
adjustment:

```bash
cargo run -p fragcap-cli -- schema validate crates/fragcap-profile/tests/fixtures/schema/export-valid.json
```

Expected: valid, exit 0. This is the contract #78 builds against.

## Scenario 5: the binary emits the schema it enforces

```bash
cargo run -p fragcap-cli -- schema print > /tmp/emitted.json
diff crates/fragcap-profile/assets/target-schema.v1.json /tmp/emitted.json
```

Expected: no diff. The emitted schema equals the embedded asset, which equals the
repository-published copy (a drift is caught by an automated check in the gate).

## Scenario 6: broken JSON is not a schema violation

```bash
cargo run -p fragcap-cli -- schema validate crates/fragcap-profile/tests/fixtures/schema/not-json.json
```

Expected: a syntax error, clearly distinguished from a schema violation, exit
non-zero.

## Gate

```bash
cargo xtask ci
cargo xtask msrv     # builds at 1.82
```

Expected: green. The fixture-corpus conformance test (valid and invalid cases per
variant) passes, binding the published schema to the hand-rolled validator.
