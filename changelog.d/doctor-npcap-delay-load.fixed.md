`fragcap doctor` no longer aborts on a machine that has no npcap installed. The
real-interface enumeration reaches the delay-loaded `wpcap.dll`, and it was
called before npcap presence was established, so on a machine without npcap the
first call raised a delay-load exception (0xC06D007E) and the process exited
before doctor could report the very absence it was checking for. This defeated
the delay-load design, whose whole purpose is to let doctor run and say "install
npcap" on exactly that machine. doctor now probes the live backend only when
npcap's `wpcap.dll` is present; when it is absent nothing is attempted and the
npcap check carries the remediation, so doctor runs to its normal not-ready exit
instead of crashing. This surfaced only in the release build (features `live`,
`socket-table`, `etw`) run with no npcap present, which the default test build
does not exercise.
