# Implementation Plan: Header Parsing and Flow Keys

**Branch**: `feat/header-parsing-flow-keys`

**Spec**: [spec.md](spec.md)

**Created**: 2026-08-08

**Slice**: S03 (specification sections 12.5, 12.6)

## Summary

Add a `parse` module to `fragcap-core` that turns a byte slice and a link type
into either a `FlowKey` with an optional `Direction`, or a named reason neither
could be produced. Twelve rejection causes, each with its own counter. Two
undetermined-direction causes, each with its own counter. One bounded fragment
identity table so that a non-initial fragment can be attributed without
reassembling anything.

This is the first slice that computes rather than declares. The three
constraints that shape every decision below are that core stays
platform-neutral (P-2), that every path ending without a flow key is counted
(P-4), and that no path ever guesses (P-9).

## Technical Context

**Language**: Rust, edition 2021, pinned 1.96.0 toolchain, declared minimum
1.82.

**New external dependencies**: none. The parser is arithmetic over a byte
slice and needs nothing the standard library does not have. The workspace's
dependency set is unchanged at one crate, `bytes`.

**Testing**: `cargo test --workspace --locked`. Unit tests colocated with the
modules they cover, plus one integration test binary carrying a counting
global allocator, which must be a separate binary because a global allocator
is per binary and installing one in the unit test build would measure the test
harness as well as the parser.

**Target**: `x86_64-pc-windows-msvc` for the workspace; `fragcap-core`
additionally builds for `x86_64-unknown-linux-gnu`, which is the P-2 proof.

**Project type**: Rust library workspace. This slice touches one crate.

## Constitution Check

| Principle | Bearing on this slice | How it is satisfied |
| --- | --- | --- |
| P-1 Passive observation | Indirect | Parsing reads bytes fragcap was already given. It opens nothing, injects nothing, and hooks nothing. |
| P-2 Core platform-neutral | Directly binding | No new dependency, no I/O, no clock. The interface address set is supplied by the caller rather than queried, which is the one place platform knowledge could have leaked in. `cargo xtask neutral` and `cargo xtask deps` prove it. |
| P-3 Capture and attribution separate | Directly binding | The parser produces the key that attribution looks up and performs no lookup. It names no attributor type and takes no attributor argument. |
| P-4 No silent loss | Directly binding | Twelve rejection counters, one ambiguity counter, one no-local-endpoint counter, one fragment eviction counter. Every path that ends without a flow key advances exactly one, and no parse outcome drops a packet. |
| P-5 Compatibility outranks richness | Not applicable | No output format work in this slice. |
| P-6 Glossary first | Directly binding | Six terms introduced, six entries added to `docs/glossary.md` in this change. |
| P-7 Wrappers stay thin | Not applicable | No wrapper work in this slice. |
| P-8 House standards | Directly binding | SPDX headers, no dashes, `cargo xtask lint` clean. |
| P-9 Instrument does not lie | Directly binding | The parser takes `&[u8]` and cannot write to it. No port is inferred, no direction is chosen when the rule returns two answers, and no endpoint is placed in the local position unless it is on the capturing host. |

No principle requires justification for violation.

One item deserves a note rather than an exception. D-4 accepts a residual
mis-attribution risk from fragment identifier reuse that cannot be detected and
therefore cannot be counted, which puts it outside what P-4 can enforce. It is
recorded as a stated limitation rather than hidden, which is the P-9 obligation
when P-4's mechanism does not reach.

## Key Decisions

### D-1. Parsing lives in `fragcap-core`

**Decision**: A new `parse` module in `fragcap-core`, not in
`fragcap-capture`.

**Rationale**: Section 12.5 says the capture thread parses, and the capture
thread belongs to the pipeline, which specification section 8.2 places in
`fragcap-core`. Putting the parser in `fragcap-capture` would require core to
depend on it, inverting the one-directional graph in section 8.3, and
`cargo xtask deps` would reject the edge.

Nothing about parsing resists the move. It is arithmetic over a byte slice
with no I/O, no platform call, and no clock, so it carries nothing P-2
excludes. Both the replay source in S04 and the live source in S09 feed the
same pipeline, so one home serves both.

**Alternatives**: `fragcap-capture`, rejected on the dependency direction. A
new crate, rejected because section 8.2 fixes the crate topology at eight and
adding a ninth for one module is a change to the architecture of record that
buys nothing.

### D-2. The parser is an owned value with `&mut self`

**Decision**:

```rust
pub struct HeaderParser { /* address set, fragment table, counters */ }

impl HeaderParser {
    pub fn new(addrs: InterfaceAddrs) -> Self;
    pub fn set_interface_addrs(&mut self, addrs: InterfaceAddrs);
    pub fn parse(&mut self, link: LinkType, frame: &[u8]) -> ParseOutcome;
    pub fn apply(&mut self, link: LinkType, packet: &mut CapturedPacket)
        -> ParseOutcome;
    pub fn stats(&self) -> &ParseStats;
}
```

**Rationale**: The fragment table and the counters are per-capture state. A
free function would force the caller to thread three arguments through every
call and would push the table's eviction policy into S08, which is the wrong
slice to own it. `&mut self` rather than interior mutability because the
capture thread owns exactly one parser and does not share it, so a lock would
cost something and protect nothing.

`apply` exists because setting `flow` and `direction` on a `CapturedPacket`
from a `ParseOutcome` is the call site both S04 and S08 will write, and two
copies of three lines is two chances to set one field and forget the other.

**Alternatives**: A free function taking the state by reference, rejected as
above. `&self` with interior mutability, rejected because it buys sharing
nobody needs and hides mutation the capture thread performs on every packet.

### D-3. No local endpoint means no flow key

**Decision**: A flow key is produced only when at least one endpoint is in the
interface address set. When neither is, the outcome is
`Rejected(NoLocalEndpoint)`.

**Rationale**: Section 8.4 defines the key's `local` field as the endpoint on
the capturing host. When neither endpoint is, no such endpoint exists, and
writing one there asserts something untrue, which is the same fabrication
section 8.4 prohibits for UDP remote endpoints. It also buys nothing: a packet
with no local endpoint has no local socket, so no socket table lookup could
resolve it. The key would be false and useless together.

The failure mode this protects against is a stale address set after an address
change. Producing keys would yield a capture that looks like working
attribution finding nothing; producing a counter that fires once per packet
says what actually happened.

**Alternatives**: Canonical ordering into the local position, rejected as
above. Keying on wire order, rejected because it produces two keys for one
conversation and destroys the normalization the key exists for.

### D-4. The fragment identity table is a fixed 256 entry ring

**Decision**: A fixed-size array of 256 slots with a write cursor, drop-oldest
on overflow, linear scan on lookup, no allocation ever.

The table maps a fragment identity to the wire-order protocol and port pair
learned from the first fragment. It stores ports rather than an assembled
`FlowKey`, so that FR-022a's requirement to recompute direction and local
position for every fragment falls out of the design rather than needing to be
remembered.

**Rationale**: 256 entries is generous against the reconnaissance finding that
the focal titles' traffic is predominantly unfragmented, and costs about 16 KB
against the 65,536 packet ring section 12.4 already budgets. Linear scan is
faster than hashing at this size and needs no hasher, and a fixed array is the
only structure that guarantees FR-004 without qualification.

Bounding by entry count rather than by age is deliberate. An age bound needs a
clock, and a clock in `fragcap-core` is a platform surface P-2 excludes. The
packet timestamp cannot substitute, because a replay source's timestamps
advance at whatever rate the fixture was recorded at, so an age bound driven by
them would behave differently under replay than under live capture. That is
precisely the kind of divergence the offline testing strategy exists to avoid.

**Accepted cost**: a sixteen bit IPv4 identifier can be reused for the same
address pair and protocol before its entry is evicted, producing a wrong flow
key. Removing an entry when its datagram's last fragment is observed shortens
the window, and the 256 bound caps how long an unfinished entry survives.
Neither eliminates it, and it is not detectable from the capture, so it cannot
be counted. It is stated in the spec's Known limitation section and recorded
for promotion to specification section 29.

### D-5. The ambiguity ordering rule

**Decision**: When both endpoints are in the address set, the endpoint with
the smaller `(IpAddr, u16)` pair, compared by the standard library's ordering
on `IpAddr` and then by port, is written to the `local` position.

**Rationale**: This is the rule FR-029 requires be documented and the
requirements checklist flagged as owed to this plan. It is total, it is
deterministic, and it depends on nothing but the two endpoints, so both halves
of one loopback conversation produce one key, which SC-005 asserts.

Ordering across address families never fires in practice, because both
endpoints of a conversation share a family, but the comparison must be total,
and the standard library's ordering already is.

Nothing about "smaller sorts local" is meaningful. In the ambiguous case both
endpoints genuinely are local, so the choice is which one to write down and not
a claim about the world. That is why an arbitrary rule is acceptable here and
was rejected for D-3, where it would have been a claim.

### D-6. Rejection causes are one closed enumeration with one counter each

**Decision**: `ParseReject` is a closed enum of twelve variants.
`ParseStats` carries one `u64` per variant, plus `direction_ambiguous` and
`fragment_evicted`, and derives every total.

Closed rather than `#[non_exhaustive]`, unlike the error enums S02 declared. An
error enum is extended by later slices adding failure modes; this enumeration
is the complete set of ways this parser can decline, and it changes only when
the parser changes, in which case its counter must be added deliberately in the
same edit. Closing it is what makes "add a discard path, add a counter" a
compile error rather than a review note.

**Rationale**: P-4 wants a counter per cause and a remedy per counter. The
twelve are separated exactly where the remedy differs: a short header means
raise the snapshot length, a malformed header means a broken sender or a
capture bug, an unsupported EtherType means unexpected traffic, an unsupported
link type means an unexpected backend.

### D-7. Correct the link type documentation for code 0

**Decision**: `LinkType::NULL` is documented as BSD loopback encapsulation,
carrying a four byte host-order address family field. `LinkType::RAW` keeps the
description of an encapsulation with no link layer header, which is what code
101 actually is.

**Rationale**: S02 gave code 0 the description belonging to code 101. It was
harmless while nothing parsed and stops being harmless in this slice, because a
parser written from that comment would read a network header out of an address
family field. Correcting the comment rather than working around it is the P-9
answer to a document that misdescribes an observation.

The four byte field is host ordered, and a fixture may be read on a host of the
opposite order, so the parser accepts both interpretations. The known values
are 2 for IPv4 and 10, 23, 24, 28, and 30 for IPv6, the set libpcap consumers
accept. No known value in one byte order is a known value in the other, so
accepting both is unambiguous rather than a guess.

### D-8. The zero-advance guard shares the chain counter

**Decision**: The extension header walk terminates on any declared length that
would not advance the cursor, and that termination advances the same counter as
exceeding the eight header bound.

**Rationale**: FR-015 requires both terminations. Given the length encodings
actually defined, `(len + 1) * 8` for the option headers and `(len + 2) * 4`
for the authentication header, neither can evaluate to zero, so the guard is
unreachable by construction. Giving it its own counter would create a counter
no constructed frame could ever advance, which SC-002 would then fail on
reachability grounds, and rightly: an unreachable counter is noise in the
statistics an operator reads.

Keeping the guard with a shared counter preserves the safety property, which is
that the walk provably terminates regardless of what the encodings turn out to
be, without inventing a statistic that is always zero. The unreachability is
documented at the guard so a later reader does not delete it as dead code.

### D-9. No per-packet rejection cause on `CapturedPacket`

**Decision**: `ParseOutcome` carries the cause. `CapturedPacket` does not gain
a field for it.

**Rationale**: The architecture of record has no such field, the counters
answer the operator's question in aggregate, and adding a field to the one
struct that exists per packet on the hot path should be driven by the slice
that demonstrates it needs one. S07 writes JSON Lines and is the likely
candidate. Nothing is lost meanwhile: the caller receives the cause and may do
whatever it likes with it.

### D-10. Deviations recorded for specification section 29

Four, all recorded in the spec and repeated here so the promotion list is in
one place:

1. Section 12.5 requires subsequent fragments be attributed by fragment
   identifier and address pair, which presupposes a memory it does not
   describe. This slice defines one, bounded and evicting.
2. Section 12.6 defines three of the four combinations of endpoint locality and
   is silent on the fourth. This slice makes it a counted rejection.
3. The link type documentation correction in D-7.
4. The fragment identifier reuse limitation in D-4.

## Project Structure

### Documentation (this feature)

```text
specs/003-header-parsing-flow-keys/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── parse-api.md
├── tasks.md
└── checklists/
    ├── requirements.md
    └── parser.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── lib.rs              + parse module declaration and re-exports
├── link.rs             + corrected documentation for code 0 and 101
├── stats.rs            + ParseStats, held by CaptureStats by value
└── parse/
    ├── mod.rs          HeaderParser, ParseOutcome, ParseReject
    ├── link.rs         link layer dispatch: Ethernet, raw IP, BSD loopback
    ├── ip.rs           IPv4 and IPv6 headers, extension header chain walk
    ├── transport.rs    TCP and UDP ports
    ├── fragment.rs     FragmentKey, FragmentTable
    └── direction.rs    InterfaceAddrs and the locality rule

crates/fragcap-core/tests/
└── no_alloc.rs         counting global allocator, FR-004 proof
```

`ParseStats` goes in the existing `stats.rs` beside `SourceStats` and
`CaptureStats` rather than in the parse module, because `CaptureStats` holds it
and the three are read as a group. That also keeps the P-4 counter set in one
file, which is where a reviewer looks for it.

One module per layer inside `parse/` so that each layer's rejection causes are
visible next to the code that produces them, and so the extension header walk,
which is the only loop in the slice, is isolated in a file a reviewer can read
whole.

## Dependency Graph

Unchanged. No crate is added and no edge is added. `fragcap-core` keeps its
single leaf, `bytes` 1.12, and `bytes` is not used by the parser: the parse
input is `&[u8]` so that a caller holding anything sliceable can call it.

## Testing Strategy

Test-driven, in the order the requirements are grouped, so that each layer's
rejections are pinned before the layer above it is written.

**Unit tests, colocated.** Frames are built by small helpers that assemble
headers from fields, so that a test reads as the packet it describes rather
than as a byte array. One test per supported combination for SC-001, one test
per rejection cause for SC-002 asserting that exactly the expected counter
moved and no other did.

**The counter isolation assertion is the load-bearing one.** Asserting only
that the expected counter advanced would pass a parser that advances three. The
helper snapshots the whole `ParseStats`, parses, and asserts the delta is
exactly one field.

**Adversarial input.** An extension chain of nine headers, a chain that
declares lengths pointing backwards, an IPv4 header with a header length below
the minimum, a UDP header declaring a length shorter than itself, and a frame
truncated at each header boundary in turn. The chain test's failure mode
without the bound is a hang, so it is worth writing before the walk exists.

**Allocation.** A separate integration binary installs a counting global
allocator backed by a thread-local counter, so that only the calling thread's
allocations are measured. The parser is constructed, the counter is snapshotted,
the corpus is parsed, and the delta must be zero. Construction is deliberately
outside the measurement: allocating the address set once is permitted,
allocating per packet is not.

## Complexity Tracking

No constitution violation requires justification.

Two items are worth naming as accepted cost.

The fragment identity table is state, and state in a parser is what makes a
parser hard to reason about. It is accepted because section 12.5 requires
subsequent fragments be attributed and there is no stateless way to do that
without reassembly, which the same section forbids. It is bounded, it has no
clock, and its only observable effects are one counter and one lookup, which is
the smallest surface the requirement admits.

The twelve rejection variants are more than a smaller enumeration would need,
and each carries a counter, a test, and a line in the statistics output. The
cost is accepted because collapsing any pair loses a distinction an operator
would use to choose a remedy, and P-4 exists to preserve exactly those
distinctions.
