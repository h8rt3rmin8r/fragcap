<!-- spec-impact: 17 -->

### Launch-and-observe capture and capture-time promotion (slice S059)

A stored target whose launch chain is unresolved, the record an interactive
`targets add` writes when the user answers `no` or `unsure` to whether the executable
they pointed at holds the sockets, can now be captured. Before this it named no
client and `capture` refused it; S055 shipped the promotion mechanism but deferred
the capture-time trigger (issue #152).

Capturing such a target runs in launch-and-observe mode: fragcap builds a profile
from the executable the user did record (a two-stage profile whose observed
executable is a launcher stage and whose terminal client stage matches the process
that descends from it and holds the sockets), captures normally, and aggregates which
process image the run attributed the most packets to. When the run observes a
dominant socket-holding image, the stored target is promoted to that resolved client
at verified fidelity (capture-time promotion), so a second capture addresses the
client directly and the target reads `ready`. A run that observes nothing leaves the
target exactly as it was: promoting on no observation would record a socket holder
the tool never saw, which it does not do.

The observe-mode resolution is added to the shared `commands/target_resolve.rs` seam
S058 extracted, so both `capture` and the Wireshark `extcap` path resolve an
unresolved target identically; only `capture` writes the promotion back (extcap is a
streaming bridge, not the store owner). The run's dominant socket-holder rides on an
additive per-image tally on `CaptureStats`, folded across capture threads and kept
out of every counter total, completion summary, and written file, so every committed
golden is reproduced byte for byte.

This slice adds no direct-executable launcher: live launch stays restricted to the
existing Steam-anchored path, so an unresolved target with no platform anchor is
started by the operator (by any means) and observe-mode captures it. The whole
resolve, observe, and promote-or-leave decision is verified offline over the scripted
fixture pipeline; only the literal `steam://run` launch is Tier 2 and is not
exercised in continuous integration.

No new dependency and no `Cargo.lock` delta. New glossary terms: launch-and-observe,
observed socket-holder, capture-time promotion.
