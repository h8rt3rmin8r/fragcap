# Contract: Scoped QUIC And HTTP/3 Evidence

## Admission

A QUIC pair is admitted only when all of these are true:

1. The UDP association authenticated the current session capability.
2. The client endpoint is pinned by the association.
3. The requested destination resolves under proxy ownership and passes policy.
4. One exact certificate identity is available.
5. Capacity exists for the endpoint, connection, tasks, streams, and evidence.

Failure yields one stable refusal event and no transparent fallback.

## Connection Events

`quic.connection` records:

- pair, session, association, and half identities
- local and peer endpoints
- exact server name and ALPN when observed
- TLS 1.3 and independent trust outcome
- zero-RTT and migration policy outcome
- timestamps, terminal state, and stable failure code

The client and upstream connection identities are distinct. The pair identity joins them without claiming end-to-end QUIC identity preservation.

## HTTP/3 Stream And QUIC Datagram Events

`quic.stream` and `quic.datagram` retain exact HTTP/3 body and negotiated QUIC DATAGRAM boundaries, direction, sequence, timestamps, observed and retained lengths, optional base64 payload, retention outcome, and terminal state. Payload fields are absent when zero bytes are retained. Forwarding counters reconcile separately from observation counters.

## HTTP/3 Events

Negotiated `h3` connections produce the existing metadata, body, transaction, timing, and terminal event shapes with protocol `h3`, plus pair and QUIC stream identities. Request pseudo-fields map to method, scheme, authority, and path. Decoded field order and duplicates remain observable; QPACK wire bytes and compressed cross-name order are explicitly unavailable. Missing status, length, timing, or body facts remain absent.

## Refusal Codes

The stable minimum set is:

- `quic-route-unscoped`
- `quic-origin-changed`
- `quic-identity-unavailable`
- `quic-client-trust-rejected`
- `quic-upstream-validation-failed`
- `quic-client-certificate-required`
- `quic-pinning-suspected`
- `quic-zero-rtt-refused`
- `quic-migration-refused`
- `quic-alpn-unsupported`
- `quic-capacity-exhausted`
- `quic-transport-failed`
- `http3-protocol-failed`

## Accounting

For each pair:

```text
accepted stream bytes = forwarded stream bytes + transport-failed stream bytes
observed stream bytes = retained stream bytes + omitted stream bytes
accepted datagrams = forwarded datagrams + refused datagrams + transport-failed datagrams
observed evidence events = persisted events + queue-dropped events + storage-dropped events
```

Every counter names its authority. Packet loss is never inferred from proxy loss.

## Unsupported Boundary

Unrouted QUIC, unknown ALPN, 0-RTT application data, unsafe migration, changed destinations, missing identity, unsupported address families, and policy-refused origins have no application inspection claim. Packet capture remains the packet authority.
