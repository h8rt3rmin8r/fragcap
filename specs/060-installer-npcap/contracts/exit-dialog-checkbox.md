# Contract: Exit-dialog npcap checkbox

## Detection

A WiX property populated by `AppSearch` over a `FileSearch`:

```xml
<Property Id="NPCAP_WINPCAP_PRESENT" Secure="yes">
  <DirectorySearch Id="System64Dir" Path="[System64Folder]" Depth="0">
    <FileSearch Id="WpcapDll" Name="wpcap.dll" />
  </DirectorySearch>
</Property>
```

- Non-empty (the found path) when `System32\wpcap.dll` exists; empty otherwise.
- Matches `doctor`'s `wpcap_loadable` marker (`gather_windows`).
- Core WiX 3; no extension.

## Checkbox default

```xml
<Property Id="WIXUI_EXITDIALOGOPTIONALCHECKBOX" Secure="yes" />
<SetProperty Id="WIXUI_EXITDIALOGOPTIONALCHECKBOX" Value="1"
             After="AppSearch" Sequence="ui">NOT NPCAP_WINPCAP_PRESENT</SetProperty>
```

- Driver absent: property becomes `1` -> checkbox pre-checked.
- Driver present: property stays unset -> checkbox shown unchecked.
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

- **Present -> not pushed**: a machine with the WinPcap-API driver shows the option
  unchecked and does not open the page unless the user opts in.
- **Absent -> guided**: a machine without it shows the option pre-checked and opens the
  page on Finish, unchanged from today.
- **Policy unchanged (P-1)**: the installer downloads, bundles, and installs no npcap;
  the checkbox only opens the vendor page.
- **No extension added**: `FileSearch`, `AppSearch`, and `SetProperty` are core WiX 3,
  so the cargo-wix extension set is unchanged and `light` does not fail.

## Verification boundary

The MSI is built only at release, so this contract is checked by WiX-schema review and
confirmed at release-build time, not by `cargo xtask ci`.
