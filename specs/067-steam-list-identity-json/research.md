# Phase 0 Research: Steam list identity and JSON output

No `NEEDS CLARIFICATION` markers remain in the Technical Context; this phase
records the decisions the plan depends on, each with rejected alternatives.

## Decision: identity join key

**Decision**: Join each `InstalledTitle` to the local store by its exact Steam
anchor, `identifier::steam_anchor(app_id)` (`steam:<app_id>`), via
`Store::target_by_anchor`.

**Rationale**: This is the existing exact-join mechanism `targets add --steam`
already uses; `target_by_anchor` already exists
(`crates/fragcap-targets/src/store.rs:695`) and needs no new query. Issue
#171 explicitly calls this join "exact, not heuristic."

**Alternatives considered**: Name matching between `InstalledTitle::name` and
`TargetEntry::name`, rejected because two different app ids can share a
display name (soundtrack/redistributable entries commonly do), which the
issue's edge cases explicitly warn against.

## Decision: row-index reverse lookup

**Decision**: Add `Store::listing_snapshot_position(&self, stable_id: i64) ->
Result<Option<usize>, TargetsError>`, a new read-only query:
`SELECT position FROM listing_snapshot WHERE stable_id = ?1`.

**Rationale**: The existing `listing_snapshot_nth` (store.rs:923) reads
position → stable_id; this slice needs the reverse. The table already stores
both columns per row (`write_listing_snapshot` writes `(position, stable_id,
handle)` rows, store.rs:906), so the reverse query needs no schema change,
matching the issue's own note that "this issue needs the reverse direction,
... which is a new read on the store rather than a new table."

**Alternatives considered**: Loading the whole snapshot into a `HashMap` in
the CLI layer via repeated `listing_snapshot_nth` calls, rejected as it
would need to guess an upper bound on position or call `listing_snapshot_len`
first and loop, which is more code than one indexed query, for a table that
is already position-and-stable-id keyed for exactly this kind of lookup.

## Decision: snapshot is read-only from this command

**Decision**: `steam list` never calls `write_listing_snapshot`. The join uses
whatever snapshot the last `fragcap targets` run left behind, including "no
snapshot" or "stale snapshot" as legitimate states.

**Rationale**: Issue #171 states this as a hard constraint: "There is one
snapshot table and it is the meaning of `fragcap capture <n>`. If `steam
list` rewrote it with its own ordering, a number the user read from the hero
listing a moment earlier would silently resolve to a different target."
Constitution P-9 (the instrument does not lie) backs this: `capture <n>`
must keep meaning what the operator last saw.

**Alternatives considered**: A second, namespaced snapshot, the issue itself
flags this as "an architecture decision that should be taken explicitly
rather than as a side effect of adding a column," and the acceptance criteria
require only that "steam list does not change what `fragcap capture <n>`
resolves to," which the read-only approach satisfies without the added
schema surface. Deferred as out of scope.

## Decision: three-state identity rendering

**Decision**: Model the join result as an enum with three variants
(`Positioned { handle, position }`, `Unpositioned { handle }`,
`Unregistered`), computed once per title and shared by both the human
renderer and the JSON serializer, so the two output modes cannot drift apart
(this is exactly what checklist item CHK007 in `identity-json.md` flags as a
risk).

**Rationale**: FR-003 through FR-005 and FR-011 both describe the same three
states; a shared type is the only way to guarantee the two renderers agree
without duplicated conditional logic.

**Alternatives considered**: Two independent `Option<Option<...>>`-shaped
computations, one per renderer, rejected because it is exactly the drift
risk the checklist calls out, and the campaign's own recurring-bugs memory
records that duplicated state logic across renderers has caused defects
before in this codebase (S066's install-root gap, per the S066 review round).

## Decision: JSON record shape and framing

**Decision**: Newline-delimited JSON (JSON Lines), one record per installed
title, hand-rolled via the existing `fragcap::write_json_string` helper,
matching `doctor --json`'s `render_json` (`crates/fragcap-cli/src/doctor/mod.rs:360`)
rather than introducing `serde_json::to_string` for this record (the crate
already depends on `serde_json` at runtime, but `doctor`'s precedent is
hand-rolled field-by-field construction for exactly this kind of small,
stable record shape).

**Rationale**: Issue #172 names `doctor` as the precedent and states "the
house precedent is newline delimited." Matching it avoids introducing a third
JSON-writing convention into the CLI crate (the `AGENTS.md` non-negotiable
"wrappers stay thin" spirit extends to not multiplying output-serialization
approaches).

**Alternatives considered**: `serde_json::to_string` over a `#[derive(Serialize)]`
struct, rejected only for this slice to match the existing hand-rolled
convention `doctor` and the emitter already use in this crate; revisiting the
convention itself is out of scope here.

## Decision: sort order

**Decision**: Sort installed titles by name, case-insensitive ordinal
comparison (`str::to_lowercase` compare, consistent with the workspace's
existing non-locale-aware string handling), tie-broken by app id (numeric
comparison, since `app_id` is stored as a numeric string but two different
titles cannot share an app id, so the tiebreak is definitional rather than
a real collision path, the plan case is name collisions across different
app ids per the spec's Edge Cases).

**Rationale**: Issue #171 requests "order becomes a visible property and
should be chosen deliberately: by name is the readable default, and it
matches the hero listing's sort by handle" (both are human-readable, stable
identifiers a user chose or Steam assigned).

**Alternatives considered**: Sorting by app id (today's incidental order),
rejected per the issue's own critique that it is "lexicographic inside one
library... and undefined across two," which is neither deliberate nor
readable.

## Decision: store-open-failure fallback

**Decision**: On any store resolution or open failure, `steam list` falls
back to the pre-slice two-column-equivalent output (every row `Unregistered`
in the new three-state model), plus one warning emitted through the existing
`Emitter::warn` (honoring `--json`'s NDJSON diagnostic shape automatically,
since the emitter already does this, see `emit.rs`).

**Rationale**: Issue #171's acceptance criteria state plainly: "With no
local store, or an unopenable one, the listing still succeeds and still
names every installed title." Using the emitter (rather than a raw stderr
write) means this slice needs no new diagnostic-formatting code at all,
`Emitter::warn` already branches on the ambient `--json` format.

**Alternatives considered**: Failing the command outright when the store is
absent, explicitly rejected by the issue: "on a fresh install there is
legitimately nothing to join against," so failure would break the common
first-run path.
