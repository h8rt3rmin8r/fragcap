# Contract: RingSink retention and dump

`RingSink` in `fragcap-sink` implements the existing `fragcap_core::traits::Sink`
trait. It is constructed with a dump path, a `RingWindow`, and a `SinkFactory`
(pcapng, built from the declared interfaces).

## Construction

```text
RingSink::create(path: PathBuf, window: RingWindow, factory: SinkFactory) -> RingSink
```

- Holds no open file during capture; the file is created at `finish`.
- `window` is `Duration(d)` or `Size(bytes)`, exactly one.

## Sink trait behavior

### `write(&mut self, packet: &CapturedPacket) -> Result<(), SinkError>`

1. Clone `packet` onto the back of the retained deque; add `packet.captured_len()`
   to the retained-bytes total.
2. Evict from the front until the retained set is within the window, but never
   below one packet:
   - **Size window**: while `len > 1` and `retained_bytes > size`, pop front,
     subtract its captured length, count one eviction.
   - **Duration window**: with `newest` = the back packet's instant, while
     `len > 1` and `front.ts` is more than `window` before `newest`, pop front,
     subtract, count one eviction.
3. Return `Ok(())` **unconditionally**. A ring never fails a packet and is never
   retired for its own eviction (P-4: conservation preserved; the eviction is the
   sink's own counted accounting).

### `flush(&mut self) -> Result<(), SinkError>`

No-op; returns `Ok(())`. Nothing is written to disk until `finish`.

### `finish(self: Box<Self>, stats: &CaptureStats) -> Result<(), SinkError>`

1. Create the dump file at `path`.
2. Build a pcapng encoder via `factory.build(...)`, writing the Section Header
   Block and one Interface Description Block per declared interface.
3. `write` each retained packet to the encoder in front-to-back (capture) order.
4. `encoder.finish(stats)` to write the run's real statistics trailer.
5. Any IO failure is returned as `SinkError`; the failure names the path.

## Guarantees

- **G1 (validity)**: the dump file is a single pcapng an unmodified analyzer
  opens: SHB, IDBs, then the retained packet blocks in capture order (P-5).
- **G2 (recent tail)**: after any `write`, the retained set is exactly the newest
  packets fitting the window, with at least the newest packet always retained.
- **G3 (whole-input equivalence)**: with a window larger than the input, no
  packet is evicted, and the dumped packet records equal a plain `--out` file
  capture of the same input, none lost, reordered, or duplicated (FR-012).
- **G4 (conservation)**: `write` accepts every packet, so the pipeline invariant
  (received + buffer_dropped + refusals = captured) holds unchanged; the evicted
  count is the sink's own accounting, distinct from `sink_dropped`.
- **G5 (no alteration)**: retained packets are dumped byte-for-byte as the file
  sink would write them; the only omission is the counted eviction of old packets
  the operator's window excluded (P-9 declared omission).

## Local conservation identity (asserted in tests)

For a completed ring capture: `evicted + retained == packets the sink accepted`.
This is the sink-local analogue of the pipeline's conservation check and is what a
new eviction path lacking a counter would fail.
