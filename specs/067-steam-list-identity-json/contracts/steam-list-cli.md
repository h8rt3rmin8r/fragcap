# Contract: `fragcap steam list` output

## Human mode (`fragcap steam list`, no `--json`)

Standard output, one header line then one data row per installed title,
sorted by name (case-insensitive), tie-broken by app id:

```text
APP ID   NAME                                  STATE       TARGET
1190600  Captain Hardcore                       registered  captain_hardcore (#4)
1203620  Enshrouded                              unregistered
228980   Steamworks Common Redistributables      registered  steamworks_common (no position)
```

Column contract:

- `APP ID`: `InstalledTitle::app_id`, verbatim.
- `NAME`: `InstalledTitle::name`, verbatim.
- `STATE`: one of `registered` (Positioned or Unpositioned) or
  `unregistered` (Unregistered), the coarse distinction a reader scans for
  first.
- `TARGET`: empty for `unregistered`; `<handle> (#<position>)` for
  `Positioned`; `<handle> (no position)` for `Unpositioned`. The three
  renderings are textually distinct by construction (no row ever prints a
  bare handle with nothing else, and no row ever looks like a positioned row
  without a `#`).

Zero installed titles: unchanged from today, `no installed titles
enumerated` and exit 0.

Store absent/unopenable: every row renders `unregistered` (the
`Unregistered` fallback), and one warning reaches standard error through the
emitter: `local store unavailable; showing installation state only`.

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

This is a breaking change to the human table's exact byte shape (issue #171
explicitly accepts this: "the two views cannot be correlated by eye" is the
defect being fixed). The `--json` mode is new; nothing depended on `--json`
changing `steam list`'s output before this slice, so there is no prior JSON
contract to preserve.
