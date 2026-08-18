# Phase 0 Research: Installer npcap exit-dialog reconciliation

## Decisions

### D-1. Detect the WinPcap-API `wpcap.dll`, not npcap's own directory copy

**Decision**: The presence marker is `wpcap.dll` in the native system directory
(`[System64Folder]`), the copy the npcap "WinPcap API compatible mode" option installs
into `System32`.

**Why**: `crates/fragcap-cli/src/doctor/probe.rs::gather_windows` distinguishes two
markers: `System32\Npcap\wpcap.dll` (npcap installed at all) and `System32\wpcap.dll`
(the WinPcap-API copy fragcap's delay-loaded live backend actually resolves by name).
fragcap requires the latter (spec 20.3); npcap installed without the compatibility
option cannot be used, and the download page (to reinstall with it) is still the right
destination. Gating on `System32\wpcap.dll` makes the installer agree with `doctor` on
what "present" means, so the two never disagree in front of the user.

**Alternatives rejected**: A `RegistrySearch` for npcap's registry key would report
npcap-installed-at-all, which over-counts the not-usable case and diverges from
`doctor`.

### D-2. `[System64Folder]`, not `[SystemFolder]`

**Decision**: Address the native system directory through `[System64Folder]`.

**Why**: `doctor` reads the real `%SystemRoot%\System32\wpcap.dll` (the 64-bit system
directory). `[SystemFolder]` resolves to `SysWOW64` for a 32-bit MSI on 64-bit Windows,
which would miss the 64-bit `wpcap.dll`; `[System64Folder]` always maps to the native
`System32` regardless of the MSI's own bitness, matching `doctor`.

### D-3. Gate the default with `SetProperty`, keep the checkbox visible

**Decision**: Replace the static `<Property Id="WIXUI_EXITDIALOGOPTIONALCHECKBOX"
Value="1" />` with an unset property plus a `SetProperty` that sets it to `1` only when
the marker is absent, sequenced after `AppSearch` in the UI sequence. The checkbox text
(`WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT`) stays non-empty, so the checkbox remains
visible in both states; only its default checked state changes.

**Why**: The WixUI ExitDialog shows the optional checkbox when its text is non-empty and
initializes its checked state from `WIXUI_EXITDIALOGOPTIONALCHECKBOX`. Setting that
property only when the driver is absent yields pre-checked-when-absent,
unchecked-when-present, while leaving the user free to toggle it. `SetProperty` and
`FileSearch` are core WiX 3 (no extension), so the linker's already-linked extension set
is unchanged (re-passing an extension makes `light` fail with duplicate-symbol errors).
`AppSearch` is a standard action in both sequences, so the marker property is resolved
before the exit dialog renders.

**Alternatives rejected**: Hiding the checkbox entirely when present (blanking the text)
was considered but rejected: keeping it visible-but-unchecked lets a user who wants to
reinstall npcap still reach the page, and is the smaller behavioral change. Conditioning
the `DoAction` publish differently is unnecessary; it already fires only when the box is
checked at Finish.

### D-4. Reword to state the why, not an unconditional requirement

**Decision**: Change the label from "Open the npcap download page (required before
capturing traffic)" to text that names npcap as the capture driver and frames the page
as for a user who does not already have it.

**Why**: The reported confusion is the unconditional "required" reading to a user who
just satisfied the prerequisite. Naming the driver and qualifying "if you do not already
have it" makes the label true in both the pre-checked (absent) and unchecked (present)
states.

## Verification boundary (stated, not hidden)

`cargo xtask ci` does not build the MSI; candle/light run only in the release job. So
the `FileSearch`, the `SetProperty` gating, and the rendered checkbox state are not
exercised by CI. Verification here is: `cargo xtask ci` green (docs and no-regression),
`cargo xtask spec` green (the distribution note lockstep), the docs site build, and a
careful review of the WiX XML against the WiX 3 schema (property/AppSearch/SetProperty
authoring). The install-time behavior is confirmed when the release job builds the MSI.
This is the reason #133 is its own slice rather than folded into a Rust slice.
