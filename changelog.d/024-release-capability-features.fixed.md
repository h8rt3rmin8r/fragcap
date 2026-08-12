Release builds now enable the live capture, socket-table attribution, and
process-event tracing features, so the distributed binary can capture and
attribute traffic instead of failing every capture with "no live capture
backend" (issue #62). The binary delay-loads npcap's `wpcap.dll`, so it still
starts and runs `fragcap doctor` on a machine where npcap is not yet installed
rather than failing to launch, and doctor can tell the operator to install it.
