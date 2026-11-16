# Research: Direct Route Reliability

## Existing evidence

Issue #619 records selected game flows whose helper reported SOCKS failure while the native proxy recorded successful upstream CONNECT 2.6-14.9 ms later. The helper uses a 50 ms socket read bound; proxy stages took 60-68 ms. The route currently discards the actual I/O error. Raising that bound on the packet worker would exceed its 90 ms progress watchdog and delay unrelated work.

S210 already separates a selected packet's digest and disposition from passive Npcap observation. Its T011 physical package matrix remains undone. Source tests alone cannot establish physical driver ordering.

## Decisions

1. Use finite off-worker helper admission and an exact pending-flow owner, with terminal results returned to the packet worker. Do not block the worker on upstream latency.
2. Bound helper work and cancellation. On failure, preserve the actual error stage and kind, retire the one-use receipt, and pass through or account for held selected packets exactly once.
3. Extend the existing protected route journal additively. Avoid credential, payload, and unrelated-process details.
4. Run the physical packet-ordering matrix from a candidate portable package against a disposable client and finite external peer. Preserve the installed-default journey as a separate claim.

## Alternatives rejected

Increasing the current 50 ms read timeout on the packet worker reproduces the worker-stall risk. A generic `socks5-connect-failed` reason loses the precise failure boundary. Reusing historical installed bundles cannot qualify changed candidate behavior.
