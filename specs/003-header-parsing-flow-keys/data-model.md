# Data Model: Header Parsing and Flow Keys

**Slice**: S03

**Created**: 2026-08-08

The types this slice adds, and the two it modifies. Signatures are indicative;
the contract is in [contracts/parse-api.md](contracts/parse-api.md).

## New types

### `HeaderParser`

The parser itself, owned by the caller. Holds the interface address set, the
fragment identity table, and the counters.

| Field | Type | Purpose |
| --- | --- | --- |
| `addrs` | `InterfaceAddrs` | Locality test for direction, replaced wholesale |
| `fragments` | `FragmentTable` | Fragment identity to wire-order ports |
| `stats` | `ParseStats` | One counter per outcome that is not a plain success |

Not `Clone`, deliberately: two parsers sharing a copy of a fragment table would
each learn half the first fragments and each miss half the subsequent ones,
which is a bug that would present as an intermittently low attribution rate.

### `ParseOutcome`

```rust
pub enum ParseOutcome {
    Parsed { flow: FlowKey, direction: Option<Direction> },
    Rejected(ParseReject),
}
```

Owns its contents. Borrows nothing from the frame, so the caller may forward or
drop the frame independently of the outcome, which FR-003 requires.

`direction` is `Option` inside the `Parsed` variant rather than a third
top-level variant, because an ambiguous direction is a successful parse with
one field undetermined, not a failure. Which of the two ambiguity causes fired
is in the counters; the packet itself carries the same information either way,
namely a key and no direction.

### `ParseReject`

Closed enumeration. One variant per cause, one counter per variant.

| Variant | Cause | Layer |
| --- | --- | --- |
| `UnsupportedLinkType` | Link type is not one of the three handled | Link |
| `UnsupportedEtherType` | Ethernet type is not IPv4 or IPv6, including VLAN tags | Link |
| `UnsupportedAddressFamily` | Loopback address family value is not recognized in either byte order | Link |
| `UnsupportedIpVersion` | Raw IP version nibble is neither 4 nor 6 | Network |
| `MalformedNetworkHeader` | Network header's own fields contradict each other | Network |
| `ExtensionChainTooLong` | Chain exceeds eight headers, or a length would not advance | Network |
| `NoNextHeader` | IPv6 chain terminates with no transport, legitimately | Network |
| `UnsupportedTransport` | Transport is neither TCP nor UDP, including encrypted payloads | Transport |
| `MalformedTransportHeader` | Transport header's own fields contradict each other | Transport |
| `ShortHeader` | Captured bytes end before a field the parser needs | Any |
| `UnmatchedFragment` | Non-initial fragment with no recorded identity | Fragment |
| `NoLocalEndpoint` | Neither endpoint is in the interface address set | Direction |

Closed rather than `#[non_exhaustive]`. See plan.md D-6: this is the complete
set of ways this parser declines, and closing it makes adding a decline path
without a counter a compile error.

### `InterfaceAddrs`

The addresses belonging to the capturing host.

```rust
pub struct InterfaceAddrs(Vec<IpAddr>);

impl InterfaceAddrs {
    pub fn new(addrs: impl IntoIterator<Item = IpAddr>) -> Self;
    pub fn contains(&self, addr: &IpAddr) -> bool;
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
}
```

Addresses only, no ports: section 12.6 matches on address, and the port-based
loopback resolution it describes needs an attribution and therefore a later
slice.

Construction allocates once. `contains` is a linear scan and allocates nothing,
which is the property FR-004 needs.

### `FragmentKey`

The identity shared by every fragment of one datagram. Different by address
family, because the two standards define different reassembly keys.

```rust
enum FragmentKey {
    V4 { src: Ipv4Addr, dst: Ipv4Addr, proto: u8, ident: u16 },
    V6 { src: Ipv6Addr, dst: Ipv6Addr, ident: u32 },
}
```

Addresses in wire order, not normalized to local and remote. Every fragment of
a datagram carries the same wire-order pair, and normalizing would make the key
depend on an address set that may change between the first fragment and the
last.

### `FragmentTable`

```rust
struct FragmentTable {
    slots: [Option<(FragmentKey, FragmentPorts)>; 256],
    cursor: usize,
}

struct FragmentPorts { proto: Proto, src_port: u16, dst_port: u16 }
```

Fixed size, write cursor, drop-oldest, linear scan on lookup. Allocates never.
Ports are stored in wire order for the same reason the addresses are.

Operations: `record` on an initial fragment, `take` on a last fragment, `lookup`
on a non-initial one. `record` returns whether it evicted, so the caller
advances the eviction counter at the one site that knows.

## Modified types

### `ParseStats` (new, in `stats.rs`)

One `u64` per `ParseReject` variant, twelve in all, plus:

| Field | Meaning |
| --- | --- |
| `direction_ambiguous` | Both endpoints local. A key was produced with no direction. |
| `fragment_evicted` | The table dropped an entry to admit a newer one. |

Neither is a rejection. `direction_ambiguous` accompanies a successful parse,
and `fragment_evicted` is a table event that may occur on a parse of any
outcome.

There is deliberately no counter for a successful parse. The count of keys
produced is the captured count less the twelve rejection counters, and S02's
rule against stored totals applies to it for the same reason it applies to the
others: a stored total can drift from its parts.

Totals are methods. `rejected()` sums the twelve. `total()` is not offered,
because summing a rejection count with an ambiguity count and an eviction count
produces a number with no meaning.

### `CaptureStats` (existing, in `stats.rs`)

Gains one field:

```rust
pub parse: ParseStats,
```

Held by value, matching how `SourceStats` is already held, and for the same
reason: composition keeps each component's accounting legible instead of
blending three vocabularies into one flat struct.

`fragcap_dropped()` and `total_dropped()` are unchanged and must stay
unchanged. No parse outcome is a drop, so folding a parse counter into either
total would report loss that did not occur, which is a P-9 problem and not only
an arithmetic one. A test asserts that advancing every parse counter leaves
both totals at zero.

### `LinkType` (existing, in `link.rs`)

No signature change. The documentation on `NULL` and `RAW` is corrected: code 0
is BSD loopback encapsulation with a four byte host-order address family field,
and code 101 is the one with no link layer header. See plan.md D-7.

## Relationships

```text
HeaderParser
├── InterfaceAddrs        locality test, replaced wholesale by the caller
├── FragmentTable         FragmentKey -> FragmentPorts, bounded at 256
└── ParseStats            counters, also reachable through CaptureStats

HeaderParser::parse(LinkType, &[u8]) -> ParseOutcome
                                        ├── Parsed { FlowKey, Option<Direction> }
                                        └── Rejected(ParseReject)
```

`FlowKey` and `Direction` are S02 types and are unchanged. That is the point of
the slice: S02 declared the shape, S03 fills it, and nothing about the shape
needed renegotiating to do so.

## Validation rules

Stated as the parser enforces them, in the order it enforces them, because the
order determines which counter fires when a frame is wrong in more than one
way.

1. **Link.** The link type is handled, the link header fits in the captured
   bytes, and the network protocol it names is IPv4 or IPv6.
2. **Network.** The version matches the protocol the link layer named, the
   header length is legal, and the address pair is readable. A header length
   that is illegal, meaning below the fixed header's own size, is malformed. A
   header length that is legal but extends past the captured bytes is short.
   The two are separated here rather than merged because they are the pair most
   easily conflated and they have opposite remedies.
3. **Fragment classification.** Initial, non-initial, or not fragmented.
4. **Transport.** For an initial or unfragmented packet, the protocol is TCP or
   UDP, its header fits, and both ports are present. For a non-initial
   fragment, the identity resolves in the table.
5. **Locality.** At least one endpoint is in the address set.

A frame wrong at more than one layer is counted at the first, which is why the
order is documented rather than incidental. A truncated frame carrying an
unsupported EtherType is an unsupported EtherType, because the parser never got
far enough to find the truncation.
