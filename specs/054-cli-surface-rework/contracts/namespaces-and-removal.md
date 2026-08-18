# Contract: namespaces, relocations, and removals

## `catalog` (writes/reads `catalog.db`)

| Subcommand | Origin | Contract |
| --- | --- | --- |
| `catalog import <seed> --db <p>` | moved from `targets import` | Same behaviour, catalog store. |
| `catalog export --db <p>` | moved from `targets export` | Same behaviour. |
| `catalog seed …` | moved from `targets seed` | Same args, same net gating. |
| `catalog seed-engine …` | moved from `targets seed-engine` | Same args, same net gating. |
| `catalog seed-signatures --db <p>` | moved from `targets seed-signatures` | Same behaviour (S053). |
| `catalog update` | NEW | Fetch the published catalog into `catalog.db`; net-gated (S035 seeder), compiled-not-run in CI; honest report when no catalog reachable. |

Each moved command MUST no longer resolve under `targets`.

## `targets` (writes/reads `local.db`)

| Subcommand | Change |
| --- | --- |
| `targets add <name> --db <p> [--anchor] [--exe] [--handle] [--steam <app_id>]` | Adds `--steam <app_id>`: resolve installed Steam title, register with a `steam:<app_id>` anchor. |
| `targets list --db <p>` | Unchanged. |
| `targets show <selector>\|--id --db <p>` | Unchanged (S051 resolution). |
| `targets discover …` | Unchanged (S052). |
| `targets scan <dir> [--catalog-db]` | Unchanged (S052/S053). |

Catalog operations MUST NOT resolve under `targets`.

## `steam` (Steam-specific only)

`steam profile <app_id>` REMOVED (replaced by `targets add --steam`). `steam`
retains installed-title enumeration and Steam metadata reads.

## `schema` (unchanged)

`schema validate <file>` and `schema print` remain. `schema validate` is the general
JSON-artifact validator, documented under an advanced section for sharing a JSON
export.

## Removals (must be rejected as unknown; a testable negative)

- `run`, `tap`, `watch` (and their arg structs / command modules).
- `profile` command and every subcommand (`validate`, `list`, `show`).
- The `--profile-dir` global flag, the file-backed profile provider, and the
  `--profile` capture selector.

## Stale-reference obligation

No shipped documentation example, and no line of master-spec section 17, may name a
removed or relocated-under-old-namespace command after this change (FR-017).
