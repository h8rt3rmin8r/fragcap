# Contract: Generic Stream Evidence

## Application JSON Lines Record

```json
{
  "type": "generic.stream_chunk",
  "proxy_connection_id": 7,
  "event_time_ns": 123,
  "direction": "client-to-upstream",
  "provenance": "tls-decrypted",
  "offset": 0,
  "observed_len": 4,
  "retained_len": 4,
  "outcome": "complete",
  "payload_encoding": "base64",
  "payload": "AQIDBA=="
}
```

`payload_encoding` and `payload` are absent when retention is intentionally omitted or no bytes fit the retention budget. Existing correlation fields remain on every record.

## Required Outcomes

| Outcome | Meaning |
| --- | --- |
| `complete` | Every represented byte is retained |
| `intentionally-omitted` | Capture policy disabled payload retention |
| `retention-limit` | Some or all represented bytes exceeded a retention budget |

Queue loss remains authoritative in the existing application writer accounting because no record exists to carry a per-record outcome after the queue refuses it.
