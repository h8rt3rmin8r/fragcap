# Phase 1 Data Model: CLI targets convergence

No persistent data model change (no store schema change, no new dependency). The
"entities" are the CLI arg surfaces and the shared resolution seam.

## Entity: targets subcommand args (modified)

`db` becomes `Option<PathBuf>` on `TargetsAddArgs`, `TargetsShowArgs` (Show +
Remove), `TargetsExportArgs`, `TargetsCommand::List`, `TargetsCommand::Import`.
Resolution order: explicit `--db` > `FRAGCAP_LOCAL_DB` > `%APPDATA%\fragcap\local.db`.
`None`: error for add/show/remove/export/import; empty listing for list. See
[contracts/targets-db-default.md](contracts/targets-db-default.md).

## Entity: shared target-resolution seam (new module)

`commands/target_resolve.rs` (`pub(crate)`): the one implementation of stored-target
resolution (selector -> validated `Profile`), moved from `capture.rs`, called by both
`capture` and `extcap`. Functions: `resolve_stored`, `setup_stores`, `build_resolver`,
`resolve_from_install`, `synthesize_named_profile`, `synthesize_profile`,
`steam_app_id`, `StoredRef`. Behavior preserved for `capture`.

## Entity: ExtcapArgs (modified)

Selection field `profile: Option<String>` -> `target: Option<String>`; adds
`catalog_db`/`local_db` overrides. Config block selection arg `--profile` -> `--target`.
The capture handler resolves the target via the shared seam, not a profile file. See
[contracts/extcap-target-selection.md](contracts/extcap-target-selection.md).

## Entity: extcap config block (modified selection arg only)

The four-arg config block keeps its structure; the number=0 arg changes from a
profile reference to a target selector. Interfaces, DLTs, and FIFO streaming
unchanged.

## Relationships

- Both `capture` and `extcap` DEPEND ON the shared `target_resolve` seam.
- The targets subcommands DEPEND ON the `paths` default-resolution helpers (reused,
  unchanged).
- The CLI reference and spec section 17 DEPEND ON the converged extcap behavior (the
  legacy callout removed only when the code converges).
- Slice S059 (launch-and-observe) DEPENDS ON this slice's clean `target_resolve` seam.
