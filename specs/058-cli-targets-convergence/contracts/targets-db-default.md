# Contract: default `--db` for the targets subcommands (#157)

## Arg surface (crates/fragcap-cli/src/cli.rs)

`db` changes from required `PathBuf` to `Option<PathBuf>` on:

- `TargetsAddArgs.db`
- `TargetsShowArgs.db` (shared by the `Show` and `Remove` subcommands)
- `TargetsExportArgs.db`
- `TargetsCommand::List { db }`
- `TargetsCommand::Import { file, db }`

Unchanged: `TargetsCommand::Scan { .., db }` (already `Option<PathBuf>`);
`TargetsDiscoverArgs { catalog_db, local_db }` (separate two-store pattern, out of
scope).

## Resolution (crates/fragcap-cli/src/commands/targets.rs)

Each affected handler resolves, in order:

1. an explicit `--db <path>` (flag wins),
2. else `FRAGCAP_LOCAL_DB` (`paths::local_db_path(None)`),
3. else `%APPDATA%\fragcap\local.db` (`paths::default_local_db_path()`).

Implemented as the existing chain
`db.map(Path::to_path_buf).or_else(|| paths::local_db_path(None)).or_else(paths::default_local_db_path)`.

## Behavior on `None`

- `add`, `show`, `remove`, `export`, `import`: a `None` result is a named failure via
  `.ok_or_else(|| CliError::failure("the local store path could not be determined"))`
  (the `run_discovery_default` precedent). No panic, no silent no-op.
- `list`: degrades to the empty listing (`empty_listing`), matching the bare
  `fragcap targets` hero command it mirrors.

## Invariants

- An explicit `--db` always overrides the env override and the default.
- `add`/`import` against a defaulted, not-yet-created store still work: `Store::open`
  creates a fresh DB at a nonexistent path (unchanged).
- No new `paths` helper is added.

## Test expectations (crates/fragcap-cli/tests/cli_targets.rs)

- Existing explicit-`--db` tests pass unchanged (flag still wins).
- New: at least one subcommand invoked with no `--db` resolves the store named by an
  isolated `FRAGCAP_LOCAL_DB` (per-test temp path; isolate to avoid the parallel-test
  env race). Confirm the effect (e.g. `add` then `show`/`list` against the same
  defaulted store).
