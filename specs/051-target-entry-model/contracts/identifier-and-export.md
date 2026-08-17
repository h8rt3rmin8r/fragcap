# Contract: Anchor canonicalization, the 63-bit identifier, and export/import merge

The stable identifier is a durable contract: once shipped it must not change for a
given anchor, or exports made by one version stop merging in another. This file
pins the observable behavior; the algorithm is BLAKE3 (research D-1).

## Canonical anchor string

`<platform>:<platform-id>` with a lowercase platform prefix and the platform's
native id verbatim:

| Platform | Canonical form | Example |
| --- | --- | --- |
| Steam | `steam:<appid>` | `steam:2221490` |
| Epic | `epic:<catalogItemId>` | `epic:fn` (illustrative) |
| GOG | `gog:<productId>` | `gog:1207658930` |

Only Steam is populated by a source today; Epic and GOG forms are fixed now so the
identifier scheme is stable before a second platform arrives. Canonicalization is
total and deterministic: the same logical title always produces the same string.

## Identifier

- **Anchored**: `stable_id = BLAKE3(utf8_bytes(canonical_anchor))` truncated to the
  low 63 bits. Deterministic and derived **only** from the anchor (never name,
  handle, or install path).
  - Contract test: two `TargetEntry` values built independently from `steam:2221490`
    have identical `stable_id`.
  - Contract test: `steam:2221490` and `steam:620` have different `stable_id`.
- **Unanchored**: `stable_id` is a random 63-bit value with a fixed locality bit
  set, so an unanchored id is distinguishable from an anchored one and two
  unanchored entries do not collide.

63 bits (not 64) keeps the value non-negative in SQLite's signed 64-bit integer
column and leaves the top bit free.

## Merge and supersession

- Storing an entry whose anchor already exists (same `stable_id`) merges into the
  existing entry rather than inserting a duplicate row (FR-010, P-10).
- When an unanchored entry gains an anchor, its `stable_id` becomes the anchored
  value and the former random value is inserted into `target_id_aliases`
  (superseded, never reissued). `--id <old>` still resolves to the merged entry.

## JSON export / import

- Export includes `stable_id` (as a JSON number or string; the encoding is settled
  in implementation but must round-trip exactly) and the `anchor`.
- Import merges on the active `stable_id`, and consults `target_id_aliases` so an
  import carrying a superseded id lands on the right entry.
- Contract test: export an entry, re-import into a fresh store, and confirm one
  entry with the same `stable_id`, handle, and anchor (no duplicate).

## Relationship to the published schema (`schema validate`)

The exported entry is the JSON target document the published master schema
describes: one document shape for export, import, and validation (P-10). The
schema is extended with the entry fields; `handle` and `stable_id` are optional on
input (computed when absent), while `name`, `classification`,
`classification_source`, `fidelity`, and (when present) `anchor` follow the entry
model.

- Contract test: an exported entry passes `schema validate` (exit 0).
- Contract test: import(export(entry)) yields one entry (round-trip identity).
- A schema-version bump, if the extension requires one, is an implementation
  detail; the validation role is unchanged.
