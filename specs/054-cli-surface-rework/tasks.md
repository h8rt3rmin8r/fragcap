# Tasks: CLI surface rework

**Feature**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md) | **Branch**: `054-cli-surface-rework`

**Scope**: `fragcap-cli` argument grammar, dispatch, and assembly seam, plus
documentation, master-spec section 17, glossary, and a changelog fragment. No
capture/attribution/pipeline/sink/core code changes.

**Local build/test** (no MSVC linker here): `cargo +1.96.0-x86_64-pc-windows-gnu
{build,test,clippy} -p fragcap-cli`. CI runs the real MSVC `cargo xtask ci`.

**Tests**: included per story; the spec's success criteria require each capture,
each removal negative, each namespace move, and the presentation behaviour to be
tested.

---

## Phase 1: Setup

- [x] T001 Read the current `fragcap-cli` surface end to end to anchor the refactor: `crates/fragcap-cli/src/cli.rs`, `crates/fragcap-cli/src/lib.rs` (dispatch), `crates/fragcap-cli/src/assemble.rs`, and `crates/fragcap-cli/src/commands/{mod.rs,run.rs,tap.rs,watch.rs,profile.rs,targets.rs,steam.rs,catalog scaffolding}`; confirm the offline-substrate flags and the existing `tests/` that exercise `run`/`tap`/`watch` so they can be retargeted, not deleted.
- [x] T002 Inventory every documentation example and master-spec reference that names `run`, `tap`, `watch`, `profile`, `steam profile`, or a catalog op under `targets`, producing a checklist for the Phase 6 sweep: scan `docs/**`, `README*`, `crates/fragcap-cli/**` help/doc comments, and `docs/fragcap-specification.md` section 17.

---

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: the assembly seam that both `--target` and `--process` capture on. Must land before US1's command wiring.

- [x] T003 Add a single capture-assembly entry point in `crates/fragcap-cli/src/assemble.rs` that produces `EffectiveConfig` from the unified capture arguments plus a synthesized/resolved one-stage `Profile`, folding `effective_config_for_tap` and `effective_config_for_watch` into it (carry `--wait`, path anchors, mode/ring/duration/bounds/scoping orthogonally); keep `effective_config_for_extcap` unchanged. Do not delete the old functions yet if `tap`/`watch` still reference them; mark them for removal in T017.
- [x] T004 Extend `build_launch` in `crates/fragcap-cli/src/assemble.rs` to source the Steam anchor from a resolved target (the `steam:<app_id>` anchor) rather than a profile field, and to return a named usage error when `--launch` is requested with no launchable anchor (the `--process` and anchorless-target cases); preserve the non-Windows "unsupported" refusal.

---

## Phase 3: User Story 1 - One capture verb (Priority: P1) MVP

**Goal**: a single `capture` verb expressing all five section-9.1 captures; the three old verbs and the whole profile-file surface removed.

**Independent test**: the five captures each run through the offline substrate; `run`/`tap`/`watch`/`profile` each fail to parse.

- [x] T005 [US1] Add `CaptureArgs` and the `Capture` command variant to `crates/fragcap-cli/src/cli.rs`: the mutually exclusive required `--target`/`--process` group, `--path`/`--path-regex`, and all orthogonal capture flags (`--mode`, `--ring`, `--duration`, `--wait`, `--launch`, `--out`, `--sink`, `--max-packets`, `--max-bytes`, `--roles`, `--direction`, `--interface`, `--loopback`, `--no-payload`), plus the flattened hidden `OfflineArgs`.
- [x] T006 [US1] Create `crates/fragcap-cli/src/commands/capture.rs`: resolve `--target` via the S051 selector resolution against `local.db` and reduce the entry's `launch_entries` to a client image name (reuse `fragcap-targets` reduction), or take `--process` directly; synthesize the one-stage `Profile` (the existing `tap`/`watch` synthesis, now with optional path anchors); assemble via T003 and run the pipeline. Fold in the attach-to-running behaviour `watch` had.
- [x] T007 [US1] Register `Command::Capture` in the dispatch in `crates/fragcap-cli/src/lib.rs` and add `pub mod capture;` to `crates/fragcap-cli/src/commands/mod.rs`.
- [x] T008 [US1] Remove `Run`/`Tap`/`Watch` variants and `RunArgs`/`TapArgs`/`WatchArgs` from `crates/fragcap-cli/src/cli.rs`, delete `crates/fragcap-cli/src/commands/{run.rs,tap.rs,watch.rs}`, drop their `mod` lines and dispatch arms, and remove the `--profile`/`--install-dir`/`--steam` capture selectors (they do not move onto `capture`).
- [x] T009 [US1] Remove the profile-file surface: the `Profile`/`ProfileArgs`/`ProfileCommand` grammar in `crates/fragcap-cli/src/cli.rs`, `crates/fragcap-cli/src/commands/profile.rs`, the `--profile-dir` global, its dispatch arm and `mod` line, and the file-backed profile provider wiring (retire the provider/dir lookups in `paths.rs`/resolution used only by the removed selector). Keep the internal `Profile` type used for one-stage synthesis.
- [x] T010 [P] [US1] Retarget the existing capture integration tests from `run`/`tap`/`watch` to `capture` in `crates/fragcap-cli/tests/` (and any `fragcap` facade tests), driving the offline substrate.
- [x] T011 [US1] Add tests for the five section-9.1 captures (target+ring, process+ring, process+wait, target+launch, target+give-up-timeout) in `crates/fragcap-cli/tests/`, each asserting parse + assembly + offline run.
- [x] T012 [P] [US1] Add usage-error tests (exit 2): neither `--target` nor `--process`, both together, `--process --launch` (no anchor), and `--mode ring` without `--out`/`--ring`, in `crates/fragcap-cli/tests/`.
- [x] T013 [P] [US1] Add removal-negative tests asserting `run`, `tap`, `watch`, and `profile validate` are rejected as unknown while `schema validate <file>` still parses, in `crates/fragcap-cli/tests/`.

**Checkpoint**: `capture` is the MVP; the tool captures via one verb and the old verbs are gone.

---

## Phase 4: User Story 2 - Namespaces follow the stores (Priority: P2)

**Goal**: `catalog` owns catalog.db ops; `targets` owns local.db ops; `targets add --steam` replaces `steam profile`.

**Independent test**: relocated commands resolve under their new namespace and not the old one; `targets add --steam <id>` registers a local.db target equivalent to the old scaffold.

- [x] T014 [US2] Add a `Catalog` command and `CatalogCommand` subcommands to `crates/fragcap-cli/src/cli.rs` (`import`, `export`, `seed`, `seed-engine`, `seed-signatures`, `update`), moving the arg structs verbatim from `TargetsCommand`; remove those five from `TargetsCommand`.
- [x] T015 [US2] Create `crates/fragcap-cli/src/commands/catalog.rs` by moving the catalog-op handlers out of `crates/fragcap-cli/src/commands/targets.rs`; add the `Catalog` dispatch arm in `lib.rs` and the `mod` line in `commands/mod.rs`; leave `targets.rs` with add/list/show/discover/scan only.
- [x] T016 [US2] Implement `catalog update` in `crates/fragcap-cli/src/commands/catalog.rs`: wire the net-gated published-catalog fetch to the existing S035 seeder (`#[cfg(feature = "net")]`), report honestly when no catalog is reachable, and give the non-net build a clear "requires the net feature" message.
- [x] T017 [US2] Add `--steam <app_id>` to `TargetsAddArgs` in `crates/fragcap-cli/src/cli.rs` and implement it in `crates/fragcap-cli/src/commands/targets.rs`: resolve the installed Steam title (reuse the `steam` enumeration), register via the existing `targets add` path with a `steam:<app_id>` anchor. Remove the `steam profile` subcommand and its handler from `crates/fragcap-cli/src/commands/steam.rs` and the grammar; keep the Steam-specific enumeration ops. Delete the now-dead `effective_config_for_tap`/`for_watch` if unused.
- [x] T018 [P] [US2] Tests: each `catalog` subcommand resolves and writes `catalog.db`; the same op no longer resolves under `targets`; `targets add --steam <id>` produces a local.db target with the expected anchor; `steam profile` no longer resolves. In `crates/fragcap-cli/tests/`.

**Checkpoint**: the command surface teaches the two-store model.

---

## Phase 5: User Story 3 - A discoverable surface (Priority: P3)

**Goal**: grouped `--help`; bare `fragcap` lists targets with a footer.

**Independent test**: `--help` shows the four headings; bare `fragcap` prints the listing plus footer; explicit `targets` omits the footer.

- [x] T019 [US3] Add `help_heading` groupings to the top-level command variants in `crates/fragcap-cli/src/cli.rs` (Capture: `capture`, `replay`; Targets: `targets`, `technologies`, `steam`; Environment: `doctor`, `extcap`; Data: `catalog`, `schema`), hiding nothing.
- [x] T020 [US3] Make the top-level subcommand optional in `crates/fragcap-cli/src/cli.rs` and implement the bare-invocation path in `crates/fragcap-cli/src/lib.rs`: with no subcommand, run the shared `targets` listing renderer and append a `--help` footer; thread a "footer" boolean so explicit `targets` omits it. Handle the empty-`local.db` case as a coherent empty listing plus footer.
- [x] T021 [P] [US3] Tests: `--help` contains the four headings with every command present; bare `fragcap` output equals `fragcap targets` output plus exactly the footer line; empty store still lists + footers. In `crates/fragcap-cli/tests/`.

**Checkpoint**: the surface is discoverable and self-directing.

---

## Phase 6: Polish & cross-cutting concerns

- [x] T022 Sweep every doc example and help/doc comment from the T002 inventory to the new surface: `docs/**`, `README*`, and `crates/fragcap-cli` doc comments; no example may name a removed or relocated-under-old-namespace command (FR-017, SC-006).
- [x] T023 Rewrite master-specification section 17 (the command surface) in `docs/fragcap-specification.md` to describe `capture`, the `catalog`/`targets` namespaces, the retired verbs and profile surface, the grouped help, and the bare invocation (P-11).
- [x] T024 Add glossary entries for any new term this slice introduces (the unified `capture` verb; the namespace-to-store binding if named) under `docs/glossary/`, and regenerate the index via `bash scripts/lint-docs.sh fix` (P-6).
- [x] T025 Write the changelog fragment `changelog.d/S054-cli-surface-rework.added.md` with a leading `<!-- spec-impact: 17 -->` line (add other sections only if their prose names a changed command); describe the collapse, the namespace realignment, and the profile-surface retirement.
- [x] T026 Run the full gate and record evidence: locally the GNU-host `cargo +1.96.0-x86_64-pc-windows-gnu {fmt --check, clippy -p fragcap-cli, test -p fragcap-cli}` plus `cargo xtask {lint,deps,license,spec}` where runnable; note the MSVC-only checks (`clippy --all-features`, full-workspace MSVC test, `cargo deny`, `msrv`) deferred to CI. Confirm `cargo xtask spec` passes (Applies-To + spec-impact).

---

## Dependencies & execution order

- **Setup (T001-T002)** → **Foundational (T003-T004)** → **US1 (T005-T013)** → **US2 (T014-T018)** → **US3 (T019-T021)** → **Polish (T022-T026)**.
- US1 is the MVP and must land first (it removes the verbs the others assume gone).
- US2 and US3 both edit `cli.rs`/`lib.rs`; run US2 before US3 so the final command set (including `catalog`) exists when the help groupings and bare-invocation default are wired.
- Within a story, `[P]` tasks touch different files (mostly separate `tests/` files) and can run together; grammar/dispatch edits to `cli.rs`/`lib.rs` are sequential.

## Parallel opportunities

- US1: T010, T012, T013 (separate test files) parallel after T005-T009 land.
- US2: T018 after T014-T017.
- US3: T021 after T019-T020.

## MVP scope

**User Story 1 alone** (T001-T013) is a shippable MVP: one `capture` verb expressing
all five captures, with the old verbs and the profile-file surface gone. US2 and US3
are coherence and discoverability increments on top.
