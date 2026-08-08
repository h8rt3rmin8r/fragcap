The JSON Lines writer of specification section 13.5. `fragcap-sink` gains
`JsonLinesWriter`, emitting a header object, one object per packet, and a
trailer object, with a payload-free mode for metadata-only streams.

This is the second output format, and the interesting part is not that it
exists but that it agrees with the first. Section 13.3's pcapng annotation and
section 13.5's JSON object answer the same question about the same packet, and
two independent derivations of "which keys are present" would drift silently,
because each would be internally consistent. S06 split deriving an annotation
from rendering it so there would be one derivation; this slice is the first
consumer of that split, and `crates/fragcap/tests/agreement.rs` checks it over
every packet of every fixture. The goldens catch a format that changed; only
that test catches two formats that drifted apart.

Three differences from the pcapng profile, all deliberate and all confined to
rendering. The interface name appears on every record, because a JSON line is
self-contained by design and a consumer that split the stream would otherwise
lose it, where a pcapng file holds the interface in its container. Hex is
lowercase, following the section 13.5 example, where the annotation
percent-encodes in uppercase following that encoding's convention. And
endpoints are named for what is known about them.

That last one is the slice's one real disagreement with the specification.
Section 13.5's example shows `src` and `dst`, but `FlowKey` normalized endpoint
position to `local` and `remote` so it would be stable across both directions
of a conversation, which means wire order is recoverable only in combination
with the direction. When the direction is undetermined, which is every loopback
packet, wire order is not merely unavailable but unknown to the whole pipeline,
and emitting `src` and `dst` anyway would present a coin flip as an
observation. A record carries `src` and `dst` when direction is known and
`local` and `remote` when it is not, never both, so the key names themselves
say which claim is being made.

Timestamps are exact, and the reasoning was measured rather than assumed. A
float path renders whole-microsecond present-era timestamps correctly, so the
usual argument for avoiding one does not apply as stated. What does apply is
rounding: a capture driver reports nanoseconds, the declared resolution is
microseconds, and this writer floors as the pcapng writer does while dividing
into an `f64` and printing to six places rounds. For 1754500000.123456789 the
two disagree by a microsecond, today, on ordinary input, which would have meant
the two output formats describing one packet differently. The timestamp is
built by integer arithmetic and never passes through a float.

Every counter is in the trailer, present even at zero, so a consumer who never
sees the pcapng file can still tell whether the capture is short and where it
was lost.

No runtime dependency. `serde_json` is added as a dev-dependency and parses
every line the writer emits, which is a stronger independent check than S06
could have for pcapng, where the structural validator had to be hand-written.
The writer itself is hand-rolled because the exact byte shape is the
deliverable: fixed key order and an exact decimal number both require
non-default `serde_json` features that change the crate's behavior globally.
