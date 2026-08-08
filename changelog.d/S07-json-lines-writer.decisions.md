**2026-08-08** Recorded for promotion to specification section 29: section
13.5's example record shows `src` and `dst`, but `FlowKey` in section 8.4
carries `local` and `remote`. The normalization is deliberate and load-bearing,
since it is what makes the key stable across both directions of a conversation
and therefore usable as an attribution lookup, but it means wire order is not
stored and is recoverable only in combination with the direction. Slice S07
emits `src` and `dst` when the direction is known and `local` and `remote` when
it is not, never both. When direction is undetermined, wire order is not merely
unavailable to the writer, it is unknown to the whole pipeline, and choosing an
ordering would present a coin flip as an observation, which P-9 forbids. This
is the same finding as the `dir=unknown` decision in S06, in a different field,
and it is concrete rather than theoretical: every packet in `loopback.pcap` has
a flow key and no direction.

**2026-08-08** Recorded for promotion to specification section 29: the JSON
record carries the interface name unconditionally, where the pcapng annotation
carries it only in a multi-interface capture. Section 13.5's example shows it
unconditionally and section 13.3 marks it conditional, so the two are already
inconsistent in the specification; S07 follows each. The reason is structural
rather than stylistic. A pcapng file holds exactly one Interface Description
Block in the single-interface case, so the key would repeat what the container
states, while a JSON line is self-contained by design and a consumer who split
the stream would lose the interface entirely. Both writers read the same
derivation and differ only in rendering, which is where a format difference
belongs.

**2026-08-08** Timestamps are rendered by integer arithmetic and never pass
through a floating point value. The reasoning was revised twice under
measurement and is worth recording accurately, because two plausible versions
of it are wrong. It is not true that an `f64` renders present-era microsecond
timestamps incorrectly; it does not, and a test built on that claim passed
against both paths. It is also nearly irrelevant that an `f64` loses exactness
above a 53-bit significand around the year 2255, since an `i64` nanosecond
timestamp overflows around 2262 regardless. The actual defect is rounding: a
capture driver reports nanoseconds, the declared resolution is microseconds,
and something must discard the remainder. This writer floors, as the pcapng
writer does, so a timestamp orders the same way in both outputs. Dividing into
an `f64` and printing to six places rounds. For 1754500000.123456789 they
differ by a microsecond, which would be the two output formats of one capture
disagreeing about one packet, today, on ordinary input.

**2026-08-08** `serde_json` is added as a dev-dependency of `fragcap-sink` and
`fragcap`, and the writer is hand-rolled. The workspace claim becomes "one
runtime dependency, one dev-dependency" rather than "one external dependency",
which is stated plainly rather than elided. The writer is hand-rolled because
the exact byte shape is this slice's deliverable and two of its requirements
are non-default `serde_json` features that change the crate's behavior
globally: `preserve_order` for the section 13.5 key order, since `Value` sorts
keys through a `BTreeMap`, and `arbitrary_precision` for an exact decimal, since
`Number` constructs from `f64` for any non-integer. The dev-dependency is taken
for the opposite reason: verification is worth more the less it shares with
what it verifies, and a third-party parser reading every emitted line is a
stronger check than S06 could obtain for pcapng, where the structural validator
had to be written here. The `BTreeMap` behavior was not taken on faith; a key
order test written against parsed values passed regardless of what the writer
emitted, which is how it was confirmed.

**2026-08-08** Carried forward from S06 rather than resolved: section 13.5
specifies the header object as declaring the fragcap version, the session
anchor, and the interface set. The anchor is absent for the same reason it is
absent from the pcapng output. There is no session in this slice, and giving it
a placeholder would leave a consumer unable to distinguish an absent anchor
from a null one that meant something. S08 owns capture start and supplies it to
both formats.
