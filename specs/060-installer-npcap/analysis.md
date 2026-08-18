# Cross-Artifact Analysis: Installer npcap exit-dialog reconciliation (S060)

**Date**: 2026-08-18 | Non-destructive consistency check across spec.md, plan.md,
tasks.md, the contract, and the constitution. Gate: MUST pass, no CRITICAL/HIGH.

## Requirement -> task coverage

| Requirement | Covered by | Status |
| --- | --- | --- |
| FR-001 detect the WinPcap-API wpcap.dll (doctor's marker) | T002 | OK |
| FR-002 pre-check only when absent | T003 | OK |
| FR-003 reworded label states why | T004 | OK |
| FR-004 unchanged policy (page-only, no install) | T004 (action untouched), review T009 | OK |
| FR-005 docs reconciled | T005 | OK |
| FR-006 spec note reconciled + xtask spec | T006 | OK |
| FR-007 no new WiX extension, no Cargo.lock change | T002-T004 (core WiX 3), T007 | OK |
| FR-008 ci green; MSI verified at release | T010, T009 | OK |

Every task maps to a requirement or is setup (T001) / polish (T007-T010). No orphans.

## Consistency findings (grounded in WiX 3)

- **C1 (resolved)** `FileSearch`, `AppSearch`, and `SetProperty` are core WiX 3 (default
  namespace), so no WiX extension is added and the cargo-wix extension set is unchanged;
  re-passing an extension is what breaks `light`, and this change re-passes none. FR-007
  holds.
- **C2 (resolved)** `AppSearch` is a standard action in the UI sequence and runs before
  the dialogs, so `NPCAP_WINPCAP_PRESENT` is resolved before the `SetProperty`
  (`After="AppSearch"`) and before the ExitDialog renders, so the checkbox default
  reflects detection. FR-002 holds.
- **C3 (resolved)** The checkbox stays visible in both states because
  `WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT` stays non-empty; only the default checked state
  (`WIXUI_EXITDIALOGOPTIONALCHECKBOX`) changes. A present-driver machine shows it
  unchecked; the user can still opt in. FR-002/FR-003 hold.
- **C4 (resolved)** The marker matches `doctor` (`system_wpcap = System32\wpcap.dll`,
  the `wpcap_loadable` gate), so the installer and `doctor` agree on "present", including
  the npcap-without-WinPcap-compat edge case (treated as absent by both). FR-001,
  Edge Cases.
- **C5 (resolved)** `[System64Folder]` maps to the native `System32` regardless of MSI
  bitness, matching what `doctor` reads; `[SystemFolder]` could resolve to `SysWOW64`.
  Assumptions.
- **C6 (noted)** The `DoAction` publish already fires only when the box is checked at
  Finish, so no change is needed there; a present-machine unchecked box opens nothing,
  and a user who checks it still reaches the page. FR-004.

## Constitution alignment

- P-1: detection and page-opening only; downloads/bundles/installs nothing; the change
  makes the prompt more conservative. The npcap rule explicitly permits detecting and
  reporting the official download location. PASS.
- P-6: no new term. PASS.
- P-9: the label no longer asserts a requirement the machine has met; pre-selected only
  when actually absent. PASS.
- P-11: docs and spec reconciled; `cargo xtask spec` gated. PASS.
- Pinned artifacts: `wix/main.wxs` is not pinned, so a normal changelog fragment; no
  dated decision. PASS.

## Verification-boundary honesty

The MSI is not built in `cargo xtask ci` (release-only), so the `FileSearch`/
`SetProperty`/rendered-state behavior is verified by WiX-schema review (T009) and
confirmed at release-build time, stated openly in the spec (FR-008), plan, research, and
quickstart. This is the disclosed limitation, not a hidden gap.

## Verdict

No CRITICAL or HIGH findings. All requirements covered, all WiX-feasibility risks
resolved against the WiX 3 authoring model and the existing `main.wxs` patterns. Ready
for `/speckit-implement`.
