# Tasks: Anti-cheat detection and machine-scope presence

**Input**: Design documents from `/specs/068-anticheat-machine-scope/`
**Prerequisites**: plan.md, research.md, data-model.md,
`contracts/detection-and-rendering.md`, quickstart.md

Tests are written first per this project's TDD convention; each
implementation task follows its test task.

## Phase 1: Setup

- [X] T001 Re-read `crates/fragcap-profile/src/signature.rs` (lines 213-260,
  296-344, 377-460), `crates/fragcap-targets/src/volume.rs` (lines 130-150,
  the `VolumeInventory`/`FixtureInventory` seam), `crates/fragcap-steam/src/appinfo.rs`
  (lines 46-68, 443-520, 602-852 the fixtures module), `crates/fragcap-steam/src/library.rs`
  (lines 116-200, 275-370), `crates/fragcap/src/discovery.rs` (lines 1-170),
  `crates/fragcap-steam/src/lib.rs` (lines 162-260, the `read_reg_sz`
  registry precedent), and `crates/fragcap-cli/src/commands/targets.rs`'s
  hero listing (`hero_listing`/`render_table` region) to confirm no
  signature has drifted since `research.md`/`data-model.md` were written.

## Phase 2: Foundational (blocking prerequisites)

**Purpose**: The shared merge primitive and the probe seam both User Story
1's combine step (FR-005) and User Story 3 (FR-006/007) depend on.

- [X] T002 [P] Add a failing test in `crates/fragcap-profile/src/signature.rs`'s
  test module asserting a new `pub fn merge_finding` behaves identically to
  today's inline dedup: two findings for the same `(category, product)`
  collapse to the stronger-fidelity one; two findings for different products
  both survive.
- [X] T003 Extract `merge_finding(findings: &mut Vec<DetectionFinding>,
  candidate: DetectionFinding)` from `SignatureSet::detect`'s inline match
  block (lines ~440-451) into a standalone function; have `detect` call it.
  Run the full existing `fragcap-profile` test suite to confirm zero
  behavior change, then confirm T002 passes.
- [X] T004 [P] Add a failing test in a new `crates/fragcap-targets/src/machine_probe.rs`
  asserting `FixtureMachineAntiCheatProbe::new(vec![...]).detect()` returns
  exactly the findings it was constructed with, and that an empty fixture
  returns an empty `Vec`.
- [X] T005 Implement `MachineAntiCheatFinding`, the `MachineAntiCheatProbe`
  trait, and `FixtureMachineAntiCheatProbe` in
  `crates/fragcap-targets/src/machine_probe.rs` per `data-model.md`; export
  the three from `crates/fragcap-targets/src/lib.rs` and from the facade's
  `pub mod targets { pub use fragcap_targets::{ .. } }` block
  (`crates/fragcap/src/lib.rs`), making T004 pass.

**Checkpoint**: `cargo test -p fragcap-profile signature` and `cargo test -p
fragcap-targets machine_probe` green.

---

## Phase 3: User Story 1 - See anti-cheat a title actually ships (Priority: P1)

**Goal**: The signature set matches the measured EAC bootstrapper artifacts;
`EOSSDK-Win64-Shipping.dll` alone never triggers a finding.

**Independent Test**: Extend the fixture install tree with the measured
artifacts and assert `SignatureSet::detect` reports Easy Anti-Cheat; assert
a tree with only the EOSSDK dll reports nothing.

### Tests for User Story 1

- [X] T006 [P] [US1] Add a failing test in `crates/fragcap-targets/tests/signatures.rs`
  building a tree shaped like the issue's measured Division 2 layout
  (`EasyAntiCheat/EasyAntiCheat_EOS_Setup.exe`, `EACLaunch.exe`) and asserting
  `detect` reports an Easy Anti-Cheat finding.
- [X] T007 [P] [US1] Add a failing test in the same file building a tree
  shaped like the measured Arc Raiders layout
  (`Installers/AntiCheatInstaller.exe`) and asserting the same.
- [X] T008 [P] [US1] Add a failing test in the same file building a tree
  containing only `EOSSDK-Win64-Shipping.dll` and asserting `detect` reports
  **no** anti-cheat finding at all (SC-002's standing regression test).
- [X] T009 [P] [US1] Extend `all_markers_tree()` (line 70) with touches for
  `EACLaunch.exe`, `AntiCheatInstaller.exe`, `start_protected_game.exe`, and
  an `EasyAntiCheat_EOS/` directory, so the "detect everything from one
  scan" test (SC-003) still covers every new row.

### Implementation for User Story 1

- [X] T010 [US1] Add six rows to `crates/fragcap-targets/assets/signatures.json`
  under the existing `anti-cheat` category: `filename` rows for
  `EasyAntiCheat*.exe`, `EACLaunch.exe`, `AntiCheatInstaller.exe`,
  `start_protected_game.exe`, and `directory-shape` rows for `EasyAntiCheat/`
  and `EasyAntiCheat_EOS/` (issue #170 explicitly measured that no
  `directory-shape` row for `EasyAntiCheat/` existed at all before this
  slice; tasks.md's earlier draft mis-stated this as already present).
  Making T006, T007, T009 pass; confirm T008 stays passing (no row anywhere
  names `EOSSDK-Win64-Shipping.dll`).
- [X] T011 [US1] Run `cargo test -p fragcap-targets signatures`; confirm
  T006-T009 pass and the pre-existing signature tests (dedup, coverage,
  Appendix B corpus) are unaffected.

**Checkpoint**: User Story 1 independently functional. `fragcap targets`
already reports Easy Anti-Cheat for both measured titles at this point,
through the existing directory-scan rendering path with zero rendering-code
change.

---

## Phase 4: User Story 2 - Corroborate from Steam's own launch metadata (Priority: P2)

**Goal**: The launch-entry classifier produces the same finding shape from
appinfo data alone, narrowly enough to correctly abstain on the issue's own
EAC-disabled counter-example, and merges cleanly with directory-scan
findings.

**Independent Test**: Feed synthetic `SteamLaunchEntry` values through
`classify_launch_entries` in isolation.

### Tests for User Story 2

- [X] T012 [P] [US2] Add a failing test module in a new
  `crates/fragcap-steam/src/anti_cheat.rs` asserting an entry with
  `arguments: Some("-anticheat_settings=SettingsProfile.json --bundle-dir data --release".into())`
  yields one Easy Anti-Cheat finding whose evidence contains
  `-anticheat_settings=`.
- [X] T013 [P] [US2] Add a failing test in the same file asserting an entry
  with `executable: "start_protected_game.exe".into()` (any case) yields an
  Easy Anti-Cheat finding.
- [X] T014 [P] [US2] Add a failing test in the same file asserting an entry
  with `arguments: Some("-force_enable_eac_module -force_enable_eos_sdk -anticheat_settings=Settings_Release_PROD.json".into())`,
  `description: Some("eac-release".into())` yields exactly one Easy
  Anti-Cheat finding (both the arguments rule and the description rule fire
  on this entry; `merge_finding` collapses them, this test proves the
  classifier's own caller-side merge behavior, not just single-rule
  matching).
- [X] T015 [P] [US2] Add a failing test in the same file asserting the
  issue's own measured counter-example, `arguments: Some("-no-eac".into())`,
  `description: Some("Halo: MCC Anti-Cheat Disabled (Mods and Limited Services)".into())`,
  `executable: "mcc\\binaries\\win64\\mcc-win64-shipping.exe".into()`,
  yields **no** finding at all (SC-004's standing regression test).
- [X] T016 [P] [US2] Add a failing test in the same file asserting an empty
  `entries` slice and a slice of entries with every optional field `None`
  both yield an empty `Vec` with no panic.
- [X] T017 [P] [US2] Add a failing test in `crates/fragcap-steam/src/library.rs`'s
  test module (or extend the existing `discovers_titles_across_two_libraries`
  fixture) asserting that an installed title whose appinfo launch entry
  matches the classifier carries the finding on `InstalledTitle.anti_cheat`,
  using the `appinfo::fixtures` encoder
  (`FixtureApp`/`FixtureLaunch`/`appinfo_bytes`) to construct a synthetic
  cache. `FixtureLaunch` needs a `description` field added (it does not have
  one today; `app_node()` at appinfo.rs line 760 needs to emit it) to
  exercise the description-based rule from a fixture.

### Implementation for User Story 2

- [X] T018 [US2] Add `description: Option<String>` to
  `crates/fragcap-steam/src/appinfo.rs`'s `fixtures::FixtureLaunch` (line
  ~610) and emit it in `app_node()` (line ~769), making the fixture able to
  encode a description at all. Add a `FixtureLaunch::windows` builder
  variant or setter for it if the existing `windows(executable)` constructor
  does not already allow setting optional fields (check the existing
  pattern first; extend rather than replace if one exists).
- [X] T019 [US2] Implement `pub fn classify_launch_entries(entries:
  &[SteamLaunchEntry]) -> Vec<DetectionFinding>` in new
  `crates/fragcap-steam/src/anti_cheat.rs` per the exact rules in
  `data-model.md`, making T012-T016 pass.
- [X] T020 [US2] Add `pub anti_cheat: Vec<DetectionFinding>` to
  `InstalledTitle` in `crates/fragcap-steam/src/library.rs`; change
  `appinfo_index`'s value type from `(Option<String>, Option<String>)` to
  `(Option<String>, Option<String>, Vec<DetectionFinding>)` at its build site
  in `discover_in` (computing the third element via
  `anti_cheat::classify_launch_entries(&a.launch)` before `a.launch.first()`
  is read) and its consumption site in `read_manifest`; update both
  functions' type signatures and the module doc comment. Making T017 pass.
- [X] T021 [US2] In `crates/fragcap/src/discovery.rs`'s `SteamSource::discover`,
  after `detect_evidence(...)` produces `evidence`, merge in each of
  `title.anti_cheat.iter().cloned()` via `fragcap_profile::signature::merge_finding`,
  then re-sort `evidence` by the same `(category order, product)` key
  `SignatureSet::detect` uses (extract that sort into a small shared helper
  next to `merge_finding` if that keeps the two call sites from drifting,
  per the same reasoning as T003).
- [X] T022 [US2] Add a failing-then-passing test in `crates/fragcap/tests/`
  (extend the existing Steam-source discovery test file, or add one)
  asserting a synthetic Steam title with both a directory-scan-matchable
  file and an appinfo launch entry matching the classifier produces exactly
  one Easy Anti-Cheat finding on the resulting candidate (FR-005's
  standing regression test).
- [X] T023 [US2] Run `cargo test -p fragcap-steam anti_cheat` and `cargo
  test -p fragcap-steam library` and `cargo test -p fragcap discovery`;
  confirm T012-T017 and T022 pass.

**Checkpoint**: User Story 2 independently functional; the classifier is
correct in isolation and correctly merges with User Story 1's directory-scan
findings.

---

## Phase 5: User Story 3 - Know when anti-cheat is present on the machine itself (Priority: P3)

**Goal**: A real Windows registry-backed probe exists, and `fragcap targets`
renders its result as a distinct, never-per-title machine-scope fact.

**Independent Test**: Inject a fixture probe into the hero-listing render
path and assert the `Machine:` section's presence/absence and shape; assert
no target row changes.

### Tests for User Story 3

- [X] T024 [P] [US3] Add a failing test in `crates/fragcap-cli/src/commands/targets.rs`'s
  test module (or a new one if none exists for this file) asserting a
  render helper, given a non-empty `Vec<MachineAntiCheatFinding>`, produces
  a `Machine:` heading followed by one indented `<product> (<evidence>)`
  line per finding, and given an empty `Vec` produces zero bytes of output.
- [X] T025 [P] [US3] Add a failing test in the same module asserting that
  rendering a machine-scope section does not alter the byte output of
  `render_table`/`print_target` for any target row (construct a case with
  both a target row and a non-empty machine finding, and diff the target
  rows' bytes against a run with an empty machine finding).
- [X] T026 [P] [US3] Add a failing tier-2-style test in `crates/fragcap/tests/`
  gated `#![cfg(all(feature = "targets", windows))]` (matching
  `windows_volumes.rs`) that constructs a real `WindowsMachineAntiCheatProbe`
  and asserts `detect()` returns without panicking (inconclusive on findings
  content, same posture as the volume inventory tier-2 test).

### Implementation for User Story 3

- [X] T027 [US3] Add `features = ["Win32_System_Registry"]` to the
  `windows-sys` dependency line in `crates/fragcap/Cargo.toml`'s
  `[target.'cfg(windows)'.dependencies]` block, with a comment recording
  that this is additive on the already-resolved 0.36 pin (no `Cargo.lock`
  package added), per `research.md`.
- [X] T028 [US3] Implement `WindowsMachineAntiCheatProbe` in new
  `crates/fragcap/src/machine_probe.rs`, `#[cfg(windows)]`-gated, checking
  registry key existence at
  `HKLM\SYSTEM\CurrentControlSet\Services\EasyAntiCheat_EOS` via
  `RegOpenKeyExW`/`RegCloseKey` (existence only, no value read), following
  `fragcap-steam::lib.rs`'s `read_reg_sz` style for the unsafe FFI block.
  Making T026 compilable and passing on a Windows CI runner.
- [X] T029 [US3] Add a render helper (e.g. `render_machine_section`) in
  `crates/fragcap-cli/src/commands/targets.rs` implementing the
  `contracts/detection-and-rendering.md` rendering contract, making T024 and
  T025 pass.
- [X] T030 [US3] Wire the real probe into the hero-listing path
  (`#[cfg(windows)]`-gated call site calling
  `fragcap::WindowsMachineAntiCheatProbe.detect()`, feeding the render
  helper from T029) in `crates/fragcap-cli/src/commands/targets.rs`'s
  `hero_listing` function, after the per-target table and before the
  next-command footer line.
- [X] T031 [US3] Run `cargo test -p fragcap-cli targets` and `cargo test -p
  fragcap targets`; confirm T024, T025 pass (T026 requires a Windows CI
  runner, matching the existing tier-2 posture; do not skip or weaken it to
  make it pass on a non-Windows dev machine).

**Checkpoint**: All three user stories independently functional and
composed correctly: a title's row, the machine-scope section, and their
non-interaction are all covered by standing tests.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T032 [P] Add a "Machine scope" entry to
  `docs/glossary/anti-cheat-and-security.md`, cross-linked with "Detection
  signature" and "Coverage state", explaining the title-scope vs
  machine-scope distinction and why they never merge (P-9), matching the
  file's existing `## Term` / definition / `{: .matters }` / `**See also:**`
  format.
- [X] T033 [P] Update the module doc comments touched by this slice
  (`crates/fragcap-targets/src/machine_probe.rs`, `crates/fragcap-steam/src/anti_cheat.rs`,
  `crates/fragcap/src/machine_probe.rs`, and the `discover_in`/`SteamSource::discover`
  doc comments) to describe the new behavior accurately; none should claim
  BattlEye or Vanguard machine-wide detection, since neither is implemented.
- [X] T034 [P] Add a changelog fragment
  `changelog.d/S068-anticheat-machine-scope.md` (a `fixed` entry for the
  signature/classifier corrections, an `added` entry for the machine-scope
  probe and rendering) per `AGENTS.md`'s changelog-fragment convention.
- [X] T035 Run the full gate set (`cargo fmt --all -- --check`, `cargo
  clippy --all-targets --all-features -- -D warnings`, `cargo test
  --workspace --locked`, `cargo xtask lint`, `cargo xtask deps`, `cargo
  xtask license`, `cargo xtask ci`) in the foreground and confirm every
  step is green before the slice's pre-push halt.

## Dependencies

- Phase 2 (T002-T005) blocks Phase 4 (US2 needs `merge_finding`) and Phase 5
  (US3 needs the probe seam), but not Phase 3 (US1 needs neither).
- Phase 3 (US1) has no dependency on Phase 4 or Phase 5 and is independently
  shippable (spec's own framing: it alone satisfies the issue's headline
  acceptance criterion).
- Phase 4 (US2) depends on Phase 2's `merge_finding` (T003) and on Phase 3's
  signature rows being present only for T022's combined-evidence test to be
  meaningful (it is not a hard code dependency, since the classifier and the
  directory scanner are independent inputs to the same merge step).
- Phase 5 (US3) depends on Phase 2's probe seam (T005) but not on Phase 3 or
  4's findings.
- Phase 6 depends on Phases 3, 4, and 5 all being complete.

## Parallel execution examples

- T002 (profile test) and T004 (targets test) are independent, different
  crates.
- Within Phase 3, T006, T007, T008, T009 are independent additions to one
  test file and should be written together in one pass.
- Within Phase 4, T012-T017 are independent test additions across two files
  (`anti_cheat.rs`, `library.rs`); T018 (fixture field addition) is a
  prerequisite only for T017, not for T012-T016.
- Within Phase 5, T024, T025 (CLI render tests) and T026 (facade tier-2
  test) touch different crates and are independent.
- T032, T033, T034 (Phase 6) are independent of each other and of T035.

## Implementation strategy

**MVP scope**: User Story 1 alone (Phases 1-3) already satisfies the issue's
headline acceptance line ("`arc_raiders` and `tom_clancys_the_division_2`
report Easy Anti-Cheat") with a six-row data change and zero new runtime
code. User Story 2 adds a second, corroborating evidence source; User Story
3 adds the structural fix for titles with no on-disk trace at all. Given the
issue frames all three as one coherent fix for one root cause (anti-cheat
detection is broken), this plan implements all three before the slice's
verification gate, with the phase boundaries above marking where a scope
cut would land if time ran short.
