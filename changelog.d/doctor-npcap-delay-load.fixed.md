<!-- spec-impact: none -->

`fragcap doctor` no longer aborts on a machine where the capture DLL cannot be
loaded. The real-interface enumeration reaches the delay-loaded `wpcap.dll`, and
it was called before that DLL's presence was established, so the first call
raised a delay-load exception (0xC06D007E) and the process exited before doctor
could report the very condition it was checking for. This defeated the delay-load
design, whose whole purpose is to let doctor run and say what to install on
exactly that machine. doctor now probes the live backend only when the
WinPcap-API-compatible `wpcap.dll` the backend actually delay-loads (the System32
copy, installed by npcap's WinPcap API compatibility option) is present; when it
is absent nothing is attempted and the npcap and WinPcap-API checks carry the
remediation, so doctor runs to its normal not-ready exit instead of crashing.
This covers both a machine with no npcap at all and npcap installed without the
compatibility option. It surfaced only in the release build (features `live`,
`socket-table`, `etw`) run without a loadable `wpcap.dll`, which the default test
build does not exercise.
