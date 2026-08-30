# S103 Data Model

## SessionCapability

Opaque fixed-length random bytes plus a non-secret listener generation. It exposes protocol-specific authorization encoding without exposing secret bytes through display, debug, logs, or observations. Cleanup invalidates and zeroizes owned bytes.

## DestinationAuthority and DestinationDecision

`DestinationAuthority` carries a validated DNS name or IP literal, port, and verification identity. `DestinationDecision` records the ordered resolved address, rule, exact controlled-test grant when present, and allow/refuse result. Resolution changes are re-evaluated before connect.

## UpstreamAttempt

One cancellation-aware DNS, TCP, and optional TLS lifecycle. Separate finite stage budgets produce one typed terminal outcome and preserve the requested authority, selected address, verification state, and elapsed stage without fabricating an HTTP result.

## SessionCertificateAuthority

Owns session generation, certificate parameters, public DER, zeroized signing key, SHA-1 store thumbprint, SHA-256 provenance fingerprint, validity, protected-storage inventory, and cleanup state. Trust authorization is not part of this entity.

## LeafCache and LeafEntry

The cache owns CA and policy generations, maximum entries, bytes, lifetime, current bytes, and least-recently-used order. Each entry carries exact SAN identity, validity, public DER, signing object, byte cost, last-use order, and eviction reason. Rotation invalidates entries from an older generation.

## TrustRecord

Exact certificate DER, SHA-1 thumbprint, requested current-user Root scope, authorization state, observed current-user and wrong-store state, mutation result, Win32 error when present, and cleanup obligation. Only exact owned DER plus thumbprint can be removed.

## RawObservation

Version, session id, optional connection id, monotonic sequence, timestamp, provenance, payload state, and event family. Families cover lifecycle, connection, DNS, TCP, TLS, HTTP, stream, message, refusal, error, loss, unknown, and malformed source.

## ObservationAccounting

Finite queue and payload limits plus admitted, emitted, dropped-oldest, truncated, refused, unparsed, and projection-gap counters. `complete` is true only when every gap counter is zero.

## ProtocolScenario and TruthLedger

A scenario has a stable id/version, protocol family, fidelity class, case kind, topology, deterministic action script, synthetic inputs, and independent output expectations. Its ledger records symbolic client/origin reads and writes, datagram boundaries, logical order, protocol facts, and task/resource terminal outcomes.

## State Transitions

```text
capability: generated -> active -> invalidated -> zeroized
upstream:   declared -> resolving -> policy-checked -> connecting -> secured/failed -> closed
CA:         generating -> protected -> available -> trust-authorized(optional) -> cleaning -> removed/residue
leaf:       missing -> issuing -> cached -> evicted/invalidated -> dropped
event:      admitted -> queued -> emitted | dropped-oldest
scenario:   declared -> running -> terminal -> compared -> clean/residue
```
