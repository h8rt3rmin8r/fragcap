# Contract: extcap capture uses target selection (#156)

## Shared resolution seam (crates/fragcap-cli/src/commands/target_resolve.rs, NEW)

The stored-target resolution logic moves out of `capture.rs` into a `pub(crate)`
module declared in `commands/mod.rs`:

- `resolve_stored`, `setup_stores`, `build_resolver`, `resolve_from_install`,
  `synthesize_named_profile`, `synthesize_profile`, `steam_app_id`, and the
  `StoredRef` helper.
- Behavior is preserved byte-for-byte for `capture` (the S057 positional-selector
  path and every existing capture test unchanged).
- Both `commands/capture.rs` and `commands/extcap.rs` call this one implementation
  (no duplicated body). The seam is the place slice S059 later adds an
  unresolved-entry branch.

## extcap args (crates/fragcap-cli/src/cli.rs, ExtcapArgs)

- Rename the selection field `profile: Option<String>` -> `target: Option<String>`.
- Add `catalog_db: Option<PathBuf>` and `local_db: Option<PathBuf>` (mirroring
  `CaptureArgs`), defaulting the same way `capture` does, so the resolver reaches the
  same stores and tests can isolate a scratch store.
- `roles`, `direction`, `loopback`, `fifo`, `extcap_interface`, the query flags, and
  the flattened `OfflineArgs` are unchanged.

## config block (crates/fragcap-cli/src/commands/extcap.rs, config_block)

- The selection arg changes from
  `{call=--profile}{display=Profile}{tooltip=The profile to capture with: a path, a name, or a game id}`
  to a target selector: `{call=--target}{display=Target}{tooltip=The target to
  capture: a handle, a name, or a row index}`. It stays `{number=0}{type=string}` and
  the analyzer round-trips a single config string.
- `--roles` (number=1), `--direction` (number=2 selector), `--loopback` (number=3
  boolflag) are unchanged. The "exactly four options" shape is preserved.

## capture handler (extcap.rs::capture)

- Replace the `resolve(profile_ref, &search, &bundled)` profile-file resolution with
  the extracted `target_resolve` call: resolve the `--target` selector against the
  local store (via the shared seam), producing the synthesized `Profile`.
- No code path in the extcap capture handler resolves a profile file through the
  `paths::search_path` / `paths::bundled` cascade.
- `assemble::effective_config_for_extcap(&ExtcapArgs, &Profile)` is unchanged.

## Preserved wire contract (must not change)

- `--extcap-interfaces`, `--extcap-dlts`, `--extcap-config` declaration outputs
  (interfaces, link types, the config block as a set of arg lines) and `--capture
  --fifo` streaming stay structurally identical, so unmodified Wireshark still drives
  fragcap. Only the meaning of the one selection arg changes.

## Docs (site/content/docs/reference/cli.mdx)

- Remove the S057 "Live capture from the analyzer dialog is a legacy path" callout in
  the extcap section; document the converged options (extcap selects a stored target,
  like `capture`). No page describes extcap capture as a legacy profile-file path.

## Test expectations (crates/fragcap-cli/tests/cli_extcap.rs)

- `offline_substrate()` registers a target in a temp local store and passes the
  target selector + `--local-db` instead of `--profile <game.json>`.
- The config-block assertions update from `{call=--profile}` to `{call=--target}`;
  the "exactly four options" count stays.
- `a_malformed_profile_is_a_configuration_error_before_capture` converts to the
  target-selection surface (a no-match / unresolvable-target usage error before
  capture).
- `capture_without_a_fifo_is_a_usage_error` and the interface/DLT declaration tests
  keep working.
