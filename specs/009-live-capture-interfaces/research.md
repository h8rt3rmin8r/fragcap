# Research: Live Capture Source and Interfaces (S09)

**Slice**: S09

**Date**: 2026-08-09

**Phase**: 0

This slice has one question that needs evidence rather than reasoning, and
four that need a decision recorded. The first is how fragcap reaches the
capture driver, because it is the project's first runtime dependency with a
platform surface and the constitution constrains it from three directions at
once: P-1 bounds what the dependency may be able to do, P-2 bounds where it may
live, and the licensing section bounds what may be shipped alongside it.

Everything below that names a version, a license, or an API was checked against
crates.io and docs.rs on 2026-08-09. Nothing here is recalled.

## R-1: How fragcap reaches the capture driver

**Decision**: The `pcap` crate, version 2.4.0, as an optional dependency of
`fragcap-capture` behind a feature that is off by default.

### What was measured

| Property | `pcap` 2.4.0 | Required |
| --- | --- | --- |
| License | MIT OR Apache-2.0 | On the allowlist |
| Declared Rust version | 1.64 | At or below 1.82 |
| Last release | 2025-11-26 | Maintained |
| Downloads | 7.5 million | Not abandoned |

Its runtime dependency graph is `bitflags` 1.3, `errno` 0.2, `libc` 0.2, and
`windows-sys` 0.36.1 on Windows. Its build dependencies are `libloading` 0.8,
`pkg-config`, and `regex`. Every one of those is MIT OR Apache-2.0, so the
whole graph clears the constitution's allowlist without an exception. The
optional `capture-stream` and `lending-iter` features pull `tokio`, `futures`,
and `gat-std`; neither feature is enabled, so none of those enter the graph.

One detail is worth stating because it would otherwise be found the hard way.
`libloading`'s current release is 0.9.0 and it declares Rust 1.88, which is far
above this workspace's 1.82 floor. `pcap` depends on `^0.8`, which resolves to
0.8.9 declaring 1.71, so the floor holds. A future proposal to use `libloading`
directly at `"0.9"` would break `cargo xtask msrv`, and the failure would
appear in a check most contributors cannot run locally.

### How the API maps onto the seam

The mapping is close enough that the adapter is mostly renaming, which is the
strongest argument for the crate and was verified method by method.

| `PacketSource` obligation | `pcap` 2.4.0 |
| --- | --- |
| `next_packet` returning `Ok(None)` on timeout | `Error::TimeoutExpired` |
| Driver timestamp and original length | `PacketHeader { ts, caplen, len }` |
| `set_filter` on an open handle | `Capture::filter(program, optimize)` |
| `link_type` | `Capture::get_datalink() -> Linktype` |
| `stats` | `Capture::stats() -> Stat` |
| Snapshot length and promiscuous mode at open | `snaplen`, `promisc`, `open` |

`Stat` carries `received`, `dropped`, and `if_dropped`, all `u32`, documented
as counts from the start of the run to the time of the call. They map onto
`SourceStats::received`, `kernel_dropped`, and `interface_dropped`, widening
`u32` to `u64` losslessly.

That the counts are cumulative rather than per-call deltas settles checklist
item CHK019: fragcap copies a value the driver already maintains and never
accumulates one of its own, so there is no arithmetic in which an alteration
could hide. Had they been deltas, relaying them "unaltered" would have required
fragcap to sum them, and the sum would have been fragcap's number wearing the
driver's name.

Interface enumeration comes from `Device::list()`, which returns `Device {
name, desc, addresses, flags }` with `DeviceFlags` exposing `is_loopback()`,
`is_up()`, `is_running()`, and `is_wireless()`. That covers every field FR-001
asks for except the virtual classification and the default route, which are
R-3 and R-4 below.

### What the API does not supply

`pcap::Error` has thirteen variants and **none of them names a device that has
gone away**. A removed interface surfaces as `PcapError(String)`, the general
"the underlying library returned an error" case. FR-019 requires that a
disappeared interface be reported as a lost device rather than as an
unmodelled backend failure, and the binding cannot tell fragcap which it is.

The tempting fix is to match on the message text. That is rejected: the string
comes from the driver, varies by version and locale, and a match that silently
stops matching would downgrade a lost device to a generic failure without
anyone noticing. See D-5 for what is done instead.

### Alternatives considered

**Hand-rolled FFI linking `wpcap` at build time.** Adds no dependency, which is
this project's usual preference and the reason S03, S04, S06, and S08 added
none. Rejected here because the thing being hand-rolled is not arithmetic over
a byte slice; it is a C ABI whose struct layouts must be transcribed by hand
with no compiler checking them against the header. A wrong offset in
`pcap_pkthdr` produces plausible timestamps and plausible lengths that are
quietly wrong, which is the exact failure P-9 exists to prevent and the exact
failure a test over synthetic data would not catch. The dependency buys a
layout that someone else keeps correct.

**Hand-rolled FFI with runtime dynamic loading via `libloading`.** Genuinely
attractive: loading `wpcap.dll` at runtime removes the software development kit
from the build entirely, so no contributor and no runner needs it. Rejected on
two counts. It carries the same hand-transcribed ABI risk as the option above,
plus a symbol table to get right. And it would put fragcap's capture path
behind a hand-written dynamic loader, which is a shape that reads like the
techniques P-1 forbids even though it is not one of them, and a security
posture that has to be explained is worth less than one that is obvious.

**`rawsock`.** MIT, dynamically loads, 11,846 downloads total and no declared
Rust version. Rejected on adoption: a capture path is not where this project
should be the crate's main consumer.

### Consequences accepted

Building `fragcap-capture` with the feature enabled requires the npcap software
development kit on Windows, because `pcap`'s build script links against it.
That is exactly what the constitution's licensing section already directs
continuous integration to do, and what `.github/workflows/platform.yml` was
scaffolded in S01 to perform. The feature being off by default is what keeps
that requirement off every contributor's machine. See D-2.

`pcap` exposes packet transmission on an active capture. See D-8 for why that
does not fail the dependency audit and what makes the answer enforceable rather
than asserted.

## R-2: What the feature gate is called and what it gates

**Decision**: One feature, `live`, declared by `fragcap-capture` and re-exported
by the `fragcap` facade. Off by default.

One feature rather than two. The obvious alternative is separating "compile the
binding" from "run the tests that need a driver", and it is not worth it: the
tests cannot run without the binding, so the second feature would only ever be
enabled together with the first.

`.github/workflows/platform.yml` anticipated a feature named `platform-tests`
and says so in a comment. That name is not adopted, because the feature gates a
capability rather than a test suite, and a capability named for its tests
invites someone to enable it in order to run tests and be surprised that it
changed what the library does. The workflow is a pinned artifact; changing it
carries a dated decision, recorded in the changelog fragment.

Tier 2 tests are `#[cfg(feature = "live")]` and additionally check at runtime
that a driver is present, returning early with a printed reason when it is not.
Rust's test harness has no skip, and a test that fails on a machine without a
driver would make the feature unusable for local development. That resolves
CHK038.

## R-3: How the default-route interface is determined

**Decision**: Ask the operating system's routing table which source address it
would choose for an off-link destination, by binding an unconnected UDP socket
and calling `connect`, then match that address against the enumerated
interfaces. No IP Helper, no `windows-sys`, no added dependency.

`connect` on a UDP socket sends nothing. It performs a route lookup and binds
the local end to the address the routing table selected, which `local_addr`
then reports. This is a query, not traffic, so it is passive in the sense P-1
cares about, and it is `std::net` only, so it works identically on the targets
section 28 has in view.

Alternatives: `GetBestRoute2` through `windows-sys` would be the direct answer
and would add a platform dependency plus a second `windows-sys` major version
to the graph, since `pcap` pins 0.36 and current is 0.61. A route-table parsing
crate would add a dependency to answer a question three lines of `std::net`
already answer.

When the machine has no route, `connect` fails, and the specification's edge
case applies: automatic selection has no interface to choose and says so.

## R-4: How an interface is classified as virtual

**Decision**: A documented substring match over the interface description,
against a list held as data in one place, with the verdict recorded per
interface rather than applied silently.

This is a heuristic and the plan says so rather than dressing it up. `pcap`
supplies a name and a description; neither carries a "this is a hypervisor
adapter" bit. The descriptions are stable enough in practice (adapters from
VMware, Hyper-V, VirtualBox, and the Windows Subsystem for Linux all name
themselves) and unreliable enough in principle that fragcap must not present
the verdict as fact.

Two things keep the heuristic honest. The classification is only ever used to
exclude from **automatic** selection, and an explicitly named interface is
captured whatever the rule concluded, per FR-006. And FR-004 requires the
verdict be recorded, so an operator whose adapter was misclassified can see
that it was and why, rather than discovering an empty capture.

`DeviceFlags::is_wireless()` is deliberately not used as a virtual signal. A
wireless adapter is a real adapter.

## R-5: Which statistics are per-interface

**Decision**: `SourceStats` becomes per-interface within `CaptureStats`, and
the capture-wide figure becomes a computed sum rather than a stored field.

Each handle has its own driver buffer, so `kernel_dropped` is a per-interface
quantity and always was; there has simply never been more than one interface to
reveal it. FR-029 forbids claiming per-interface precision for a capture-wide
counter, and the reverse error is just as bad: folding four interfaces' kernel
drops into one number tells an operator that a driver buffer is undersized
without telling them which, which is the diagnosis P-4 exists to make possible.

The change respects `stats.rs`'s own standing rule that no aggregate is stored.
`CaptureStats::source` stops being a field and becomes a method summing the
per-interface entries, so the capture-wide view cannot drift from its parts.

`buffer_dropped` and `sink_dropped` stay capture-wide, because the buffer and
the sinks are capture-wide. There is one buffer by section 12.4 and attributing
its evictions to whichever interface produced the evicted packet would be a
true statement that invites a false inference, namely that the busy interface
is the problem rather than the slow sink.

This is a third deviation, discovered in planning rather than before it. It is
recorded in the specification's deviation list alongside the other two.

## R-6: What is deferred, and to where

- Filter narrowing and maintenance, section 12.2 phases two and three: S13.
- The `doctor` presentation of the capture driver report: S14.
- The session anchor of section 12.7: the slice that builds correlation with
  external event logs. This slice carries driver timestamps only.
- Command line and profile wiring of the selection settings: S14.
- Linux and macOS live sources: section 28. The `pcap` crate covers libpcap on
  those targets, so the seam this slice fills is the one they will fill, but
  nothing here claims to have tested it.
