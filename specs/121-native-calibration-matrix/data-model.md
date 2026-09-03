# Data Model: Complete Native Calibration Matrix

## Calibration Protocol

A closed protocol selection aligned to the S120 traffic-family contract.

Values: `routing`, `http1`, `http2`, `https`, `http3`, `websocket`, `sse`, `grpc`, `non-http-tls`, `socks5-tcp`, `socks5-udp`, `generic-tcp`, `generic-udp`, `quic`, and `not-applicable`.

`routing` is a plan selection for route-only reachability. `not-applicable` is stored only on fact classes whose meaning is protocol-independent. Unknown and unrouted classifications cannot become positive protocol facts.

## Compatibility Case

| Field | Requirement |
| --- | --- |
| target id | Existing local target row identity |
| launch case | Exact supported cold launch case |
| backend name | Non-empty native backend identity |
| backend version | Exact non-empty backend version |
| routing strategy | Closed strategy token, currently `child-environment` |
| address family | Exactly `ipv4` or `ipv6` |
| selected protocol | One calibration protocol token |
| fragcap version | Current product version |
| target version | Exact version evidence when trustworthy and available |
| phase | Reachability or TLS |

The case is immutable from plan authorization through fact append and artifact finalization.

## Compatibility Fact

The existing append-only fact gains three optional storage columns: routing strategy, address family, and protocol family. Optionality exists only so pre-S121 rows can survive migration. New observed calibration facts validate that every field applicable to their fact class is present.

### Protocol applicability

| Fact class | Stored protocol |
| --- | --- |
| Proxy routing, propagation, variables, launch case, final owner | `not-applicable` |
| TLS trust behavior | Exact selected S120 family |
| Protocol behavior | Exact selected S120 family |
| Inspectability | Exact selected S120 family |

### Applicability states

- `applicable`: explicit current state and every applicable dimension matches.
- `stale`: stale flag or stale-observation source.
- `legacy-incomplete`: one or more required dimensions are absent.
- `mismatch`: a named applicable dimension differs.

Applicability is derived and never overwrites the stored row.

## Latest Applicable Evidence

For a prerequisite key, candidates are filtered to applicable rows and ordered by durable row id. The last row is authoritative for that exact case. Negative, partial, and conflicting rows remain stored. A later mismatched row does not supersede an earlier exact row because it belongs to another case.

## State Transitions

```text
planned exact case
  -> refused before effects
  -> authorized
     -> observed partial/negative/positive outcome
     -> fact proposals
        -> appended rows
        -> failed appends retained in bundle
     -> finalized artifacts and cleanup

stored row
  -> applicable for an exact current case
  -> stale by explicit marker/source
  -> legacy-incomplete after schema expansion
  -> mismatched for a different case
```
