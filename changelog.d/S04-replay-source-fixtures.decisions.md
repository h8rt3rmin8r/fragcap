**2026-08-08** Recorded for promotion to specification section 29: section 25.3
requires an attribution script per fixture, declaring what the scripted
attributor returns for each flow at each point in time, without defining one.
Slice S04 defines a line-oriented format with three statements and half-open
time windows. TOML was rejected for now: adopting it means adopting a parser and
its proc-macro dependencies on behalf of S05, which owns the profile schema and
should choose against the profile's requirements rather than inherit a choice
made for a test fixture.

**2026-08-08** Recorded for promotion to specification section 29: section
25.3's `burst.pcap` must both exceed a 65,536 packet buffer and be small, and
those cannot both hold, since a faithful fixture would run to several megabytes.
Backpressure is a relationship between a rate and a capacity rather than a
property of a file, so the fixture supplies the rate and S08's test supplies a
small capacity. The narrowing is deliberate and is recorded rather than applied
silently.

**2026-08-08** The flow attributor seam was left unwidened. It carries no
timestamp, because a real attributor reads a socket table that is already
current, but a scripted one has to be told what "now" is and port reuse depends
on it. Adding a timestamp parameter would have been the easy fix and would have
handed every real implementation a parameter it does not want, for a testing
convenience. The clock is an inherent method on the double instead, and a test
asserts the trait is still what S02 fixed it as.

**2026-08-08** `.gitattributes` gains `*.script text eol=lf`. The existing
wildcard already covered it; listing it matches the file's own stated convention
of naming every format whose parsing depends on line endings, and keeps the
corpus drift check from depending on autodetection.
