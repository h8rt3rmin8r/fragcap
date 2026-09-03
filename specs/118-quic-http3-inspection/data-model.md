# Data Model: Scoped QUIC And HTTP/3 Inspection

## ScopedQuicPair

- `pair_id`: session-unique logical identity
- `session_id`: owning Deep Capture session
- `association_connection_id`: authenticated SOCKS5 control connection
- `pair_id` plus `half`: stable proxy identity for each distinct QUIC half
- `client_endpoint`: immutable pinned outer client endpoint
- `origin_endpoint`: immutable policy-approved selected destination
- `certificate_identity`: admitted DNS name or IP identity
- `alpn`: exact negotiated application protocol or absent
- `state`: preparing, handshaking, active, draining, refused, closed
- `created_at` and `closed_at`: observed lifecycle instants

Validation rules:

- One pair belongs to one session and one authenticated association.
- Origin endpoint and certificate identity cannot change after admission.
- Client and upstream connection identities are distinct through their half discriminator.
- Active state requires both TLS halves to have succeeded.
- Refused state carries one stable refusal code and never transitions to active.

## QuicConnectionEvidence

- `pair_id` and `half`: client or upstream
- `pair_id` plus `half`: stable proxy identity for this half
- `peer_endpoint` and approved `origin_endpoint`: observed route endpoints
- `server_name`: requested or verified identity when available
- `alpn`: negotiated bytes rendered losslessly
- `tls_version`: TLS 1.3 on success
- `zero_rtt`: refused, not-attempted, or unavailable
- `migration`: disabled, refused, or unchanged
- `terminal`: complete, refused, reset, timed-out, transport-failed, cancelled
- `failure_code`: stable product code when terminal is not complete

## QuicStreamEvidence

- `pair_id`, half identities, and `stream_id`
- `kind`: HTTP/3 request or response
- `direction`: client-to-upstream or upstream-to-client
- `sequence`: monotonic within stream direction
- `timestamp`
- `observed_length`, `retained_length`, and optional retained bytes
- `retention_outcome`: complete, intentionally-omitted, or retention-limit
- `terminal`: finished, reset, stopped, failed, timed-out, or cancelled

Conservation: observed bytes equal retained plus omitted bytes. Forwarded bytes are independent and reconcile separately.

## QuicDatagramEvidence

- `pair_id`, both connection identities, and directional sequence
- `direction` and timestamp
- `observed_length`, `retained_length`, optional retained bytes, and outcome
- `terminal`: forwarded, unsupported-by-peer, oversized, queue-dropped, storage-dropped, failed, or cancelled

Each accepted QUIC DATAGRAM remains one evidence unit and is never split or merged.

## Http3Transaction

- Existing transaction and stream identities, joined to `pair_id` by the proxy connection
- Request method, scheme, authority, path, and ordered fields
- Response status and ordered fields
- Request and response body segment evidence
- Trailers when observed
- Timing and terminal outcomes for both directions

The existing application transaction model remains authoritative. HTTP/3 adds transport provenance and does not create another transaction store.

## QuicLoss

- `authority`: connection, stream, datagram, event queue, storage, or retention
- `direction`, `pair_id`, connection identity, and stream identity when localized
- exact lost units and bytes
- stable reason
- bounded localized map plus exact aggregate overflow

## State Transitions

```text
preparing -> handshaking -> active -> draining -> closed
     |            |          |          |
     +------------+----------+----------+-> refused
                              +-----------> cancelled
```

No refused pair becomes active. Migration or immutable endpoint change transitions an active pair to refused or closed and cannot create a replacement without fresh admission.
