# Contracts: Targets Hint Database (foundation)

Two contract surfaces: the `fragcap targets` CLI, and the `fragcap-targets`
library. The JSON contract is the published schema
(`docs/schema/target-schema.v1.json`), `kind: "export"`; it is not restated here,
only referenced. See data-model.md for the column-to-JSON mapping.

## CLI contract

The subcommand is present only when the binary is built with the `targets`
feature (the shipped CLI enables it). All paths are local; no network access.

### `fragcap targets import <SEED> --db <DB>`

- `<SEED>`: path to a local JSON seed document (the `export` shape the loader
  understands).
- `--db <DB>`: path to the SQLite store file; created if absent, opened and
  migrated if present.
- Behavior: loads the seed transactionally. A duplicate appid within the seed is
  an error that rolls back the whole import. An appid already present in the store
  is replaced wholesale. A record the schema/store rules reject (bad engine
  source or confidence, launch entry missing its executable) fails the import with
  a diagnostic and leaves the store unchanged.
- Exit codes: `0` success; `1` operational failure (unreadable seed, malformed
  record, store I/O error) with a diagnostic to stderr; `2` usage error (missing
  argument, unknown flag). Consistent with the CLI's existing 0/1/2 contract.

### `fragcap targets export --db <DB>`

- `--db <DB>`: path to an existing SQLite store.
- Behavior: projects every game in the store into a single `kind: "export"`
  document and writes it to stdout (pretty-printed, trailing newline). The
  document is validated against the embedded schema before it is written; the
  command never emits a document the validator rejects.
- Exit codes: `0` success; `1` operational failure (store missing/unreadable,
  or, as an internal-error guard, an export that failed self-validation); `2`
  usage error.
- Empty store: exports a valid envelope with an empty `records` array.

## Library contract (`fragcap-targets`)

Stable surface the CLI and later seeding slices build on. Exact names settle in
implementation; the contract is the behavior.

- `Store::open(path) -> Result<Store, TargetsError>`: open or create the file,
  enable foreign keys, apply/verify migration version 1. Errors on a
  newer-than-known `user_version`.
- `Store::open_in_memory() -> Result<Store, TargetsError>`: for tests.
- `Store::upsert_game(&Game) -> Result<(), TargetsError>`: insert or wholesale
  replace one game and its launch/technology rows, transactionally. Rejects an
  out-of-set engine value or an empty launch executable before writing.
- `Store::games(&self) -> Result<Vec<Game>, TargetsError>`: read all games with
  their launch entries (ordered) and technologies.
- `Store::seed_state(tier) / set_seed_state(...)`: read/write per-tier resume
  state (structural; no fetch writes it this slice).
- `export(&Store) -> Result<String, TargetsError>`: build the `export` document,
  validate it via `fragcap_profile::jsonschema::validate_value`, return the
  pretty-printed JSON. A self-validation failure is an internal error surfaced,
  never a silently emitted document.
- `import(&mut Store, seed_text) -> Result<ImportSummary, TargetsError>`: parse
  the seed, enforce the duplicate/replace rules, write transactionally.

### Error contract

`TargetsError` distinguishes, at minimum: usage/parse errors (map to exit 2 at
the CLI), operational errors (I/O, malformed record, schema violation; exit 1),
and an internal-invariant error for a self-validation failure. No error path
leaves a partially written store (transactions), and no path silently drops or
coerces a record (P-4/P-9).
