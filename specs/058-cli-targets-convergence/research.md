# Phase 0 Research: CLI targets convergence

All unknowns are resolved by reading the shipped `fragcap-cli` source; this is a
convergence slice, so "research" is establishing the existing patterns to reuse.

## Decision 1: Default-db resolution reuses the existing chain (#157)

**Decision**: For each affected subcommand, when `--db` is omitted, resolve
`db.map(Path::to_path_buf).or_else(|| paths::local_db_path(None)).or_else(paths::default_local_db_path)`.
`add`/`show`/`remove`/`export`/`import` treat a `None` result as a named failure
(`.ok_or_else(...)`); `list` degrades to an empty listing on `None`, matching the
bare hero command.

**Rationale**: This is the exact chain the `scan` variant (targets.rs ~264-267) and
the bare hero `list_default` (~59-71) already use; `run_discovery_default` (~221-224)
is the `.ok_or_else` precedent for a store that must open. `paths::local_db_path(flag)`
already gives flag precedence over `FRAGCAP_LOCAL_DB`, and `default_local_db_path()`
resolves `%APPDATA%\fragcap\local.db`. No new helper is needed.

**Alternatives considered**: A new `resolve_local_db(flag)` helper in paths.rs
(rejected: the inline chain is already used verbatim in three places; a helper is a
nicety, not required, and adds surface); making `--db` default via clap
`default_value` (rejected: the default is environment-derived and may be `None`, which
clap `default_value` cannot express, and it would bypass the flag>env>default order).

## Decision 2: `Option<PathBuf>` on five arg sites (#157)

**Decision**: Change `db: PathBuf` to `db: Option<PathBuf>` on `TargetsAddArgs`,
`TargetsShowArgs` (shared by `Show` and `Remove`), `TargetsExportArgs`, the
`TargetsCommand::List` variant, and the `TargetsCommand::Import` variant. `Scan` is
already `Option<PathBuf>`; `Discover`'s two-store `catalog_db`/`local_db` pair is out
of scope.

**Rationale**: These are the five sites the exploration found still required. The
handlers then apply Decision 1. `Store::open` creating a fresh DB at a nonexistent
path (existing behavior) keeps defaulted `add`/`import` working (FR-004).

**Alternatives considered**: Also defaulting `Discover` (rejected: it uses a distinct
two-store pattern and is explicitly out of #157's scope).

## Decision 3: Extract the resolution seam into `commands/target_resolve.rs` (#156)

**Decision**: Move the private stored-target resolution functions out of `capture.rs`
(`resolve_stored`, `setup_stores`, `build_resolver`, `resolve_from_install`,
`synthesize_named_profile`, `synthesize_profile`, `steam_app_id`, and the small
`StoredRef` helper) into a new `commands/target_resolve.rs` as `pub(crate)`, declared
in `commands/mod.rs`. `capture.rs` calls the extracted functions; behavior for
`capture` is preserved (the S057 positional-selector path unchanged).

**Rationale**: FR-009 requires one shared implementation; a module keeps the seam
cohesive and gives S059 (launch-and-observe) one place to add its unresolved-entry
branch. Making the fns `pub(crate)` in place (no move) would work but leaves the
resolution logic entangled with capture's run loop; a dedicated module is the clean
seam the sprint plan calls for.

**Alternatives considered**: `pub(crate)` in `capture.rs` without moving (rejected:
weaker seam, harder for extcap and S059 to depend on without importing capture's
unrelated internals); a new crate (rejected: overkill, and would perturb `cargo xtask
deps`).

## Decision 4: extcap selects a target; the wire contract is preserved (#156)

**Decision**: Rename the extcap config selection arg from `--profile` to `--target`
(display "Target", tooltip naming a handle/name/row-index/id) in `config_block()`,
and rename the `ExtcapArgs.profile` field to `target`; add `catalog_db`/`local_db`
overrides to `ExtcapArgs` mirroring `CaptureArgs`. The capture handler replaces
`resolve(profile_ref, &search, &bundled)` with the extracted
`target_resolve::resolve_stored`-equivalent call. The extcap control grammar
otherwise (interfaces, DLTs, config block as arg lines, FIFO streaming) is unchanged;
the analyzer still round-trips a single config string, now a target selector.

**Rationale**: FR-006/FR-007/FR-008. Naming it `--target` matches `capture`'s selector
flag so the analyzer dialog and the command line select capture identically (the
claim S057's callout said was not yet true). `effective_config_for_extcap(&ExtcapArgs,
&Profile)` only consumes roles/direction/loopback/fifo + the resolved profile, so it
is unaffected.

**Alternatives considered**: Keep the arg call name `--profile` with a new meaning
(rejected: misleading now that it is a target selector, and inconsistent with
`capture --target`); add full `--id`/`--process` parity to extcap (rejected: extcap's
one-config-string dialog carries a single selector; `--process` and a distinct `--id`
are capture-command conveniences, out of scope here -- the single selector string
resolves handle/name/row-index via the shared resolver, and `--id` selection can ride
the same string if the resolver already accepts it, otherwise it is deferred).

## Decision 5: Verification and docs reconciliation

**Decision**: Verify with `cargo xtask ci` (GNU locally). Confirm no profile-file
resolver call remains in the extcap capture handler by inspection/grep. Confirm no
`Cargo.lock` delta. Replace the S057 extcap "legacy" callout in
`site/content/docs/reference/cli.mdx` with the converged options; reconcile spec
section 17 and run `cargo xtask spec`.

**Rationale**: Establishes the objective gates for SC-001..SC-005 and P-11.

**Alternatives considered**: none material.
