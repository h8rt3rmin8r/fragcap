# Tasks: Launch-and-observe promotion (S059)

**Feature dir**: `specs/059-launch-and-observe/`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Local build/test: `cargo +1.96.0-x86_64-pc-windows-gnu ...`. CI runs the MSVC
`--all-features` `cargo xtask ci`. Dependency order: core tally (bottom) ->
targets accessor -> cli resolve/orchestrate/write-back (top) -> tests -> docs/spec.

## Phase 1: Setup

- [ ] T001 Re-read the current attribution site in `crates/fragcap-core/src/pipeline/mod.rs` (~line 899-911), `CaptureStats` in `crates/fragcap-core/src/stats.rs`, the `authoring` module in `crates/fragcap-targets/src/authoring.rs`, `commands/target_resolve.rs` (the S058 seam), `commands/capture.rs::run`, `orchestrator.rs::capture`/`capture_prerecorded`/`capture_live`, and `commands/extcap.rs::capture`, so every edit mirrors existing idioms exactly (no file changes).

## Phase 2: Core dominant-holder tally (fragcap-core, US1/US2 foundation)

- [ ] T002 [US1] In `crates/fragcap-core/src/stats.rs`, add `holder_tally: BTreeMap<Arc<str>, u64>` to `CaptureStats` (import `std::collections::BTreeMap` and `std::sync::Arc`). Document it as additive: a per socket-holding-image count of attributed packets, ordered for determinism, contributing to no drop total, no writer, no completion summary. Keep the existing derives (`Clone, Debug, Default, PartialEq, Eq`).
- [ ] T003 [US1] In `crates/fragcap-core/src/stats.rs`, fold `holder_tally` in `CaptureStats::absorb` (add each key's count from `other`). Do NOT touch `buffer_dropped`/`sink_dropped`/`gate_dropped` (still output-owned). Do NOT add it to `sample()` in a way that changes drop-total tests; the existing drop/conservation tests must stay green unchanged.
- [ ] T004 [US1] In `crates/fragcap-core/src/pipeline/mod.rs`, at the `AttributionState::Resolved` arm (where `packets_attributed` increments), increment `stats.holder_tally` for `packet.attribution`'s `process` (clone the `Arc<str>`; `*entry += 1` via `entry(...).or_default()`). No other arm changes.
- [ ] T005 [US1] Add fragcap-core unit tests in `stats.rs`: (a) `absorb` sums per-image tallies across two stats; (b) adding to `holder_tally` changes neither `fragcap_dropped`, `total_dropped`, nor `lost_anything` (the additive invariant); (c) `CaptureStats` still compares equal/unequal by tally. `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-core` green.
- [ ] T006 [US2] In `crates/fragcap/tests/pipeline.rs` (or the corpus pipeline test), assert the tally rides in `PipelineReport.stats.holder_tally` over a fixture whose flows attribute to a process, and that the goldens are unchanged (the tally leaks into no written byte). `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap` green.

**Checkpoint (core):** the dominant-holder tally exists, is incremented and folded, is additive (no golden/counter change), and rides in the pipeline report.

## Phase 3: Targets accessor (fragcap-targets, US1)

- [ ] T007 [US1] In `crates/fragcap-targets/src/authoring.rs`, add `pub fn observed_executable(entry: &TargetEntry) -> Option<&str>` reading `launch_entries` object keys `observed_exe` (unsure) then `executable` (no). Add a unit test that it returns the stored exe for both a `no`- and an `unsure`-authored entry and `None` for a resolved (array) or empty chain. `cargo +1.96.0-x86_64-pc-windows-gnu test -p fragcap-targets` green.

## Phase 4: Orchestrator CaptureOutcome (fragcap-cli, US1)

- [ ] T008 [US1] In `crates/fragcap-cli/src/orchestrator.rs`, add `pub struct CaptureOutcome { pub exit: Exit, pub observed_holder: Option<Arc<str>> }` and a `fn dominant_holder(stats: &CaptureStats) -> Option<Arc<str>>` (arg-max by count over the ordered `holder_tally`, deterministic tiebreak). Change `capture`, `capture_prerecorded`, and `capture_live` return types to `Result<CaptureOutcome, CliError>`; compute `observed_holder` from `report.stats` on paths with a report, `None` on the no-target-acquired paths and the launch-failure error path (which stays `Err`).
- [ ] T009 [US1] Add an orchestrator unit test for `dominant_holder`: empty -> `None`; single -> that image; two-way tie -> the deterministic (ordered) winner; clear max -> the max.

## Phase 5: Resolve seam + write-back (fragcap-cli, US1/US3)

- [ ] T010 [US1] In `crates/fragcap-cli/src/commands/target_resolve.rs`, add `ResolvedTarget { profile: Profile, promotion: Option<Promotion> }` and `Promotion { target_id: i64, local_db: PathBuf }`. Change `resolve_stored` to return `Result<ResolvedTarget, CliError>`. Existing (yes/steam/`--process`) paths set `promotion: None`.
- [ ] T011 [US1] In `target_resolve.rs`, add the fourth branch in the non-Steam arm: before `entry_windows_clients`, if `fragcap::targets::authoring::launch_is_unresolved(&entry)` and `authoring::observed_executable(&entry)` is `Some(e)`, synthesize the two-stage observe-mode profile from `e` (launcher `exe`, terminal client `descends_from:"launcher"`) via `Profile::parse`, and set `promotion = Some(Promotion { target_id: entry.id, local_db })` (the resolved local store path). A genuinely empty/no-exe entry keeps the existing `[] -> refuse` message.
- [ ] T012 [US1] Add a `synthesize_observe_profile(exe) -> Result<Profile, CliError>` helper in `target_resolve.rs` (mirrors `synthesize_named_profile`, builds the two-stage JSON, validates via `Profile::parse`). Unit tests: it validates, has two stages (launcher + terminal client), the client stage carries `descends_from` and no `exe`, and it is not `authored` fidelity.
- [ ] T013 [US1] In `crates/fragcap-cli/src/commands/capture.rs::run`, take `ResolvedTarget` from `resolve_stored` (the `--process` path still uses `synthesize_named_profile` with no promotion), call `orchestrator::capture` and bind the returned `CaptureOutcome`, then: if `promotion` is `Some` and `outcome.observed_holder` is `Some(image)`, reopen `Store::open(&local_db)` and `promote_target_launch(target_id, &authoring::resolved_client_launch(&image), FidelityTier::Verified)`, emitting a progress line / event naming the promotion. `observed_holder == None` -> no write. Return `outcome.exit`.
- [ ] T014 [US3] In `crates/fragcap-cli/src/commands/extcap.rs::capture`, take `ResolvedTarget` from `resolve_stored`, use `.profile`, drop `.promotion`, and map `orchestrator::capture`'s `CaptureOutcome` to `.exit` (extcap never promotes and ignores `observed_holder`).

**Checkpoint (cli):** an unresolved target resolves an observe-mode profile through the shared seam; `capture` promotes on an observed holder; extcap resolves the same profile and does not write back; the build compiles.

## Phase 6: End-to-end offline tests (fragcap-cli, US1)

- [ ] T015 [US1] Add a process-script fixture under `crates/fragcap-cli/tests/data/` where a launcher (e.g. `launcher.exe`) starts and a child (e.g. `game.exe`, parent = launcher pid) starts and descends from it, plus reuse an attr-script that attributes the fixture's flows to the child pid/image (the udp-gameplay flow owner). Keep image names distinct from any existing golden's.
- [ ] T016 [US1] In `crates/fragcap-cli/tests/cli_capture.rs`, add the promotion acceptance test: register an unresolved target (`targets add ... --exe launcher.exe --socket-holder no`), capture it offline with the launcher+child substrate, assert exit 0 and attributed packets, then reopen the store and assert the target's launch chain is `[{ executable: <child image>, role: client }]` at `verified` fidelity.
- [ ] T017 [US1] Add the no-observe acceptance test (matching US1 scenario 2 "completes without error"): the same registered target captured with the launcher+child process script (so the target IS acquired) but an attr-script that attributes nothing (empty/non-matching), so `holder_tally` stays empty and `dominant_holder` is `None`. Assert exit 0 and that the stored target is unchanged (still unresolved, original fidelity). This is the P-9 branch: acquired and captured, nothing observed, nothing promoted.

## Phase 7: Docs, spec, glossary, polish

- [ ] T018 Add glossary entries (P-6) for "launch-and-observe", "observed socket-holder", and "capture-time promotion" in the master specification glossary, and reconcile sections 17.2 (capture) and 17.7 (targets command) to describe observe-mode capture and capture-time promotion. Run `cargo +1.96.0-x86_64-pc-windows-gnu xtask spec` and `bash scripts/lint-docs.sh check`.
- [ ] T019 If the operator-facing CLI reference or getting-started describes capturing a target, add a short note that a `no`/`unsure` target is now capturable and self-promotes; keep it minimal (docs describe what shipped, P-11).
- [ ] T020 Encoding sweep: every edited/new file is UTF-8 without BOM, LF, no em-dashes or en-dashes (including comments and the new fixture). Confirm `git diff --stat Cargo.lock` is empty (no dependency delta).
- [ ] T021 Add `changelog.d/S059-launch-and-observe.added.md` (with the `spec-impact: 17` marker) noting observe-mode capture of unresolved targets, the dominant-holder tally, and capture-time promotion; state the Tier 2 `steam://run` boundary.
- [ ] T022 Run the full gate: `cargo +1.96.0-x86_64-pc-windows-gnu xtask ci` locally (MSVC `--all-features` gate in CI); confirm green (FR-013, SC-004, SC-005).

## Dependencies

- T002-T004 (core tally) block T006, T008 (the report/holder computation).
- T007 (accessor) blocks T011 (observe branch reads it).
- T008 (CaptureOutcome) blocks T013/T014 (call sites) and T009.
- T010-T012 (resolve seam) block T013/T014.
- T013 (write-back) + T015 (fixture) block T016/T017.
- T018-T022 (docs/spec/polish) come after the code compiles and tests pass.
