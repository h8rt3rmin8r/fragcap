# Quickstart / Validation Guide: Hint-Record Schema Revision

Validates the slice offline against the conformance corpus. Implementation lives
in `tasks.md`.

## Prerequisites

- The workspace builds (`cargo build --workspace`).

## 1. The revised subschema validates a full hint

```sh
cargo test -p fragcap-profile --test schema_conformance
```

Expected: `hint-loose-valid.json` (a hint with a multi-entry launch array,
`launcher_mediated`, and a valid engine object) validates with no diagnostics.

## 2. Bad engine enums and a missing executable are rejected

Expected (corpus): `engine-bad-source.json` is rejected with `InvalidEngineSource`;
`engine-bad-confidence.json` with `InvalidEngineConfidence`; `launch-no-executable.json`
with `MissingField`.

## 3. The strict format stays clean

Expected (corpus): `profile-with-launch.json` (a profile carrying `launch`) is
rejected as an `UnknownKey`; every pre-existing fixture (profile/package/hint/export)
keeps its outcome.

## 4. The two schema copies stay byte-identical

```sh
cargo test -p fragcap-profile --test schema_conformance the_embedded_schema_matches_the_published_repository_copy
```

Expected: the embedded and `docs/schema` copies match (no version bump, additive
change applied to both).

## 5. Vocabularies are independent

Expected (corpus): a hint with `fidelity: heuristic-unverified` and an independent
`engine.confidence: low` validates; the two are separate fields.

## 6. Full gate

```sh
cargo xtask ci
cargo xtask msrv
```

Expected: fmt, clippy, tests, lint, deps, license, docs all pass; MSRV 1.82 green;
no new dependency.
