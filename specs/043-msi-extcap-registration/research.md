# Research: Slice 043

Design is carried forward from `specs/041-extcap-registration/research.md` decision
D-4; this file records the WiX-level choices that implement it.

## R1. The wizard control: a custom dialog, not the exit-dialog checkbox

**Decision**: Add a custom dialog `ExtcapDlg` to the WixUI_InstallDir flow (between
`InstallDirDlg` and `VerifyReadyDlg`) carrying two checkboxes (per-user, machine-wide)
and the scope note and fallback text.

**Rationale**: WixUI_InstallDir has exactly one free optional checkbox, on the exit
dialog, and `main.wxs` already uses it for the npcap download link
(`WIXUI_EXITDIALOGOPTIONALCHECKBOX`). A second opt-in cannot reuse it, so a small
custom dialog is the standard WiX v3 mechanism for a real wizard checkbox. The
backing properties (`REGISTEREXTCAP_USER`, `REGISTEREXTCAP_MACHINE`) are public, so
an administrator can also set them from the command line for a silent install.

**Alternatives considered**: A command-line-only property with no UI (rejected: the
operator asked for a real checkbox); a second exit-dialog checkbox (not supported by
WixUI_InstallDir).

## R2. Impersonation per scope

**Decision**: Per-user registration runs `Impersonate="yes"`; machine-wide runs
`Impersonate="no"`.

**Rationale**: `fragcap extcap install` with no `--dir` resolves the per-user
Wireshark extcap directory from `%APPDATA%` (`paths::extcap_dir`), so it must run as
the installing user, not SYSTEM, or `%APPDATA%` resolves to the wrong profile (D-4).
Machine-wide writes into `[WIRESHARK_DIR]extcap` under Program Files, which needs the
installer's elevation, so it must not impersonate.

## R3. Detecting Wireshark for the machine-wide path

**Decision**: A WiX `RegistrySearch` reads Wireshark's install directory into
`WIRESHARK_DIR`; the machine-wide action is conditioned on `WIRESHARK_DIR` being set,
and targets `[WIRESHARK_DIR]extcap`.

**Rationale**: Wireshark records its install location in the Windows registry under
its own key; a registry search is the standard WiX detection with no external
process. When Wireshark is absent the property is empty and the machine-wide action
is a clean no-op, satisfying the degrade-cleanly edge case. The exact key and value
name are set in the WiX and validated at the manual build (this environment cannot
run the search).

## R4. The custom-action pattern: mirror the Defender exclusion

**Decision**: For each scope, an immediate action sets a property whose name equals
the deferred action's Id to the full command line, and a deferred `WixQuietExec`
action runs it with `Return="ignore"`; a rollback action runs `extcap uninstall`, and
an uninstall action removes the registration.

**Rationale**: `main.wxs` already carries exactly this pattern for the Defender
exclusion (immediate `Set...` action, deferred `WixQuietExec`, `Return="ignore"`,
paired rollback and remove). Reusing it keeps the file consistent, keeps a failure
non-fatal (the install must succeed even if Wireshark is missing or the directory is
locked), and needs no new WiX extension. The command line is built by the immediate
action so the deferred action reads it as CustomActionData, the only way a deferred
action sees per-install values.

## R5. No new WiX extension

**Decision**: Introduce no new `-ext`; use only core WiX plus WixUtilExtension
(`WixQuietExec`), both already linked by cargo-wix.

**Rationale**: The release workflow passes no extension flags on purpose, because
cargo-wix already links WixUIExtension and WixUtilExtension and passing them again
makes light fail with duplicate-table collisions (documented in `main.wxs` and
`release.yml`). Impersonated custom actions and `RegistrySearch` are core WiX, so no
new extension is needed.

## R6. Verification boundary

**Decision**: Verify `cargo xtask ci`, `main.wxs` XML well-formedness, and
release-workflow consistency here; enumerate the WiX build and the per-user and
machine-wide install tests as manual verification at the halt.

**Rationale**: There is no WiX toolchain in this environment (D-4), so the MSI cannot
be built or install-tested. Reporting the build or the registration as verified would
be a P-9 violation; they are reported unverified-pending-manual.
