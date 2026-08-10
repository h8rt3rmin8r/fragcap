### Added

- **The session decision now gates the sink writes, so a volume bound produces an
  exactly-bounded file.** A generic `WriteGate` seam in `fragcap-core` is consulted
  by the pipeline output loop before the per-sink fan-out; a facade `SessionGate`
  admits a packet only while the capture session is capturing and the configured
  `--max-packets` or `--max-bytes` bound has not been reached, discarding and
  counting every other packet by cause. Because the admit-or-discard decision is
  made synchronously on the write path, the produced pcapng and JSON Lines contain
  exactly the bound and the completion summary matches what is on disk, rather than
  the S14 soft bound that could write more than the bound while counting the
  overflow as discarded. A new `retained` line on the summary reports the packets on
  disk. Resolves issue #22 (the deferred half of the PR #21 review, findings C2 and
  C3).
- **A packet the gate withholds is counted in a new `gate_dropped` counter, folded
  into the pipeline conservation identity.** `CaptureStats` gains `gate_dropped`,
  and the identity checked in every pipeline test is now, for every sink,
  `received + buffer_dropped + gate_dropped + refusals == packets_captured`. The
  counter is distinct from the two loss counters because a gate drop is an intended
  discard (outside the capture window or beyond the bound), not loss to be remedied,
  so it does not reach `fragcap_dropped`, `total_dropped`, or `lost_anything`. A run
  with no gate attached leaves the term zero and the identity in its prior form.
- **The live driver runs the packet path from arm, so watch-time frames are read and
  counted.** On a live capture the handle is open from arm; the pipeline is now
  spawned before acquisition and the gate discards and counts the pre-acquisition
  frames in `watching_discarded` rather than never observing them. The offline
  driver keeps its two-phase shape (acquire, then start the pipeline), so its
  behavior and the committed goldens are byte-identical. The live path is compiled
  and linked in CI but not executed there (tier 2).
