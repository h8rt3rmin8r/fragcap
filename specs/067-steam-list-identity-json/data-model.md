# Phase 1 Data Model: Steam list identity and JSON output

No schema migration. This slice adds one new query against existing tables
and one new in-memory type; no `local.db` schema version bump.

## New in-memory type: `SteamListingIdentity`

Lives in `crates/fragcap-cli/src/commands/steam.rs`. Computed once per
installed title, consumed by both the human table renderer and the JSON
record writer, so the two never describe a title's identity differently
(closes the drift risk `research.md` names).

```text
enum SteamListingIdentity {
    Positioned { stable_id: i64, handle: String, position: usize },
    Unpositioned { stable_id: i64, handle: String },
    Unregistered,
}
```

- `Positioned`: the title is registered in the local store (an exact
  `steam:<app_id>` anchor match) and its `stable_id` appears in the most
  recent `listing_snapshot`. Carries the stable id, the handle, and the
  1-based position. The human table renders the handle and position;
  `stable_id` rides along only for the JSON record (FR-011 asks for it there,
  the human acceptance criteria in issue #171 do not).
- `Unpositioned`: the title is registered but its `stable_id` is absent from
  the most recent snapshot (never listed by `targets`, or listed and then the
  snapshot was overwritten by a later `targets` run that dropped it).
  Carries the stable id and handle, no position.
- `Unregistered`: no `TargetEntry` resolves for this anchor. Also the
  fallback state for every title when the local store cannot be resolved or
  opened at all (FR-008), in that fallback case the command additionally
  emits one warning distinguishing "actually unregistered" ambiguity from
  "could not check."

Derivation, per installed title:

```text
match store.target_by_anchor(&steam_anchor(app_id))? {
    None => Unregistered,
    Some(entry) => match store.listing_snapshot_position(entry.stable_id)? {
        Some(position) => Positioned { stable_id: entry.stable_id, handle: entry.handle, position },
        None => Unpositioned { stable_id: entry.stable_id, handle: entry.handle },
    },
}
```

A store-query error (not "not found," an actual `TargetsError`) for one title
is not silently folded into `Unregistered`: it is surfaced as a warning
through the emitter (per the spec's Edge Cases) and that title's identity
falls back to `Unregistered` only for rendering purposes, with the warning
carrying the distinction, the row itself cannot carry a fourth on-screen
state without breaking the three-state contract the issues specify, so the
side-channel warning is what preserves P-9 here.

## New store read: `Store::listing_snapshot_position`

`crates/fragcap-targets/src/store.rs`, alongside `listing_snapshot_nth` and
`listing_snapshot_len`.

```text
pub fn listing_snapshot_position(&self, stable_id: i64) -> Result<Option<usize>, TargetsError>
```

- Query: `SELECT position FROM listing_snapshot WHERE stable_id = ?1`.
- Returns `Ok(None)` when the stable id is not in the current snapshot (not
  an error, the `Unpositioned` state is a legitimate outcome, not a
  failure).
- Read-only: no `INSERT`/`UPDATE`/`DELETE` reaches `listing_snapshot` from
  this path, matching FR-006.
- `position` is stored as SQLite `INTEGER` (matching the existing
  `write_listing_snapshot`/`listing_snapshot_nth` column type); returned as
  `usize` after a checked, non-negative conversion consistent with how
  `listing_snapshot_nth` already returns a `usize` position argument.

## Existing entities reused, unchanged

- **`InstalledTitle`** (`crates/fragcap-steam/src/library.rs:25`): `app_id`,
  `name`, `install_dir`, `installdir`. No field added; `install_dir` is what
  the JSON record surfaces that the human table has never shown (FR-010).
- **`TargetEntry`** (`crates/fragcap-targets/src/entry.rs:175`): `stable_id`,
  `handle` read; nothing else from this type is needed for the listing.
- **`listing_snapshot` table** (`crates/fragcap-targets/src/schema.rs:142`):
  read only, via the new `listing_snapshot_position` query and the existing
  `write_listing_snapshot` remains the sole writer (owned by the `targets`
  hero listing, untouched by this slice).

## JSON record shape (informal; the formal contract is in `contracts/`)

One object per line, fields present unconditionally for the always-known
facts and present-or-absent (never null-as-placeholder) for identity:

```text
{"app_id":"1190600","name":"Captain Hardcore","install_dir":"C:\\...\\Captain Hardcore",
 "handle":"captain_hardcore","stable_id":8391027465103,"position":4}
```

```text
{"app_id":"228980","name":"Steamworks Common Redistributables",
 "install_dir":"C:\\...\\Steamworks Shared"}
```

The second example omits `handle` and `position` entirely (an absent key,
not a `null` value) for an `Unregistered` title, matching FR-011's
"distinguishable from a present-but-empty or zero value" requirement, an
absent JSON key is unambiguous in a way `null` or `0` would not be, since a
row index of `0` could otherwise be misread as a real position.
