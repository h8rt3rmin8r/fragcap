<!-- spec-impact: 20 -->

### Installer npcap exit-dialog prompt reconciled with the docs (slice S060)

The Windows installer's exit-dialog npcap prompt no longer reads as an unconditional
requirement to a user who already installed the capture driver. It now detects
whether the WinPcap-API-mode `wpcap.dll` fragcap loads is present (the same
`System32\wpcap.dll` marker `fragcap doctor` probes, via a core WiX `FileSearch` under
`[System64Folder]`) and pre-checks the "open the npcap download page" option only when
that driver is absent. On a machine that installed npcap as the prerequisite (through
Wireshark), the option is shown unchecked so the user can click Finish straight
through; on a machine without it, the option is pre-checked and Finish opens the
vendor's page as before. The label is reworded to name npcap as the capture driver and
frame the page as for a user who does not already have it, rather than asserting it is
"required before capturing traffic" (issue #133).

The policy is unchanged: fragcap still downloads, bundles, and installs no npcap; the
checkbox only opens the vendor's page, and only when left checked. This is a
presentation fix. No new WiX extension is introduced (`FileSearch`, `AppSearch`, and
`SetProperty` are core WiX 3), and there is no code, dependency, or `Cargo.lock`
change. The getting-started guide and the specification's distribution note are
reconciled with the conditional behavior.
