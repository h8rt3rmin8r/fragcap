# Implementation Plan: JSON Lines Writer

**Branch**: `feat/json-lines-writer` | **Date**: 2026-08-08 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/006-json-lines-writer/spec.md`

**Slice**: S07 (specification section 13.5)

## Summary

Add a JSON Lines writer to `fragcap-sink` beside the pcapng writer, emitting a
header object, one object per packet, and a trailer object, with a payload-free
mode. It obtains which attribution keys are present from the `Annotation`
derivation S06 exposed, so the two formats cannot disagree about the same
packet, and renders them under this format's own conventions.

No runtime dependency. The writer is hand-rolled because the exact byte shape
is the deliverable, and `serde_json` enters as a dev-dependency so every
emitted line is parsed by a third-party reader in the test suite. Timestamps
are built by integer arithmetic and never pass through a float.

## Technical Context

**Language/Version**: Rust, edition 2021. Toolchain 1.96.0; minimum 1.82.

**Primary Dependencies**: None added at runtime. `serde_json` 1.0 as a
dev-dependency of `fragcap-sink` and `fragcap`, for output validation only.
Verified available and its `arbitrary_precision` and `preserve_order` features
confirmed non-default, which is the substance of research R-1.

**Storage**: A caller-supplied `std::io::Write`.

**Testing**: `cargo test --workspace --locked` over the S04 fixture corpus.
Three layers per research R-6: unit tests on escaping and formatting,
third-party parse of every line, and cross-format agreement against the pcapng
writer.

**Target Platform**: Any target the standard library supports. No
platform-specific code.

**Project Type**: Library crate within a Cargo workspace.

**Performance Goals**: None set. Section 2.3 ranks fidelity above throughput and
the pipeline that would make a number meaningful is S08.

**Constraints**: Byte-identical output across runs and architectures. No clock,
environment, locale, or host property read. No floating point on the timestamp
path.

**Scale/Scope**: One crate gains two modules. Eight goldens. Largest fixture is
400 packets.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Applies how | Status |
| --- | --- | --- |
| P-1 Passive observation | No handles, no injection. The writer formats text. | Pass, not engaged |
| P-2 Core stays platform-neutral | Writer is in `fragcap-sink`. Core gains nothing, and the dev-dependency is not on core. | Pass |
| P-3 Capture and attribution separate | A sink is neither. | Pass, not engaged |
| P-4 No silent loss | FR-030 and FR-031 put every counter in the trailer, present even at zero. FR-032 keeps unattributed packets. FR-033 makes a refusal an error. | Pass, load-bearing |
| P-5 Compatibility outranks richness | The format's whole point is off-the-shelf consumers. FR-029 verifies it against a real parser. | Pass, load-bearing |
| P-6 Glossary first | New terms: JSON Lines, payload-free mode, trailer record. Entries land in the same change. | Pass, tracked in tasks |
| P-7 Wrappers stay thin | No wrapper. | Pass, not engaged |
| P-8 House standards | Enforced by `cargo xtask lint`. | Pass |
| P-9 The instrument does not lie | Load-bearing three times: no float on the timestamp, no guessed wire order under an unknown direction, no fidelity invented. | Pass, load-bearing |

**Post-design re-check**: Pass, unchanged. The design adds no runtime
dependency and no platform code. The one edge worth naming is the
dev-dependency, which the constitution governs by license rather than
prohibiting; `serde_json` and its tree are MIT or Apache-2.0, inside the
allowlist. `cargo xtask deps` does not inspect dev-dependencies, which S06
recorded as a blind spot; this slice's use is a third-party crate rather than a
sibling, so it is not the case that blind spot hides.

## Project Structure

### Documentation (this feature)

```text
specs/006-json-lines-writer/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── jsonl-format.md  # Phase 1 output
├── checklists/
│   ├── requirements.md
│   └── format.md
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/fragcap-sink/
├── src/
│   ├── lib.rs           # Re-exports
│   ├── annotation.rs    # Unchanged; S06 derivation, consumed here
│   ├── error.rs         # Gains the JSON writer's named conditions
│   ├── json/
│   │   ├── mod.rs       # JsonLinesWriter, the Sink implementation
│   │   ├── escape.rs    # String escaping and hex encoding
│   │   └── number.rs    # Exact decimal timestamp, integer only
│   └── pcapng/          # Unchanged
└── Cargo.toml           # Gains [dev-dependencies] serde_json

crates/fragcap/tests/
├── common/mod.rs        # Gains a JSON render alongside the pcapng one
├── goldens.rs           # Gains the JSON corpus goldens
├── structure.rs         # Unchanged
└── agreement.rs         # New: the two formats carry the same facts

fixtures/goldens/        # Gains one .jsonl per fixture
```

**Structure Decision**: `escape.rs` and `number.rs` are separate from `mod.rs`
because they are the two places this format can be wrong in a way that is
invisible by reading. Both are pure functions over values, unit-testable
without constructing a packet, and both have a third-party oracle available:
`serde_json` for escaping, exact decimal expectations for numbers.

The cross-format agreement test is a new target in the facade rather than a
case inside `goldens.rs`, because it answers a different question. `goldens.rs`
asks whether output changed; `agreement.rs` asks whether the two outputs still
mean the same thing, which can fail while every golden passes.

The corpus tests stay in the facade for the reason S06 recorded as D-7: they
need a replay source and a scripted attributor, which are siblings of
`fragcap-sink`.

## Design decisions

**D-1: Hand-rolled writer, third-party parser in tests.** Research R-1. The
exact bytes are the deliverable, and the two `serde_json` features required to
produce them are both non-default and both global to the crate.

**D-2: Timestamps are integer arithmetic, end to end.** Research R-2. There is
no `f64` anywhere on the path, so there is no site at which precision could be
lost and no reviewer question about whether a given conversion is safe.

**D-3: The writer holds the interface set, not one interface.** Section 13.5's
header declares an interface set, and each record names its own. The type is a
slice of names in declaration order, indexed the same way the pcapng writer
indexes its declarations, so S08 can drive both from one structure. Unlike the
pcapng writer, nothing here breaks with more than one, because a JSON record
carries its interface explicitly and the trailer's counters are not attributed
per interface.

**D-4: `finish` consumes the writer**, as in S06 and for the same reason: the
trailer is written exactly once.

**D-5: Payload mode is fixed at construction.** Section 14.1 sets it per sink
in the destination specification, so it is a property of the sink rather than a
per-record argument. Making it per-record would let one stream mix modes, which
no consumer could interpret.

**D-6: No `Deserialize` for the record types.** A round trip through this
format is not a capability this project offers, and defining one would create a
second definition of the format for the parser to drift from. The tests parse
into `serde_json::Value` and assert on fields, which tests the bytes rather
than a symmetric implementation.

## Complexity Tracking

> No constitution violations.

The one addition worth naming is the dev-dependency. It is justified by
FR-029's requirement that validity be judged by something other than this
writer, and confined by FR-037 to test code. The alternative, a hand-written
JSON validator, would have reproduced S06's weaker arrangement in a place where
a better one is freely available.
