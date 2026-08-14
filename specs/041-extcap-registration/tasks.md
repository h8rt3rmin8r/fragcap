---
description: "Task list for slice 041 extcap registration"
---

# Tasks: extcap registration

**Input**: Design documents from `specs/041-extcap-registration/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md,
contracts/extcap-cli.md, quickstart.md

**Tests**: Included (TDD). Register/uninstall are covered by integration tests
over a scratch directory; the analyzer protocol non-regression by an explicit
parser test.

## Format: `[ID] [P?] [Story] Description`

---

## Phase 1: Setup

- [x] T001 Confirm the branch `041-extcap-registration` is checked out and
  `.specify/feature.json` points at this feature.

## Phase 2: Foundational (blocking prerequisites)

- [x] T002 Add a public `EXTCAP_BINARY` constant to
  `crates/fragcap-cli/src/paths.rs` (`fragcap.exe` on Windows, `fragcap`
  elsewhere) and change `crates/fragcap-cli/src/doctor/probe.rs` to reference
  `crate::paths::EXTCAP_BINARY` instead of its private copy, so the register
  target and the doctor probe share one name (R-6). Behavior-preserving.

## Phase 3: User Story 1 - Register with one command (P1)

- [x] T003 [US1] In `crates/fragcap-cli/src/cli.rs`, add
  `#[command(subcommand)] pub action: Option<ExtcapAction>` to `ExtcapArgs`, and
  define `ExtcapAction { Install(ExtcapInstallArgs), Uninstall(ExtcapInstallArgs) }`
  and `ExtcapInstallArgs { #[arg(long)] dir: Option<PathBuf> }`. Do not change any
  existing flag.
- [x] T004 [US1] In `crates/fragcap-cli/src/commands/extcap.rs` `run`, dispatch
  `args.action` FIRST (before the protocol-flag checks): `Some(Install)` ->
  install, `Some(Uninstall)` -> uninstall, `None` -> the existing protocol
  dispatch unchanged.
- [x] T005 [US1] Implement `install` in `crates/fragcap-cli/src/commands/extcap.rs`:
  resolve the target dir (`--dir` else `paths::extcap_dir()`; `None` -> error),
  resolve `std::env::current_exe()` (`Err` -> error), `create_dir_all`, copy the
  binary to `<dir>/paths::EXTCAP_BINARY` (overwrite = refresh), and print the
  destination path to `out`. Errors go through the emitter and return a non-zero
  `CliError` (FR-008).
- [x] T006 [US1] In `crates/fragcap-cli/tests/cli_extcap.rs`, add integration
  tests: install into a tempdir via `--dir` creates the binary and prints the
  path; install twice is exit 0 both times (idempotent/refresh); an undetermined
  binary/location or unwritable dir path errors (non-zero) and does not report
  success.

## Phase 4: User Story 2 - Unregister (P2)

- [x] T007 [US2] Implement `uninstall` in `commands/extcap.rs`: remove
  `<dir>/EXTCAP_BINARY` if present (report removed), else report a no-op; both
  exit 0. A remove failure against an existing file errors.
- [x] T008 [US2] Add integration tests: uninstall removes the registered binary
  (exit 0); uninstall when nothing is registered is exit 0 and reported as a
  no-op.

## Phase 5: User Story 4 - Protocol non-regression (P1)

- [x] T009 [US4] In `crates/fragcap-cli/tests/cli_extcap.rs`, add an explicit
  parser-regression test asserting all four analyzer invocations still parse and
  run in the bare top-level form (`--extcap-interfaces`, `--extcap-dlts`,
  `--extcap-config`, `--capture --fifo ...`) now that the `install`/`uninstall`
  subcommands exist, complementing the existing direct-invocation tests.

## Phase 6: User Story 1 end-to-end - doctor agreement (P1)

- [x] T010 [US1] Add an end-to-end test: with `FRAGCAP_EXTCAP_DIR` set to a
  tempdir, `extcap install` then `doctor` reports the analyzer extcap check
  installed; after `extcap uninstall`, `doctor` reports not registered.

## Phase 7: User Story 3 - Installer option (P2) - DEFERRED

Split to a dedicated installer slice on operator direction (2026-08-14) so it
gets a real WiX build and a multi-user install test, an installer checkbox, an
at-install per-user note, and the "otherwise run `fragcap extcap install`"
guidance, with both per-user and machine-wide scopes offered. Not delivered here.

- [~] T011 [US3] DEFERRED: optional extcap registration in
  `crates/fragcap-cli/wix/main.wxs` (dedicated installer slice).
- [~] T012 [US3] DEFERRED: the release-adjacent WiX decisions fragment (dedicated
  installer slice).

## Phase 8: Docs and polish

- [x] T013 [P] Document `fragcap extcap install` / `uninstall` (and `--dir`) in
  `site/content/docs/reference/cli.mdx`, same-change (FR-009).
- [x] T014 [P] Add `changelog.d/104-extcap-registration.added.md` describing the
  new commands for the operator.
- [x] T015 Run `cargo xtask ci` in the foreground to completion; resolve any
  finding. Confirm `cargo test -p fragcap-cli --test cli_extcap` passes including
  the parser-regression and doctor-agreement tests.

## Dependencies and order

- Setup (T001) -> Foundational (T002) -> stories.
- T002 is blocking for T005 and T010 (shared constant / doctor agreement).
- T003 -> T004 -> T005/T007 (grammar before dispatch before behavior).
- US4 (T009) and the e2e (T010) depend on T003-T007.
- MSI (T011/T012) is independent of the test path and cannot be verified in CI;
  its smoke test is surfaced at the pre-push halt.

## Parallel opportunities

- T013 and T014 are independent docs/changelog files [P].
- The MSI work (T011/T012) can proceed in parallel with the doc/changelog tasks.

## MVP scope

User Story 1 (register with one command) plus User Story 4 (protocol
non-regression) is the MVP: it delivers the supported registration path and
proves it did not break the analyzer. Uninstall, the installer option, and docs
layer on top.
