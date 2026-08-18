# Phase 0 Research: Installer npcap exit-dialog reconciliation

## Decisions

### D-1. Require both of `doctor`'s markers, not just the WinPcap-API copy

**Decision**: Suppress the pre-check only when npcap is fully present in the form
fragcap needs: `System32\Npcap\wpcap.dll` (npcap's own driver copy, `NPCAP_INSTALLED`)
**and** `System32\wpcap.dll` (the WinPcap-API-mode copy the loader resolves,
`NPCAP_WINPCAP_API`), both under `[System64Folder]`. Pre-check when either is absent.

**Why**: `crates/fragcap-cli/src/doctor/probe.rs::gather_windows` distinguishes two
markers, and `doctor`'s checks (`checks.rs`) read them independently: the **npcap**
check fails when `System32\Npcap\wpcap.dll` is absent ("npcap is not installed"), and
the **winpcap-api** check fails when npcap is present but `System32\wpcap.dll` is
absent ("reinstall npcap with the WinPcap API-compatible Mode option"). The installer
must agree with `doctor` on when the prerequisite is satisfied, which is only when both
pass. This was refined during PR #160 review (Codex): an earlier draft gated on
`System32\wpcap.dll` alone, which a **legacy WinPcap** install also satisfies (it drops
a `System32\wpcap.dll` with no `Npcap` directory), so it would have left the option
unchecked on a machine `doctor` reports as "npcap is not installed". Requiring
`NPCAP_INSTALLED` as well closes that case, and also correctly pre-checks the
npcap-without-WinPcap-API case (the `Npcap` copy present, the `System32` copy absent).

**Alternatives rejected**: Gating on `System32\wpcap.dll` alone (the loadability
marker) reads a legacy WinPcap as satisfied and diverges from `doctor`. Gating on
`Npcap\wpcap.dll` alone reads npcap-without-compat as satisfied, though fragcap cannot
load it. A `RegistrySearch` for npcap's key would not match `doctor`'s filesystem
markers.

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
