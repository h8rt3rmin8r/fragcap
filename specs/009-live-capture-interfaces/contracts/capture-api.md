# Contract: The Capture Surface After S09

**Slice**: S09

**Date**: 2026-08-09

**Phase**: 1

This slice is a library slice, so its contract is the public Rust surface other
crates and later slices compile against. Three parts of that surface move, and
one is new. Signatures are given in the shape they are intended to land; the
narrative reasons are in [plan.md](plan.md) and [data-model.md](data-model.md).

## 1. The behavioral seam, `fragcap-core::traits`

```rust
pub trait PacketSource: Send {
    fn next_packet(&mut self, timeout: Duration) -> Result<Option<RawPacket>, SourceError>;
    fn set_filter(&mut self, filter: &FilterProgram) -> Result<(), SourceError>;
    fn stats(&self) -> SourceStats;
    fn link_type(&self) -> LinkType;
}
```

Only the bound changes. The method set is untouched, which matters because
section 8.5 intends this surface to reach 1.0.0 and every method removed or
added is a larger commitment than a bound that every existing implementor
already satisfies.

**Compatibility**: source-breaking for any implementor that is not `Send`.
There are none in the workspace. Asserted by a `static_assertions`-style
compile-time check placed next to the existing dyn-compatibility test, so the
failure names the trait rather than surfacing three layers up in the pipeline.

## 2. The interface vocabulary, `fragcap-core::interface`

New module. The full field lists are in [data-model.md](data-model.md); this is
the callable surface.

```rust
pub struct InterfaceId(u32);

pub fn select(
    inventory: &InterfaceInventory,
    settings: &SelectionSettings,
) -> Result<SelectionOutcome, SelectionError>;
```

`select` is the whole of the section 12.1 precedence and it is a pure function.
It opens nothing, enumerates nothing, and touches no platform surface, which is
FR-010 and what lets SC-002 be verified on any machine.

**Invariant**, asserted rather than documented: for any inventory and settings
that do not produce an error,

```text
outcome.selected.len() + outcome.excluded.len() == inventory.interfaces.len()
```

No interface is unaccounted for. This is the selection-side analogue of the
conservation identity S08 established for packets, and it fails loudly if a
future precedence rule drops an interface on the floor.

## 3. The pipeline, `fragcap-core::pipeline`

```rust
pub struct SourceBinding {
    pub id: InterfaceId,
    pub source: Box<dyn PacketSource>,
}

impl Pipeline {
    pub fn new(
        sources: Vec<SourceBinding>,
        attributor: Box<dyn FlowAttributor>,
        config: PipelineConfig,
    ) -> Result<Self, ConfigError>;
}
```

The link type is not carried on the binding: `PacketSource::link_type` already
answers it per source, and duplicating it would create two answers that can
disagree. Each capture thread reads it once at start and parses against it,
which is FR-026.

**Compatibility**: breaking for the single-source constructor S08 shipped.
Every caller in the workspace is a test, and the tests that worked around
`Pipeline` not being movable across threads can stop doing so.

A `ConfigError::NoSources` variant is added, so that constructing a pipeline
over nothing fails at construction rather than running to completion having
captured nothing.

## 4. Statistics, `fragcap-core::stats`

```rust
pub struct CaptureStats {
    // ... unchanged fields ...
    pub sources: Vec<(InterfaceId, SourceStats)>,
}

impl CaptureStats {
    pub fn source(&self) -> SourceStats;              // summed, never stored
    pub fn source_for(&self, id: InterfaceId) -> Option<SourceStats>;
}
```

**Compatibility**: breaking for `stats.source` as a field. Both writers read it
and become callers of `source()`.

The summed view widens each `u32` the driver reports into `u64` before adding,
so a long capture on several busy interfaces cannot overflow the total into a
smaller number than one of its parts.

## 5. The live source, `fragcap-capture::live`

Gated `#[cfg(all(windows, feature = "live"))]`. Absent, not stubbed, everywhere
else, which is FR-021.

```rust
pub struct LiveSource { /* ... */ }

impl LiveSource {
    pub fn open(
        record: &InterfaceRecord,
        options: LiveOptions,
    ) -> Result<Self, SourceError>;
}

pub struct LiveOptions {
    pub snaplen: u32,
    pub promiscuous: bool,
    pub read_timeout: Duration,
}

pub fn enumerate() -> Result<InterfaceInventory, SourceError>;
pub fn detect_driver() -> DriverReport;
```

`enumerate` requires no open handle, which is FR-003: an operator can be told
what exists before anything is captured.

`detect_driver` returns a report rather than a `Result`, because absence is an
answer and not a failure. It never downloads, installs, or invokes an
installer, under any path, which is FR-043 and is the sharpest edge in this
slice.

### Error mapping

The mapping is part of the contract because a wrong one is invisible.

| `pcap` outcome | `SourceError` |
| --- | --- |
| `Error::TimeoutExpired` | `Ok(None)`, not an error |
| `Error::NoMorePackets` | `Closed` |
| `Error::PcapError`, interface gone on re-enumeration | `DeviceLost { detail }` |
| `Error::PcapError`, interface still present | `Backend { detail }` |
| Filter rejected by the backend | `FilterRejected { detail }` |

The third and fourth rows are plan decision D-5. The binding cannot distinguish
them, so fragcap determines it by looking rather than by matching the error
string, and reports the honest answer when it cannot tell.

## 6. Output, `fragcap-sink`

The pcapng writer accepts more than one `InterfaceDeclaration` and resolves
each packet's `InterfaceId` to the block it declared. The error variant
refusing a second interface is removed; the one refusing an undeclared
interface stays, because a packet naming something never declared is still a
defect and still must not be written against a fabricated declaration.

The JSON Lines writer names the interface on every record when the capture
holds more than one, and omits the key when it holds exactly one. Section 13.3,
unchanged in intent from S06 and S07.

**Guarantee**: a single-interface capture produces byte-identical output to
what S06 and S07 produce today for the same input. That is SC-005, and it is
checked against the committed goldens rather than reasoned about.

## 7. Repository checks

`cargo xtask lint` gains one check: no fragcap crate's source may name the
capture binding's transmit API. Plan decision D-8. This is the mechanical form
of the P-1 argument that the dependency is acceptable because fragcap never
transmits, and it exists so that the argument cannot quietly stop being true.

`cargo xtask deps` is unchanged and must continue to pass, with no edge from
`fragcap-capture` to any sibling. SC-013.
