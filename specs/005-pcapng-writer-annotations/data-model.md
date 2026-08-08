# Data Model: pcapng Writer and Annotation Encoding

**Slice**: S06

**Created**: 2026-08-08

**Feature**: [spec.md](spec.md)

The entities below are the ones this slice introduces. Types S02 already fixed
(`CapturedPacket`, `Attribution`, `Direction`, `Timestamp`, `LinkType`,
`CaptureStats`, `SinkError`) are inputs, not new entities, and are referenced
rather than restated.

## Annotation

The attribution facts for one observation, as a value rather than a string.
Derived from a `CapturedPacket`; rendered to the section 13.3 grammar as a
separate step, so S07 can render the same value as JSON.

| Field | Type | Presence | Source |
| --- | --- | --- | --- |
| `pid` | `u32` | When attributed | `Attribution::pid` |
| `process` | `Arc<str>` | When attributed | `Attribution::process` |
| `role` | `Option<Arc<str>>` | When present | `Attribution::role` |
| `stage` | `Option<StageId>` | When present | `Attribution::stage` |
| `direction` | `AnnotatedDirection` | Always | `CapturedPacket::direction` |
| `fidelity` | `Fidelity` | Always | `Attribution::fidelity`, or `None` when unattributed |
| `interface` | `Option<Arc<str>>` | When multi-interface | Interface declaration |

**Derivation rules**, which are the part FR-025 makes reusable:

- `pid` and `process` are present exactly when `CapturedPacket::attribution` is
  `Some`. They are never present individually.
- `role` and `stage` are decided independently of each other, each present when
  the corresponding field on `Attribution` is `Some`. FR-018. The
  specification presents them as a pair; the type does not, and the type is
  what the data actually looks like.
- `interface` is present exactly when the writer holds more than one interface
  declaration. FR-021.
- `direction` and `fidelity` are always present. FR-019, FR-020.
- `fidelity` is copied from the attribution, never computed from whether one
  exists. FR-029. The one case this crate decides is absence: no attribution
  means `None`, because nothing was found.

**Rendering rules**, which are the part that produces bytes:

- Keys appear in the order `pid`, `proc`, `role`, `stage`, `dir`, `attr`,
  `iface`, with present keys keeping that relative order. FR-016a.
- Values are percent-encoded for `;`, `=`, `%`, any code point below 0x20, and
  0x7F, with uppercase hexadecimal digits. FR-022, FR-023, FR-023a.
- An empty value renders as an empty value with its key present. FR-023b.

## AnnotatedDirection

The direction as the file records it. Distinct from core's `Direction` because
the file must express two states the type does not.

| Variant | Rendered | Meaning |
| --- | --- | --- |
| `In` | `in` | `Direction::Inbound` |
| `Out` | `out` | `Direction::Outbound` |
| `Local` | `local` | Both endpoints on the capturing host |
| `Unknown` | `unknown` | The pipeline determined no direction |

`Local` is defined and encodable here and is not produced by this slice.
Section 12.6 leaves loopback direction undetermined until it can be resolved
from the attributed process's endpoint, which is later work. Carrying the
variant now means that slice supplies data rather than widening a grammar.

`CapturedPacket::direction` maps as `Some(Inbound) -> In`, `Some(Outbound) ->
Out`, `None -> Unknown`. FR-019a forbids mapping `None` to `Local`: "not
determined" and "loopback" are different facts, and asserting the second from
the first is the substitution P-9 exists to block.

## Fidelity

How attribution was obtained. Section 13.4. Never inferred, never defaulted,
never upgraded. FR-029.

**Lives in `fragcap-core`, on `Attribution`.** Moved there during review of
pull request 8, which found the writer deriving `Live` from the mere presence
of an attribution. That was an inference, and a wrong one: the scripted
attributor resolves from a declared script rather than a socket table, so every
golden claimed an endpoint was in a table that did not exist. Only the
attributor knows how it reached an answer, so the value travels with the
answer. `Attribution::new` takes it as a required argument, because a defaulted
field is the same inference wearing a different hat.

| Variant | Rendered | Meaning |
| --- | --- | --- |
| `Live` | `live` | Endpoint present in the socket table at resolution time |
| `Retained` | `retained` | Resolved from the section 11.4 grace period map |
| `None` | `none` | Not attributable |

`None` implies the absence of `pid`, `proc`, `role`, and `stage`, rather than
their presence with an empty value. FR-028.

This slice writes `Live` and `None`, and writes whichever the attributor
supplied rather than choosing. `Retained` becomes reachable when the grace
period map lands with the socket table attributor. As with
`AnnotatedDirection::Local`, the value exists here so the later slice supplies
data rather than extending the grammar.

## InterfaceDeclaration

One capture interface, as declared by the caller before any packet references
it.

| Field | Type | Notes |
| --- | --- | --- |
| `link_type` | `LinkType` | Written to the block as u16 |
| `snap_len` | `u32` | Declared, never enforced against packet contents |
| `name` | `Arc<str>` | Written as `if_name`; also the annotation `iface` value |
| `id` | `u32` | Assigned by the writer in declaration order from zero |

`id` is assigned rather than supplied, because pcapng identifies interfaces by
declaration order and a caller-supplied identifier could disagree with it.
FR-006.

Declaring a second interface, identical or not, is refused. pcapng identity is
positional, so the question of deduplication does not arise while there is one.

## PcapngWriter

The sink itself.

| Field | Purpose |
| --- | --- |
| Output target | Any `std::io::Write` |
| Interface declarations | In declaration order, indexed by assigned id |
| Last timestamp per interface | Source of the Interface Statistics Block timestamp, FR-008a |
| Header written flag | The Section Header Block is emitted once, before anything else |

**State transitions**:

```text
new  ->  declare_interface  ->  write*  ->  finish
           (exactly once)      (any number)   (consumes)
```

- `declare_interface` succeeds once. A second call is an error, per FR-006a.
  Late declaration was permitted in the first draft of this slice and is not:
  a packet block already written cannot gain the `iface` key that a
  now-multi-interface capture requires of it.
- `write` against an undeclared identifier is an error, not an invented
  interface. FR-033.
- `finish` writes one Interface Statistics Block per declared interface and
  consumes the writer, so the trailing blocks are written exactly once. D-4.
- A writer dropped without `finish` leaves the blocks already written intact
  and readable. Nothing buffers across a block boundary.

**Invariants**:

- The Section Header Block is the first thing in the file, always.
- Every Enhanced Packet Block references an identifier whose Interface
  Description Block appears earlier in the file. FR-004.
- No method reads a clock, the host byte order, the environment, or a locale.
  Research R-4.

## Golden

A committed file of expected output for one corpus fixture. Not a runtime
entity; recorded here because it is data the repository carries and the drift
check compares.

| Property | Value |
| --- | --- |
| Location | `fixtures/goldens/<fixture>.fcapng` |
| Produced by | The committed generator, from the fixture and its script |
| Checked by | A drift check in `cargo xtask ci`, FR-038a |
| Reviewed | Once by a human, at the commit that introduces it |

The pairing is one golden per fixture in the S04 corpus, all eight, so that
"which fixtures have goldens" is never a question a contributor has to answer.
