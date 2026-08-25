# Contract: Application JSONL

Application JSONL is the canonical application-layer sidecar for proxy observations.

## Header record

The first record declares the stream.

```json
{"type":"application.header","session_id":"fcap-session-00000001","manifest_version":1}
```

## Event record

Every application event record carries the correlation anchors.

```json
{
  "type": "application.http",
  "session_id": "fcap-session-00000001",
  "target_id": 42,
  "flow_id": "flow-00000001",
  "proxy_connection_id": "proxy-conn-00000001",
  "process_id": 1234,
  "role": "client",
  "attribution": "live",
  "started_at": "2026-08-25T00:01:00.000000Z",
  "ended_at": "2026-08-25T00:01:00.250000Z",
  "http": {
    "method": "GET",
    "scheme": "https",
    "authority": "example.invalid",
    "path": "/status",
    "status": 200
  }
}
```

If `process_id` or `role` is unavailable, the field may be null only when
`attribution` explains why, such as `none`, `proxy-only`, or
`not-yet-correlated`. A `retained` attribution still identifies the owning
process and MUST carry the retained process id.

## Trailer record

The final record summarizes record counts and writer status.

```json
{"type":"application.trailer","session_id":"fcap-session-00000001","records":1,"writer_status":"complete"}
```
