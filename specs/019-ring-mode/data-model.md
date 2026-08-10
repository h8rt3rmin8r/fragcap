# Phase 1 Data Model: Ring mode and triggers

The slice introduces two types in `fragcap-sink` and no new type in
`fragcap-core`. It reuses `CapturedPacket`, `SinkFactory`, `Sink`, and
`CaptureStats` unchanged.

## RingWindow

The bound on the retained set.

| Field / variant | Meaning |
| --- | --- |
| `Duration(std::time::Duration)` | Retain packets whose capture instant is within this window measured back from the newest retained packet. |
| `Size(u64)` | Retain the newest packets whose total captured length (`CapturedPacket::captured_len()`) is within this many bytes. |

Invariants:

- A `RingWindow` is always explicit; ring mode has no implicit default window
  (the absence of `--ring` is a configuration error, not a default).
- The two variants are mutually exclusive: `--ring` parses to exactly one, a
  duration tried first then a size (the existing `parse_ring` in the CLI already
  yields this shape as `crate::args::RingWindow`; the sink's `RingWindow` is the
  library-side equivalent the CLI maps onto).

Note: the CLI already carries a `RingWindow` enum in `crates/fragcap-cli/src/args.rs`
(`Duration | Size`). The sink defines its own `RingWindow` (the library API is not
allowed to depend on the CLI crate, per P-2 dependency direction), and the CLI maps
its parsed value onto the sink's when constructing the `RingSink`.

## RingSink

A `Sink` that retains a rolling window in memory and materializes it at finish.

| Field | Meaning |
| --- | --- |
| `path: PathBuf` | The `--out` dump target. |
| `window: RingWindow` | The retention bound. |
| `factory: SinkFactory` | Builds the pcapng encoder at dump time (header preamble + IDBs). |
| `retained: VecDeque<CapturedPacket>` | The rolling window, oldest at the front. |
| `retained_bytes: u64` | Running sum of `captured_len()` over `retained`, for the size window. |
| `evicted: u64` | Count of packets evicted from the window, surfaced as the sink's own accounting. |

State / behavior:

- **`write(packet)`**: clone the packet to the back of `retained`, add its
  `captured_len()` to `retained_bytes`, then evict from the front per the window
  (below), and return `Ok(())` unconditionally.
- **Eviction (size)**: while `retained.len() > 1` and
  `retained_bytes > size`, pop the front, subtract its length, increment
  `evicted`.
- **Eviction (duration)**: let `newest` be the back packet's instant; while
  `retained.len() > 1` and the front packet's instant is more than `window`
  before `newest`, pop the front, subtract its length, increment `evicted`.
- **`flush()`**: a no-op (nothing is written until finish); returns `Ok(())`.
- **`finish(self, stats)`**: create the `--out` file, build an encoder via
  `factory.build(...)`, `write` each retained packet in front-to-back (capture)
  order, then `encoder.finish(stats)`. Propagate any IO error as `SinkError`.

Invariants:

- After every `write`, the retained set is within the window, except that at
  least one packet (the newest) is always retained (R5). Therefore
  `retained.len() >= 1` whenever at least one packet has been written.
- `retained` is ordered by arrival (push order), which for the offline replay
  source is capture order; the dump preserves that order.
- The bytes the dump emits are produced solely by the unchanged pcapng writer, so
  a whole-input dump (window larger than the input) is byte-comparable to a plain
  `--out` file capture (FR-012).
- `evicted` plus `retained.len()` equals the number of packets `write` accepted;
  this is the sink's local conservation identity and is distinct from the
  pipeline's.

## Reused, unchanged

- **`CapturedPacket`** (`fragcap-core`): `Clone`; `ts: Timestamp`,
  `captured_len() -> usize`, `interface: InterfaceId`. Retained by value (cheap:
  the payload is a reference-counted `Payload`).
- **`SinkFactory`** (`fragcap-sink`): `build(Box<dyn Write + Send>) -> Box<dyn Sink>`
  writing the header preamble. The `RingSink` holds a `Pcapng` factory built from
  the declared interfaces.
- **`Sink`** (`fragcap-core`): the trait `RingSink` implements.
- **`CaptureStats`** (`fragcap-core`): passed through to the encoder's `finish`,
  so the dump carries the run's real statistics trailer exactly as the file sink
  does.
