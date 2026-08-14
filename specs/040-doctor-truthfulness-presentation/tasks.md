---
description: "Task list for slice 040 doctor truthfulness and presentation"
---

# Tasks: doctor truthfulness and presentation

**Input**: Design documents from `specs/040-doctor-truthfulness-presentation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md,
contracts/doctor-output.md, quickstart.md

**Tests**: Included (the slice is implemented under TDD; the doctor is tested by
hand-built `Inputs` through `checks::run` plus two golden files).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel (different files, no dependency on incomplete work)
- Paths are repository-relative.

---

## Phase 1: Setup

- [x] T001 Confirm the branch `040-doctor-truthfulness-presentation` is checked
  out and `.specify/feature.json` points at this feature; no code scaffolding is
  needed (single-crate change).

## Phase 2: Foundational (blocking prerequisites for all stories)

- [x] T002 Add the `virtual_verdict` function and `VirtualVerdict` type to the
  `fragcap::core` re-export in `crates/fragcap/src/lib.rs` (single-source the
  virtuality heuristic for the CLI; do not replicate `VIRTUAL_PATTERNS`).
- [x] T003 Extend `Inputs` in `crates/fragcap-cli/src/doctor/mod.rs`: add
  `fragcap_version: String`, `binary_path: Option<PathBuf>`,
  `profile_dir: Option<PathBuf>`, `hint_db_path: Option<PathBuf>`, and change the
  loopback signal to a three-valued `loopback: Option<bool>` on `Inputs` (moved
  off `NpcapInfo.loopback_adapter`). Keep `IfaceInfo` shape unchanged.
- [x] T004 Update BOTH duplicated fixtures so the crate compiles with the new
  `Inputs` fields: `ready()` in `crates/fragcap-cli/tests/cli_doctor.rs` and
  `ready_inputs()` in `crates/fragcap-cli/src/doctor/checks.rs`. Use a fixed
  version string (for example "0.0.0-test") so goldens do not churn on release
  bumps (R-2), a representative binary path, profile dir, and hint-db path, and
  `loopback: Some(true)`.

## Phase 3: User Story 1 - Truthful capture-readiness diagnosis (P1)

**Goal**: doctor lists real interfaces and reports loopback truthfully.
**Independent test**: on a live+windows build, `fragcap doctor` lists real
adapters and the loopback line reflects the real state; empty only when genuinely
empty.

- [x] T005 [US1] In `crates/fragcap-cli/src/doctor/checks.rs`, add unit tests for
  the loopback classifier: `Some(true)` -> ok, `Some(false)` -> warn,
  `None` -> warn with a "could not be determined" detail; assert none is a
  blocking failure. (Tests first.)
- [x] T006 [US1] Rewrite the loopback classifier `loopback()` in
  `crates/fragcap-cli/src/doctor/checks.rs` to consume `Inputs.loopback:
  Option<bool>` per data-model.md; remove the old `NpcapInfo.loopback_adapter`
  read.
- [x] T007 [US1] In `crates/fragcap-cli/src/doctor/probe.rs`, delete the
  `npcap_wifi.sys` loopback derivation (line ~260) and the `interfaces:
  Vec::new()` stub (line ~282).
- [x] T008 [US1] In `crates/fragcap-cli/src/doctor/probe.rs`, add a
  `#[cfg(all(feature = "live", windows))]` helper that calls `fragcap::enumerate()`
  and `fragcap::detect_driver()`, mapping each `InterfaceRecord` to `IfaceInfo`
  (name; `addresses.first()` -> addr; `is_up` -> up; `virtual_verdict(&record)`
  -> is_virtual) and setting `loopback` from `DriverReport::loopback_supported`.
  Add the `#[cfg(not(all(feature = "live", windows)))]` fallback returning empty
  interfaces and `loopback: None`. **R-1: this gate is what keeps the default
  `cargo test --workspace` and the Linux `fragcap-core` neutrality build
  compiling; mirror `live_availability()` exactly.**
- [x] T009 [US1] Confirm the interfaces classifier in
  `crates/fragcap-cli/src/doctor/checks.rs` still emits the "no interfaces were
  found" warning only for the empty case and retains the live-absent attribution
  message; add a unit test for a populated interface fixture.

## Phase 4: User Story 2 - A report that identifies itself (P2)

**Goal**: the report opens with version, binary path, profile dir, hint-db path.
**Independent test**: `fragcap doctor` shows the Identity section; `--json` keeps
one record per check.

- [x] T010 [US2] In `crates/fragcap-cli/src/doctor/probe.rs` `gather`, populate
  the identity fields from `env!("CARGO_PKG_VERSION")`,
  `std::env::current_exe()`, `paths::user_profile_dir()`, and
  `paths::default_hint_db_path()`.
- [x] T011 [US2] In `crates/fragcap-cli/src/doctor/checks.rs`, add an `Identity`
  section pushed FIRST in `run()`: one `Check::ok` per fact (version, binary,
  profile dir, hint db); an unresolvable path renders as an ok row whose detail
  says "undetermined". Never blocking.
- [x] T012 [US2] Add unit tests asserting the Identity rows are present, carry
  the fixture values, are `ok`, and do not change `report.exit()`; assert
  `--json` still yields `lines == report.checks.len()`
  (the_json_form_is_one_record_per_check).

## Phase 5: User Story 3 - A report a person can read at a glance (P3)

**Goal**: legible human output; plain when piped or NO_COLOR or JSON.

- [x] T013 [US3] In `crates/fragcap-cli/src/doctor/mod.rs` `render_human`, insert
  a blank line before each section except the first, and wrap detail/remediation
  at a fixed 80 columns with an indented continuation aligned under the detail
  column. Keep this function byte-plain (no color).
- [x] T014 [US3] Add a unit/asserted test that `render_human` output contains no
  ANSI escape bytes and that no line exceeds 80 columns on the ready fixture.
- [x] T015 [US3] In `crates/fragcap-cli/src/commands/doctor.rs`, add a TTY-gated
  presentation layer that colors each status word by severity (ok green, warn
  yellow, skip dim, fail red) and bolds section headings, applied around the
  plain `render_human` output only when `std::io::stdout().is_terminal()` and
  `NO_COLOR` is unset. Never style the `--json` branch. Hand-rolled ANSI, no new
  dependency.

## Phase 6: User Story 4 - Guidance that points the right way (P3)

- [x] T016 [P] [US4] In `crates/fragcap-cli/src/doctor/checks.rs`, extend the
  npcap-absent guidance to note that the recommended analyzer's installer
  (Wireshark) also provides the driver, keeping the official npcap source.
- [x] T017 [P] [US4] In `crates/fragcap-cli/src/doctor/checks.rs` `integration()`,
  reword the not-installed detail away from "copy the fragcap binary into {dir}"
  toward running the forthcoming `fragcap extcap install`, and stop implying the
  analyzer lacks the extcap framework.

## Phase 7: Polish and cross-cutting

- [x] T018 Regenerate the goldens:
  `FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap-cli --test cli_doctor`, then
  read the diff of `crates/fragcap-cli/tests/goldens/doctor-ready.{txt,ndjson}`
  and confirm it shows the Identity section, section spacing, and the extra
  identity records, with no color bytes.
- [x] T019 [P] Add changelog fragments under `changelog.d/`:
  `102-doctor-interfaces.fixed.md`, `103-doctor-loopback.fixed.md`,
  `105-106-doctor-output.changed.md`.
- [x] T020 [P] Add `changelog.d/dependency-taxonomy.decisions.md` recording the
  required/recommended/optional model (npcap required, Wireshark recommended,
  extcap optional), dated, referenced by slice 042.
- [x] T021 Run the R-1 guard explicitly: `cargo test --workspace --locked` (no
  features) and confirm it compiles and passes; if a live-only symbol leaked out
  of the cfg gate, fix it here.
- [x] T022 Run the full gate `cargo xtask ci` in the foreground and watch it to
  completion; resolve any fmt/clippy/test/lint/deps/license finding.

## Dependencies and order

- Setup (T001) -> Foundational (T002-T004) -> stories.
- Foundational is blocking: T003 changes `Inputs`, so T004 must update both
  fixtures in the same step or the crate will not compile.
- US1 (T005-T009) depends only on Foundational. US2 (T010-T012) depends on
  Foundational. US3 (T013-T015) depends on Foundational. US4 (T016-T017) depends
  only on Foundational.
- Polish (T018-T022) runs last; T018 (goldens) must follow all output-affecting
  tasks (T006, T008, T011, T013, T016, T017).

## Parallel opportunities

- T016 and T017 are independent edits to different functions and can proceed in
  parallel [P].
- T019 and T020 are independent new files [P].
- Within a story, the test task precedes its implementation task (TDD).

## MVP scope

User Story 1 alone (truthful interfaces and loopback) is the MVP: it resolves the
two correctness bugs (#102, #103) that make the command untrustworthy. US2, US3,
US4 layer identity, legibility, and guidance on top.
