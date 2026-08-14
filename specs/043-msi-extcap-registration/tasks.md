# Tasks: MSI extcap registration, both scopes

**Feature**: 043-msi-extcap-registration | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Organized by user story. This is a Windows-installer (WiX) slice: the deliverable
cannot be built or install-tested here, so foreground verification is `cargo xtask
ci`, WiX XML well-formedness, and release-consistency; the MSI build and installs
are manual at the halt. No Rust or CLI change.

## Phase 1: Setup

- [ ] T001 Confirm the registration mechanism needs no Rust change: `extcap install` / `extcap uninstall` / `--dir` already exist (`crates/fragcap-cli/src/commands/extcap.rs`, `paths::extcap_dir`), and the per-user default resolves to `%APPDATA%\Wireshark\extcap`. Record if any gap is found (should be none).

## Phase 2: Foundational (blocking prerequisites)

- [ ] T002 In `crates/fragcap-cli/wix/main.wxs`, add the public properties (`REGISTEREXTCAP_USER`, `REGISTEREXTCAP_MACHINE`) and a `RegistrySearch` resolving Wireshark's install directory into `WIRESHARK_DIR`.
- [ ] T003 Add the custom dialog `ExtcapDlg` (per-user checkbox, machine-wide checkbox, the per-user-scope note, and the `fragcap extcap install` fallback text) and insert it into the WixUI_InstallDir flow between `InstallDirDlg` and `VerifyReadyDlg` with the correct `Publish` NewDialog/Back wiring. Never pre-force registration.

## Phase 3: User Story 1 - Register from the installer, for me (P1)

**Goal**: The per-user opt-in registers via an impersonated action; failures never fail the install.

**Independent test (manual)**: MSI install as a normal user with the option on -> `doctor` extcap row `ok` in the user's `%APPDATA%\Wireshark\extcap`.

- [ ] T004 [US1] Add the per-user custom actions in `main.wxs`, mirroring the Defender pattern: an immediate `SetRegisterExtcapUser` setting the CustomActionData to `"[INSTALLDIR]fragcap.exe" extcap install`, and a deferred `WixQuietExec` `RegisterExtcapUser` with `Impersonate="yes"` and `Return="ignore"`, conditioned on `REGISTEREXTCAP_USER=1`.
- [ ] T005 [US1] Add the per-user rollback (`extcap uninstall`) queued before the deferred action, and an uninstall action that unregisters on product removal; sequence all after `InstallFiles` in `InstallExecuteSequence`.

## Phase 4: User Story 2 - Register for every user on this machine (P2)

**Goal**: The machine-wide option registers into Wireshark's system extcap dir when detected.

**Independent test (manual)**: MSI machine-wide install with Wireshark present -> a second user's `doctor` row `ok`.

- [ ] T006 [US2] Add the machine-wide custom actions in `main.wxs`: an immediate `SetRegisterExtcapMachine` setting the command line to `"[INSTALLDIR]fragcap.exe" extcap install --dir "[WIRESHARK_DIR]extcap"`, and a deferred `WixQuietExec` `RegisterExtcapMachine` with `Impersonate="no"` and `Return="ignore"`, conditioned on `REGISTEREXTCAP_MACHINE=1 AND WIRESHARK_DIR`.
- [ ] T007 [US2] Add the machine-wide rollback and uninstall actions (`extcap uninstall --dir "[WIRESHARK_DIR]extcap"`), sequenced after `InstallFiles`; confirm the action is a clean no-op when `WIRESHARK_DIR` is empty.

## Phase 5: User Story 3 - Skip it and still know what to do (P3)

**Goal**: The installer and docs state the per-user scope and the `fragcap extcap install` fallback.

**Independent test**: Read `ExtcapDlg` text and the docs: both state the scope and the fallback.

- [ ] T008 [P] [US3] Document the installer's optional extcap registration and both scopes in `site/content/docs/reference/cli.mdx` (in or beside the existing `extcap install` section), keeping the slice 042 dependency model (extcap optional; doctor warns).
- [ ] T009 [P] [US3] Note the installer option in `site/content/docs/getting-started.mdx` alongside `fragcap extcap install`, including that per-user registration is for the current user only.

## Phase 6: Polish & Cross-Cutting

- [ ] T010 Add `changelog.d/043-msi-extcap-registration.decisions.md` (dated; main.wxs is release-adjacent and pinned: record both scopes, impersonated per-user action, machine-wide via detected Wireshark extcap dir, ignore-on-failure) and `changelog.d/043-msi-extcap-registration.added.md` (feature line).
- [ ] T011 Verify `main.wxs` is well-formed XML (`python -c "import xml.dom.minidom; xml.dom.minidom.parse('crates/fragcap-cli/wix/main.wxs')"`) and that the release workflow still invokes `cargo wix` over it with no added extension flags.
- [ ] T012 Run `cargo xtask ci` in the foreground; fix any repo-lint finding over the WiX/docs text (no dashes, UTF-8, LF).
- [ ] T013 Walk `checklists/wix.md`: check every automatable box; leave the MANUAL items unchecked and carry them to the pre-push halt as operator verification. Confirm `git diff` touches only WiX, docs, changelog, and specs (no `.rs`/`Cargo`).

## Dependencies & order

- Phase 2 (properties, registry search, dialog) blocks Phases 3 and 4 (the actions
  reference the properties and `WIRESHARK_DIR`).
- Phase 3 and Phase 4 are independent of each other once Phase 2 exists.
- Phase 5 (docs) is independent; T008 and T009 are [P].
- Phase 6 gates the commit; T011 and T012 are the foreground verification.

## Parallel opportunities

- T008 and T009 (docs) can be authored in parallel.
- The per-user (Phase 3) and machine-wide (Phase 4) action blocks can be authored
  independently after Phase 2.

## MVP scope

User Story 1 (per-user opt-in) is the MVP: it makes the integration reachable from
the installer for the common case. User Story 2 (machine-wide) and User Story 3
(skip guidance and docs) complete the slice.

## Manual verification (carried to the halt)

The WiX build and the per-user and machine-wide install tests (SC-001, SC-002,
SC-003) require the WiX toolchain and Windows and are not runnable here; they are
enumerated for the operator at the pre-push halt.
