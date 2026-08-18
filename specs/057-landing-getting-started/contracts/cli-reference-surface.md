# Contract: The CLI reference command surface

The CLI reference page (`site/content/docs/reference/cli.mdx`) must mirror this
surface, derived from `crates/fragcap-cli/src/cli.rs`. Grouped as the binary's own
`--help` groups them. No `run`, `tap`, `watch`, `profile`, or `steam profile`.

## Global options

`--quiet`, `--silent`, `--json` (global on every command).

## Capture

- **`capture`** - Capture a target's traffic. Target input is exactly one of
  `--target <selector>` (handle | case-insensitive name | 1-based row index),
  `--id <stable-id>`, or `--process <image>` (clap group, exactly one required).
  Path anchors `--path <substr>` / `--path-regex <re>` disambiguate a shared image
  name. Output/scope flags: `--out <path>`, `--mode file|stream|ring`, `--sink
  <spec>` (repeatable: `file:`, `jsonl:`, `pipe:`, `tcp://`), `--duration`,
  `--wait`, `--max-packets`, `--max-bytes`, `--roles a,b`, `--direction
  in|out|both`, `--interface` (repeatable), `--loopback`, `--no-payload`, `--ring
  <window>`, `--launch` (requires a `--target` carrying a Steam anchor). Store
  overrides: `--catalog-db`, `--local-db`. Direction filtering is recorded but not
  yet enforced; roles scoping is enforced.
- **`replay`** - Run a capture file back. Not yet implemented; parses and reports
  unavailable.

## Targets (local.db)

- **`targets`** (no subcommand) - the hero listing of registered targets, numbered,
  ending by naming the next command.
- **`targets add [name] --db <local.db>`** - register a target. `--steam <app_id>`
  registers an installed Steam title (anchors `steam:<app_id>`, supplies the name);
  `--anchor <platform:id>` gives a deterministic identity (mutually exclusive with
  `--steam`); `--exe <name>`; `--handle <override>`; `--socket-holder yes|no|unsure`
  (non-interactive form of the socket-holder question; requires `--exe`).
- **`targets list --db`**, **`targets show <selector>|--id --db`**,
  **`targets remove <selector>|--id --db`**, **`targets export [selector]|--id
  --db`**, **`targets import <file> --db`**.
- **`targets discover --catalog-db --local-db [--steam-root]`** - walk Steam and
  known roots, list candidates (reads only).
- **`targets scan <dir> [--catalog-db] [--db]`** - treat one directory as a single
  candidate; with `--catalog-db` detect engine/anti-cheat/DRM; with `--db` register.
- **`technologies --path <dir> --catalog-db <catalog.db>`** - detect engine,
  anti-cheat, and DRM in an install directory from the signature table.
- **`steam list`** - enumerate installed Steam titles (inspection only; registering
  a title is `targets add --steam`).

## Environment

- **`doctor`** - report environment readiness (read-only). `--fix` adds an
  interactive action layer that offers the remediations the report named, one at a
  time, under confirmation; `--yes` pre-confirms (unattended, still needs a terminal
  stdout). `--fix` is refused with `--json` and non-interactive stdout. (S056.)
- **`extcap`** - Wireshark analyzer integration. The protocol form (driven by
  `--extcap-interfaces` / `--extcap-dlts` / `--extcap-config` / `--capture --fifo`)
  is invoked by the analyzer, not by hand. `extcap install` / `extcap uninstall`
  register/unregister fragcap as an extcap source; scope selectors `--user`
  (default), `--system`, `--dir <path>` are mutually exclusive.

## Data

- **`catalog`** (catalog.db) - `import <seed> --db`, `export --db`, `seed`,
  `seed-engine`, `seed-signatures --db`, `update --db` (net-gated fetch).
- **`schema`** - `validate <file>` (structural, every violation in one pass),
  `print` (emit the embedded master schema).

## Notes for the page

- The page stays "the readable version"; the binary's `--help` is authoritative.
- Durations share one grammar (`30s`, `5m`, `2h`); sizes take `b`/`kb`/`mb`/`gb`.
- Non-file sinks name their format (`,format=pcapng` / `,format=jsonl`).
- Do not document the hidden offline flags (`--replay-source`, `--attr-script`,
  `--process-script`, `--local-addr`, `--fire-interrupt`); they are test-only.
