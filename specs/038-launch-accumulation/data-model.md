# Phase 1 Data Model: launch-data accumulation

## New and reused entities

### AppInfoApp (fragcap-steam, new)

One app's parsed appinfo section.

- `appid: u32` - the Steam application id from the section header.
- `change_number: u32` - the section's change-number; the staleness key.
- `launch: Vec<SteamLaunchEntry>` - the parsed launch configuration, in cache
  order, verbatim.

### SteamLaunchEntry (fragcap-steam, new)

A crate-local, boundary-neutral launch entry (fragcap-steam may not depend on
fragcap-targets, so it does not reuse targets' `LaunchEntry`; the facade maps
between them).

- `executable: String` - required, non-empty; the one field an entry must have.
- `arguments: Option<String>`
- `launch_type: Option<String>` - the appinfo `type`.
- `os: Option<String>` - the appinfo `config/oslist`, verbatim.
- `osarch: Option<String>` - the appinfo `config/osarch`, verbatim.
- `beta_branch: Option<String>` - the appinfo `config/betakey`, verbatim.
- `description: Option<String>`

### AppInfoParse (fragcap-steam, new)

The result of parsing a whole appinfo file, keeping failures rather than dropping
them.

- `apps: Vec<AppInfoApp>` - every section parsed cleanly.
- `failures: Vec<AppInfoFailure>` - sections that could not be parsed.

### AppInfoFailure (fragcap-steam, new)

- `appid: Option<u32>` - the section's appid when the header was readable, else
  `None` (a fault before the appid was read ends the file).
- `reason: String` - a positioned, human-readable cause.

### LaunchEntry (fragcap-targets, reused unchanged)

Already exists (`model::LaunchEntry`): `executable` (via `new`), `os`, `osarch`,
`launch_type`, `beta_branch`, `arguments`, `description`. The facade maps
`SteamLaunchEntry` to this field-for-field.

### games table (fragcap-targets, one additive column)

Add `appinfo_change_number INTEGER` (nullable). No CHECK. It is store-internal
bookkeeping, never exported, never surfaced on the `Game` model.

- Fresh store (user_version 0): the column is in the v2 DDL.
- Existing v1 store (user_version 1): `ALTER TABLE games ADD COLUMN
  appinfo_change_number INTEGER`, then stamp user_version 2. Existing rows get
  NULL, so they read as "never learned from appinfo" and refresh on first walk.
- user_version 2: opened as-is. Greater than 2: the existing
  `SchemaVersion { found }` error.

`SCHEMA_VERSION` becomes `2`. The `launch_entries` and `seed_state` tables are
unchanged (`seed_state` already permits the `'launch'` tier value from S034).

### LaunchAccumulationSummary (fragcap facade, new)

The reconciled account of one walk. Conservation: `considered == written +
skipped + failed + empty`.

- `considered: u64` - installed apps examined (the intersection of the installed
  library with appinfo presence is examined; an installed app absent from appinfo
  is still considered and falls in `empty`).
- `written: u64` - apps whose launch data was (re)written because it was missing
  or stale and had at least one storable entry.
- `skipped: u64` - apps whose stored change-number already matched the cache
  (the fast path).
- `failed: u64` - apps whose appinfo section could not be parsed.
- `empty: u64` - apps considered that yielded no storable launch entry (absent
  from appinfo, or present with no executable). Not a failure (FR-009).

## State transition per considered app

```text
installed app X
  |
  ├─ not present in appinfo ............................ empty
  ├─ present, section parse failed .................... failed
  ├─ present, parsed, cache.change <= stored.change ... skipped
  └─ present, parsed, cache.change > stored.change
        ├─ >= 1 storable launch entry ................. written  (merge_launch)
        └─ 0 storable launch entries .................. empty
```

`merge_launch(appid, change_number, entries)` on the write path:

1. Ensure a `games` row exists (`INSERT OR IGNORE (appid)`), so `launch_entries`'
   foreign key holds without touching any Tier 1/3 column.
2. Set `games.appinfo_change_number = change_number`.
3. Delete this appid's `launch_entries`, then insert the new entries in order.
4. Leave `name`, metrics, engine columns, `launcher_mediated`, `token_required`
   untouched. All in one transaction.

## Validation rules

- An entry with an empty or absent executable is not stored (schema CHECK
  `length(executable) > 0` is the backstop; the mapping drops it first).
- Entries are stored in cache order; no dedup, reorder, or normalization (P-9).
- `appinfo_change_number` is written only alongside launch data, so a game with a
  stored change-number always has (had) launch rows learned at that version.
