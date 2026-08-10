# Implementation Plan: Live Capture Source and Interfaces

**Branch**: `feat/live-capture-interfaces` | **Date**: 2026-08-09 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`/specs/009-live-capture-interfaces/spec.md`

## Summary

Give fragcap a live packet source, and give the pipeline more than one of them.
The parts that already exist do not change shape: the bounded buffer, the drop
accounting, the parser, and both writers keep their behavior, and the
single-interface capture keeps its byte-for-byte output. What changes is that a
packet now knows which interface it arrived on, that `PacketSource` can cross a
thread boundary, and that `fragcap-capture` has a Windows backend behind a
feature that is off by default.

The technical approach is settled in [research.md](research.md): the `pcap`
crate 2.4.0 supplies the driver binding, `std::net` supplies the default route,
and a documented substring rule supplies the virtual-interface verdict. No
other dependency is added.

Three type changes cross the architecture of record and are recorded as
deviations: `Send` on `PacketSource`, an interface identifier on
`CapturedPacket`, and per-interface `SourceStats` inside `CaptureStats`.

## Technical Context

**Language/Version**: Rust, edition 2021, workspace floor 1.82 (enforced by
`cargo xtask msrv`)

**Primary Dependencies**: `pcap` 2.4.0, optional, `fragcap-capture` only,
behind the `live` feature. `bytes` remains core's only dependency.

**Storage**: None. Output goes to the sinks S06 and S07 built.

**Testing**: `cargo test` at tiers 0 and 1 on any machine; tier 2 behind
`--features live` on a Windows runner with npcap installed. Specification
section 25.2.

**Target Platform**: Windows for the live source. `fragcap-core` and
`fragcap-capture` build for a target with no capture backend, which
`cargo xtask neutral` proves.

**Project Type**: Rust workspace, library plus facade plus command line.

**Performance Goals**: None stated for this slice. The capture thread must not
wait on the sink, which S08 already guarantees and this slice must not regress.

**Constraints**: No packet interception driver, no injection, no hooking, no
process handle carrying memory rights. npcap detected, never bundled,
downloaded, installed, or vendored. `fragcap-core` platform-neutral. Every
discard path counted.

**Scale/Scope**: One thread per selected interface, all feeding one 65,536
packet ring. Ordinary use is one or two interfaces; broad capture on a
development machine may reach a dozen.

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1. Both passes
recorded.*

| Principle | Pre-design | Post-design | Notes |
| --- | --- | --- | --- |
| P-1 Passive observation only | PASS | PASS | See D-8. The binding exposes a transmit call fragcap never makes, and a repository lint makes that mechanical rather than asserted. |
| P-2 Core stays platform-neutral | PASS | PASS | `pcap` enters `fragcap-capture` only. `fragcap-core` gains no dependency. `cargo xtask deps` and `cargo xtask neutral` both cover this. |
| P-3 Capture and attribution separate | PASS | PASS | Nothing in this slice touches `fragcap-attr`. Selection takes plain values rather than reaching for a sibling, per FR-012. |
| P-4 No silent loss | PASS | PASS | See D-10. Source statistics become per-interface so a kernel drop names its own driver buffer. Retirement is reported and correctly advances no drop counter. |
| P-5 Compatibility outranks richness | PASS | PASS | Multi-interface pcapng uses the format's own interface blocks. A single-interface capture is byte-identical to today's, per SC-005. |
| P-6 Glossary first | PASS | PASS | Six terms identified in D-11, all written in this change. |
| P-7 Wrappers stay thin | PASS | PASS | No wrapper work in this slice. |
| P-8 House standards apply | PASS | PASS | `cargo xtask lint` unchanged in what it demands of this slice's files. |
| P-9 The instrument does not lie | PASS | PASS | See D-5 and D-9. Device loss is determined by observation rather than by matching an error string, and the virtual-interface verdict is presented as the heuristic it is. |

**Licensing gate**: `pcap` and its entire transitive graph are MIT OR
Apache-2.0. No copyleft. The npcap software development kit is acquired by the
workflow at build time and never committed; no repository file changes that.

**Deviation gate**: three deviations, all recorded in the specification and
carried into a changelog decisions fragment for promotion to specification
section 29. None of them weakens a principle; each widens a type that predates
the case it now has to serve.

## Decisions

### D-1: `pcap` 2.4.0 supplies the driver binding

Measured in [research.md](research.md) R-1 against a hand-rolled binding
linking at build time, a hand-rolled binding loading at runtime, and `rawsock`.
The deciding argument is that the alternative to a dependency here is not
arithmetic over bytes, as it was in S03 and S06, but a C ABI transcribed by
hand with nothing checking the layout. A wrong offset yields plausible
timestamps that are wrong, which is a P-9 failure that no test over synthetic
data would catch.

### D-2: One feature, `live`, off by default

`fragcap-capture` declares `live`, the facade re-exports it, nothing enables it
by default. `cargo xtask ci` therefore passes on a machine with neither the
driver nor the software development kit, which is SC-011.

The workflow's anticipated name `platform-tests` is not adopted. The feature
gates a capability, not a test suite, and naming a capability for its tests
invites someone to enable it for the tests and be surprised that the library
changed.

### D-3: The default route comes from `std::net`

A UDP socket bound and connected to an off-link address reports the source
address the routing table chose. `connect` on UDP transmits nothing. This adds
no dependency, needs no IP Helper, and behaves identically on the targets
section 28 has in view. Research R-3.

### D-4: `InterfaceId` is a newtype over a small integer, assigned at selection

Selection assigns each chosen interface an index, and that index is the
identity for the run. It does not come from the platform, because platform
names are not guaranteed unique, which the specification's edge cases call out.
It is not a string, because it is compared once per packet.

The mapping from `InterfaceId` to the platform's name and description lives in
the selection outcome, which is what the writers use to declare interfaces.

### D-5: Device loss is determined by observation, not by string matching

`pcap::Error` has no variant for a device that has gone away; the case arrives
as the general `PcapError(String)`. Matching on the message would work until a
driver update or a non-English locale changed the text, and would then
downgrade a lost device to an unmodelled failure silently.

Instead, on a terminal backend error the source re-enumerates and asks whether
its interface is still present. Absent means `SourceError::DeviceLost` with the
interface named; present means `SourceError::Backend` carrying the driver's own
detail. This is an observation rather than a guess, it costs one enumeration on
a path that is already terminal, and it degrades to the honest answer rather
than the flattering one when it cannot tell.

### D-6: The pipeline takes a vector of sources, each with its identity

`Pipeline::new` accepts a collection of sources, each paired with its
`InterfaceId` and its link type, and spawns one thread per source. The
alternative, a multiplexing source presenting one `PacketSource`, was rejected
during clarification: it needs a second buffer where section 12.4 specifies
one, and it would have to invent a side channel for the identifier the pipeline
is about to attach anyway.

The parser is per-thread and takes its link type from its own interface, which
is FR-026. That falls out of the arrangement rather than needing machinery: each
capture thread already owns its own parser state.

### D-7: `Send` on `PacketSource`, and what it costs

Adding the bound is one line. What it constrains is every implementor: the
replay source, the stub sources in the traits module's own tests, and anything
S15 or S16 adds. `ReplaySource` owns a `PcapReader` over a file handle and is
already `Send`; the test stubs are plain data. The live source holds a
`pcap::Capture<Active>`, which the crate documents as `Send`.

The bound is added to the trait, and a compile-time assertion is added
alongside the existing dyn-compatibility test so that an implementor that stops
being `Send` fails at the trait rather than at the pipeline.

### D-8: The transmit capability, and why a lint rather than an argument

`pcap` exposes packet transmission on an active capture. The constitution says
a dependency providing a prohibited capability fails the dependency audit, so
this needs an answer rather than a shrug.

The answer is that transmission is not on the section 19.3 denylist. That list
names packet interception and filtering drivers, code injection, function
hooking, process handles carrying memory rights, layered service providers, and
executable image modification. npcap's NDIS capture driver is explicitly
permitted by section 19.2, and the crate that binds it offering a send call is
the same kind of fact as the standard library offering file deletion.

That argument is correct and it is also exactly the kind of argument that
decays. So `cargo xtask lint` gains a check that fails if any fragcap crate's
source names the transmit API. The posture stops being a claim in a plan and
becomes a gate a future change has to defeat deliberately.

### D-9: The virtual-interface rule is a heuristic and says so

A documented substring match over the adapter description, held as data in one
place. Research R-4. Two things keep it honest: it only ever excludes from
automatic selection, never from explicit selection; and the verdict is recorded
per interface so a misclassification is visible rather than silently producing
an empty capture.

### D-10: Source statistics become per-interface, and the total becomes computed

`CaptureStats::source` stops being a field and becomes a method summing
per-interface entries. Each handle has its own driver buffer, so a kernel drop
was always per-interface; there has simply never been a second interface to
show it. Folding them would tell an operator a driver buffer is undersized
without saying which.

`buffer_dropped` and `sink_dropped` stay capture-wide. There is one buffer and
the sinks are shared, and attributing an eviction to the interface that
produced the evicted packet would be true in a way that invites the false
inference that the busy interface is at fault rather than the slow sink.

The change respects `stats.rs`'s standing rule that no aggregate is stored, so
the capture-wide view cannot drift from its parts. Research R-5.

### D-11: Terms introduced, per P-6

Six terms need glossary entries in this change: bootstrap filter, interface
identifier, interface inventory, selection outcome, virtual interface, and
interface retirement. Each gets an entry in `docs/glossary.md` following the
section 4.3 template.

### D-12: The `platform` workflow changes, with a dated decision

`.github/workflows/platform.yml` is a pinned artifact. This slice gives it real
triggers, enables the `live` feature, and keeps the software development kit
acquisition step it has always had. The change is recorded as a dated decision
in `changelog.d/`, per the constitution and `CONVENTIONS.md`.

## Project Structure

### Documentation (this feature)

```text
specs/009-live-capture-interfaces/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── capture-api.md   # Phase 1 output, the public surface this slice moves
├── checklists/
│   ├── requirements.md
│   └── acquisition.md
└── tasks.md             # Phase 2, produced by /speckit-tasks
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── traits.rs            # PacketSource gains Send (D-7)
├── packet.rs            # CapturedPacket gains InterfaceId (D-4)
├── stats.rs             # CaptureStats gains per-interface source stats (D-10)
├── interface.rs         # NEW. InterfaceId, InterfaceRecord, inventory,
│                        # selection settings, selection outcome, the
│                        # selection decision itself. Platform-neutral: a
│                        # decision over a value, per FR-010.
└── pipeline/
    └── mod.rs           # Several sources, one thread each (D-6)

crates/fragcap-capture/src/
├── lib.rs               # Feature gate wiring
├── replay.rs            # Carries an InterfaceId; unchanged otherwise
└── live/                # NEW, cfg(all(windows, feature = "live"))
    ├── mod.rs           # LiveSource, the PacketSource implementation
    ├── enumerate.rs     # Device::list adapted into an InterfaceInventory
    ├── driver.rs        # Presence and version detection, never installation
    └── route.rs         # Default route by std::net (D-3)

crates/fragcap-sink/src/
├── pcapng/interface.rs  # More than one declaration
└── json.rs              # Interface named per record in multi-interface runs

crates/fragcap/tests/
├── corpus_pipeline.rs   # Extended: two sources, two interfaces
└── multi_interface.rs   # NEW, tier 1: identity through to both writers

crates/fragcap-capture/tests/
└── live.rs              # NEW, tier 2, cfg(feature = "live")

xtask/src/lint.rs        # The transmit-API check (D-8)
docs/glossary.md         # Six entries (D-11)
.github/workflows/platform.yml  # Real triggers, live feature (D-12)
```

**Structure Decision**: The interface vocabulary and the selection decision go
in `fragcap-core`, not in `fragcap-capture`. Selection is a pure decision over
an inventory value, per FR-010, and the pipeline that consumes the outcome is
in core by specification section 8.2. Putting it in the capture crate would
force core to name a capture type to describe its own packets, inverting
section 8.3 exactly as putting the parser there would have in S03.

The live backend and the enumeration that produces an inventory from a real
machine stay in `fragcap-capture`, because both touch the platform. The seam
between them is the inventory value, which is also what makes selection
testable without a driver.

## Complexity Tracking

No constitution violations require justification. The table is empty by design.

Two things worth naming as costs rather than violations:

| Cost | Why accepted |
| --- | --- |
| Three type changes crossing section 8.4 and 8.5 | Each is required by section 12.1's multi-interface capture, each was identified before implementation, and all three are recorded as deviations for promotion to section 29. The alternative is a live source that cannot say where a packet came from. |
| A runtime dependency with a platform surface | The first in the project. Bounded by being optional, off by default, confined to one crate, and covered by `cargo xtask deps` and `cargo xtask neutral`. Research R-1 records what was measured. |
