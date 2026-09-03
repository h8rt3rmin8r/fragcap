# Data Model: Generic UDP Observations

## GenericUdpDatagram

- Existing `session_id` and `connection_id`
- `direction`: `client-to-upstream` or `upstream-to-client`
- `sequence`: zero-based monotonic ingress ordinal within that direction
- `client_endpoint`: immutable pinned client socket address
- `remote_endpoint`: selected destination or exact observed contacted peer
- `observed_len`: complete application payload length
- `retained_len`: retained payload prefix length
- `bytes`: bounded retained prefix
- `outcome`: `complete`, `intentionally-omitted`, or `retention-limit`
- Event timestamp supplied by `ApplicationEvent`

## UdpSocketError

- Existing session and connection identity
- Direction and operation: receive or send
- Known endpoint, when available
- OS error kind and stable product error code
- Visibility: `platform-observed`
- No inferred ICMP fields

## GenericUdpAccounting

- observed datagrams and bytes
- retained bytes
- omitted bytes
- truncated datagrams
- queue-dropped datagrams and bytes
- storage-failed datagrams and bytes
- platform-observed socket errors

## Invariants

- One accepted ingress maps to one datagram event or one named evidence-loss outcome.
- `observed_len = retained_len + omitted_len` for every event.
- Sequences are contiguous within each direction, including duplicates.
- One record never contains bytes from another datagram.
- Forwarded length is independent from retained length.
- Refused ingress retains no payload.
- Endpoints come from the pinned client and selected or observed exact peer.
- Packet capture remains the only packet-level authority.

## State Transitions

```text
authenticated association -> accepted ingress -> observe under bounds -> forward complete payload
                                |                       |
                                |                       +-> persisted / queue lost / storage failed
                                +-> refused by S115 -> named transport loss, no payload retention
```
