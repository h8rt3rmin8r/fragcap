<!-- spec-impact: 20 -->

### Installer npcap exit-dialog prompt reconciled with the docs (slice S060)

The Windows installer's exit-dialog npcap prompt no longer reads as an unconditional
requirement to a user who already installed the capture driver. It now detects, via
core WiX `FileSearch`es under `[System64Folder]`, the same two markers `fragcap doctor`
probes: npcap's own driver copy (`System32\Npcap\wpcap.dll`) and the WinPcap-API-mode
copy the live backend loads (`System32\wpcap.dll`). The "open the npcap download page"
option is pre-checked unless both are present. On a machine that installed npcap in
WinPcap-API mode as the prerequisite (through Wireshark), the option is shown unchecked
so the user can click Finish straight through; on a machine without npcap, with npcap
but no WinPcap-API mode, or with only a stray legacy WinPcap copy, the option is
pre-checked and Finish opens the vendor's page, matching what `doctor` reports. The
label is reworded to name npcap as the capture driver and frame the page as for a user
who does not already have it, rather than asserting it is "required before capturing
traffic" (issue #133).

The policy is unchanged: fragcap still downloads, bundles, and installs no npcap; the
checkbox only opens the vendor's page, and only when left checked. This is a
presentation fix. No new WiX extension is introduced (`FileSearch`, `AppSearch`, and
`SetProperty` are core WiX 3), and there is no code, dependency, or `Cargo.lock`
change. The getting-started guide and the specification's distribution note are
reconciled with the conditional behavior.
