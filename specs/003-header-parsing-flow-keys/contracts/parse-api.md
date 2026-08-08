# Contract: the `fragcap-core` parse API

**Slice**: S03

**Created**: 2026-08-08

The public surface this slice adds to `fragcap-core`, and the guarantees a
caller may rely on. This is the interface S04, S08, and S09 write against.

## Surface

```rust
pub mod parse {
    pub struct HeaderParser { /* private */ }

    impl HeaderParser {
        pub fn new(addrs: InterfaceAddrs) -> Self;
        pub fn set_interface_addrs(&mut self, addrs: InterfaceAddrs);
        pub fn interface_addrs(&self) -> &InterfaceAddrs;

        pub fn parse(&mut self, link: LinkType, frame: &[u8]) -> ParseOutcome;
        pub fn apply(&mut self, link: LinkType, packet: &mut CapturedPacket)
            -> ParseOutcome;

        pub fn stats(&self) -> &ParseStats;
    }

    pub enum ParseOutcome {
        Parsed { flow: FlowKey, direction: Option<Direction> },
        Rejected(ParseReject),
    }

    impl ParseOutcome {
        pub fn flow(&self) -> Option<FlowKey>;
        pub fn direction(&self) -> Option<Direction>;
        pub fn reject(&self) -> Option<ParseReject>;
    }

    pub enum ParseReject { /* twelve variants, see data-model.md */ }

    impl ParseReject {
        pub fn as_str(&self) -> &'static str;
    }

    pub struct InterfaceAddrs { /* private */ }

    impl InterfaceAddrs {
        pub fn new(addrs: impl IntoIterator<Item = IpAddr>) -> Self;
        pub fn contains(&self, addr: &IpAddr) -> bool;
        pub fn is_empty(&self) -> bool;
        pub fn len(&self) -> usize;
    }
}

// re-exported at the crate root
pub use parse::{HeaderParser, InterfaceAddrs, ParseOutcome, ParseReject};
```

`ParseStats` lives in `stats` beside `SourceStats` and `CaptureStats`, and is
re-exported at the crate root with them.

## Guarantees

**The frame is not modified.** `parse` takes `&[u8]`. There is no interface
through which the parser could write to a frame, and no overload taking a
mutable slice will be added.

**The outcome borrows nothing.** `ParseOutcome` is `Copy`. A caller may drop or
forward the frame the instant `parse` returns.

**Parsing allocates nothing.** No call to `parse` or `apply` performs a heap
allocation, on any input, including malformed and adversarial input.
`HeaderParser::new` and `InterfaceAddrs::new` each allocate once. This is
asserted by a test under a counting allocator, not merely intended.

**Parsing terminates.** The extension header walk is bounded at eight headers
and additionally cannot fail to advance. No input causes a hang.

**Reads stay inside the captured bytes.** A declared length larger than the
captured bytes never causes a read past the end. It causes a `ShortHeader`
rejection or, where the field contradicts itself rather than merely exceeding
the capture, a malformed rejection.

**Exactly one counter moves per rejection.** A `Rejected` outcome advances the
counter for its variant and no other rejection counter. A `Parsed` outcome
advances no rejection counter, and advances `direction_ambiguous` only when
both endpoints are local.

**No parse outcome is a drop.** `CaptureStats::fragcap_dropped` and
`total_dropped` are unaffected by every parse counter. A packet that produced no
flow key is retained and marked by the caller, per P-4.

**A conversation has one key.** Both directions of a conversation produce the
same `FlowKey`, including a loopback conversation where direction is
undetermined.

## Behavioral contract by input class

| Input | `flow` | `direction` | Counter |
| --- | --- | --- | --- |
| Ethernet, IPv4, TCP, source local | present | `Outbound` | none |
| Ethernet, IPv6, UDP, destination local | present | `Inbound` | none |
| Raw IP, IPv4, UDP, source local | present | `Outbound` | none |
| BSD loopback, IPv4, TCP, both local | present | absent | `direction_ambiguous` |
| Any supported, neither endpoint local | absent | absent | `NoLocalEndpoint` |
| IPv6 with extension chain, TCP, source local | present | `Outbound` | none |
| IPv4 initial fragment, UDP | present | per locality | none |
| IPv4 non-initial fragment, identity known | present | per locality | none |
| IPv4 non-initial fragment, identity unknown | absent | absent | `UnmatchedFragment` |
| Link type 108 | absent | absent | `UnsupportedLinkType` |
| Ethernet carrying a VLAN tag | absent | absent | `UnsupportedEtherType` |
| Loopback with an unknown address family | absent | absent | `UnsupportedAddressFamily` |
| Raw IP whose version nibble is 5 | absent | absent | `UnsupportedIpVersion` |
| IPv4 with header length 4 | absent | absent | `MalformedNetworkHeader` |
| IPv6 with nine extension headers | absent | absent | `ExtensionChainTooLong` |
| IPv6 chain ending in next header 59 | absent | absent | `NoNextHeader` |
| IPv4 protocol 1 | absent | absent | `UnsupportedTransport` |
| IPv6 chain ending in next header 50 | absent | absent | `UnsupportedTransport` |
| UDP declaring length 4 | absent | absent | `MalformedTransportHeader` |
| Truncated within the TCP header | absent | absent | `ShortHeader` |
| IPv4 with a legal header length extending past the captured bytes | absent | absent | `ShortHeader` |
| IPv4 unfragmented packet | present | per locality | no table entry recorded |

Each row is a test. The table is the coverage obligation SC-001 and SC-002
state, written out so that a reviewer can count it rather than trust it.

## What this contract does not cover

**When the address set is refreshed.** The parser accepts a replacement at any
point between packets. Deciding when to call it, and obtaining the addresses in
the first place, is platform work owned by S09 and S13.

**Where the parser is called from.** The pipeline that owns a `HeaderParser`,
drains a source into it, and feeds the results to sinks is S08.

**What is done with a rejection.** The caller receives the cause and decides.
The obligation this slice imposes is that the packet is retained and marked, not
what marking looks like downstream.

**Loopback direction resolution.** Section 12.6 resolves an ambiguous direction
from the attributed process's endpoint. That needs an attribution, so it follows
S10 and belongs with the loopback work in S13. This contract's obligation is to
report the ambiguity rather than to resolve it.

## Stability

`ParseReject` is closed. Adding a variant is a breaking change, and that is
intentional: the counter, the test, and the statistics line have to be added in
the same edit, and a `#[non_exhaustive]` enumeration would let the first happen
without the others.

`ParseOutcome` and `InterfaceAddrs` may gain methods. `HeaderParser` may gain
configuration, for instance a fragment table capacity, without breaking a caller
that uses `new`.
