# Tasks: doctor recognizes machine-wide extcap registration

**Feature**: 044-doctor-machine-wide | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

TDD: write the failing classifier tests first, then implement until green, then
regenerate goldens. All in `fragcap-cli`; no core/CLI/MSI change.

## Phase 1: Tests first (TDD)

- [ ] T001 [US1] Add unit tests to `crates/fragcap-cli/src/doctor/checks.rs` for the four scope combinations: per-user only -> ok naming current user; system only -> ok naming machine-wide; both -> ok naming both; neither -> optional Warn with the `fragcap extcap install` guidance. Drive them through `Inputs` fixtures (`extcap_installed`/`extcap_dir` + new `extcap_system_installed`/`extcap_system_dir`). These fail to compile until Phase 2 adds the fields.

## Phase 2: Foundational (Inputs + paths)

- [ ] T002 Add `extcap_system_installed: bool` and `extcap_system_dir: Option<PathBuf>` to `Inputs` in `crates/fragcap-cli/src/doctor/mod.rs` (doc-comment them; note `extcap_installed`/`extcap_dir` are the per-user scope).
- [ ] T003 Add `paths::system_extcap_dir()` and `SYSTEM_EXTCAP_DIR_ENV` (`FRAGCAP_SYSTEM_EXTCAP_DIR`) to `crates/fragcap-cli/src/paths.rs`: honor the override on all platforms; Windows default `%ProgramFiles%\Wireshark\extcap`; a conventional Unix default; `None` when undeterminable. Add a unit test for the override.

## Phase 3: US1 - classifier + probe

- [ ] T004 [US1] Rewrite `integration()` in `checks.rs` as a match over (`extcap_installed`, `extcap_system_installed`): ok on either, detail naming the scope(s) and directory; the neither arm keeps the existing optional Warn text verbatim.
- [ ] T005 [US1] Widen `extcap_status()` in `crates/fragcap-cli/src/doctor/probe.rs` to also read `paths::system_extcap_dir()` for `EXTCAP_BINARY`, returning both scopes; set the new fields at both `Inputs {` construction sites in probe.rs (`gather` and `gather_windows`).
- [ ] T006 [US1] Update the `ready_inputs()` fixture in `checks.rs` and the `ready()` fixture in `crates/fragcap-cli/tests/cli_doctor.rs` with the new fields (per-user installed, system not installed, to preserve the ready scenario's meaning).

## Phase 4: Goldens + docs

- [ ] T007 Regenerate the goldens: `FRAGCAP_UPDATE_GOLDENS=1 cargo test -p fragcap-cli --test cli_doctor`, then re-run clean. Read the diff: only the `analyzer extcap` detail wording should change.
- [ ] T008 If the per-user detail wording changed, update the sample doctor block in `site/content/docs/getting-started.mdx` to match, keeping it dash-free.

## Phase 5: Polish & verification

- [ ] T009 Add `changelog.d/044-doctor-machine-wide.changed.md` describing the widened detection.
- [ ] T010 Run `cargo xtask ci` in the foreground to green (fmt, clippy, tests, lint, deps, license, docs). Confirm the default no-feature build and the Linux `fragcap-core` neutrality build still compile (the change is cli-only and platform-tolerant).
- [ ] T011 Confirm `git diff --stat` touches only `fragcap-cli`, the goldens, docs, changelog, and specs (no `fragcap-core`, no CLI/MSI surface).

## Dependencies

- T001 (tests) first, then T002/T003 make them compile, then T004-T006 make them pass.
- T007 after the classifier is final. T008 depends on T007's diff. T009-T011 gate the commit.

## MVP

The classifier change (T001, T002, T004) plus the probe (T003, T005) is the whole
feature; fixtures and goldens (T006, T007) keep the gate green.
