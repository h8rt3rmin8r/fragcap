# Contract: Generic UDP Evidence

## Application JSON Lines Record

```json
{
  "schema_version": 2,
  "type": "generic.udp_datagram",
  "proxy_connection_id": 7,
  "event_time_ns": 123,
  "direction": "client-to-upstream",
  "datagram_sequence": 0,
  "client_endpoint": "127.0.0.1:41000",
  "remote_endpoint": "127.0.0.1:42000",
  "observed_len": 4,
  "retained_len": 4,
  "outcome": "complete",
  "payload_encoding": "base64",
  "payload": "AQIDBA==",
  "inspectability": "protocol-unknown"
}
```

`payload_encoding` and `payload` are absent when no bytes are retained. `remote_endpoint` is absent only when no exact peer is observable. Existing session and correlation envelope fields remain on every record.

## Required Outcomes

| Outcome | Meaning |
| --- | --- |
| `complete` | Every byte in this datagram is retained |
| `intentionally-omitted` | Payload capture policy retained no bytes |
| `retention-limit` | The retained prefix is shorter than the observed datagram |

Queue and storage loss remain authoritative in application writer accounting because no successfully persisted record exists to carry that outcome.

## Boundary Rules

- One record represents exactly one ingress datagram.
- Sequence is directional and advances once per accepted ingress.
- Forwarding always receives the complete payload even when retention is partial or absent.
- No field implies a request-response pairing, application protocol, remote receipt, or packet count.
