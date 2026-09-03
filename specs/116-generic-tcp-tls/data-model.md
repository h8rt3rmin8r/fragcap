# Data Model: Generic TCP And Non-HTTP TLS Evidence

## GenericStreamChunk

- Existing `session_id` and `connection_id`
- `direction`: `client-to-upstream` or `upstream-to-client`
- `provenance`: `tcp-plaintext`, `tls-encrypted`, or `tls-decrypted`
- `offset`: zero-based directional observed-byte offset
- `observed_len`: bytes represented by this event
- `retained_len`: bytes retained in `bytes`
- `bytes`: bounded payload, absent from serialized output when empty by policy
- `outcome`: `complete`, `intentionally-omitted`, or `retention-limit`
- Event timestamp supplied by `ApplicationEvent`

## GenericStream

- Existing connection identity and open/close timestamps
- Requested destination authority and route
- Mode: `plain-tcp`, `opaque-tls`, `intercepted-tls`, or `refused`
- Directional forwarded, observed, retained, and omitted byte totals
- Existing connection and TLS terminal outcomes

## Invariants

- For each chunk, `observed_len = retained_len + omitted_len`.
- Directional offsets are monotonic and contiguous over observed bytes.
- Retained bytes never exceed the per-connection or session budget.
- Forwarded bytes are independent from retained bytes.
- Encrypted and decrypted provenance never occur on the same observation path.
- Protocol-unknown records carry no request, response, message, field, or schema claim.

## State Transitions

```text
approved route -> connected -> classified -> forwarding -> terminal
                                  |              |
                                  |              +-> chunks retained or omitted
                                  +-> plain / opaque TLS / client TLS -> upstream TLS -> protocol unknown
                                                                   |
                                                                   +-> explicit refusal
```
