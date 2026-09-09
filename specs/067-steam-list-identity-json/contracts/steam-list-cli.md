# Contract: `fragcap steam list` output

## Human mode (`fragcap steam list`, no `--json`)

Standard output is one width-aware headed listing, sorted by name (case-insensitive) and tie-broken by numeric app id. It contains no tab characters and never truncates a value.

When the maximum complete row fits the selected display width, the listing uses an aligned table with two spaces between columns:

```text
APP ID  NAME              STATE         TARGET
730     Counter-Strike 2  registered    cs2 (no position)
570     Dota 2            unregistered
620     Portal 2          registered    portal_2 (#4)
```

When the complete table would exceed the selected width, the entire listing uses labeled vertical records:

```text
INSTALLED STEAM TITLES

APP ID: 1190600
NAME: Captain Hardcore: The Complete Collection
STATE: registered
TARGET: captain_hardcore (#4)
```

Interactive terminal width is clamped to 40 through 80 display columns; redirected output, an unavailable width, and an unsupported-host measurement use 80. Visible display cells determine fit and padding, so combining and wide characters do not move later columns. Tab, carriage return, and line feed inside human values are shown as `\t`, `\r`, and `\n`, while every other character is preserved. See the [S136 human output contract](../../136-steam-list-layout/contracts/steam-list-human.md).

Column contract:

- `APP ID`: `InstalledTitle::app_id`, with layout controls visibly represented.
- `NAME`: `InstalledTitle::name`, with layout controls visibly represented.
- `STATE`: one of `registered` (Positioned or Unpositioned) or
  `unregistered` (Unregistered), the coarse distinction a reader scans for
  first.
- `TARGET`: empty for `unregistered`; `<handle> (#<position>)` for
  `Positioned`; `<handle> (no position)` for `Unpositioned`. The three
  renderings are textually distinct by construction (no row ever prints a
  bare handle with nothing else, and no row ever looks like a positioned row
  without a `#`).

Zero installed titles: unchanged from today, `no installed titles enumerated` and exit 0.

Store absent/unopenable: every row renders `unregistered` (the `Unregistered` fallback), and one warning reaches standard error through the emitter: `local store unavailable; showing installation state only`.

## JSON mode (`fragcap steam list --json`)

Standard output: zero or more newline-delimited JSON objects, one per
installed title, nothing else. No trailing sentence, no summary line.

Per-record fields:

| Field | Type | Presence |
|---|---|---|
| `app_id` | string | always |
| `name` | string | always |
| `install_dir` | string | always |
| `handle` | string | present iff registered (Positioned or Unpositioned) |
| `stable_id` | number | present iff registered (Positioned or Unpositioned) |
| `position` | number | present iff Positioned |

Field order is not part of the contract (a consumer parses by key, matching
every other structured output this tool emits); the order shown in
`data-model.md`'s examples is illustrative.

Zero installed titles: zero bytes of record output (not the human sentence,
not an empty-array marker).

Enumeration warnings and the store-unavailable warning: reach standard error
as NDJSON diagnostic records through the existing emitter, exactly as
`doctor --json` and every other JSON-mode command already do. Never appear
on standard output.

Exit codes (both modes): 0 on success (including zero titles); 2 (usage /
configuration refusal) when no Steam installation is found, unchanged by
`--json`; 1 on an unexpected I/O failure during enumeration.

## Backward compatibility note

S067 accepted a breaking change to the human table's exact byte shape because the prior views could not be correlated by eye. S136 completes that intended contract by replacing the remaining raw tab separators with the aligned-or-vertical layout above. JSON remains the stable machine-readable contract.
