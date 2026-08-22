<!-- spec-impact: 15.7, 16.2 -->
`fragcap targets` now reports Easy Anti-Cheat for titles that ship it, which it
did not before, even for titles measured shipping a visible EAC bootstrapper
(`EACLaunch.exe`, `AntiCheatInstaller.exe`, an `EasyAntiCheat/` or
`EasyAntiCheat_EOS/` directory). The in-tree signature set previously matched
only the runtime's `.dll`/`.sys` files, none of which the two measured titles
actually ship. `EOSSDK-Win64-Shipping.dll` alone continues to never report
anti-cheat: it is the Epic Online Services SDK, shipped by many titles with no
anti-cheat at all, and remains excluded by every new signature row and by a
standing regression test.

A second evidence source now runs alongside the directory scan: Steam's own
`appinfo.vdf` launch-entry metadata, which fragcap already parses, is
classified for anti-cheat signals in its `arguments` and `description` fields
(the enabling flags and launcher name a genuinely protected launch entry
carries). The classifier matches only specific, unambiguous tokens, never a
broad substring on words like "anti-cheat," so a launch variant that
explicitly disables anti-cheat (measured on a Halo: MCC entry) correctly
reports nothing. When both sources agree on a title, it is reported once, at
the stronger evidence's fidelity.
