# Data Model: Live Capture Source and Interfaces (S09)

**Slice**: S09

**Date**: 2026-08-09

**Phase**: 1

Everything new in this slice lives in `fragcap-core`, in a new `interface`
module, except the live source itself. That placement is argued in
[plan.md](plan.md); the short form is that selection is a decision over a value
and the pipeline that consumes its outcome is in core.

Three existing types change. Those are listed last, with their blast radius,
because they are what a reviewer should look at first.

## New types

### `InterfaceId`

A newtype over `u32`, `Copy`, ordered, hashable. Assigned by selection, unique
within a run, meaningless outside it.

Not the platform's name, because platform names are not guaranteed unique and
the specification's edge cases require identity to survive a collision. Not a
string, because it is compared once per packet and carried on every one.

### `InterfaceRecord`

One entry of what the machine reports. Produced by enumeration, consumed by
selection.

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | `Arc<str>` | The platform's own name for the interface |
| `description` | `Option<Arc<str>>` | The adapter description, when the platform supplies one |
| `link_type` | `LinkType` | The encapsulation frames will arrive in |
| `addresses` | `Vec<IpAddr>` | Addresses configured on the interface |
| `is_up` | `bool` | The interface is administratively up |
| `is_running` | `bool` | The interface has a carrier |
| `is_loopback` | `bool` | The platform reports it as a loopback adapter |

There is deliberately no `is_virtual` field. Virtuality is a verdict this
project reaches by heuristic, not a property the platform reports, and storing
it here would let the verdict travel without the reasoning that produced it.
See `VirtualVerdict`.

### `InterfaceInventory`

A `Vec<InterfaceRecord>` plus the default-route address, if one was determined.

Being a value rather than a query is the point: selection over an inventory is
testable without a machine, which is FR-010 and SC-002. Enumeration on a real
machine produces one; a test writes one.

| Field | Type | Meaning |
| --- | --- | --- |
| `interfaces` | `Vec<InterfaceRecord>` | Every capture-capable interface |
| `default_route_source` | `Option<IpAddr>` | The source address the routing table chooses for an off-link destination, per plan D-3 |

### `SelectionSettings`

What the caller asks for. Plain values, because `fragcap-capture` may not read
a profile, per FR-012 and specification section 8.3.

| Field | Type | Meaning |
| --- | --- | --- |
| `explicit` | `Vec<String>` | Interfaces named by the operator. Non-empty takes precedence over everything else. |
| `loopback` | `bool` | Include the loopback adapter alongside the default-route interface |
| `broad` | `bool` | Select every interface that is up, addressed, and not virtual |

### `VirtualVerdict`

The heuristic's answer for one interface, carried rather than folded into a
boolean, per FR-004 and plan D-9.

| Variant | Meaning |
| --- | --- |
| `NotVirtual` | No pattern matched |
| `Virtual { pattern }` | The description matched a documented pattern, which is named |

### `SelectedInterface`

An interface chosen for the run.

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | `InterfaceId` | Identity for the run |
| `record` | `InterfaceRecord` | What the machine said about it |
| `reason` | `SelectionReason` | Why it was chosen |

`SelectionReason` is one of `NamedExplicitly`, `DefaultRoute`, `Loopback`, or
`Broad`, mirroring the section 12.1 precedence so that a report can say which
step chose each interface.

### `ExclusionReason`

Why an enumerated interface was not chosen. A closed enumeration, so that a new
exclusion path cannot be added without naming itself, which is the same
discipline `ParseReject` follows.

| Variant | Meaning |
| --- | --- |
| `NotNamed` | Explicit names were given and this was not among them |
| `NotDefaultRoute` | Automatic selection chose the default-route interface and this is not it |
| `LoopbackNotRequested` | A loopback adapter, and the settings did not ask for loopback |
| `Down` | Not up |
| `NoAddress` | Up, but no address configured |
| `Virtual { pattern }` | Excluded by the heuristic, with the pattern that matched |

### `SelectionOutcome`

The whole answer, which is what makes an unexpectedly empty capture
diagnosable. FR-009 and SC-003.

| Field | Type | Meaning |
| --- | --- | --- |
| `selected` | `Vec<SelectedInterface>` | In selection order, ids assigned by position |
| `excluded` | `Vec<(InterfaceRecord, ExclusionReason)>` | Every interface not chosen, with its reason |

The invariant that matters: `selected.len() + excluded.len()` equals the
inventory's length. No interface is unaccounted for, and a test asserts it
rather than trusting it.

### `SelectionError`

Returned rather than panicking, and turned into a failed run by the caller.
FR-007, FR-011, and checklist item CHK034.

| Variant | Meaning |
| --- | --- |
| `UnknownInterface { requested, available }` | A name matched nothing, carrying what was available so the message can say |
| `NothingSelected` | The settings and the inventory together chose nothing |

### `InterfaceRetirement`

Recorded when a capture thread ends. FR-027 and FR-028.

| Field | Type | Meaning |
| --- | --- | --- |
| `interface` | `InterfaceId` | Which one |
| `reason` | `RetirementReason` | `SourceClosed`, `DeviceLost { detail }`, or `Backend { detail }` |

Advances no drop counter, because nothing was observed and then discarded. That
is stated on the type, so that a later reader does not helpfully add one.

### `DriverReport`

What detection concluded. FR-041, FR-042, FR-045. Presentation is S14's.

| Field | Type | Meaning |
| --- | --- | --- |
| `present` | `bool` | A capture driver was found |
| `version` | `Option<String>` | Its version, as the driver reports it |
| `loopback_supported` | `Option<bool>` | Whether the loopback option is installed, `None` when it cannot be determined |
| `winpcap_compatible` | `Option<bool>` | Whether the compatibility mode option is installed, `None` when it cannot be determined |

The two `Option<bool>` fields are three-valued deliberately. "We could not tell"
is not "no", and reporting it as "no" would make a diagnostic assert something
it did not observe.

## Changed types

### `PacketSource` gains `Send`

`crates/fragcap-core/src/traits.rs`. One line, and the blast radius is every
implementor: `ReplaySource`, the two stub sources in the traits module's own
tests, and the new `LiveSource`. All are already `Send`; none needs a change
beyond compiling under the bound.

Recorded as a deviation from specification section 8.5.

### `CapturedPacket` gains `interface`

`crates/fragcap-core/src/packet.rs`. Non-optional `InterfaceId`, attached at
the lift from `RawPacket`. `RawPacket` is unchanged: a source knows only its
own interface, so the identifier belongs where the pipeline attaches it.

Blast radius: `CapturedPacket::from_raw` gains a parameter, and every call site
in the workspace changes. That is roughly forty sites, nearly all of them
tests, and the compiler finds all of them. A default would have been kinder and
would have let a real capture ship with the wrong identity, so there is no
default.

Recorded as a deviation from specification section 8.4.

### `CaptureStats::source` becomes per-interface

`crates/fragcap-core/src/stats.rs`. The field becomes
`sources: Vec<(InterfaceId, SourceStats)>`, and `source()` becomes a method
summing them.

This follows the module's own standing rule that no aggregate is stored, so the
capture-wide figure cannot drift from its parts. It is what lets a kernel drop
name the driver buffer that is undersized rather than reporting that one of
several is.

Blast radius: both writers read `stats.source`, and the S07 corpus helper
composes one. All become calls to `source()` or constructions of a one-entry
vector.

Recorded as a deviation, discovered in planning rather than before it.

## Corrected in review of pull request 12

**`SourceBinding` carries an `InterfaceAddrs`.** Specification section 12.6
determines direction by matching against the capturing interface's address set,
and `PipelineConfig` held one set for the whole run. `PipelineConfig::addrs` was
removed rather than kept as a fallback, so the ambiguous form cannot be written.

**A capture thread that panics stops the others through a `StopOnPanic` guard.**
Distinct from the existing `StopOnDrop`: a thread ending normally must leave the
other interfaces running, and a thread that unwinds must not, because it holds a
producer that would otherwise keep the buffer open forever. The guard fires
inside the panicking thread rather than at the join, because join order is
arbitrary and waiting for the join is what let a survivor run on unbounded.

**`LiveSource` records the timeout it was activated with.** libpcap fixes the
read timeout at activation, so `next_packet`'s argument cannot be honoured.
`LiveOptions::for_pipeline` makes the two agree by construction and
`LiveSource::configured_timeout` exposes the one that governs.

## What does not change

`RawPacket`, `Timestamp`, `SourceStats` itself, `ParseStats`, `FlowKey`,
`Attribution`, the bounded buffer, the sink thread, sink retirement, and the
drop-oldest semantics. The conservation identity S08 established holds
unchanged and is asserted again with several capture threads running, which is
SC-006.
