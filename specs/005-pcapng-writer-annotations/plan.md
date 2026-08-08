# Implementation Plan: pcapng Writer and Annotation Encoding

**Branch**: `feat/pcapng-writer-annotations` | **Date**: 2026-08-08 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from
`/specs/005-pcapng-writer-annotations/spec.md`

**Slice**: S06 (specification sections 13.1 to 13.4)

## Summary

Add a pcapng writer to `fragcap-sink` implementing the core `Sink` trait, so
fragcap produces a file for the first time. The file carries per-packet
attribution in the Enhanced Packet Block `opt_comment`, which unmodified
analyzers already display, and carries every section 12.4 loss counter in the
Interface Statistics Block, using standard fields where they exist and a
declared comment where they do not.

The technical approach is settled by research: no new dependency, little-endian
output regardless of host, a clock never read, and an annotation modelled as a
value with an encoder and a decoder rather than as a format string. The pcapng
structures the writer emits were verified against Wireshark 4.6.3 before this
plan was written, so the implementation starts from a table known to be correct
rather than one recited from a specification.

## Technical Context

**Language/Version**: Rust, edition 2021. Toolchain pinned at 1.96.0; minimum
supported 1.82, checked separately by `cargo xtask msrv`.

**Primary Dependencies**: None added. The workspace stays at its single
external dependency, `bytes`, which this slice does not use. Encoding is
`to_le_bytes` and `std::io::Write` from the standard library.

**Storage**: Files, written through a caller-supplied `std::io::Write`. The
writer never opens a path itself.

**Testing**: `cargo test --workspace --locked`, over the committed S04 fixture
corpus at `fixtures/`. Unit tests for the annotation grammar, structural
validation tests for the block layout, and golden comparison for byte-level
stability. tshark verification is documented in
[quickstart.md](quickstart.md) as a manual step and is deliberately not a gate;
see research R-6.

**Target Platform**: Any target the standard library supports. This slice adds
no platform-specific code, and `cargo xtask neutral` continues to prove core
builds without a capture backend.

**Project Type**: Library crate within a Cargo workspace.

**Performance Goals**: None set for this slice. Specification section 2.3 ranks
fidelity of observation above throughput, and the pipeline that would make a
throughput number meaningful arrives in S08. Recording a target here would be
inventing a requirement.

**Constraints**: Byte-identical output across runs and architectures, which
forbids reading a clock, the host byte order, the environment, or a locale. No
allocation requirement is imposed; the writer is on the sink thread, not the
capture thread.

**Scale/Scope**: One crate gains roughly four modules. Eight golden files, one
per corpus fixture. The largest fixture is 1,000 packets.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Applies how | Status |
| --- | --- | --- |
| P-1 Passive observation | No process handles, no injection, no driver. The writer touches a byte sink. | Pass, not engaged |
| P-2 Core stays platform-neutral | Writer lives in `fragcap-sink`, which already depends on `fragcap-core`. No new edge, no reverse edge. `cargo xtask deps` proves it. | Pass |
| P-3 Capture and attribution separate | A sink is neither. It consumes what the pipeline resolved. | Pass, not engaged |
| P-4 No silent loss | FR-031 puts fragcap's own counters in the file where no standard field exists. FR-030 keeps unattributed packets. FR-033 makes a refused packet an error, not a discard. | Pass, load-bearing |
| P-5 Compatibility outranks richness | The entire slice. `opt_comment` over custom options, verified against Wireshark 4.6.3. | Pass, load-bearing |
| P-6 Glossary first | New terms: annotation, attribution fidelity, golden. Entries land in the same change. | Pass, tracked in tasks |
| P-7 Wrappers stay thin | No wrapper in this slice. | Pass, not engaged |
| P-8 House standards | UTF-8 without BOM, LF, no em-dashes or en-dashes, 80 columns. Enforced by `cargo xtask lint`. | Pass |
| P-9 The instrument does not lie | Load-bearing three times: the timestamp narrowing is declared and confined to one site, a pre-epoch timestamp fails rather than being clamped, and fragcap losses are never reported as operating system losses. | Pass, load-bearing |

**Post-design re-check**: Pass, unchanged. The design added no dependency, no
platform code, and no path that discards or alters an observation. The one
place the design touches a constitution edge is FR-031, where section 13.2 as
written would have left two counters out of the file; the resolution satisfies
both the section and P-4, and is recorded as a decision rather than applied
silently.

## Project Structure

### Documentation (this feature)

```text
specs/005-pcapng-writer-annotations/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── writer-api.md    # Phase 1 output
├── checklists/
│   ├── requirements.md  # Spec quality
│   └── format.md        # Format correctness and output fidelity
└── tasks.md             # Phase 2 output, /speckit-tasks
```

### Source Code (repository root)

```text
crates/fragcap-sink/
├── src/
│   ├── lib.rs           # Re-exports; module documentation naming later slices
│   ├── annotation.rs    # Annotation value, derivation, encoder, decoder
│   ├── pcapng/
│   │   ├── mod.rs       # PcapngWriter, the Sink implementation
│   │   ├── block.rs     # Block framing and option framing primitives
│   │   └── interface.rs # Interface declaration and identifier assignment
│   └── error.rs         # Writer errors, mapped to core SinkError
└── (unit tests live in the modules above; no tests/ directory)

crates/fragcap/tests/
├── structure.rs         # Independent structural validator, FR-039
└── goldens.rs           # Golden generation and drift check, FR-038a

fixtures/goldens/        # One .fcapng per corpus fixture, committed
```

**Structure Decision**: The writer lives in `fragcap-sink` because
specification section 8.2 assigns sink implementations there and the crate's
module documentation already names S06 as the slice that fills it. The
annotation module sits beside the pcapng module rather than inside it, because
FR-025 requires S07 to reuse the derivation without reaching through a pcapng
namespace for it.

The structural validator lives in `tests/` rather than in `src/`, deliberately.
It is a test-only reader that must not become a supported capability, and
FR-039 requires it be independent of the writer's encoding functions; putting
it in the crate's public surface would invite the writer to share helpers with
it, which is exactly the shared-assumption problem it exists to avoid.

**The corpus-driven tests live in the `fragcap` facade, not in
`fragcap-sink`.** They need a `ReplaySource` from `fragcap-capture` and a
`ScriptedAttributor` from `fragcap-attr` to turn a fixture into packets worth
writing, and the facade is the only crate that legitimately depends on all
three. Putting them in `fragcap-sink` would mean a dev-dependency on a sibling,
which is the edge constitution P-3 exists to prevent.

This one is worth stating rather than assuming, because it would not be caught:
`cargo xtask deps` deliberately ignores `[dev-dependencies]`
(`xtask/src/deps.rs`, and its own test `ignores_dev_dependencies_in_core`), so
the violation would pass the mechanical gate and be visible only to a reviewer
who went looking. S04 met the same problem and placed its end-to-end test the
same way, for the same reason.

`fragcap-sink` therefore has no `tests/` directory. Its unit tests cover
framing and the annotation grammar, neither of which needs a fixture: a
synthetic packet is enough to assert bytes, and using a fixture there would buy
nothing but a dependency.

Goldens live under `fixtures/goldens/` rather than in the crate, next to the
corpus they are derived from, following the placement S04 established.

## Design decisions

Recorded here per the autopilot decision policy. Each was resolved against the
constitution, the specification, and existing patterns rather than raised.

**D-1: The writer is generic over `std::io::Write`, not over a file path.** A
test needs an in-memory buffer to compare bytes without touching a filesystem,
and section 25.1 requires tier 1 tests run unprivileged. A file is one such
target and the CLI supplies one in S14.

**D-2: Interfaces are declared explicitly, before packets.** The alternative,
inferring an interface from the first packet, would make the writer guess a
link type and a name it was never told, which is a fabricated observation under
P-9. FR-006 assigns identifiers in declaration order, which is also how pcapng
identifies them, so the two cannot drift.

**D-3: The Interface Statistics Block timestamp is derived from the last packet
written on that interface.** The block header carries a timestamp that has to
hold something; the obvious something is the current time, and a writer that
reads a clock there breaks every golden on the second run. See research R-4.

**D-4: `finish` consumes the writer.** The core `Sink` trait already takes
`self: Box<Self>` for exactly this reason, and it is what guarantees the
trailing statistics blocks are written once. A writer dropped without finishing
leaves a file whose completed blocks are readable, which is the bounded failure
story 4 scenario 4 asks for.

**D-5: The annotation decoder is liberal about percent-encoding case and strict
about everything else.** It reads files other tools may have written, so
rejecting lowercase hexadecimal would be wrong; it is the round-trip partner
for the encoder, so accepting a missing sentinel or an uppercase key would let
an encoder defect pass.

**D-7: The corpus-driven tests live in the facade, not in `fragcap-sink`.**
Found while starting implementation, after the analyze gate had passed. Writing
a fixture requires a `ReplaySource` and a `ScriptedAttributor`, which are
siblings of `fragcap-sink`; reaching them from its `tests/` directory means a
dev-dependency on a sibling, which is the edge P-3 exists to prevent. It would
not have been caught, because `cargo xtask deps` deliberately ignores
`[dev-dependencies]`. Recorded rather than quietly relocated, because the
mechanical gate's blind spot is the interesting part, not the file move.

**D-6: No `epb_flags` option.** pcapng defines a per-packet flags option
carrying a direction field, and writing direction there as well as in the
annotation would put the same fact in two places that can disagree. Section
13.3 places direction in the annotation. Recorded because the option is
discoverable and the duplication is tempting.

## Complexity Tracking

> No constitution violations. This section is empty by design.

The one place the design could have grown complexity is the structural
validator, which is a reader inside a project that does not read pcapng. It is
justified by FR-039: a writer verified only by its own encoding functions has
proven that two functions agree, not that the file is valid. It is confined to
`tests/`, is not part of the crate's surface, and is not a supported
capability.
