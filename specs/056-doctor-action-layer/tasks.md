# Tasks: doctor gains an action layer (--fix)

**Feature**: S056 (issue #143) | **Branch**: `056-doctor-action-layer` | **Spec**:
[spec.md](spec.md) | **Plan**: [plan.md](plan.md)

All source paths are under `crates/fragcap-cli/` unless noted. Tests are Tier 1
(no capture driver, no elevation, no network) unless marked Tier 2. Local build/test
uses `cargo +1.96.0-x86_64-pc-windows-gnu {build,test,clippy} -p fragcap-cli`; the
committing gate is the MSVC `cargo xtask ci`.

## Phase 1: Setup

- [X] T001 Add `--fix` and `--yes` boolean flags to `DoctorArgs` in `src/cli.rs`,
  documenting that `--yes` has no meaning without `--fix` and that `--fix` is an
  interactive mode refused with `--json` or a non-terminal stdout.
- [X] T002 Create module skeletons `src/doctor/action.rs` and `src/doctor/fix.rs` and
  declare them `pub mod action; pub mod fix;` in `src/doctor/mod.rs`.

## Phase 2: Foundational (blocking prerequisites for all user stories)

- [X] T003 In `src/doctor/mod.rs`, add an optional `action: Option<Action>` field to
  `Check`, keeping `Check::ok/warn/skip/fail` signatures unchanged and defaulting the
  field to `None`; add a paired constructor (e.g. `Check::warn_action` /
  `fail_action`) that sets `remediation` and `action` together so they cannot drift
  (FR-004). Do not modify any existing check test.
- [X] T004 [P] In `src/doctor/action.rs`, define `Action { kind, label, net_required,
  degraded }`, `ActionKind` (`ObtainNpcap`, `RelaunchNpcapInstaller`,
  `RelaunchElevated`, `InstallExtcap { scope }`, `FetchCatalog`, `RunDiscovery`),
  `ExtcapScope { User, Machine }`, `ActionOutcome { Performed, Skipped, Degraded,
  Failed { reason } }`, and `Capabilities { net: bool }`.
- [X] T005 [P] In `src/doctor/mod.rs` add `target_entry_count: Option<usize>` to
  `Inputs`; in `src/doctor/probe.rs` gather it by counting registered entries in
  `local.db` (thin, not unit tested), reporting `None` when it cannot be determined
  (never a fabricated zero, P-9). Update the classifier test fixture
  (`ready_inputs`) to set the new field, without changing any existing assertion.
- [X] T006 In `src/doctor/action.rs`, implement the pure
  `offered_actions(report: &Report, caps: Capabilities) -> Vec<Action>`: collect each
  check's `action` in report order, and mark a net-required action `degraded` with a
  fallback label when `caps.net` is false. The output MUST be a subset of the actions
  carried by checks in `report` (FR-003), MUST order a `RelaunchElevated` action first
  when present (FR-014), and MUST surface a degraded catalog action as guidance rather
  than a confirm prompt (FR-016).
- [X] T007 In `src/doctor/fix.rs` (or a small `confirm` submodule), define the
  `ActionConfirm` trait and `ConsoleConfirm` (reads stdin yes/no, mirroring
  `commands/targets.rs::prompt_socket_holder`, default No), `YesConfirm` (always
  true), and `ScriptedConfirm` (scripted answers for tests).

## Phase 3: User Story 1 - Fix what doctor named (Priority: P1) MVP

**Goal**: `doctor --fix` runs the classifier, prints the report, offers each
report-named action under confirmation, performs the confirmed no-network/reused
actions, and prints the updated verdict.

**Independent test**: with an injected report and a `ScriptedConfirm`, confirmed
actions record `Performed`/`Degraded`, declined actions record `Skipped` and change
nothing, and the classifier is re-run for the final verdict, all with no terminal.

- [X] T008 [US1] In `src/doctor/checks.rs`, add the two new additive pure checks and
  attach actions to existing findings: a catalog-store-missing check (from
  `catalog_db_present`, WARN, carrying `FetchCatalog`); a no-target-entries check
  (from `target_entry_count == Some(0)`, WARN, carrying `RunDiscovery`; `None` renders
  undetermined and carries no action); and attach `ObtainNpcap` to the npcap-absent
  fail, `RelaunchNpcapInstaller` to the winpcap-api fail, `RelaunchElevated` to the
  not-elevated privilege warn, and `InstallExtcap` to the not-registered integration
  warn, all via the paired constructor. Neither new check may push a ready machine to
  a failing verdict (FR-001, FR-019).
- [X] T009 [US1] In `src/doctor/fix.rs`, implement the `--fix` driver: run the
  classifier, print the report (existing render), compute `offered_actions`, and if
  empty state nothing to fix and exit 0; else loop offering each action via the seam,
  performing on confirm, recording an `ActionOutcome`, printing it; then re-run the
  classifier and print the updated verdict and return its exit (FR-005, FR-006,
  FR-010, FR-011). (Refusal gates land in US2.)
- [X] T010 [US1] Implement the no-network and reused action performers: `InstallExtcap`
  reusing the `extcap install` path at the chosen scope, `RunDiscovery` reusing the
  S055 discovery composition (`compose_and_discover` / `register_from_discovery`,
  tiers 1 and 2), the degraded `ObtainNpcap` / `RelaunchNpcapInstaller` (open the
  official download page), and the `RelaunchElevated` selection + handoff message
  (the elevation side effect itself is Tier 2). A performer returns an
  `ActionOutcome`.
- [X] T011 [US1] Route `--fix` in `src/commands/doctor.rs` to the driver, passing the
  compile-time `Capabilities { net }` and constructing `ConsoleConfirm` (or
  `YesConfirm` when `--yes`).
- [X] T012 [US1] In `tests/cli_doctor.rs`, drive the action phase with a
  `ScriptedConfirm` and an injected report: assert performed/skipped/degraded/failed
  outcomes, that a declined action changes nothing, and that the verdict is re-run.

## Phase 4: User Story 2 - It can only do what it said (Priority: P2)

**Goal**: the guardrails that make an elevated action layer safe: subset invariant,
refusal with `--json` and non-terminal stdout, `--yes` gating.

**Independent test**: purely testable, no side effects.

- [X] T013 [US2] In `src/commands/doctor.rs`, implement the refusal gates before any
  action: `--fix` + `--json` -> usage error exit 2; `--fix` + non-terminal stdout
  (`std::io::stdout().is_terminal()` false) -> usage error exit 2 (holds with
  `--yes`); `--fix` without `--yes` + non-terminal stdin -> usage error exit 2; `--yes`
  without `--fix` -> usage error exit 2 (FR-007, FR-008, FR-009).
- [X] T014 [US2] In `tests/cli_doctor.rs`, assert the subset invariant: an
  `ActionKind` whose check is absent from the injected report is never in
  `offered_actions` output (FR-003, SC-003).
- [X] T015 [US2] In `tests/cli_doctor.rs`, assert the refusal rules: `--fix --json`
  and `--fix` with a non-terminal stdout both exit 2 and perform no action; `--yes`
  without `--fix` exits 2; `--fix --yes` pre-confirms in an interactive context
  (SC-004).

## Phase 5: User Story 3 - The action catalog (Priority: P3)

**Goal**: the network-dependent primary forms, gated on `net`, degrading in a default
build.

**Independent test**: the selection/degradation is testable from an injected report
and `Capabilities`; the side effects are Tier 2.

- [X] T016 [US3] Implement the net-gated primary performers behind the `net` feature:
  `ObtainNpcap` / `RelaunchNpcapInstaller` fetch the vendor's own signed installer
  from the official location and launch it (storing nothing in a fragcap artifact,
  redistributing nothing, per amended rule 2 and D-1), and `FetchCatalog` reuses
  `catalog update`; each degrades to its no-network form when the feature is absent.
- [X] T017 [US3] In `tests/cli_doctor.rs`, assert each finding maps to the right
  action and that a net-required action is presented in its degraded form when
  `Capabilities { net: false }` and its primary form when true (FR-012, FR-016).

## Phase 6: Polish and cross-cutting concerns

- [X] T018 Amend `.specify/memory/constitution.md` Licensing rule 2 to the
  user-confirmed vendor-installer carve-out (exact text in research.md D-2), bump the
  version 1.2.0 -> 1.3.0, and add a Sync Impact Report entry. Preserve rules 1, 3, 4
  and P-1/P-9.
- [X] T019 [P] Add a `changelog.d/S056-doctor-action-layer.decisions.md` fragment
  recording the npcap license determination (D-1) and the rule-2 amendment (D-2),
  dated 2026-08-18 (SC-005).
- [X] T020 [P] Add a `changelog.d/S056-doctor-action-layer.added.md` fragment
  describing the action layer.
- [X] T021 [P] Add glossary entries for "action layer", "structured action", and
  "action outcome" under `docs/glossary/`, and add them to the glossary index (P-6,
  FR-018).
- [X] T022 Update the master specification's `doctor` section (26.3) and the Licensing
  section to describe the action layer and the amended rule 2, and make
  `cargo xtask spec` pass the lock-step (P-11).
- [X] T023 [P] Update `README.md` (and any getting-started doc) to mention
  `doctor --fix` where `doctor` is described.
- [X] T024 Run the full gate: `cargo xtask ci` green (fmt, clippy all-features,
  workspace tests, lint, deps, license), plus `cargo xtask spec`. Verify text hygiene
  (UTF-8 no BOM, LF, no em/en dashes) on every added file (P-8).

## Dependencies and order

- Setup (T001-T002) -> Foundational (T003-T007) -> user stories.
- US1 (T008-T012) is the MVP and depends only on Foundational.
- US2 (T013-T015) depends on Foundational (T006 for the subset test) and the shell
  routing from US1 (T011); its gates are independent of the action performers.
- US3 (T016-T017) depends on Foundational and the US1 driver (T009-T010) it extends
  with the net-gated forms.
- **Governance precedes the fetch code (C1/I1)**: T018 (amend constitution rule 2)
  and T019 (decisions fragment) MUST land before T016 (the net-gated vendor-installer
  fetch), so no committed code depends on the carve-out before the rule permits it.
  The other Polish tasks (T020-T024) run after the code stories.

## Parallel opportunities

- T004 and T005 are independent (`[P]`): different types/fields in different files.
- T019, T020, T021, T023 are independent docs/fragments (`[P]`).
- Within US1, T008 (checks) and T010 (performers) touch different files and can
  progress in parallel once T003/T004 exist.

## Implementation strategy

MVP is US1 (the offer-and-perform mechanism with the no-network/reused actions),
which delivers "fix what doctor named" end to end for the actions fragcap fully
controls. US2 adds the safety guardrails. US3 adds the network-dependent forms behind
`net`. Governance (T018-T019) is authorized, lands with the slice, and (per C1/I1)
lands before the net-gated fetch code in T016.
