# Handoff and Evidence Contract

The process trace retains the original observed process events, declared stage matches and exits, process-instance identifiers, and causal parents. A successor is a later match of the terminal role under the exited predecessor's exact ancestry. The top-level reason distinguishes an incomplete handoff from an ordinary terminal exit. Historical role stamps must not apply to a later process that reuses the PID.

`terminalstageexited` means a true terminal client exit with no proven active handoff; a provisional exit is not a complete observation outcome. A timed-out handoff reports `handoff-incomplete` while the process trace retains the first match, exit, and observed descendants. It keeps packet and cleanup status separately visible.

Packet records retain their original timestamps and process-instance-bound role stamps; the process timeline identifies pre-client, provisional, eligible final-client, owner-release, and cleanup periods. Native proxy observations carry explicit pre-client and provisional classifications when their connection opened before the enduring client bound. Only final-client `observation` records can support positive calibration. Current Doctor recovery may update live resource status but cannot rewrite the original partial session verdict.
