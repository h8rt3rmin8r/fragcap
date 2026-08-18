# Phase 0 Research: The targets hero command and interactive authoring

**Feature**: S055 | **Date**: 2026-08-18 | **Branch**: `055-targets-hero-command`

This resolves the design unknowns the spec raised, grounded in the current code
map (store, selector, CLI, discovery, detection, schema). Each decision names the
concrete anchor it builds on so the plan and tasks inherit exact seams.

## D1. Durable row-index listing snapshot

**Decision**: Add a `listing_snapshot` table to `local.db` (targets store) via a
new `MIGRATE_5_TO_6` step (schema version 5 -> 6). Any listing path (bare
`fragcap`, `fragcap targets`, `targets list`) replaces the snapshot with the
ordered set it displayed. The row-index branch of `selector::resolve_positional`
resolves a bare integer against the snapshot (position -> stored stable_id ->
`target_by_stable_id`) instead of the live `targets()` order.

**Shape**: `listing_snapshot(position INTEGER PRIMARY KEY, stable_id INTEGER NOT
NULL, handle TEXT NOT NULL)`. One snapshot per store; a new listing does
`DELETE FROM listing_snapshot` then inserts positions `1..n`. Storing `stable_id`
(not rowid) makes the reference survive a supersede; `handle` is carried for a
clear out-of-range/removed-row message.

**Rationale**: FR-004/FR-005 require `capture <n>` to name the row the user saw.
Today `resolve_positional` (selector.rs:32-39) takes `targets()` order
(store.rs:545, `ORDER BY id`), which is stable only while the set is unchanged;
an add/remove between listing and capture shifts every later index silently.
Pinning positions in a table is the only way a number keeps its meaning across a
mutation. Keeping the change inside `resolve_positional` means `capture`
(capture.rs:123) and `show` (targets.rs:351/354) both inherit it with no
call-site change (P-10, one resolution path).

**Alternatives considered**:
- *Deterministic order-by-handle, no table*: makes two listings of an unchanged
  set match, but does not survive add/remove (indices still shift). Rejected: the
  issue explicitly says "writes the last-listing snapshot to local.db," and the
  staleness edge (spec Edge Cases) needs a pinned reference.
- *Snapshot in a temp/session file*: a second storage shape for one datum,
  against P-10's spirit; local.db already exists and is the natural home.

**Out-of-range contract**: a row index past the snapshot length, or a snapshot
that does not exist (never listed), is `Selection::NoMatch` returned on the
row-index path, which the callers already map to a usage error (exit 2) for a
bare integer (targets.rs:351 `is_row_index`; capture maps a stored miss the same
way). No new exit-code semantics.

## D2. CAPTURE readiness column (`ready` vs `needs a target`)

**Decision**: A derived, presentational status. `ready` when the entry has a
launch chain that reduces to a client image (reuse the S054 windows-executables
reduction, `fragcap::targets::entry_windows_clients` per lib.rs:51) OR a resolved
`steam:<app_id>` anchor whose install root is discoverable; `needs a target` when
the launch chain is unresolved (the `unsure` state) and no anchor gives a client.
Never stored; computed at listing time from `launch_entries` + `anchor`.

**Rationale**: FR-002/FR-021/CHK031 want a bounded vocabulary and a stated
derivation, and want every row to be capturable in principle (the column reports
closeness, not validity). Deriving it keeps it honest (P-9): it reflects what the
entry actually carries, not a guess.

## D3. KNOWN evidence column

**Decision**: A neutral, human-readable summary derived, in order, from: (a) the
entry's `evidence` findings (`DetectionFinding` list: engine / anti-cheat / drm
products, e.g. "Denuvo, EasyAntiCheat"); (b) launcher mediation and the resolved
client image ("launcher-mediated -> eso64.exe"); (c) the fallback "no online mode
recorded" / "no launch data known" when neither is present. Presentational; the
phrasing MUST NOT read as a blocker or an endorsement (FR-021).

**Rationale**: Matches the §9.5 sample rows exactly and keeps the column as
neutral evidence (P-9: report what was observed, do not editorialize). The
anti-cheat/drm product names come straight from `DetectionFinding.product`
(signature.rs:202), so the column is observed evidence, not commentary.

## D4. Interactive `targets add` flow and testability seam

**Decision**: Build a dedicated interactive authoring flow behind a small,
injectable prompt seam (a `Prompt`/console trait with a scripted test double,
mirroring the existing `Confirm`/`ScriptedConfirm` pattern at
sources/interactive.rs:24-47). The flow: resolve an executable (path arg, or
Enter to browse via guided path entry) -> run detection on its directory
(`SignatureSet::compile` + `detect`, signature.rs:251/322) and print the
engine/anti-cheat/drm findings inline -> prompt display name and handle (offering
the derived default, handle.rs) -> ask "Is the executable above the process that
holds the sockets? [Y/n/unsure]" -> construct a `TargetEntry` and persist it via
the same `Store::insert_target` path every other source uses (store.rs:488).

The interactive branch runs only when stdin is a terminal; otherwise the existing
flag-driven `add` form handles it (FR-015). The three answers are reachable in
tests by driving the scripted prompt seam, with no real terminal (CHK022).

**Rationale**: P-10 requires one creation operation and one stored form for every
source, so authoring reuses `insert_target` rather than a bespoke write. The
existing `InteractiveSource` (interactive.rs:49) wraps a `DirectorySource` and
confirms discovery *candidates*; the add flow is a different shape (point at one
executable, then one socket-holder decision), so it gets its own seam rather than
being forced through `InteractiveSource`. The console/scripted split is what keeps
it CI-testable, exactly as `ScriptedConfirm` already is.

**Note**: `targets add` today wires no detection and stores `evidence: None`
(targets.rs:315); the inline scan and evidence write are net-new.

## D5. The Y / n / unsure answer and the unresolved launch chain

**Decision**: The socket-holder answer determines the stored `launch_entries`
shape and fidelity, never fabricating a holder (P-9, FR-012):
- **Y** (this exe holds the sockets): store the executable as the resolved
  client/socket-holder launch entry; fidelity `Authored`.
- **n** (a different, unknown process holds them): store the executable as a
  launcher/non-client stage with the socket holder explicitly unresolved;
  fidelity `Authored`; CAPTURE shows `needs a target` until a capture resolves it.
- **unsure**: store the executable with the launch chain marked unresolved (no
  socket-holder claim at all); fidelity `Authored`; CAPTURE shows
  `needs a target`.

The unresolved marker is a field inside the `launch_entries` JSON value
(`launch_entries` is `Option<serde_json::Value>` carried whole, entry.rs:140), so
no schema/DDL change is needed to represent it.

**Rationale**: The `unsure` branch is the reason the prompt exists (spec US2); it
must register a real, honest partial answer, not a guess. `n` and `unsure` differ
in what they assert (n: "not this one"; unsure: "unknown"), and both leave the
holder unresolved for a capture to fill.

## D6. Promotion on capture (the `unsure` -> `verified` write-back)

**Decision**: Add a `Store` update method (e.g.
`promote_target_launch(id, launch_entries, fidelity)`) that rewrites an entry's
`launch_entries` and raises its `fidelity` to `Verified`. When `capture` runs
against a target whose launch chain is unresolved, after the run it takes the
observed dominant socket-holding image from the run's attributions and writes it
back through that method. The promotion logic (given an observed image + an
unresolved entry, produce the resolved entry and the fidelity bump) is a pure
function unit-tested directly; the end-to-end demonstration runs over the existing
fixture pipeline (crates/fragcap/tests/pipeline.rs replays a fixture and resolves
flows with no live driver, per spec section 25.1), so FR-013 is testable without
npcap.

**Rationale**: FR-013 and the issue's acceptance criterion require the first
capture to promote the row. No write-back path exists today: `capture` never
writes the `targets` table (its only local.db write is `merge_launch` into the
catalog `games`/`launch_entries` tables, capture.rs:449), and the sole existing
target mutation is `supersede_with_anchor` (id/anchor only, store.rs:634). This is
the heaviest, highest-risk workstream in the slice and is sequenced last in US2.

**Risk / boundary**: If, during implementation, promotion is found to require a
live capture backend rather than the fixture pipeline, the store method + pure
promotion function still land and are unit-tested, and the live demonstration is
marked Tier 2 (not run in CI), consistent with the S010 socket-table precedent.
This boundary is called out so it is not reported as a passing check it is not.

## D7. Target-entry export / import representation

**Decision** (operator, 2026-08-18): A dedicated JSON array of target-entry
objects, each carrying the entry identity (`stable_id`, `handle`, `name`,
`classification`, `classification_source`, `fidelity`, `anchor`, `launch_entries`,
`install_root`, `evidence`). It is NOT the published capture schema
(`target-schema.v1.json`). `export <selector>` emits a one-element array,
`export` with no selector all entries; `import` reads the array and merges each
element on `stable_id` (update in place; a new id inserts). A non-conforming file
is rejected with diagnostics, not partially applied (FR-019). S055 does not change
or version the published capture schema.

**Rationale**: The published schema's `export` records are catalog games
(export.rs:32-93) and deliberately omit the entry identity (`game.id` is never
emitted; export.rs:59) that merge-on-id round-trip needs, so it cannot carry a
target entry. A dedicated array keeps target export separate from catalog export,
consistent with the S050 two-store split and S054's namespaces-follow-stores rule,
and avoids mutating a durable versioned contract in this slice. `TargetEntry` does
not derive serde today (entry.rs:119), so a `TargetEntry` <-> JSON mapping is
net-new; it lives in `fragcap-targets` next to the store.

**Alternatives considered**: Extending the published schema to carry target
identity (mutates a versioned contract, bumps schema version, conflates two record
kinds) and deferring export/import entirely (P3, lowest value) were both weighed
and declined by the operator in favor of the dedicated array.

**Merge mechanics**: import iterates the array; for each element,
`target_by_stable_id` decides update-vs-insert. Update reuses (or extends) the
store's row-write path; insert reuses `insert_target`. Round-trip identity
(FR-020/SC-005) is asserted by export -> fresh store import -> compare id sets.

## D8. `targets remove`

**Decision**: Add `Store::delete_target(id)` (net-new; no delete exists today).
`targets remove <selector>` resolves the selector, and on `Selection::Ambiguous`
lists the matches and refuses to act (exit 2), consistent with `show`
(targets.rs:369-378) and FR-017/CHK034. The `target_id_aliases` FK is
`ON DELETE CASCADE` (schema.rs:117), so alias rows clean up automatically.

## D9. `is_row_index` duplication

**Decision**: Promote the row-index predicate to one shared location
(`fragcap-targets::selector`) and have `targets.rs` and any new consumer use it,
removing the local duplicate at targets.rs:385. Keeps the row-index rule single-
sourced now that `capture`, `show`, `remove`, and `export` all consult it.

**Rationale**: Minor cleanup that prevents the snapshot change (D1) from having to
be mirrored in two predicates.

## D10. Facade re-exports

**Decision**: Ensure the new/changed target surfaces used by the CLI
(interactive prompt seam, delete, export/import, promotion, snapshot writer) are
reachable through the `fragcap::targets` facade the CLI imports (the CLI does not
import `fragcap-targets` directly; targets.rs:16-20). Verified as a plan-time
task; `InteractiveSource` is exported from `fragcap-targets` (lib.rs:81) but its
facade re-export must be confirmed when wiring.

## Summary of net-new vs reused

| Area | Reused | Net-new |
| --- | --- | --- |
| Listing snapshot | `resolve_positional` row-index branch | `listing_snapshot` table, MIGRATE_5_TO_6, writer + nth reader |
| Hero listing output | `list_default` path, discovery run | CAPTURE/KNOWN derivation + table renderer |
| Interactive add | `insert_target`, handle/identifier, `Confirm`/scripted pattern | prompt seam, inline detection wiring, socket-holder question |
| unsure state | `launch_entries` JSON value | unresolved-chain marker, three-answer mapping |
| Promotion | fixture pipeline, attributions | `promote_target_launch`, capture write-back |
| Export/import | `insert_target`, `target_by_stable_id` | `TargetEntry` <-> JSON, dedicated array, merge-on-id |
| Remove | selector resolution | `Store::delete_target` |
| Steam add | existing `add --steam` | (already present; keep) |
