# S204 Controlled Evidence (2026-11-12)

## External signed route

The elevated ignored `controlled_direct_runtime_routes_external_peers` probe ran with a verified signed WinDivert component against one temporary, unprivileged, non-loopback IPv4 and IPv6 peer. For each family it exercised diagnostics on and off. The client received the exact origin reply, sent an additional exact client message after that reply, and observed clean EOF. The peer counted two selected exchanges and six unrelated sentinel exchanges per family, with no peer errors. Every route reported zero post-receive packet loss and zero synthetic-reply send loss. The four route runs consumed 7, 5, 5, and 5 selected packets respectively. The diagnostic-enabled route streams wrote all 31 and 23 accepted events without writer or queue loss. Pre-receive loss remains unmeasurable by this probe.

The remote script was sent over SSH standard input; it created no remote file or service. Both temporary listener ports were verified closed after its successful exit. The source script is `integration/controlled-route-peer.py`. This is a controlled route probe, not a private-title or default installed-storage test.

A second complete four-route pass after the final timing assertion again yielded two selected exchanges and six sentinels per family with no peer errors or post-receive loss. Selected transaction elapsed times were 4,217 and 2,933 ms for IPv4, and 2,586 and 2,931 ms for IPv6 (diagnostics on, then off). The four route runs consumed 10, 8, 6, and 8 packets; the diagnostic streams wrote all 40 and 30 accepted events. This is transaction duration under the local machine's concurrent traffic, not a paired sentinel p99 latency measurement. Both remote listener ports were verified closed again.

## Failure and reconciliation cases

Elevated signed-driver probes for cancellation and restart, proxy refusal, and route writer retirement each passed. The driver-free direct-route library suite passed 239 active tests with 28 explicitly ignored tests; all nine route-event integration tests passed. The zero-byte UDP regression fails before the route-proof change and passes after it. The candidate flood regression fails before queue reservation and passes after it.

The signed broad pass-through sentinel probe completed 200 baseline, 200 active, and 200 post-route exchanges plus 2,000 parallel active exchanges with exact echoed bytes and no missing replies. Baseline, active, and post-route p99 values were 93, 527, and 55 microseconds; the parallel active p99 was 355 microseconds. All are below the versioned 50 ms or four-times-baseline bound. This generic pass-through probe is separate from the selected external transaction timings above.

The first two hosted Windows native-integration attempts each failed one TLS classification probe with an unknown issuer, while the other 22 signed-driver probes passed. Five isolated local repetitions passed. The ordered local matrix initially exposed a startup race and a test-only loopback ACK overconstraint; after a bounded packet-worker readiness handoff and limiting the strict ACK assertion to the external round trip, the full ordered 23-probe matrix passed locally with zero skipped rows. Hosted confirmation of that revision remains pending.

## Outstanding qualification

The hosted MSI and ZIP certification, exact-head Windows integration, CI, and third-party reviews remain pending at this record's creation. Do not use this file alone to claim package, installed default-path, or review completion.
