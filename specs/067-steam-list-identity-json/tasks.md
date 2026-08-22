# Tasks: Steam list identity and JSON output

**Input**: Design documents from `/specs/067-steam-list-identity-json/`
**Prerequisites**: plan.md, research.md, data-model.md, contracts/steam-list-cli.md, quickstart.md

Tests are written first per this project's TDD convention (`AGENTS.md`
verification discipline); each implementation task follows its test task.

## Phase 1: Setup

- [X] T001 Read `crates/fragcap-cli/src/commands/steam.rs`,
  `crates/fragcap-cli/src/commands/targets.rs` (for `default_local_store`
  and `render_table`/hero-listing patterns), `crates/fragcap-targets/src/store.rs`
  (for `target_by_anchor`, `listing_snapshot_nth`, `listing_snapshot_len`),
  `crates/fragcap-targets/src/identifier.rs` (for `steam_anchor`), and
  `crates/fragcap-cli/src/doctor/mod.rs` (`render_json` as the JSON Lines
  precedent) to confirm no signature has drifted since `research.md` and
  `data-model.md` were written.

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The store read both user stories depend on.

- [X] T002 [P] Add a failing test `listing_snapshot_position_reverse_lookup`
  in `crates/fragcap-targets/src/store.rs`'s test module (alongside
  `listing_snapshot_round_trips_and_replaces`), asserting: after
  `write_listing_snapshot(&[(1001, "alpha"), (1002, "bravo")])`,
  `listing_snapshot_position(1001) == Ok(Some(1))`,
  `listing_snapshot_position(1002) == Ok(Some(2))`, and
  `listing_snapshot_position(9999) == Ok(None)` for a stable id never
  written.
- [X] T003 Implement `pub fn listing_snapshot_position(&self, stable_id: i64)
  -> Result<Option<usize>, TargetsError>` in `crates/fragcap-targets/src/store.rs`
  next to `listing_snapshot_nth`, querying `SELECT position FROM
  listing_snapshot WHERE stable_id = ?1`, making T002 pass.

**Checkpoint**: `cargo test -p fragcap-targets listing_snapshot` green.

---

## Phase 3: User Story 1 - Read a labeled, joined listing (Priority: P1)

**Goal**: `fragcap steam list` (human mode) shows a header and the
three-state identity (registered+positioned / registered-only /
unregistered) per title, sorted by name, never writing the snapshot.

**Independent Test**: In an in-module unit test, resolve identity and render
against a synthetic `InstalledTitle` list and a real `Store::open_in_memory`
seeded with `insert_target` and `write_listing_snapshot`; assert the header,
the three distinct renderings, and the sort order.

**Testing note (from `/speckit-analyze`)**: `fragcap::steam::discover()`
has no override reachable from `steam list` (unlike `targets discover
--steam-root`), so a CI-machine CLI-integration test cannot inject synthetic
Steam titles. The join/sort/render logic is therefore tested as in-module
`#[cfg(test)]` unit tests in `commands/steam.rs` itself, `InstalledTitle`'s
fields are all `pub`, and `Store::open_in_memory` plus `insert_target` /
`write_listing_snapshot` give a real, controllable store. The existing
`crates/fragcap-cli/tests/cli_steam.rs` (not `tests/steam.rs`) stays for
wiring/exit-code smoke tests only, machine-state-dependent like its current
tests.

### Tests for User Story 1

- [X] T004 [P] [US1] Add a failing `#[cfg(test)] mod tests` (or extend one)
  in `crates/fragcap-cli/src/commands/steam.rs` with a test that: opens
  `Store::open_in_memory()`, inserts three `TargetEntry` values via
  `insert_target` (one whose `stable_id` is then written into a
  `write_listing_snapshot` call, one left out of the snapshot, and one
  `InstalledTitle` app id left with no `TargetEntry` at all), builds three
  matching synthetic `InstalledTitle` values by hand, and asserts
  `resolve_identity` (T007) returns `Positioned`, `Unpositioned`, and
  `Unregistered` respectively for the three.
- [X] T005 [P] [US1] Add a failing test in the same module asserting the
  human-mode render function (T008's rendering half) produces a header line
  first, and that the three identity states render as textually distinct
  `TARGET` cells per `contracts/steam-list-cli.md` (a bare handle never
  appears without either `(#N)` or `(no position)`, and an unregistered row
  never shows a handle at all).
- [X] T006 [P] [US1] Add a failing test in the same module asserting rows
  sort by name (case-insensitive) then app id: build titles named `"beta"`,
  `"Alpha"`, `"alpha"` (app ids `"2"`, `"1"`, `"3"`) and assert the rendered
  order is `Alpha`/`"1"`, `alpha`/`"3"`, `beta`/`"2"` (or an equivalent
  order-preserving assertion over the resolved+sorted list, not the
  rendered string, if that is more direct to assert).
- [X] T007a [P] [US1] Add a failing test in the same module asserting that
  after a resolve-and-render pass, the store's `listing_snapshot` contents
  (read via `listing_snapshot_nth`/`listing_snapshot_position`) are
  unchanged from before the pass, i.e. the function under test never calls
  `write_listing_snapshot`.
- [X] T007b [P] [US1] Add a failing test in the same module asserting that
  when store resolution/open fails (pass a path that cannot be opened, or
  call the fallback path directly with `None`), every title resolves to
  `Unregistered` and one `emitter.warn(...)` call is recorded (assert via a
  captured `Emitter` writing to an in-memory buffer, per FR-008).

### Implementation for User Story 1

- [X] T007 [US1] Define the `SteamListingIdentity` enum (`Positioned {
  stable_id, handle, position }`, `Unpositioned { stable_id, handle }`,
  `Unregistered`) in `crates/fragcap-cli/src/commands/steam.rs`, plus a
  function `resolve_identity(store: &Store, app_id: &str) ->
  Result<SteamListingIdentity, TargetsError>` implementing the derivation in
  `data-model.md` (`target_by_anchor` then, if found,
  `listing_snapshot_position`), making T004 pass.
- [X] T008 [US1] Rewrite `list()` in `crates/fragcap-cli/src/commands/steam.rs`
  to: open the local store via `crate::commands::targets::default_local_store`
  (falling back to the `Unregistered`-for-all-rows path with an
  `emitter.warn(...)` call when resolution or `Store::open` fails, per
  FR-008); resolve each title's `SteamListingIdentity` via T007; sort titles
  by name (case-insensitive), tie-broken by app id; render the header and
  one row per title per the `contracts/steam-list-cli.md` human-mode
  contract. Do not call any snapshot-writing method. Factor the
  resolve+sort step and the render step into functions callable from tests
  without going through `steam::discover()`, making T005, T006, T007a,
  T007b pass.
- [X] T009 [US1] Run `cargo test -p fragcap-cli commands::steam` and `cargo
  test -p fragcap-targets listing_snapshot`; confirm T002, T004, T005, T006,
  T007a, T007b now pass.

**Checkpoint**: User Story 1 independently functional, human-mode `steam
list` is fully correct and testable on its own.

---

## Phase 4: User Story 2 - Consume Steam titles as structured data (Priority: P2)

**Goal**: `fragcap steam list --json` emits one NDJSON record per installed
title carrying app id, name, install directory, and (when available)
handle/stable id/position, honoring the same identity states as User Story 1.

**Independent Test**: In an in-module unit test (same rationale as User
Story 1, see the testing note above), resolve identity against a synthetic
`InstalledTitle` list and a real `Store::open_in_memory`; render as JSON;
parse each line and assert field presence/absence matches the identity
state and zero non-JSON bytes are produced.

### Tests for User Story 2

- [X] T010 [P] [US2] Add a failing test in `crates/fragcap-cli/src/commands/steam.rs`'s
  test module asserting the JSON render function (T014) emits one JSON
  object per line for a set of resolved titles, each line parses as JSON
  (a minimal hand-rolled or `serde_json`-based parse is fine for the test
  since `serde_json` is already a dev/runtime dependency of this crate), and
  every object carries `app_id`, `name`, `install_dir` as strings.
- [X] T011 [P] [US2] Add a failing test in the same module asserting field
  presence/absence per identity state: a `Positioned` title's record has
  `handle` (string), `stable_id` (number), and `position` (number); an
  `Unpositioned` title's record has `handle` and `stable_id` but no
  `position` key; an `Unregistered` title's record has none of the three
  keys.
- [X] T012 [P] [US2] Add a failing test in the same module asserting that
  with zero titles passed in, the JSON render function produces zero bytes
  of output (not the human "no installed titles enumerated" sentence, which
  must not appear in this path at all).
- [X] T013 [P] [US2] Add a failing test in `crates/fragcap-cli/tests/cli_steam.rs`
  asserting that `fragcap steam list --json` never writes a human-readable
  line to stdout regardless of this machine's real Steam/store state (every
  stdout line, if any, must parse as JSON), confirming FR-013's existing
  emitter-routing guarantee holds through the new `json` parameter plumbing.

### Implementation for User Story 2

- [X] T014 [US2] Add a `render_json` (or equivalently named) function in
  `crates/fragcap-cli/src/commands/steam.rs` that, given the resolved
  `(InstalledTitle, SteamListingIdentity)` pairs from User Story 1's
  resolution step, writes one `fragcap::write_json_string`-built NDJSON
  record per title to `out`, matching `contracts/steam-list-cli.md`'s field
  table and `doctor`'s `render_json` construction style
  (`crates/fragcap-cli/src/doctor/mod.rs:360`), making T010-T012 pass.
- [X] T015 [US2] Change `pub fn run(args: &SteamArgs, out: &mut dyn Write,
  emitter: &mut Emitter)` in `crates/fragcap-cli/src/commands/steam.rs` to
  `pub fn run(args: &SteamArgs, json: bool, out: &mut dyn Write, emitter:
  &mut Emitter)`, branching `list()` between the human renderer (T008) and
  the JSON renderer (T014) on `json`, matching `doctor::run`'s existing
  `json: bool` parameter pattern.
- [X] T016 [US2] Update the call site in `crates/fragcap-cli/src/lib.rs`'s
  `dispatch` (`Command::Steam(args) => commands::steam::run(&args, out,
  emitter)`) to pass the ambient `json` flag through:
  `commands::steam::run(&args, json, out, emitter)`.
- [X] T016a [US2] Add a failing test in `crates/fragcap-cli/tests/cli_steam.rs`
  asserting `fragcap steam list --json` exits 2 when no Steam installation
  is found, identical to the existing exit-2 branch
  `steam_list_is_wired_and_not_a_stub` already covers for human mode
  (FR-014's "unaffected by `--json`" requirement, flagged as a coverage gap
  by `/speckit-analyze`), then make it pass (it should already pass once
  T015/T016 land, since `map_steam_error` is unchanged; this task confirms
  it explicitly rather than leaving it implicit).
- [X] T017 [US2] Run `cargo test -p fragcap-cli`; confirm T010-T013 and
  T016a now pass alongside the User Story 1 tests still passing.

**Checkpoint**: Both user stories independently functional; `steam list` and
`steam list --json` both correct.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [X] T018 [P] Update the module doc comment at the top of
  `crates/fragcap-cli/src/commands/steam.rs` to describe the new header,
  three-state identity join, deterministic sort, and `--json` mode,
  replacing the stale "one per line" description.
- [X] T019 [P] Add a changelog fragment
  `changelog.d/S067-steam-list-identity-json.md` (feature entry) describing
  the header, join, sort, and `--json` additions, per `AGENTS.md`'s
  changelog-fragment convention (`cargo xtask changelog --release` later
  consumes it).
- [X] T020 Run the full gate set (`cargo fmt --all -- --check`, `cargo
  clippy --all-targets --all-features -- -D warnings`, `cargo test
  --workspace --locked`, `cargo xtask lint`, `cargo xtask deps`, `cargo
  xtask license`) in the foreground and confirm every step is green before
  the slice's pre-push halt.

## Dependencies

- Phase 2 (T002-T003) blocks both Phase 3 and Phase 4, both stories'
  identity resolution depends on `listing_snapshot_position`.
- Phase 3 (US1, T004-T009) has no dependency on Phase 4.
- Phase 4 (US2, T010-T017) depends on Phase 3's `SteamListingIdentity` type
  and `resolve_identity` function (T007) but not on Phase 3's rendering code
  (T008), the JSON renderer is a sibling of the human renderer, not built
  on top of it.
- Phase 5 depends on Phases 3 and 4 both being complete.

## Parallel execution examples

- T002 (targets store test) can run alongside T004-T006 (cli steam tests)
  once T001's read-only survey is done, different crates, no file overlap.
- Within Phase 3, T004, T005, T006 are independent test additions to the
  same new test file and should be written together in one pass rather than
  three separate edits, but are logically parallel (no dependency between
  them).
- Within Phase 4, T010-T013 are similarly independent additions to the same
  test file.
- T018 and T019 (Phase 5) are independent of each other and of T020.

## Implementation strategy

**MVP scope**: User Story 1 alone (Phases 1-3) is a complete, shippable
improvement, issue #171 without #172. User Story 2 (Phase 4) is additive
and does not require re-touching Phase 3's rendering code. Given both issues
are filed together, cross-referenced, and scoped as one slice (S067 per the
campaign plan), this plan implements both before the slice's verification
gate, but the phase boundary above is where a scope cut would land if
needed.
