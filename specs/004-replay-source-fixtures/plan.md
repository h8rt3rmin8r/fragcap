# Implementation Plan: Replay Source and Fixture Corpus

**Branch**: `feat/replay-source-fixtures`

**Spec**: [spec.md](spec.md)

**Created**: 2026-08-08

**Slice**: S04 (specification sections 25.1, 25.3)

## Summary

Fill two of the eight crates for the first time. `fragcap-capture` gains a
classic pcap reader and a `ReplaySource` implementing `PacketSource`.
`fragcap-attr` gains a `ScriptedAttributor` implementing `FlowAttributor` from
a declared script. Between them and the eight committed fixtures they read,
the pipeline becomes a deterministic function from fixture input to output,
which is the claim specification section 25.1 has been making since S01.

No new dependency. A pcap file is a twenty-four byte header followed by
sixteen-byte record headers, and the script format is deliberately trivial.

Three constraints shape the decisions below. The substrate must be
deterministic, because every later slice's tests inherit whatever
nondeterminism gets in. Every skip must be counted (P-4), because a reader that
quietly drops a record turns a damaged fixture into a passing test. And nothing
may alter what a fixture records (P-9), because the fixture is the observation
as far as every tier 1 test is concerned.

## Technical Context

**Language**: Rust, edition 2021, pinned 1.96.0 toolchain, declared minimum
1.82.

**New external dependencies**: none. The workspace stays at one, `bytes`.

**Testing**: `cargo test --workspace --locked`. Unit tests colocated; the
corpus generator and its drift check live in one integration test target in
`fragcap-capture`.

**Target**: `x86_64-pc-windows-msvc` for the workspace; `fragcap-core` still
additionally builds for `x86_64-unknown-linux-gnu`, unchanged by this slice.

**Project type**: Rust library workspace. This slice touches two crates and
adds a committed `fixtures/` corpus.

## Constitution Check

| Principle | Bearing on this slice | How it is satisfied |
| --- | --- | --- |
| P-1 Passive observation | Indirect | Reading a file opens nothing, injects nothing, hooks nothing. No fixture is captured from a real session. |
| P-2 Core platform-neutral | Indirect | Nothing is added to `fragcap-core`. File I/O lives in `fragcap-capture`, where P-2 does not reach. |
| P-3 Capture and attribution separate | Directly binding | The replay source and the scripted attributor are the canonical demonstration that the two are separable: they are in different crates, neither names the other, and the test that wires them together does so from outside both. |
| P-4 No silent loss | Directly binding | Four named reader counters, one per way a record is not delivered as-is, in their own type. The backend drop counts stay zero, so fragcap's accounting is never presented as the backend's. |
| P-5 Compatibility outranks richness | Not applicable | No output format work. |
| P-6 Glossary first | Directly binding | Seven terms introduced, seven entries added in this change. |
| P-7 Wrappers stay thin | Not applicable | No wrapper work. |
| P-8 House standards | Directly binding | SPDX headers, no dashes, `cargo xtask lint` clean. Fixture scripts are text and follow the repository's line ending rule. |
| P-9 Instrument does not lie | Directly binding | The reader delivers what the file records: no reordering, no length reconciliation, no resolution rounding, no dropping a record for being unusual. Fixtures carry only filler payloads and documentation or loopback addresses, and the drift check proves the committed bytes are what the generator describes. |

No principle requires justification for violation.

## Key Decisions

### D-1. The reader and the replay source live in `fragcap-capture`

**Decision**: A `pcap` module and a `replay` module in `fragcap-capture`.

**Rationale**: Specification section 8.2 defines that crate as the home of
packet source backends, "live capture and replay". There is no tension here;
it is written down.

Reading a file is I/O, which is why this is not in `fragcap-core`: P-2 excludes
I/O crates from core, and while `std::fs` is not a crate, putting file reading
in the platform-neutral crate would invite the next thing that is.

### D-2. Classic pcap only, whole file in memory

**Decision**: Read classic pcap, all four magic numbers, and read the whole
file into memory at open.

**Rationale**: Section 25.3 names `.pcap` for the corpus. Reading pcapng is
nobody's requirement: fragcap writes it in S06 and nothing reads it back.

Whole-file reading is chosen for determinism and simplicity. Fixtures are
capped at 64 KiB by FR-031, so the memory is irrelevant, and a reader with no
buffering has no buffering bug and no partial-read path to get wrong. The live
source in S09 has entirely different constraints and shares nothing with this.

The four magic numbers encode two independent choices, byte order and timestamp
resolution, and the reader must handle all four combinations rather than the
two it is likely to meet.

| Magic | Byte order | Resolution |
| --- | --- | --- |
| `0xa1b2c3d4` | same as reader | microseconds |
| `0xd4c3b2a1` | swapped | microseconds |
| `0xa1b23c4d` | same as reader | nanoseconds |
| `0x4d3cb2a1` | swapped | nanoseconds |

### D-3. Exhaustion is `Closed`, not a timeout

**Decision**: When the file is exhausted, `next_packet` returns
`Err(SourceError::Closed)`.

**Rationale**: The seam already distinguishes these. `Ok(None)` means the
timeout elapsed and the caller should keep going; `Closed` means the source
will produce nothing further and is terminal. A finished file is the second.

Returning `Ok(None)` would be the tempting choice and would make any pipeline
loop spin forever on a finished fixture, which is a hang rather than a failure.

**Alternative**: a new `SourceError` variant for end of file. Rejected: the
enum's variants exist so a caller can act differently, and there is no action
that differs between "closed" and "end of file".

### D-4. Open failures reuse `SourceError::Backend`

**Decision**: A file that cannot be opened, or whose magic is unrecognized,
produces `SourceError::Backend { detail }` with precise detail text.

**Rationale**: Same reasoning as D-3 from the other side. The enum is
`#[non_exhaustive]` so adding a variant is cheap and non-breaking, but a
variant is only worth having when a caller would act differently on it. Every
open failure here is terminal and unrecoverable, exactly like `Backend`, so a
new variant would add a name without adding a decision.

Recorded so a later slice that does need the distinction, for instance a CLI
wanting to say "this is not a capture file" differently from "permission
denied", knows the option was left open deliberately.

### D-5. Reader counters in their own type, backend drops stay zero

**Decision**: A `ReplayStats` type in `fragcap-capture` with one counter per
skip cause, reachable through an inherent method. `SourceStats::received`
reports what was delivered; `kernel_dropped` and `interface_dropped` stay zero.

The four causes:

| Counter | Cause | Delivered |
| --- | --- | --- |
| `truncated_record` | The file ends part way through a record header or its data | no |
| `impossible_length` | A record declares more data than the file can supply | no, and reading stops |
| `caplen_exceeds_wire` | A record's captured length exceeds its on-wire length | yes, both lengths unchanged |
| `caplen_exceeds_snaplen` | A record's captured length exceeds the file's declared snapshot length | yes |

**Rationale**: The delivered column is the part worth being explicit about. Two
of these are the file lying about a record whose bytes are nonetheless present,
and P-9 says an observation is not withheld because it is inconvenient. The
other two are the file being unable to supply bytes at all, where there is
nothing to deliver.

Reconciling a `caplen_exceeds_wire` record by adjusting one of its lengths is
the specific alteration this decision forbids. The contradiction is the
observation, and repairing it would hide a defect in whatever wrote the file.

Backend drop counts stay zero because there is no kernel and no interface here.
Reporting the reader's own skips in those fields would fold fragcap's
accounting into a backend's report, which is what S02 decision D-4 separated
the two types to prevent.

### D-6. Scripts key on the attribution key

**Decision**: A script entry names a protocol, a local endpoint, and a remote
endpoint or `*`. Matching goes through `FlowKey::attribution_key()` and
`AttributionKey::local_matches_bind`, the S02 machinery, rather than a parallel
comparison.

Loading rejects a UDP entry naming a concrete remote, and a TCP entry naming
`*`, because neither corresponds to anything a socket table can answer.

**Rationale**: This is what stops the double and the real attributor drifting
apart. A test that passes against a script is a test S10 has to satisfy,
because both resolve through the same key derivation and the same wildcard bind
allowance. Had the script keyed on the full flow key, it could express a UDP
attribution requiring a remote endpoint, which is exactly the fabrication
specification section 8.4 prohibits, and a test built on it would demand
behavior S10 must never implement.

### D-7. The clock lives on the double, not on the seam

**Decision**: `ScriptedAttributor::set_now(Timestamp)`, an inherent method.
`resolve` reads the stored instant. `refresh` succeeds and does nothing. The
`FlowAttributor` trait is not touched.

**Rationale**: A real attributor needs no timestamp parameter, because it reads
a socket table that is already current. Only a scripted one needs to be told
what "now" is, so the parameter belongs to the double.

S02 fixed these five traits as the part of the surface intended to reach 1.0.0
unchanged. Widening one so a test double can be written would pay an
architectural cost for a testing convenience, and would hand every real
implementation a parameter it does not want. SC-006b asserts the seam is still
unwidened after this slice, so a later attempt is noticed.

An entry with no window matches at any time, so a caller that never sets a
clock still resolves. The default instant is the epoch.

### D-8. The script format

**Decision**: A line-oriented text format. Blank lines and lines beginning with
`#` are ignored. Three statements:

```text
flow <proto> <local> <remote|*> <window> owner <pid> <name>
flow <proto> <local> <remote|*> <window> unowned
endpoint <proto> <addr>
```

`<window>` is `always` or `<from_ns>..<to_ns>`, half-open, in nanoseconds since
the Unix epoch, matching the timestamps the fixtures carry.

**Rationale**: The alternative is TOML, which the profile schema needs in S05
and which is a better format. Adopting it here means adopting a parser and its
proc-macro dependencies on behalf of a slice that has not made that decision on
its own merits, and S05 should choose against the profile's requirements rather
than inherit a choice made for a test fixture.

`unowned` exists rather than relying on the absence of a window, because a
reviewer reading a fixture should see that a flow is deliberately unattributed
rather than inferring it from silence. Both kinds participate in the
overlap check.

Comments exist so the generator can annotate a nanosecond integer with what it
means, which is the only thing making an absolute timestamp readable.

### D-9. The generator lives in the corpus test target

**Decision**: `crates/fragcap-capture/tests/corpus.rs` holds both the generator
and the drift check. The check runs by default. Setting
`FRAGCAP_UPDATE_FIXTURES=1` makes the same test write the corpus instead.

**Rationale**: Three homes were considered.

`xtask` matches the precedent section 25.4 sets for regenerating goldens, and
was declined because `xtask` has no dependencies at all today, deliberately, and
would need either a duplicate of the frame construction or an edge into the
product graph. `cargo xtask deps` would then need its expected edge set widened,
which is a change to what a constitution check enforces, for a convenience.

`fragcap-capture` proper was declined because it ships a fixture generator to
every consumer of the crate for this repository's benefit.

A new workspace member was declined because it needs a manifest, a license file
for `cargo xtask license`, and a place in the publication order, all to hold one
file.

The test target ships nowhere, needs no manifest, and puts generation and
checking in the same file so they cannot disagree about the format. The cost is
that regeneration is an environment variable rather than a subcommand, which is
less discoverable, and `quickstart.md` documents it for that reason.

### D-10. Fixture content rules

**Decision**: Addresses come from `192.0.2.0/24`, `198.51.100.0/24`,
`203.0.113.0/24`, `2001:db8::/32`, and the loopback addresses. Link layer
addresses come from a stated locally-administered pair. Payload bytes are a
repeating documented filler pattern. The timestamp base is a constant.

**Rationale**: The privacy rule in section 25.3 is only enforceable if it is
mechanical. "Contains no session token" is not something a test can evaluate;
"every payload byte is the filler pattern" is. The rule is inverted from
detecting bad content to requiring known content, which is checkable and
strictly stronger.

Loopback is in the permitted set because `loopback.pcap` cannot exercise
direction ambiguity without it, and a loopback address identifies no operator.
The first draft of the spec omitted it and thereby forbade a fixture the same
document requires; the checklist caught it.

### D-11. Deviations recorded for specification section 29

Two, both already in the spec and repeated here so the promotion list is in one
place:

1. Section 25.3's `burst.pcap` must both exceed a 65,536 packet buffer and be
   small, which cannot both hold. The fixture carries the sustained rate and
   S08's test supplies the capacity.
2. Section 25.3 requires an attribution script per fixture without defining
   one. This slice defines the format.

## Project Structure

### Documentation (this feature)

```text
specs/004-replay-source-fixtures/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── replay-api.md
├── tasks.md
└── checklists/
    ├── requirements.md
    └── fixtures.md
```

### Source Code (repository root)

```text
crates/fragcap-capture/src/
├── lib.rs              + module declarations and re-exports
├── pcap.rs             the classic pcap reader and ReplayStats
└── replay.rs           ReplaySource

crates/fragcap-capture/tests/
└── corpus.rs           the generator, the drift check, and the per-fixture
                        condition assertions

crates/fragcap-attr/src/
├── lib.rs              + module declarations and re-exports
├── script.rs           the attribution script format and its parser
└── scripted.rs         ScriptedAttributor

fixtures/
├── tcp-session.pcap        + .script
├── udp-gameplay.pcap       + .script
├── ipv6-mixed.pcap         + .script
├── fragmented.pcap         + .script
├── loopback.pcap           + .script
├── malformed.pcap          + .script
├── port-reuse.pcap         + .script
└── burst.pcap              + .script
```

The per-fixture condition assertions live beside the generator because they are
the same knowledge read twice: the generator says what it put in, and the
assertions say what must still be there. Splitting them would let one drift
from the other, which is the failure the whole corpus check exists to prevent.

## Dependency Graph

Unchanged. No crate is added and no edge is added. `fragcap-capture` and
`fragcap-attr` each keep their single edge to `fragcap-core`, and the
`cargo xtask deps` expected set needs no edit.

The corpus test target's use of the fixtures is a file path, not a dependency,
which is why `fragcap-attr` can read the scripts a `fragcap-capture` test
generated without any edge between them.

## Testing Strategy

Test-driven, bottom up, in three independent tracks that meet at the end.

**The reader**, against byte arrays built in the test rather than against
files, so every malformed case is constructible and no fixture has to be
damaged on disk to reach a counter. One test per magic number, one per skip
cause, each asserting exactly one counter moved.

**The script and the attributor**, against strings rather than files, for the
same reason. The port reuse case is the one worth writing first, because it is
the reason the time dimension exists.

**The corpus**, generated and checked in one target. The condition assertions
are what make the corpus trustworthy: each fixture parses its own contents and
asserts the property section 25.3 claims for it, so a generator change that
drops a fragment or flattens a chain fails immediately rather than in S08.

**The claim itself**, last: one test that opens a fixture, parses each packet
with the S03 parser, resolves each flow against the fixture's script, and
asserts a plausible end-to-end result. That test is SC-001, and it is the first
time the three slices are exercised together.

## Complexity Tracking

No constitution violation requires justification.

Two costs are accepted and named.

The corpus is eight fixtures and eight scripts, of which this slice consumes
perhaps half. `burst.pcap` and `port-reuse.pcap` exist for S08 and S10. Building
them now rather than when they are needed is deliberate, because section 25.3
defines the corpus as a set and a corpus assembled piecemeal across five slices
would never be reviewed as a whole. Their condition assertions run here, so they
cannot rot unnoticed in the meantime.

Regeneration through an environment variable is less discoverable than a
subcommand, and is the price of keeping the generator out of `xtask` and out of
every shipped crate. It is documented in `quickstart.md` rather than left to be
found.
