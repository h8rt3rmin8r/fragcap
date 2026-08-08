The tier 1 test substrate. `fragcap-capture` gains a classic pcap reader and a
`ReplaySource`; `fragcap-attr` gains a `ScriptedAttributor` driven by a declared
script; and `fragcap/tests/pipeline.rs` puts them together with the S03 parser
over a committed corpus of eight fixtures.

That last file is the point of the slice. Specification section 25.1 has
claimed since S01 that the whole pipeline runs with no capture driver, no
elevated privilege, and no game. It now does, and there is a test that proves
it rather than an architecture that permits it.

No new dependency. A pcap file is a twenty-four byte header and a run of
sixteen-byte records, and the script format is deliberately trivial so that S05
can choose a parser for the profile schema on the profile's merits rather than
inheriting one picked for a test fixture.

Reading is deterministic and says what it skipped. Byte order and timestamp
resolution come from the file's magic number in all four combinations, never
from the host. Four named counters cover the ways a record is not what the file
described: two mean the bytes are absent and stop reading, and two mean the file
contradicts itself about bytes that are present, where the record is delivered
anyway with both its lengths exactly as recorded. Repairing that contradiction
would hide a defect in whatever wrote the file.

The scripted attributor makes port reuse testable, which nothing in the project
could express before: one local endpoint, two processes, two windows of time.
It matches through the same key derivation and wildcard bind rule the real
attributor will use, so it cannot express an attribution the socket table could
never supply, and a test written against it is one S10 has to satisfy. The
attributor seam is unchanged: the clock is a method on the double, not a new
parameter on a trait meant to reach 1.0.0 untouched.

The corpus is generated rather than hand-made, and the generator is the readable
record of what each fixture holds. A drift check runs in the ordinary gate and
fails if a committed file stops matching it, if a capture has no script, if
anything exceeds its size ceiling, or if a fixture stops exercising the
condition section 25.3 states for it. Every address is documentation or
loopback and every payload byte is filler, which is what turns "contains no
session token" from a judgment into an assertion.
