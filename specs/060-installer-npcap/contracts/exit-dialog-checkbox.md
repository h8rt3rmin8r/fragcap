# Contract: Exit-dialog npcap checkbox

## Detection (two markers, mirroring `doctor`)

`doctor` uses two independent markers, and the installer requires both before
treating the npcap prerequisite as satisfied (PR #160 review, Codex):

```xml
<!-- npcap's own driver copy (doctor's npcap check, npcap_present). -->
<Property Id="NPCAP_INSTALLED" Secure="yes">
  <DirectorySearch Id="System64Dir" Path="[System64Folder]" Depth="0">
    <DirectorySearch Id="NpcapDir" Path="Npcap" Depth="0">
      <FileSearch Id="NpcapWpcapDll" Name="wpcap.dll" />
    </DirectorySearch>
  </DirectorySearch>
</Property>

<!-- The WinPcap-API-mode copy the loader resolves by name (doctor's
     winpcap-api check, system_wpcap / winpcap_api_mode). -->
<Property Id="NPCAP_WINPCAP_API" Secure="yes">
  <DirectorySearch Id="System64DirApi" Path="[System64Folder]" Depth="0">
    <FileSearch Id="WpcapDll" Name="wpcap.dll" />
  </DirectorySearch>
</Property>
```

- `NPCAP_INSTALLED` non-empty when `System32\Npcap\wpcap.dll` exists, matching
  `doctor`'s `npcap_present`. Its absence is what `doctor` reports as "npcap is not
  installed".
- `NPCAP_WINPCAP_API` non-empty when `System32\wpcap.dll` exists, matching `doctor`'s
  `system_wpcap` (`winpcap_api_mode`), the copy the delay-loaded backend resolves.
- Requiring both avoids two false-satisfied cases: npcap installed without the
  WinPcap-API option (only the `Npcap` copy), and a legacy WinPcap that left only the
  `System32` copy with no npcap. `doctor` fails one check in each case, so the
  installer must not read either as satisfied.
- Core WiX 3; no extension.

## Checkbox default

```xml
<Property Id="WIXUI_EXITDIALOGOPTIONALCHECKBOX" Secure="yes" />
<SetProperty Id="WIXUI_EXITDIALOGOPTIONALCHECKBOX" Value="1"
             After="AppSearch" Sequence="ui">NOT (NPCAP_INSTALLED AND NPCAP_WINPCAP_API)</SetProperty>
```

- npcap fully present (both markers): property stays unset -> checkbox shown unchecked.
- Otherwise (either marker absent): property becomes `1` -> checkbox pre-checked.
- The checkbox stays visible in both states, because
  `WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT` stays non-empty.

## Label

`WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT` reworded to name npcap as the capture driver and
qualify the page as for a user who does not already have it, e.g. "Open the npcap
download page (fragcap needs the npcap capture driver, if you do not already have it)".

## Action (unchanged)

The existing `LaunchNpcap` `WixShellExec` on `https://npcap.com`, fired by the
ExitDialog Finish `DoAction` publish when `WIXUI_EXITDIALOGOPTIONALCHECKBOX = 1 and NOT
Installed`. It opens the vendor page in the user's browser and nothing else.

## Guarantees

- **Fully present -> not pushed**: a machine with npcap installed in WinPcap-API mode
  (both markers) shows the option unchecked and does not open the page unless the user
  opts in.
- **Absent or unusable -> guided**: a machine without npcap, with npcap but no
  WinPcap-API mode, or with only a legacy WinPcap `System32` copy shows the option
  pre-checked and opens the page on Finish, matching what `doctor` reports.
- **Policy unchanged (P-1)**: the installer downloads, bundles, and installs no npcap;
  the checkbox only opens the vendor page.
- **No extension added**: `FileSearch`, `AppSearch`, and `SetProperty` are core WiX 3,
  so the cargo-wix extension set is unchanged and `light` does not fail.

## Verification boundary

The MSI is built only at release, so this contract is checked by WiX-schema review and
confirmed at release-build time, not by `cargo xtask ci`.
