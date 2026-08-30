# S102 Data Model

## NativeProxyConfig

Immutable start configuration: explicit loopback socket address, non-zero maximum live connection tasks, non-zero fixed per-connection read buffer, and non-zero shutdown timeout. Validation rejects non-loopback addresses and zero bounds before effects begin.

## BackendIdentity

Stable typed backend identity containing a non-exhaustive backend kind, stable machine-readable name, crate package version, and explicit capabilities for foundation listener, forwarding, HTTP observation, and TLS inspection. S102 reports only the foundation listener capability.

## RuntimeObservation

Cloneable point-in-time state containing lifecycle state, bound endpoint, accepted/saturated/completed/failed/forced counters, live and peak connection task counts, and ordered failures. Counters are monotonic and live work never exceeds configured capacity.

## RuntimeFailure

One preserved listener, connection, task, cancellation, or join failure, with a stable code, detail, and optional connection identifier. Failures remain present in later observations.

## ShutdownReport

Terminal, repeatable result containing the final observation, listener release status, joined/forced/incomplete task counts, and residue flag. Every accepted connection reaches one terminal category. A clean report has no listener, live task, incomplete task, or residue.

## State Transitions

```text
configured -> running -> stopping -> stopped
                 |          |          |
                 +----------+----------+-> observe
```

Start failure returns no lease. A stop deadline returns named residue while the lease retains ownership for a later cleanup retry. After the owner thread joins, repeated stop and cleanup return the cached terminal result.
