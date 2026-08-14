# Quickstart / Validation: Targets Hint Database (foundation)

How to prove this slice works end to end. All steps are offline. Commands run
from the repository root on Windows.

## Prerequisites

- The `1.82` toolchain installed (for the MSRV gate): `rustup toolchain list`
  shows `1.82-x86_64-pc-windows-msvc`.
- A C toolchain reachable by `cc` (the MSVC build tools) so the bundled SQLite
  compiles. The active `1.96` toolchain is `x86_64-pc-windows-msvc`, which
  supplies it.

## 1. Library round-trip (the MVP, User Story 1)

Run the crate's tests, which open an in-memory store, insert games across the
three tiers (including a launcher-mediated title, a title with an engine, and a
Tier-1-only title with neither), export, and assert the JSON validates with zero
diagnostics:

```bash
cargo test -p fragcap-targets
```

Expected: green. The export test asserts
`fragcap_profile::jsonschema::validate_value` returns no diagnostics for the
produced document, and asserts the record shapes (engine omitted when unknown,
launch array carried whole, fidelity always `heuristic-unverified`).

## 2. Schema-conformance fixtures (User Story 1, honesty checks)

A conformance test drives fixtures under the crate's `tests/fixtures/`:

```bash
cargo test -p fragcap-targets conformance
```

Expected: a well-formed export validates; a malformed one (an out-of-set
`engine.source`, and a launch entry missing its `executable`) is rejected with the
expected `SchemaCode`. This proves the store can never emit a document the
published schema rejects.

## 3. CLI import/export round-trip (User Story 2)

Build the CLI with the `targets` feature and round-trip the committed seed
fixture through a temporary store, with no network:

```bash
cargo run -p fragcap-cli --features targets -- targets import crates/fragcap-targets/tests/fixtures/seed.json --db %TEMP%\hint.db
cargo run -p fragcap-cli --features targets -- targets export --db %TEMP%\hint.db
```

Expected: the first command creates and populates the store; the second prints a
`kind: "export"` document to stdout that validates against the schema and reflects
the seeded titles (the launcher-mediated title carries `launcher_mediated: true`,
the engine title carries an `engine` object, the Tier-1-only title carries
neither). Re-running the import is idempotent (same appids replaced, not
duplicated).

A malformed-seed check (a launch entry missing its executable) exits non-zero and
leaves no store behind.

## 4. Feature gate: default build skips SQLite (Success Criterion SC-005)

Confirm a default build of the facade compiles neither the database engine nor its
C build:

```bash
cargo build -p fragcap
```

Expected: green, and `rusqlite`/`libsqlite3-sys` do not compile (the `targets`
feature is off by default at the facade). Building with the feature does compile
them:

```bash
cargo build -p fragcap --features targets
```

## 5. Full gate set (the slice's Done gate)

```bash
cargo xtask ci
cargo xtask msrv
```

Expected: `cargo xtask ci` passes (fmt, clippy `-D warnings` with all features,
`test --workspace --locked`, `xtask lint`, `xtask deps` accepting the new crate
edges, `xtask license` finding the new crate's LICENSE/NOTICE/README). `cargo
xtask msrv` builds the workspace (including `fragcap-targets` and the bundled
SQLite) through `rustup run 1.82` and exits 0. A `2` from `msrv` means the
toolchain was not found and is NOT a pass.
