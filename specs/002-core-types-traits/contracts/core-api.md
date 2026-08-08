# Contract: fragcap-core Public Surface

**Slice**: S02

**Created**: 2026-08-08

`fragcap-core` exposes no command line and no wire protocol. Its contract is its
public Rust surface, which the other seven crates are written against. This
document states what that surface promises, so a later slice can tell a
deliberate guarantee from an incidental detail.

## What is promised

**The type set is complete for the pipeline in specification section 8.6.**
Every stage of that diagram can be expressed against these types without adding
one. A test wires stub implementations end to end to prove it.

**The four behavioral traits are usable as trait objects.** The pipeline owns
them across thread boundaries and holds a heterogeneous set of sinks selected at
runtime. Any change making one of them generic in a method breaks this
contract.

**Optional means "not resolved", never "not applicable".** Every optional field
on `CapturedPacket` is absent because a pipeline stage did not or could not fill
it, and the reason is recoverable from the combination of fields rather than
lost.

**Named counters are the contract, not the total.** A consumer reading
statistics gets one field per discard cause, named as specification section 12.4
names them. Totals are derived. A future counter is an added field, and
consumers that match exhaustively are expected to be updated.

**Error variants are meaningful and extensible.** A caller may branch on a
variant to decide whether to continue. Each error enum is non-exhaustive, so a
new variant is not a breaking change and callers must have a fallback arm.

## What is not promised

**No behavior.** Nothing in this crate captures, resolves, parses, or writes. A
caller that expects a method to do work has misread the slice.

**Field-level stability below 1.0.0.** The workspace is pre-1.0, and
`CHANGELOG.md` states that minor increments may carry breaking changes. Types
whose documentation names a later slice as their owner are expected to change
when that slice lands: `FilterProgram` at S13, `ProcessEvent` and
`ProcessRecord` at S11, `LinkType` when S09 discovers what the backend reports.

**No serialization format.** These types are not a wire format. The pcapng
encoding at S06 and the JSON Lines encoding at S07 own their own
representations, and neither is derived automatically from these shapes.

**No platform capability.** The crate names no platform facility and offers no
way to reach one. That is P-2, and it is proven by building for a target with no
capture backend.

## Stability of the seams

The five traits are the part of this surface intended to survive to 1.0.0
unchanged. They are transcribed from the architecture of record rather than
designed here, and a change to one is a change to specification section 8.5,
which requires the deviation process rather than a local edit.

The data types are expected to gain fields. The traits are expected not to gain
methods.
