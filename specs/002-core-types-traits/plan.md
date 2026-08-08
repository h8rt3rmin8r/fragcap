# Implementation Plan: Core Types and Traits

**Branch**: `feat/core-types-traits`

**Spec**: [spec.md](spec.md)

**Created**: 2026-08-08

**Slice**: S02 (specification sections 8.4, 8.5)

## Summary

Fill `fragcap-core` with the vocabulary from specification sections 8.4 and 8.5:
the seven types those sections name outright, the eight they reference without
defining, five traits, three error enums, and two statistics types. No
behavior. The deliverable is a set of seams that sixteen later slices are
written against, plus tests that prove the seams are expressible and that the
constitution constraints they encode actually hold.

Three constraints shape every decision below. Core must stay platform-neutral
(P-2). The statistics types must make an uncounted discard hard to write (P-4).
Nothing may offer a way to alter an observation (P-9).

## Technical Context

**Language**: Rust, edition 2021, built with the pinned 1.96.0 toolchain,
declared minimum 1.82.

**New external dependencies**: exactly one, `bytes` 1.12, MIT licensed, declared
minimum 1.57, with no required transitive dependencies. This is the workspace's
first external dependency.

**Testing**: `cargo test --workspace --locked`. Unit tests colocated with the
modules they cover, following the pattern `xtask` already established of testing
pure functions against deliberately wrong input.

**Target**: `x86_64-pc-windows-msvc` for the workspace; `fragcap-core`
additionally builds for `x86_64-unknown-linux-gnu`, which is the P-2 proof.

**Project type**: Rust library workspace. This slice touches one crate.

## Constitution Check

| Principle | Bearing on this slice | How it is satisfied |
| --- | --- | --- |
| P-1 Passive observation | Traits describe what an implementor must do | No trait obliges a process handle, an injection, or a hook. The process watcher exposes subscribe and snapshot only, both satisfiable from ETW and query-only enumeration. |
| P-2 Core platform-neutral | Directly binding | One dependency, `bytes`, is pure Rust with no platform surface. `cargo xtask neutral` and `cargo xtask deps` both prove it. |
| P-3 Capture and attribution separate | Directly binding | `PacketSource` and `FlowAttributor` are declared in separate modules, neither references the other, and a test asserts the module boundary. |
| P-4 No silent loss | Directly binding | Counters are named per specification section 12.4: `kernel_dropped`, `buffer_dropped`, `sink_dropped`, plus `filter_gaps`. Totals are computed, never stored. |
| P-5 Compatibility outranks richness | Indirect | `Timestamp` carries no format resolution, so the output layer decides compatibility rather than core forcing it. |
| P-6 Glossary first | Directly binding | Every term introduced gets a `docs/glossary.md` entry in this change. |
| P-7 Wrappers stay thin | Not applicable | No wrapper work in this slice. |
| P-8 House standards | Directly binding | SPDX headers, 100-column Rust, no dashes, `cargo xtask lint` clean. |
| P-9 Instrument does not lie | Directly binding | No setter rewrites an observed field. `orig_len` is separate from payload length so truncation is self-describing. Timestamp conversion happens once, at the output boundary, not in core. |

No principle requires justification for violation. Nothing in this slice is
complex enough to need an exception.

## Key Decisions

### D-1. The packet payload is `bytes::Bytes`

**Decision**: Take `bytes` 1.12 as the workspace's first external dependency and
alias the payload type to `bytes::Bytes`.

**Rationale**: A payload is cloned from the capture thread into a bounded ring,
drained by a sink thread, and fanned out to several sinks. `Bytes` clones are a
reference count bump rather than a copy, so a three-sink fan-out costs one
allocation instead of four. That is the hot path of the entire program, and it
is the reason the architecture of record wrote `Bytes` rather than `Vec<u8>`.

The crate is unusually well suited to being the first dependency admitted:
MIT, which is on the allowlist; a declared minimum of 1.57, far below our 1.82,
so it cannot force the minimum up; and no required transitive dependencies, so
the audit surface grows by exactly one crate.

**Alternatives**: `Vec<u8>` costs a full copy per sink and would have to be
replaced before S15 ships streaming. `Arc<[u8]>` clones cheaply but cannot be
cheaply sliced, and header parsing in S03 slices constantly.

### D-2. `Timestamp` is defined locally, in nanoseconds since the Unix epoch

**Decision**: A newtype over `i64` nanoseconds since the Unix epoch, defined in
core, with no external date-time dependency and no per-packet resolution field.

**Rationale**: Section 12.7 stores microseconds in the capture file, matching
the pcapng per-interface declared resolution. Carrying that resolution on every
packet would put format knowledge in core, which P-2 forbids, and would make
arithmetic between two timestamps consult both operands' resolutions.

One canonical internal unit removes both problems. Nanoseconds is finer than any
capture backend supplies, so converting inward is lossless, and the single
outward conversion at the output boundary is the one site P-9 compliance has to
be checked at. Signed rather than unsigned so that a difference between two
timestamps is expressible in the same type without a separate duration type.

**Alternatives**: `std::time::SystemTime` cannot be constructed from a driver's
raw counter without going through a duration and cannot represent the pre-epoch
values a misconfigured clock can produce. `chrono` and `time` are both large
dependencies whose calendar handling this project never needs.

### D-3. Error types are hand-written enums, not derived

**Decision**: Three enums, each with named variants, marked `#[non_exhaustive]`,
with `Display` and `std::error::Error` implemented by hand.

**Rationale**: `thiserror` is the ecosystem default and would save perhaps forty
lines. It is a proc-macro crate, so admitting it pulls `syn`, `quote`, and
`proc-macro2` into the graph. For the workspace's first dependency set, growing
the audit surface from one crate to four to save forty lines of mechanical code
is the wrong trade, and it is a trade that is easy to make later if the count of
error types grows.

`#[non_exhaustive]` matters because S09, S15, and S16 each add failure modes
that cannot be enumerated now, and without it each addition is a breaking change
for every caller.

**Alternatives**: `thiserror` as above. Opaque error structs were rejected in
clarify: a caller must distinguish a timeout, which is normal and continues the
capture loop, from a device disappearing, which is terminal.

### D-4. Statistics are two types, composed rather than merged

**Decision**: `SourceStats` carries what the capture backend reports.
`CaptureStats` carries fragcap's own counters and holds a `SourceStats` by
value. Totals are methods, never fields.

**Rationale**: Section 12.4 names three drop counters and says they are
maintained and reported separately, because the remedy differs: kernel drops
mean an undersized driver buffer, buffer drops mean a slow sink, sink drops mean
a slow downstream consumer. An operator who sees one blended number cannot
choose a remedy.

Composition rather than merging also keeps P-9 intact. The backend's counts are
another component's observation that fragcap is relaying; folding fragcap's own
accounting into those fields would alter what that component said. Holding them
in a named sub-structure reports both faithfully.

Totals as methods rather than fields is what makes FR-025 structural: a stored
total can drift from its parts, a computed one cannot.

### D-5. The three attribution states are derived, not stored

**Decision**: Keep `Option<FlowKey>` and `Option<Attribution>` exactly as the
architecture of record writes them. Add a method that reports which of three
states a packet is in, and a test that pins the mapping.

**Rationale**: No flow key means attribution was never attempted, because there
was nothing to attempt it with. A flow key with no attribution means attempted
and unresolved. A flow key with an attribution means resolved. All three are
already distinguishable, so an explicit enum would add a discriminant to a
per-packet struct to store information the struct already carries.

**Alternatives**: A three-state enum replacing the optional attribution. It
would deviate from the architecture of record and cost memory on the hot path
for no new information.

### D-6. The process watcher subscription uses the standard library channel

**Decision**: `std::sync::mpsc::Receiver<ProcessEvent>`.

**Rationale**: The architecture of record writes `Receiver<ProcessEvent>`
without naming a crate. The standard library provides one, it is
platform-neutral, and it costs no dependency. `subscribe(&self)` taking a shared
reference means the implementor holds its sender registry behind interior
mutability, which is S11's problem and is entirely tractable.

**Alternatives**: `crossbeam-channel` is more capable, supports multiple
consumers, and would be a second dependency admitted for a trait that has no
implementation until S11. Deferring the choice to the slice that actually needs
it costs nothing now. Recorded so S11 can revisit rather than inherit silently.

### D-7. Deviation recorded: eight referenced types are never defined

**Decision**: Define all eight in core, and record the gap for promotion to
specification section 29.

Sections 8.4 and 8.5 reference `Timestamp`, `Bytes`, `StageId`, `LinkType`,
`Endpoint`, `FilterProgram`, `ProcessEvent`, and `ProcessRecord` in type
signatures, and define none of them. That is a gap in the architecture of record
rather than a decision it made, so filling it is this slice's work, and the
constitution requires the divergence be recorded rather than resolved silently.

`FilterProgram`, `ProcessEvent`, and `ProcessRecord` are declared in the minimal
shape their signatures require and are expected to grow in S13, S11, and S11
respectively. Each carries documentation naming the slice that fills it, so a
later contributor does not read a thin type as a finished one.

## Project Structure

### Documentation (this feature)

```text
specs/002-core-types-traits/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── core-api.md
├── tasks.md
└── checklists/
    ├── requirements.md
    └── constitution.md
```

### Source Code (repository root)

```text
crates/fragcap-core/src/
├── lib.rs          module declarations, re-exports, crate documentation
├── flow.rs         Proto, FlowKey, AttributionKey, Direction, Endpoint
├── packet.rs       Timestamp, RawPacket, CapturedPacket, AttributionState
├── attribution.rs  Attribution, StageId
├── link.rs         LinkType
├── filter.rs       FilterProgram
├── process.rs      ProcessEvent, ProcessRecord
├── stats.rs        SourceStats, CaptureStats
├── error.rs        SourceError, AttrError, SinkError
└── traits.rs       PacketSource, FlowAttributor, ProcessWatcher, Sink, Dissector
```

One module per concern rather than one large `lib.rs`, so that the P-3 boundary
between acquisition and attribution is visible in the file listing and a
reviewer can see at a glance that no attribution type lives in a capture module.

`traits.rs` holds all five traits together because they are the seam set and are
read as a group. The P-3 separation is enforced by neither trait referencing the
other, which a test asserts, rather than by file placement.

## Dependency Graph

Unchanged from S01 in shape. `fragcap-core` gains one leaf:

```text
fragcap-core -> bytes 1.12 (MIT, no transitive dependencies)
```

`cargo xtask deps` continues to pass: `bytes` is not a platform crate, not an
I/O crate, and not a capture library. The check's existing assertion that core
has no workspace-internal dependencies is unaffected.

## Complexity Tracking

No constitution violation requires justification.

One item is worth naming as accepted cost. Hand-writing `Display` and `Error`
for three enums (D-3) is more code than a derive would need, and that code has
to be kept correct by hand as variants are added in later slices. The cost is
accepted for a smaller first dependency graph, and the decision is explicitly
revisitable when the error type count grows.
