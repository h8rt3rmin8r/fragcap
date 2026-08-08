# Research: Core Types and Traits

**Slice**: S02

**Created**: 2026-08-08

Phase 0 findings. Each entry resolves one question the plan could not answer by
reading the architecture of record.

## R-1. What backs the packet payload type

**Question**: Sections 8.4 writes `Bytes` without saying what provides it.

**Decision**: `bytes` version 1.12, aliased in core.

**Evidence gathered**: Queried the registry directly rather than relying on
recollection. Version 1.12.1 is the current stable release, licensed MIT, with a
declared `rust-version` of 1.57 and no required transitive dependencies. The
`serde` integration is optional and stays off.

**Why it matters that the minimum is 1.57**: the workspace declares 1.82. A
dependency with a higher declared minimum would silently raise the effective
minimum and make the `msrv` check fail for a reason unrelated to fragcap's own
code. 1.57 leaves 25 minor versions of headroom.

**Why MIT matters**: specification section 20.4 restricts dependency licenses to
MIT, Apache-2.0, BSD two and three clause, ISC, Unicode-DFS, and Zlib. MIT is on
the list, so `deny.toml` needs no amendment to admit this crate. That is worth
noting: the first dependency admitted does not require relaxing the policy,
which is a good property for the precedent it sets.

**Alternatives considered**:

| Option | Why rejected |
| --- | --- |
| `Vec<u8>` | One full payload copy per sink. A three-sink fan-out costs four allocations where `Bytes` costs one. Would have to be replaced before S15. |
| `Arc<[u8]>` | Clones cheaply but slicing requires a new allocation or a separate offset type. S03 slices constantly during header parsing. |
| `Cow<'_, [u8]>` | Introduces a lifetime into every packet type, which propagates into the bounded buffer and across the thread boundary. Not viable. |

## R-2. How a timestamp should be represented

**Question**: Sections 8.4 writes `Timestamp` without defining it. Section 12.7
says capture files store microseconds, matching the pcapng declared resolution.

**Decision**: A local newtype over `i64` nanoseconds since the Unix epoch, with
no resolution field and no external dependency.

**Reasoning**: Three properties are needed. It must be constructible from a
capture driver's raw counter, which is an integer pair rather than a calendar
value. It must not carry format knowledge, because core is platform-neutral and
format-neutral. And any conversion must happen at exactly one place, so P-9
compliance is checkable by reading one function rather than auditing every write
site.

A fixed nanosecond count satisfies all three. It is finer than any backend
supplies, so the inward conversion is lossless. The outward conversion to
microseconds happens in the pcapng writer at S06, which is where the interface's
declared resolution already lives.

Signed rather than unsigned so subtraction yields the same type. An unsigned
count would need a separate signed duration type for the session anchor
correlation described in section 12.7.

**Alternatives considered**:

| Option | Why rejected |
| --- | --- |
| `std::time::SystemTime` | Not constructible from a raw driver counter without a duration round trip, and awkward for pre-epoch values a misconfigured clock produces. |
| `chrono::DateTime<Utc>` | Large dependency, calendar handling never needed, and would raise the audit surface substantially for the first dependency set. |
| `time::OffsetDateTime` | Same objection as chrono. |
| Carrying `(seconds, subsec, resolution)` | Puts pcapng's per-interface resolution into core, which P-2 forbids, and makes arithmetic consult both operands' units. |

## R-3. Whether to derive error implementations

**Question**: Three error enums need `Display` and `std::error::Error`.

**Decision**: Hand-write both. No derive crate.

**Reasoning**: `thiserror` is the ecosystem default and is genuinely good. It is
also a proc-macro crate, so admitting it adds `syn`, `quote`, and `proc-macro2`,
taking the dependency graph from one crate to four. The saving is roughly forty
lines of entirely mechanical code across three enums.

For the slice that establishes what the dependency graph looks like, and that
makes the audit meaningful for the first time, a four-fold increase in audit
surface to avoid forty mechanical lines is the wrong trade. The decision is
cheaply reversible: if the error type count grows past a handful, adding
`thiserror` later is a one-line manifest change and a mechanical edit.

**Alternatives considered**: `thiserror` as above. `anyhow` is for applications
rather than libraries and erases the variant distinction that FR-022 requires.
`snafu` is heavier than `thiserror` for the same benefit.

## R-4. What channel type backs the process watcher subscription

**Question**: Section 8.5 writes `Receiver<ProcessEvent>` without naming a
crate.

**Decision**: `std::sync::mpsc::Receiver`.

**Reasoning**: The standard library provides a `Receiver` that satisfies the
signature, is platform-neutral, and costs no dependency. `subscribe(&self)`
taking a shared reference obliges an implementor to hold its senders behind
interior mutability, which is ordinary and is S11's concern.

The honest caveat: `std::sync::mpsc` is single-consumer, so each `subscribe`
call must create a fresh channel and register its sender. If S11 finds it needs
multiple consumers of one stream, or select across channels, `crossbeam-channel`
is the answer and is MIT and Apache-2.0 dual licensed. Deferring that to the
slice that has the requirement is better than admitting a dependency now for a
trait with no implementation for nine slices.

Recorded so S11 revisits deliberately rather than inheriting the choice
unexamined.

## R-5. What the statistics counters are actually called

**Question**: The spec requires one named counter per discard cause. What are
the causes, and what are their names?

**Finding**: The architecture of record already answers this, in a place the
slice scope did not point at. Section 12.4 gives a table:

| Counter | Meaning |
| --- | --- |
| `kernel_dropped` | Dropped by the capture driver before fragcap |
| `buffer_dropped` | Dropped by fragcap's bounded buffer |
| `sink_dropped` | Dropped by a sink that could not accept |

Section 13's sample summary output adds a fourth counted quantity, filter gaps
during narrowing. Requirement FR-17 in section 7.4 adds packets captured,
attributed, and unattributed.

**Decision**: Use exactly these names. Do not invent alternatives, and do not
abbreviate.

**Why this is worth an entry**: the temptation was to design the statistics
types from the principle alone, since P-4 states the rule clearly. Doing so
would have produced plausible names that disagreed with the architecture of
record, and the disagreement would have surfaced at S08 when the pipeline
started incrementing them. Searching the specification for the counter names
before designing the type cost one grep and avoided a rename across three
slices.

## R-6. Whether the declared minimum toolchain survives

**Question**: S01 recorded that the `msrv` check was vacuous with an empty
dependency graph. Does 1.82 still hold once `bytes` is in?

**Finding**: Yes. `bytes` 1.12.1 declares 1.57. The workspace declares 1.82. No
other dependency is added.

**Consequence**: the `msrv` check stops being vacuous and starts constraining
something, but it does not yet constrain anything tightly, because the single
dependency's minimum is far below the declared one. It becomes a real constraint
when a dependency with a minimum near 1.82 enters the graph.

This must be verified rather than assumed, because it is exactly the class of
claim the constitution's evidence rule targets. `cargo xtask msrv` is run as a
task in this slice and its output read.
