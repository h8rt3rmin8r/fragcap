<!-- spec-impact: 15.7, 17.1 -->
`fragcap targets` now checks for anti-cheat products installed machine-wide,
outside any title's own install tree. Modern Easy Anti-Cheat installs once per
machine as a Windows service, which a directory scan can never see no matter
how many signature rows exist. When the `EasyAntiCheat_EOS` service is
registered, a `Machine:` section appears once, after the per-target table,
naming it. This is always kept distinct from a title's own reported evidence:
a machine-wide fact is never merged into, or used to infer, any specific
target's row, since presence on the machine does not say which installed
title, if any, is the one that put it there. When the check finds nothing, or
cannot run at all, nothing is printed; no output ever asserts a completed
"no anti-cheat found" scan the tool cannot actually vouch for.

Only Easy Anti-Cheat is checked. BattlEye and Vanguard's machine-wide service
names have not been measured on a real installation, and the probe checks
only what has been verified rather than guessed.
