# CLI Contract: technology detection subcommand

## Command

```text
fragcap technologies --path <INSTALL_DIR>
```

- `technologies`: new top-level subcommand (registered in
  `crates/fragcap-cli/src/commands/mod.rs` and the arg parser).
- `--path <INSTALL_DIR>` (required): the install directory to scan. A relative
  path is resolved against the current working directory.

The exact long/short flag spelling follows the CLI's existing conventions; the
required input is a single install-directory path.

## Behavior

1. Load the embedded, compiled ruleset (built once).
2. Scan `<INSTALL_DIR>` depth-bounded, matching relative file paths.
3. Print the findings grouped by category, in a fixed category order
   (engine, anti_cheat, sdk, framework, emulator, container, runtime, launcher),
   and within a category ordered by technology name. Each finding prints its
   technology name and the representative marker path.
4. If any subtree was unreadable, print the unreadable path(s) as a surfaced
   warning distinct from the findings.
5. If the ruleset load skipped any incompatible patterns, the skipped count is
   available; the command surfaces it (at least when non-zero) so reduced
   coverage is visible rather than implied-complete.

## Output shape (illustrative)

```text
Technologies detected in <INSTALL_DIR> (heuristic-unverified):

  engine
    Unreal            game/bin/win64/Game-Win64-Shipping.exe
  anti_cheat
    EasyAntiCheat     EasyAntiCheat/EasyAntiCheat_x64.dll
  sdk
    Steamworks        steam_api64.dll

(<n> ruleset patterns skipped as incompatible)
```

An install directory with no detected technologies prints a clear
"no technologies detected" line, not an error.

## Exit contract

- `0`: the scan completed (including a clean empty result).
- Non-zero (per the CLI's existing `exit` mapping): the target directory itself
  could not be read (does not exist / not a directory / unreadable root). An
  unreadable *subtree* under a readable root is a surfaced warning, not a
  non-zero exit (the scan still produced a partial-but-valid result).

## Constraints

- Reads file paths only; opens no process handle; reads no process memory; makes
  no network call (P-1).
- Does not modify any capture output and is not part of `run` (P-5, FR-013a).
