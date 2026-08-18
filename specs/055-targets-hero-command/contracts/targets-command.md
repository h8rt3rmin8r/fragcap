# Contract: `targets` command surface

**Feature**: S055 | **Date**: 2026-08-18

The `targets` command owns local.db operations (S054, namespaces follow stores).
This contract fixes the subcommand grammar, exit codes, and output obligations
S055 adds or changes. Existing `discover` and `show` are unchanged except that
`show <n>` inherits the snapshot-backed row index (see listing contract).

## Grammar

```
fragcap                              # bare: default listing + --help footer
fragcap targets                      # listing, no footer
fragcap targets list [--db PATH]     # listing (explicit)
fragcap targets add [PATH] [--name S] [--handle S] [--steam APPID] [--anchor S] [--exe S] [--db PATH]
fragcap targets scan <DIR> [--catalog-db PATH] [--db PATH]
fragcap targets remove <SELECTOR|--id N> [--db PATH]
fragcap targets export [SELECTOR|--id N] [--db PATH]        # stdout
fragcap targets import <FILE> [--db PATH]
fragcap targets show <SELECTOR|--id N> [--db PATH]          # unchanged surface
fragcap targets discover [...]                              # unchanged
```

`SELECTOR` is the S051 selector: a 1-based row index (snapshot-backed, see listing
contract), an exact handle, or a case-insensitive exact name. `--id N` is the
durable stable-identifier form.

## Exit codes (the S051 §5.4 contract, extended)

| Situation | Exit |
| --- | --- |
| Success | 0 |
| Handle/name selector matches nothing (clean miss) | 0 |
| Row-index selector out of snapshot range, or `--id` unknown | 2 (usage) |
| Ambiguous name on `remove`/`export`/`show` (lists matches, refuses) | 2 (usage) |
| Missing required value under non-interactive `add` | 2 (usage) |
| Malformed/ nonconforming `import` file | 1 (failure) or 2 per existing CliError mapping |
| Store I/O failure | 1 (failure) |

## Per-subcommand obligations

### `list` / bare `fragcap` / `targets`
See `contracts/listing-and-row-index.md`. Writes the listing snapshot. Ends by
naming the next command; bare `fragcap` also appends the `--help` footer,
`targets` does not (S054 footer bool at the dispatch site).

### `add`
See `contracts/interactive-add.md`. Interactive when stdin is a terminal;
flag-driven otherwise. `add --steam APPID` scaffolds a `steam:APPID`-anchored
entry for an installed title (usage error naming the app id if not installed).
Persists via `insert_target` (P-10). MUST NOT record an unobserved socket holder.

### `scan <DIR>`
Registers the titles discovered under `DIR` (reuses `DirectorySource`
+ signature classification). Discovery accounting is conserved and surfaced (P-4).

### `remove <SELECTOR>`
Removes exactly the resolved target (`delete_target`); others untouched. Ambiguous
name lists matches and refuses (exit 2). A clean miss on a handle/name reports
"no target matches" and exits 0; an out-of-range row index or unknown `--id`
exits 2.

### `export [SELECTOR]`
Emits the target-entry array (see `contracts/export-import.md`) to stdout. No
selector: all entries. With a selector: a one-element array. A selector that
matches nothing emits an empty array and exits 0 (a listing/export of "nothing"
is not an error); an ambiguous name exits 2.

### `import <FILE>`
Reads the target-entry array and merges each element on `stable_id`. A
nonconforming file is rejected with diagnostics and applies nothing (all-or-
nothing; no partial import). Reports the count inserted vs updated.

## Invariants

- Every listed row is capturable in principle; CAPTURE reports closeness, never
  validity (FR-021).
- One creation operation and one stored form for all sources (P-10).
- No fabricated socket holder anywhere (P-9, FR-012).
- Every discard/decline in scan/discover is counted and surfaced (P-4).
