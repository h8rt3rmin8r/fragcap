# Contract: the replay source and scripted attributor

**Slice**: S04

**Created**: 2026-08-08

What S06, S07, S08, S13, and S16 write their tests against.

## Surface

```rust
// fragcap-capture
pub mod pcap {
    pub struct PcapReader { /* private */ }

    impl PcapReader {
        pub fn from_bytes(data: Vec<u8>) -> Result<Self, SourceError>;
        pub fn next_record(&mut self) -> Option<RawPacket>;
        pub fn link_type(&self) -> LinkType;
        pub fn snaplen(&self) -> u32;
        pub fn stats(&self) -> &ReplayStats;
    }

    pub struct ReplayStats {
        pub truncated_record: u64,
        pub impossible_length: u64,
        pub caplen_exceeds_wire: u64,
        pub caplen_exceeds_snaplen: u64,
    }

    impl ReplayStats {
        pub fn skipped(&self) -> u64;
    }
}

pub mod replay {
    pub struct ReplaySource { /* private */ }

    impl ReplaySource {
        pub fn open(path: impl AsRef<Path>) -> Result<Self, SourceError>;
        pub fn from_bytes(data: Vec<u8>) -> Result<Self, SourceError>;
        pub fn replay_stats(&self) -> &ReplayStats;
    }

    impl PacketSource for ReplaySource { /* the seam, unchanged */ }
}

// fragcap-attr
pub mod script {
    pub struct AttributionScript { /* private */ }

    impl AttributionScript {
        pub fn parse(text: &str) -> Result<Self, ScriptError>;
        pub fn load(path: impl AsRef<Path>) -> Result<Self, ScriptError>;
    }

    pub enum ScriptError { /* named causes, each carrying its line */ }
}

pub mod scripted {
    pub struct ScriptedAttributor { /* private */ }

    impl ScriptedAttributor {
        pub fn new(script: AttributionScript) -> Self;
        /// Not on the seam, and must not be. See plan D-7.
        pub fn set_now(&mut self, now: Timestamp);
        pub fn now(&self) -> Timestamp;
    }

    impl FlowAttributor for ScriptedAttributor { /* the seam, unchanged */ }
}
```

## Guarantees

**The seams are unchanged.** Neither `PacketSource` nor `FlowAttributor` gains
a method, a parameter, or a bound in this slice. `set_now` is inherent to the
scripted attributor, and a test asserts the trait definitions are untouched.

**Replay is deterministic.** The same bytes yield the same packet sequence, on
every run and every platform: same order, same timestamps, same lengths, same
payloads. Byte order and timestamp resolution are read from the file's magic
number and never from the host.

**Nothing is altered.** Records are delivered in file order, never reordered. A
timestamp is converted by the unit the file declares and never rounded. A
record whose captured length exceeds its on-wire length is delivered with both
lengths exactly as recorded. Payload bytes are delivered unmodified.

**Nothing is silently skipped.** Every record not delivered as the file
described it advances exactly one named counter. Two causes stop reading and
two do not, and which is which is fixed by the contract below.

`next_record` returns `None` both at a clean end of file and after a cause that
stopped reading early, because at that layer there is nothing more to hand back
either way. The counters are how a caller tells them apart:
`stats().skipped()` is zero for a clean end and non-zero otherwise.
`ReplaySource` maps both to `Closed`, deliberately, because a pipeline's
response is the same, and a pipeline that wants to know whether the file was
whole reads `replay_stats`.

**The two crates cannot reach into each other.** `fragcap-capture` and
`fragcap-attr` have no dependency edge between them, which `cargo xtask deps`
enforces mechanically against the expected edge set. That is the evidence for
P-3 here: the replay source could not contain attribution logic, nor the
scripted attributor packet acquisition, without an edge the check would reject.

**Backend drop counts are zero.** A replay source reports `received` and
nothing else. Its own skip counters are reachable through `replay_stats`, never
through `SourceStats`, because presenting fragcap's accounting as a backend's
observation is the confusion S02 separated those types to prevent.

**Exhaustion is terminal.** `next_packet` returns `Err(SourceError::Closed)`
when the file is spent, never `Ok(None)`, which means "timed out, keep going"
and would spin a pipeline forever.

**Filters are accepted and not applied.** `set_filter` always succeeds and
changes nothing about what is delivered. A replay source does not filter.

**A script cannot demand what a socket table cannot answer.** Loading rejects a
UDP entry naming a remote endpoint and a TCP entry without one. Matching goes
through the same key derivation and wildcard bind rule the real attributor will
use, so a test that passes against a script is one S10 must satisfy.

**An ambiguous script does not load.** Overlapping windows for one flow are a
named error, not a precedence rule.

## Behavioral contract by input class

| Input | Result | Counter |
| --- | --- | --- |
| Any of the four magic numbers | Opens; byte order and resolution from the magic | none |
| Fewer than 24 bytes, or an unknown magic | `Err(Backend)` at open | none |
| A well-formed record | Delivered with its timestamp and both lengths | none |
| A zero-length record | Delivered, empty payload | none |
| A timestamp earlier than the one before it | Delivered in file order, unreordered | none |
| A link type fragcap cannot parse | Delivered; the parser counts it later | none |
| Fewer than 16 bytes remain | Reading stops | `truncated_record` |
| Fewer than `caplen` bytes remain | Reading stops | `impossible_length` |
| `caplen` greater than `orig_len` | Delivered, both lengths unchanged | `caplen_exceeds_wire` |
| `caplen` greater than the file snaplen | Delivered | `caplen_exceeds_snaplen` |
| End of file | `Err(Closed)` | none |

| Script input | Result |
| --- | --- |
| A TCP entry with a local and remote endpoint | Loads; resolves on both |
| A UDP entry with `*` as remote | Loads; resolves on the local endpoint alone |
| A UDP entry naming a remote endpoint | `Err`, naming the line |
| A TCP entry with `*` as remote | `Err`, naming the line |
| A UDP entry bound to a wildcard address | Matches a datagram on a specific interface address |
| An `endpoint` statement | Loads; the address appears in `active_endpoints` |
| An `endpoint` statement with a malformed address | `Err`, naming the line |
| Two windows for one flow that intersect | `Err`, naming the line |
| Two windows for one flow that abut | Loads; half-open, so they do not intersect |
| `always` alongside any other window for one flow | `Err`, naming the line |
| A flow the script does not mention | `resolve` returns nothing |
| An `unowned` entry covering now | `resolve` returns nothing |
| No window set, no clock set by the caller | `always` entries still resolve |

Each row is a test. The tables are the coverage obligation SC-003, SC-004, and
SC-006 state, written out so a reviewer can count them.

## The corpus

```text
fixtures/<name>.pcap      at most 64 KiB
fixtures/<name>.script
```

Eight pairs, from specification section 25.3. The directory is at most 256 KiB.
Every address is from a documentation range or is loopback, and every payload
byte is the filler pattern.

Regeneration:

```bash
FRAGCAP_UPDATE_FIXTURES=1 cargo test -p fragcap-capture --test corpus
```

Without the variable, that test checks rather than writes, and fails naming
anything that drifted.

## What this contract does not cover

**Reading pcapng.** fragcap writes it in S06; nothing reads it.

**Filtering.** S13 owns filters and may decide whether a replay source should
ever apply one in software.

**Wiring the clock to the packet.** The scripted attributor takes an instant
from its caller. Setting it from the packet under attribution is the pipeline's
job in S08.

**Golden output.** Specification section 25.4 compares pipeline output, and
there is no pipeline until S08.

## Stability

`ReplayStats` fields are public and additive: a later slice finding a new way a
record can be malformed adds a counter, which is a minor change for a caller
reading fields by name.

`ScriptError` is expected to gain variants as the format grows and is
`#[non_exhaustive]` for that reason. The script format itself is a test fixture
format, not a user-facing surface, and may be replaced wholesale if S05's
profile parser makes a richer format free.
