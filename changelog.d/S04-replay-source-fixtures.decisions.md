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

**2026-08-08** Recorded for promotion to specification section 29:
`FlowAttributor::resolve` in section 8.5 now takes the instant the packet was
observed. S02 transcribed it without one, and this slice initially kept it that
way, carrying the clock as an inherent method on the scripted attributor and
arguing that a real attributor reads a socket table that is already current.

Review of pull request 7 refuted that twice. Section 11.4 already says capture
and socket table observation are not synchronized, and that a closing
connection produces final packets processed after the socket has left the
table; that is why the retention window exists, and it means a real attributor
is also answering about the past. Separately, section 8.6 holds the attributor
behind a trait object, so an inherent method is unreachable from the pipeline
and core cannot downcast to a backend without the dependency P-2 and P-3
forbid. Every time-windowed fixture would have sat at the epoch resolving
nothing, and the slice's own test hid that by holding the concrete type.

The change costs one parameter now, with a single real implementor. After S10,
S11, and S12 it would have cost considerably more.

**2026-08-08** `.gitattributes` gains `*.script text eol=lf`. The existing
wildcard already covered it; listing it matches the file's own stated convention
of naming every format whose parsing depends on line endings, and keeps the
corpus drift check from depending on autodetection.
