# Phase 1 Data Model: The targets hero command and interactive authoring

**Feature**: S055 | **Date**: 2026-08-18

This slice reads and authors the existing `TargetEntry` (S051); it adds one new
persisted table (the listing snapshot), one new stored-field convention (the
unresolved launch chain inside `launch_entries`), and one wire representation (the
target-entry export array). No change to the `targets` table columns.

## Existing entities (unchanged shape, referenced)

### TargetEntry (`crates/fragcap-targets/src/entry.rs:120`)

The registered target row. Fields, all persisted in the `targets` table
(schema.rs:98):

| Field | Type | Notes |
| --- | --- | --- |
| `id` | `Option<i64>` | rowid; `None` before insert |
| `stable_id` | `i64` | UNIQUE; the durable, machine-facing identity; merge key for import |
| `handle` | `String` | UNIQUE; normalized; CHECK not purely numeric |
| `name` | `String` | display name |
| `classification` | `TargetClassification` | game/launcher/tool/mod/emulator/unknown |
| `classification_source` | `ClassificationSource` | catalog/engine-signature/platform/user/unset |
| `fidelity` | `FidelityTier` | authored/verified/heuristic-unverified/observed (Ord: Authored > Verified > HeuristicUnverified > Observed) |
| `provenance` | `Option<Value>` | JSON carried whole |
| `anchor` | `Option<String>` | canonical e.g. `steam:620` |
| `launch_entries` | `Option<Value>` | JSON carried whole; **carries the resolved/unresolved chain (see below)** |
| `install_root` | `Option<String>` | filesystem root or None |
| `evidence` | `Option<Value>` | JSON carried whole; **the KNOWN column and inline scan write here** |

State transitions relevant to S055:
- **Authoring** (interactive/flag `add`, `scan`, `discover`): create with
  `fidelity = Authored` (user) or the source's default fidelity, via
  `insert_target`.
- **Promotion** (capture write-back, D6): `fidelity` raised to `Verified` and
  `launch_entries` rewritten from the observed socket holder, via the new
  `promote_target_launch`.
- **Supersede** (existing, S051): unanchored id -> anchored id via
  `supersede_with_anchor`; unchanged by S055.
- **Removal** (new): row deleted via `delete_target`; aliases cascade.

## New persisted entity

### ListingSnapshot (`listing_snapshot` table, schema version 6)

The ordered set the most recent listing displayed, so row-index selectors resolve
to what the user saw.

| Column | Type | Notes |
| --- | --- | --- |
| `position` | `INTEGER PRIMARY KEY` | 1-based row number as displayed |
| `stable_id` | `INTEGER NOT NULL` | resolves via `target_by_stable_id`; survives supersede |
| `handle` | `TEXT NOT NULL` | for a clear removed-row / out-of-range message |

- **Cardinality**: at most one snapshot per store. Writing a listing does
  `DELETE FROM listing_snapshot` then inserts `1..n` in display order.
- **Lifetime**: replaced by every listing; never auto-expires. `--id` resolution
  is independent of it.
- **Resolution**: `resolve_positional` row-index branch reads
  `listing_snapshot` by `position`, then `target_by_stable_id(stable_id)`. A
  position with no row, or a `stable_id` no longer present, is `NoMatch` (the
  row-index callers map that to a usage error).
- **Migration**: `MIGRATE_5_TO_6` adds the table; `DDL` gains the same
  `CREATE TABLE`; `SCHEMA_VERSION` 5 -> 6; a `version == 5` step is added to the
  migration driver (store.rs:52-117 pattern).

## New stored-field convention

### Unresolved launch chain (inside `launch_entries`)

`launch_entries` is an opaque JSON value in the store, so the resolved/unresolved
distinction is a convention on its content, not a DDL change. The three authoring
answers (D5) map to:

- **Y (this exe holds sockets)**: a resolved client entry, e.g.
  `[{ "executable": "<exe>", "role": "client" }]`.
- **n (different unknown process holds them)**: the exe recorded as a non-client
  stage with the holder explicitly unresolved, e.g.
  `[{ "executable": "<exe>", "role": "launcher" }]` plus an unresolved marker.
- **unsure**: an unresolved marker with no socket-holder claim, e.g.
  `{ "socket_holder": "unresolved", "observed_exe": "<exe>" }`.

The exact JSON keys are an implementation detail fixed in the plan/tasks; the
invariant (FR-012, P-9) is that no answer records a socket holder the tool did not
observe. Capture's CAPTURE-readiness read (D2) and the promotion write-back (D6)
are the two consumers of this convention.

Note the reduction consumed by `capture --target` (S054
`entry_windows_clients`/windows-executables reduction): a resolved chain reduces
to a client image; an unresolved chain yields no client, which is what drives the
`needs a target` status and the promotion path.

## New wire representation

### Target-entry export array (D7)

A JSON array; each element is one target entry:

```json
[
  {
    "stable_id": 123456789,
    "handle": "the_elder_scrolls_online",
    "name": "The Elder Scrolls Online",
    "classification": "game",
    "classification_source": "user",
    "fidelity": "authored",
    "anchor": "steam:306130",
    "launch_entries": [ { "executable": "eso64.exe", "role": "client" } ],
    "install_root": "C:\\...\\Zenimax Online\\...",
    "evidence": { "...": "carried whole" }
  }
]
```

- **Conformance**: validated structurally by required-field presence and type,
  and by round-trip (export -> import -> identical id set, no duplicates). Not
  validated against `target-schema.v1.json`.
- **Optional fields**: `anchor`, `launch_entries`, `install_root`, `evidence`,
  `provenance` are emitted only when present (mirrors the minimal-record habit of
  export.rs:119).
- **Merge key**: `stable_id`. On import, an existing `stable_id` updates that row;
  a new one inserts. Handle collisions on insert disambiguate as in `add`
  (handle.rs `disambiguate`).
- **Serde**: net-new `TargetEntry` <-> JSON mapping in `fragcap-targets` (the type
  does not derive serde today). The mapping is explicit (like export.rs) so the
  JSON key set is a deliberate, reviewed contract, not a struct-field accident.

## Derived (not persisted) values

- **CaptureReadiness** (`ready` | `needs a target`): from `launch_entries`
  resolvability + `anchor` (D2). Rendered in the CAPTURE column.
- **EvidenceSummary** (KNOWN column text): from `evidence` findings, launcher
  mediation + client image, else the "no online mode / no launch data" fallback
  (D3). Neutral phrasing (FR-021).

## Store method surface (delta)

Net-new methods on `Store`:
- `write_listing_snapshot(&mut self, rows: &[(i64 /*stable_id*/, &str /*handle*/)]) -> Result<(), TargetsError>`
- `listing_snapshot_nth(&self, position: usize) -> Result<Option<i64 /*stable_id*/>, TargetsError>` (or a resolver returning the entry)
- `delete_target(&mut self, id: i64) -> Result<bool, TargetsError>`
- `promote_target_launch(&mut self, id: i64, launch_entries: &Value, fidelity: FidelityTier) -> Result<(), TargetsError>`
- an update path used by import for an existing `stable_id` (either a general
  `update_target` or a focused method; scoped in tasks)

Reused unchanged: `insert_target`, `targets`, `target_by_handle`,
`targets_by_name`, `target_by_stable_id`, `target_by_anchor`, `handle_exists`.
