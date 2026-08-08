# Research: JSON Lines Writer

**Slice**: S07

**Created**: 2026-08-08

**Feature**: [spec.md](spec.md)

## R-1: Serializing JSON without a runtime dependency

**Decision**: Hand-roll the writer. Add `serde_json` as a dev-dependency only,
to validate output in tests.

**Rationale**: The usual argument for a serialization library is that escaping
and number formatting are easy to get subtly wrong. That argument is real, and
it is answered here by the dev-dependency rather than by the runtime one: every
line this writer produces is parsed by `serde_json` in the test suite, so a
escaping defect fails a test rather than reaching a consumer.

The argument against using it for output is specific rather than ideological.
Three properties this slice requires are awkward through `serde_json` and
trivial by hand:

- **Fixed key order.** `serde_json::Value` uses a `BTreeMap` by default, which
  would sort keys alphabetically rather than in the section 13.5 order, and
  `preserve_order` swaps in `IndexMap`, which is another dependency and another
  behavior to hold in mind.
- **An exact decimal timestamp.** `serde_json::Number` constructs from `f64`
  for any non-integer, which is the precise thing FR-012 forbids. Emitting
  `1754500000.123456` exactly requires the `arbitrary_precision` feature, which
  changes `Number`'s representation globally for the crate.
- **Byte-level stability.** Two non-default features, both affecting the shape
  of output, is a configuration to maintain rather than a decision to make once.

Writing JSON is a small problem: emit a brace, emit quoted keys with escaping,
emit integers with `itoa`-shaped formatting the standard library already does,
emit a delimiter. The reader is the hard half, and this slice does not need one.

**Alternatives considered**:

- **`serde_json` at runtime with both features.** Rejected on the reasoning
  above: it is more configuration and more dependency for output whose exact
  bytes are the deliverable.
- **`serde` with derive.** Rejected more strongly. It brings a proc-macro
  dependency tree to describe five record shapes, and the derived output would
  still need the two features above.
- **Hand-roll the validator too.** Rejected. S06 had to hand-write a pcapng
  structural validator because no reader existed to borrow; JSON has excellent
  readers, and declining to use one for verification would throw away the
  strongest independent check available for no gain.

**On the workspace dependency count**: the claim "one external dependency"
becomes "one runtime dependency, one dev-dependency". That is worth stating
plainly rather than eliding, and `cargo deny` (which has still never been
watched to completion) will see `serde_json`, `serde`, `itoa`, `ryu`, and
`memchr`, all MIT or Apache-2.0 and all inside the constitution's allowlist.

## R-2: An exact decimal timestamp without floating point

**Decision**: Build the text from integer arithmetic. Split the nanosecond
count into whole seconds and a microsecond remainder, format the seconds, a
period, and the remainder zero-padded to exactly six digits.

**Rationale**: Measured, in three passes, because the obvious argument for this
decision is wrong and the real one is better.

The first claim written here was that an `f64` cannot hold these values and the
sixth digit would differ today. That was false: for a whole-microsecond
timestamp in the present era both paths render identically, which was checked.

The second claim was that the float path breaks only at large magnitudes,
around the year 2255. True, and nearly irrelevant, because an `i64` nanosecond
timestamp overflows around 2262 anyway.

The actual defect is present-day and is about rounding rather than precision. A
capture driver reports nanoseconds, and the declared resolution is
microseconds, so something has to discard the remainder. This slice floors, as
the pcapng writer does, so a timestamp orders the same way in both outputs.
Dividing into an `f64` and printing to six places rounds. For
1754500000.123456789 the two disagree: `1754500000.123456` against
`1754500000.123457`. That is the same packet described differently by the two
output formats of the same capture, on ordinary input, today.

Integer arithmetic makes the rounding rule explicit and the result exact at
every magnitude, for the same four lines of code.

Flooring rather than rounding, consistently with the pcapng writer, so a
timestamp orders the same way in both outputs.

**Alternatives considered**:

- **Emit the timestamp as a string.** Sidesteps the precision question and
  breaks every consumer that expects a number, including the section 13.5
  example. Rejected.
- **Emit integer nanoseconds.** Exact, and diverges from the documented shape
  for no benefit a consumer asked for. Rejected.
- **Emit an `f64` and accept the loss.** Rejected under P-9: it converts a
  recorded observation into an approximation, invisibly.

## R-3: Reusing the S06 derivation

**Decision**: Call `Annotation::from_packet`, then read its fields. Do not
re-inspect the packet.

**Rationale**: FR-025 of S06 exists for this slice. The presence rules are
non-obvious in three places (identity keys are a pair, role and stage are not a
pair, fidelity comes from the attributor rather than from whether attribution
exists), and each is a place two implementations would diverge. Reading the
derived value means the rules exist once.

The divergences between the formats are then confined to rendering, where they
are visible: `iface` unconditional here, hex lowercase here, endpoint naming
here. That is the correct place for a format difference to live.

**Alternatives considered**: Deriving independently from `CapturedPacket`.
Rejected; it is the exact failure the S06 split was built to prevent, and it
would be undetectable because each writer would be internally consistent.

## R-4: Endpoint naming under an unknown direction

**Decision**: `src` and `dst` when direction is known; `local` and `remote`
when it is not; never both.

**Rationale**: `FlowKey` normalized endpoint position so the key would be
stable across both directions of a conversation, which is what makes it usable
as an attribution lookup key. The cost is that wire order is not stored, and
recovering it needs the direction.

When direction is `unknown`, wire order is not merely unavailable, it is
unknown to the whole pipeline. Emitting `src` and `dst` anyway means choosing
one of two orderings and presenting the choice as an observation. Loopback
traffic makes this concrete rather than theoretical: every packet in
`loopback.pcap` has a flow key and no direction.

Changing the key names is a stronger signal than a null or a flag, because a
consumer writing `select(.dst == ...)` simply does not match a record that never
claimed to know a destination, rather than matching one that guessed.

**Alternatives considered**:

- **Always `src` and `dst`, ordered as local-then-remote when unknown.**
  Rejected under P-9. It is the substitution the `dir=unknown` decision in S06
  already rejected in a different field.
- **Always `local` and `remote`.** Truthful, and it diverges from section 13.5
  for every ordinary packet, discarding information the pipeline does have.
- **Emit both pairs.** Redundant on every record to serve a minority case, and
  invites a consumer to trust whichever it reads first.

## R-5: Determinism

**Decision**: The writer reads no clock, environment, or locale. The interface
set is held in declaration order. Every trailer counter comes from the supplied
snapshot.

**Rationale**: The same reasoning as S06 R-4, and the same failure mode. This
writer has fewer ambient inputs than the pcapng writer did, which is precisely
why the enumeration is worth writing down: a short list looks like it does not
need one.

Two candidates existed. The header declares an interface set, which would vary
if held in a hash-ordered collection. The trailer is the only record whose
content comes from outside the packet stream, so a counter recomputed rather
than read would vary with timing.

**Alternatives considered**: Relying on the design being obviously
deterministic. Rejected; that is what was believed about the Interface
Statistics Block timestamp in S06, which was written from a clock until a
checklist asked.

## R-6: Verification

**Decision**: Three layers.

1. **Unit tests** in `fragcap-sink` over escaping, number formatting, and key
   presence, using synthetic packets.
2. **Third-party parse** of every emitted line with `serde_json`, asserting
   both that it parses and that the parsed values match what went in.
3. **Cross-format agreement** in the facade, asserting that for every packet of
   every fixture the JSON record and the pcapng comment carry the same
   attribution facts.

The third is the one that justifies the slice's design. The first two prove the
stream is well-formed; only the third proves the two formats have not drifted.

**Rationale**: S06's structural validator was hand-written because pcapng had
no reader here. JSON does, so the equivalent check is stronger and cheaper.
Layer three has no S06 equivalent because there was only one format.

**Alternatives considered**: Trusting the shared derivation to make layer three
unnecessary. Rejected: the derivation is shared, but the two renderings are not,
and a rendering can drop a field that the derivation supplied.
