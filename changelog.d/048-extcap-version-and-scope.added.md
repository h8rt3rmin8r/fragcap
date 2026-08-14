`fragcap extcap install` and `fragcap extcap uninstall` gained `--user` and
`--system` scope flags. `--user` (the default when no scope is given) registers
into the per-user Wireshark extcap directory; `--system` registers into the
machine-wide one, resolving the system Wireshark extcap directory for you rather
than requiring you to type it. The existing `--dir` remains an explicit override,
and the three selectors are mutually exclusive. This is the ergonomic form of what
was previously only expressible by pointing `--dir` at the system directory; the
default behavior is unchanged.
