# Data model: The two-store split (S050)

No new schema. The single store type and its version-2 schema are reused for both
files; the split is by file and lifecycle, not by schema.

## The two stores

| Store | Owner | Lifecycle | Written by | Read by |
| --- | --- | --- | --- | --- |
| `catalog.db` | ShruggieTech | disposable, replaced wholesale by a future `catalog update` | the MSI seed + `targets seed`/`import` (maintainer) | resolution (second) |
| `local.db` | the user | durable, never replaced | learned launch accumulation; later slices | resolution (first) |

Both hold the version-2 schema (`games`, `launch_entries`, `technologies`,
`seed_state`). In S050 `catalog.db` carries the shipped rows and `local.db` carries
the learned launch rows; later slices add `local.db`-only tables (target entries,
overrides, local detection, IGDB, volume exclusions, last-listing snapshot).

## Path resolution

| Store | Flag | Env | Default |
| --- | --- | --- | --- |
| catalog | `--catalog-db <path>` | `FRAGCAP_CATALOG_DB` | `%APPDATA%\fragcap\catalog.db` |
| local | `--local-db <path>` | `FRAGCAP_LOCAL_DB` | `%APPDATA%\fragcap\local.db` |

Flag over env over default, per store. `FRAGCAP_HINT_DB` and `--hint-db` are
removed.

## First-run bootstrap decision (per store)

- Path exists: left untouched (idempotent).
- Absent, catalog with a beside-binary template: template copied to AppData,
  read-only attribute cleared.
- Absent, no template (local always; catalog on a bare-exe build): an empty
  current-schema store is created.
- A bootstrap failure is a warning, not fatal; the path is dropped so the resolver
  proceeds without that store.

## Layered provider query

`HintDatabaseProvider` holds `Vec<Store>` in priority order `[local, catalog]`.
For a request carrying a Steam app id, it queries each store in turn with the
existing single-store logic (row lookup, launcher-mediated decline, one-executable
rule, ambiguity note) and returns the first store's usable `Target`. If every
store is absent or declines, it returns `None` and the cascade continues.

## Key entities

- **`catalog.db`** / **`local.db`**: the two files above.
- **Seed template**: the `catalog.db` the installer places beside the binary.
- **Layered `HintDatabaseProvider`**: one provider, ordered stores, local first.
