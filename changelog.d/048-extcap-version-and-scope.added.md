`fragcap extcap install` and `fragcap extcap uninstall` gained `--user` and
`--system` scope flags. `--user` (the default when no scope is given) registers
into the per-user Wireshark extcap directory; `--system` registers into the
machine-wide one, resolving Wireshark's registry-recorded install directory (and
falling back to the Program Files default), the same location `doctor` probes, so
a machine-wide install and `doctor` agree even when Wireshark is installed outside
Program Files. The existing `--dir` remains an explicit override,
and the three selectors are mutually exclusive. This is the ergonomic form of what
was previously only expressible by pointing `--dir` at the system directory; the
default behavior is unchanged.
