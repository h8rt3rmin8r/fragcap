### Decisions

**2026-08-10. Slice 017 (issue #22) reverses two S14 decisions for the two cases
they were wrong for and keeps them for the case they were right for. Recorded while
implementing the CLI capture engine's write gating.**

- **The observe-only tee becomes a synchronous write gate (reverses D-e for the
  bound and watch-time cases).** S14's `TeeCountingSink` observed each captured
  packet after the sinks had written it, so the session could count a bound only
  after the file already exceeded it. It is replaced by a `SessionGate` the output
  loop consults before the fan-out: the session's decision now gates the write. The
  gate keeps forwarding each admitted packet to the driver over the same channel the
  tee used, so `CaptureSession::on_packet` still fires `VolumeReached` and the
  duration bound in the session, which stays the single owner of the six stop
  conditions (section 10.6). For an unbounded offline run the gate is a pass-through
  and D-e's arrangement is unchanged.

- **The session's decision gates the write, but packets are still not routed through
  the `CaptureSession` object (keeps D-c's separation).** The gate reads a lock-free
  published window state (open while capturing, closed while watching or draining)
  that the driver writes as the session transitions, the same discipline section
  11.6 requires of the attribution snapshot. The gate owns the bound counting, so
  the admit-or-discard decision is made on the sink thread with no per-packet
  cross-thread call into the session. The session remains a control brain beside the
  pipeline, not inside its packet path.

- **The gate seam is generic in core; the session-aware policy is in the facade
  (constitution P-3).** `fragcap-core` gains only a `WriteGate` trait
  (`Send + Sync`, `admit(&CapturedPacket) -> bool`) that the output loop consults;
  it learns nothing of capture sessions or profiles. The `SessionGate` lives in the
  facade `session` module beside `CaptureSession` and `RoleStampingAttributor`, the
  crate already above both `fragcap-capture` and `fragcap-attr`, and is handed to the
  pipeline as an `Arc<dyn WriteGate>`. No new dependency, and core takes no platform
  dependency.

- **`gate_dropped` is a term of the conservation identity but not of loss
  (constitution P-4, P-9).** A gate discard is counted so nothing the gate withholds
  escapes the pipeline accounting, but it is an intended discard (the operator's own
  bound or the pre-acquisition window), so it is kept out of `fragcap_dropped`,
  `total_dropped`, and `lost_anything`, which separate a slow sink and an undersized
  driver buffer from a configuration choice. The writer trailers (pcapng and JSON
  Lines) are deliberately left unchanged, so the committed goldens stay
  byte-identical; the gate's discards are surfaced by cause in the completion
  summary's `watching_discarded` and `discarded_out_of_window` lines, whose sum
  reconciles with `gate_dropped`.

- **The gate's Sender lives on the gate alone, not on the shared handle.** The
  driver keeps a `GateHandle` sharing the gate's atomics (to publish the window and
  read the tallies) but holding no channel sender, so the tee channel closes when the
  pipeline finishes and the driver's per-packet read loop ends. Sharing the sender
  would keep the channel open for the whole run and hang the loop after the source
  exhausts.

These are behavioral reversals confined to this slice; they are promoted to
`docs/fragcap-specification.md` only at release, not per slice.
