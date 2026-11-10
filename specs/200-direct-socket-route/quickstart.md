# Quickstart: S200 Controlled Validation

## Driver-free authority

Run the ownership and receipt tests on the development host before live work. They must cover exact final-client ancestry, reused PID/port, delayed and missing events, receipt replay, cross-session use, concurrent same-destination unrelated traffic, and exclusive packet dispositions. The normal repository gate is `cargo xtask ci`.

## Disposable Windows authority

The Windows integration job acquires the exact S199 signed package bytes, runs under a disposable Administrator runner, and opens WinDivert only inside an explicitly authorized controlled session. A broad TCP/UDP pass-through is prearmed before the synthetic managed root starts; unrelated packets are promptly reinjected after header-only candidacy, without application-payload inspection or retention, while selected candidates wait only within finite bounds for exact process and destination proof. The fixture launches a synthetic platform root, publisher/updater, and final client. Controlled TCP and UDP origins and active unrelated application/local-service sentinels run before, during, and after the route, including a deliberately stalled worker and watchdog release.

Required rows are direct raw TCP, selected UDP/reply, mixed launcher proxy and final-client direct, same/different-destination sentinels, helper/proxy exclusion, IPv4/IPv6, cancellation/restart, delayed/missing/ambiguous/reused evidence, and route/proxy/writer/cleanup failures. Every row declares its expected original-flow and route result. Zero skipped required rows, exact byte/datagram conservation, exclusive packet accounting, zero sentinel reply loss and the spec's paired p99 latency bound are the pass condition.

## Handoff

The official PR is published only after local and hosted results match the contract. CI and all arriving Codex/security reviews are addressed, with at most one manually triggered second Codex round. The owner performs final review and merge.
