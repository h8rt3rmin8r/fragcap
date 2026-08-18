# Tasks: CLI targets convergence (S058)

**Feature dir**: `specs/058-cli-targets-convergence/`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Local build/test: `cargo +1.96.0-x86_64-pc-windows-gnu ...`. CI runs MSVC
`cargo xtask ci`. Internal order: Part A (#157) first, then Part B (#156).

## Phase 1: Setup

- [ ] T001 Re-read the current `commands/targets.rs` handlers (add/show/remove/export/import/list dispatch), the `scan`/`list_default`/`run_discovery_default` default-resolution patterns, and the `capture.rs` resolution functions + `extcap.rs` capture handler, so the edits mirror existing idioms exactly (no file changes).

## Phase 2: Part A - default `--db` for targets subcommands (#157, US1)

- [ ] T002 [US1] In `crates/fragcap-cli/src/cli.rs`, change `db: PathBuf` to `db: Option<PathBuf>` on `TargetsAddArgs`, `TargetsShowArgs` (Show + Remove), `TargetsExportArgs`, `TargetsCommand::List`, and `TargetsCommand::Import`. Leave `Scan` (already optional) and `Discover` (two-store pattern) unchanged. Update the field doc comments to say the default local store is used when omitted.
- [ ] T003 [US1] In `crates/fragcap-cli/src/commands/targets.rs`, update the `add`, `show`, `remove`, `export`, `import` handlers to resolve the default store when `--db` is omitted, using `db.map(Path::to_path_buf).or_else(|| paths::local_db_path(None)).or_else(paths::default_local_db_path)` and `.ok_or_else(...)` (mirror `run_discovery_default`) on `None`.
- [ ] T004 [US1] In `crates/fragcap-cli/src/commands/targets.rs`, update the `list` dispatch so an omitted `--db` resolves the default and degrades to the empty listing on `None` (route through the existing `list_default`/`hero_listing` path rather than requiring a `db`).
- [ ] T005 [US1] Update the `run` dispatch in `commands/targets.rs` for the changed variant shapes (`List { db: Option<PathBuf> }`, `Import { file, db: Option<PathBuf> }`) and any pattern matches / arg passing that assumed a required `PathBuf`.
- [ ] T006 [US1] In `crates/fragcap-cli/tests/cli_targets.rs`, add a default-store test: with an isolated `FRAGCAP_LOCAL_DB` temp path (per-test, to avoid the parallel-env race), run a subcommand with no `--db` and confirm it operates on that store (e.g. `add` then `show`/`list`). Confirm existing explicit-`--db` tests still pass. `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli --test cli_targets` green.

**Checkpoint (Part A):** the six subcommands run with no `--db` against the default store; explicit `--db` still wins; cli_targets tests green.

## Phase 3: Part B - extcap uses target selection (#156, US2 + US3)

- [ ] T007 [US3] Create `crates/fragcap-cli/src/commands/target_resolve.rs` and move the stored-target resolution functions from `capture.rs` into it as `pub(crate)`: `resolve_stored`, `setup_stores`, `build_resolver`, `resolve_from_install`, `synthesize_named_profile`, `synthesize_profile`, `steam_app_id`, and the `StoredRef` helper. Declare the module in `commands/mod.rs`. Preserve behavior exactly.
- [ ] T008 [US3] Update `crates/fragcap-cli/src/commands/capture.rs` to call the extracted `target_resolve::*` functions; remove the now-moved bodies. Confirm every existing capture test (cli_capture.rs, cli_watch.rs, cli_args.rs positional tests) still passes: `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli` green after the extraction, before touching extcap.
- [ ] T009 [US2] In `crates/fragcap-cli/src/cli.rs`, rename `ExtcapArgs.profile` -> `target: Option<String>` and add `catalog_db: Option<PathBuf>` + `local_db: Option<PathBuf>` (mirroring `CaptureArgs`); update field docs.
- [ ] T010 [US2] In `crates/fragcap-cli/src/commands/extcap.rs`, change `config_block()` selection arg from `{call=--profile}{display=Profile}{tooltip=...profile...}` to `{call=--target}{display=Target}{tooltip=The target to capture: a handle, a name, or a row index}`; keep it `{number=0}{type=string}` and the four-arg shape.
- [ ] T011 [US2] In `crates/fragcap-cli/src/commands/extcap.rs`, replace the `resolve(profile_ref, &search, &bundled)` block in the capture handler with the extracted `target_resolve` call resolving `args.target` against the local store (honoring `args.local_db`/`args.catalog_db`). Remove the `paths::search_path`/`paths::bundled`/profile-file `resolve` usage from the extcap capture path. Update the missing-selection usage message to name a target.
- [ ] T012 [US2] Update `crates/fragcap-cli/tests/cli_extcap.rs`: `offline_substrate()` registers a target in a temp local store and passes the selector + `--local-db` instead of `--profile <game.json>`; config-block assertions `{call=--profile}` -> `{call=--target}` (keep "exactly four options"); convert `a_malformed_profile_is_a_configuration_error_before_capture` to the target-selection surface (unresolvable target usage error); keep `capture_without_a_fifo_is_a_usage_error` and the interface/DLT tests. `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-cli --test cli_extcap` green.

**Checkpoint (Part B):** extcap resolves a stored target via the shared seam; no profile-file resolution remains in the extcap capture path; one shared resolution module; extcap tests green.

## Phase 4: Docs, spec, and polish

- [ ] T013 In `site/content/docs/reference/cli.mdx`, remove the S057 extcap "Live capture from the analyzer dialog is a legacy path" callout and document the converged extcap options (extcap selects a stored target, like `capture`). No page describes extcap capture as a legacy profile-file path.
- [ ] T014 Reconcile master specification section 17 (the extcap/targets command surface) with the converged extcap behavior and the default-`--db` subcommands; run `cargo +1.96.0-x86_64-pc-windows-gnu xtask spec` (Applies-To lockstep) and `bash scripts/lint-docs.sh check` (P-6/glossary). Add a glossary entry only if a new user-facing term appears (expected: none).
- [ ] T015 Encoding sweep: every edited/new file is UTF-8 without BOM, LF, no em-dashes or en-dashes (including the new `target_resolve.rs` comments). Confirm `git diff --stat Cargo.lock` is empty (no dependency delta).
- [ ] T016 Add `changelog.d/S058-cli-targets-convergence.added.md` (with the `spec-impact: 17` marker), noting the default-`--db` subcommands (#157), the extcap-to-target convergence + shared resolution seam (#156), and the removed legacy callout.
- [ ] T017 Run the full gate: `cargo +1.96.0-x86_64-pc-windows-gnu xtask ci` locally (MSVC gate in CI); confirm green (FR-013, SC-005).

## Dependencies

- Part A (T002-T006) is independent and lands first.
- T007 (extract seam) blocks T008 (capture uses it) and T011 (extcap uses it).
- T008 must be green before T009-T012 (settle the extraction before repointing extcap).
- Phase 4 depends on Parts A + B.

## MVP / scope note

Both parts ship together (the slice is the pack #157 + #156). Part A is the
independently-demonstrable ergonomic win; Part B is the convergence that closes the
S057 legacy callout and produces the shared seam slice S059 extends.
