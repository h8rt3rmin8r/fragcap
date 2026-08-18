# Tasks: Installer npcap exit-dialog reconciliation (S060)

**Feature dir**: `specs/060-installer-npcap/`
**Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

Local checks: `cargo +1.96.0-x86_64-pc-windows-gnu xtask {ci,spec}` and the docs
build. The MSI is not built in CI; the WiX change is verified by schema review and
confirmed at release-build time.

## Phase 1: Setup

- [ ] T001 Re-read `crates/fragcap-cli/wix/main.wxs` (the exit-dialog block ~234-257 and the existing `WIRESHARK_DIR` `RegistrySearch` pattern ~173-181), `crates/fragcap-cli/src/doctor/probe.rs::gather_windows` (~330-351, the `wpcap.dll` markers), the getting-started install paragraph, and the spec distribution note (~3398-3404), so the change mirrors existing idioms and matches `doctor` exactly (no file changes).

## Phase 2: WiX change (#133, US1 + US2)

- [ ] T002 [US1/US2] In `crates/fragcap-cli/wix/main.wxs`, add a `NPCAP_WINPCAP_PRESENT` property backed by a `DirectorySearch`(`[System64Folder]`)/`FileSearch`(`wpcap.dll`), mirroring the `WIRESHARK_DIR` `RegistrySearch` shape. Core WiX 3, no extension.
- [ ] T003 [US1/US2] Replace `<Property Id="WIXUI_EXITDIALOGOPTIONALCHECKBOX" Value="1" />` with an unset property plus a `SetProperty` that sets it to `1` `After="AppSearch" Sequence="ui"` conditioned on `NOT NPCAP_WINPCAP_PRESENT`, so the box is pre-checked only when the driver is absent.
- [ ] T004 [US1/US2] Reword `WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT` to name npcap as the capture driver and qualify the page as for a user who does not already have it (no em/en dashes). Update the surrounding comment (~234-237) to describe the conditional pre-check.

**Checkpoint:** the checkbox is pre-checked only when `System32\wpcap.dll` is absent, the label states why, the action and the no-download/bundle/install policy are unchanged, and no WiX extension is added.

## Phase 3: Docs and spec reconcile

- [ ] T005 In `site/content/docs/getting-started.mdx`, reconcile the "1. Install fragcap" npcap paragraph so it describes the conditional behavior (pre-checked only when npcap is not already present; unchecked and declinable when it is), consistent with the never-downloads/bundles/installs posture.
- [ ] T006 In `docs/fragcap-specification.md` (~3398-3404), reconcile the distribution exit-dialog note to say the prompt is pre-selected only when the npcap driver is absent, still without downloading, bundling, or installing npcap. Run `cargo +1.96.0-x86_64-pc-windows-gnu xtask spec` and `bash scripts/lint-docs.sh check`.

## Phase 4: Polish and verify

- [ ] T007 Encoding sweep: `main.wxs`, `getting-started.mdx`, the spec, and the changelog are UTF-8 without BOM, LF, no em-dashes or en-dashes. Confirm `git diff --stat Cargo.lock` is empty.
- [ ] T008 Add `changelog.d/S060-installer-npcap.fixed.md` (with the `spec-impact: 20` marker) noting the conditional pre-check and reworded label, and that the policy is unchanged.
- [ ] T009 WiX review: read the final `main.wxs` against the WiX 3 schema (property/`AppSearch`/`FileSearch`/`SetProperty`/sequence authoring), confirm no extension is introduced and the `SetProperty` runs before the exit dialog renders. Record that the MSI is built and the behavior confirmed at release time.
- [ ] T010 Run the gate: `cargo +1.96.0-x86_64-pc-windows-gnu xtask ci` (green; does not build the MSI) and the docs site build; confirm green (FR-008, SC-005).

## Dependencies

- T002 blocks T003 (the SetProperty conditions on the property) and T004 shares the block.
- T005/T006 (docs/spec) can proceed in parallel with Phase 2.
- T007-T010 come after the WiX and docs edits.
