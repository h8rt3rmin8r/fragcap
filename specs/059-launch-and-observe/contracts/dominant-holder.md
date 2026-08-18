# Contract: Dominant socket-holder aggregation

## The tally

`CaptureStats.holder_tally: BTreeMap<Arc<str>, u64>`.

- **Increment**: exactly once per `AttributionState::Resolved` packet, at the same
  site in `pipeline/mod.rs` where `packets_attributed` increments, keyed by
  `packet.attribution.process`.
- **Absorb**: `CaptureStats::absorb` adds each key's count from the other stats
  (several capture threads each hold a tally).
- **Additive invariant**: the tally contributes to no drop total, no conservation
  term, no writer output, and no completion summary. Adding a packet to it never
  changes `fragcap_dropped`, `total_dropped`, or `lost_anything`.

## Dominant-image selection

`dominant_holder(&CaptureStats) -> Option<Arc<str>>`:

- `None` when the tally is empty (nothing attributed).
- Otherwise the key with the greatest count. Ties break by the ordered key sequence
  (the `BTreeMap`'s lexical order), so the result is deterministic across runs over
  identical input.

## Guarantees

- **Deterministic**: identical input yields an identical dominant image, including in
  the two-way-tie case. This is the same total-order discipline the socket-table join
  requires (S10); a `HashMap` would fail it.
- **Golden-safe**: `CaptureStats` still derives `PartialEq`/`Eq`; no golden output or
  completion summary observes the tally. Confirmed against the JSON Lines and pcapng
  writers (named-field access, no blanket serialize) and `build_summary`.
- **Cheap**: the key is the `Arc<str>` the attribution already holds, so the increment
  is a refcount bump, not an allocation.
