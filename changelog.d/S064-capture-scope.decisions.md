<!-- spec-impact: none -->
**2026-08-20** Scoped output is the default, which is a user-visible change to
what a capture file contains. A capture taken with this release holds the
target's traffic where the same command before it held everything on the wire.
`--scope all` restores the previous behavior exactly, verified by byte-comparing
its output against a binary built from the previous commit over the same
fixture. The alternative, defaulting to `all` and making scoping opt-in, was
rejected because the tool's stated purpose is process attribution and a user
reading that claim expects the file to hold what they named; shipping the
opposite by default and requiring a flag to get it would keep the claim false.

**2026-08-20** The scope exclusion is counted in two terms, not one. A packet
attributed to a process no profile stage binds is confidently not the capture's.
A packet carrying no attribution at all might have been the target's, excluded
only because the socket table had not yet published the socket that would have
named it. They are reported separately because folding them would hide a
possible real loss inside an intended exclusion, and a scope gate is precisely
where P-4 can fail quietly. A non-zero unresolved count on a real capture is a
signal to investigate; a non-zero out-of-scope count is the feature working.

**2026-08-20** An observe-mode run is not scoped, and says so. Slice S059
promotes a target with an unresolved launch chain to the process it observes
holding the sockets, and that observation counts only packets the write gate
admitted. Scoping such a run to its target would starve the mechanism that
decides what the target is: the gate would reject everything unbound, the file
would be empty, and nothing would be promoted. Measured before the fix: a run
that captured 24 packets and attributed all of them retained none. A run that
does not yet know its target cannot scope to it, so the scope widens for that
run and the run warns that it did, because an operator who asked for a scoped
file and received an unscoped one has to be told and told why. This interaction
was found by running the S059 promotion test, not by the cross-artifact analysis
gate, which is worth recording as a limit of that gate.
