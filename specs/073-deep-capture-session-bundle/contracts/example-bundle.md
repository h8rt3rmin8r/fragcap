# Contract: Example bundle

Example bundle layout:

```text
session-fcap-session-00000001/
├── manifest.json
├── capture.fcapng
├── application.jsonl
├── tls-keylog.log
├── proxy.jsonl
├── process-trace.jsonl
├── compatibility-updates.jsonl
└── cleanup.json
```

Example manifest excerpt:

```json
{
  "manifest_version": 1,
  "session_id": "fcap-session-00000001",
  "mode": "deep-capture",
  "target": {
    "target_id": 42,
    "handle": "sample-target",
    "compatibility_updates": ["compatibility-updates.jsonl"]
  },
  "proxy": {
    "backend": "external-mitmproxy",
    "backend_version": "12.1.0",
    "mode": "explicit-env-proxy",
    "status": "completed"
  },
  "trust": {
    "ca_thumbprint": "sha256:example",
    "store": "current-user",
    "user_confirmed": true
  },
  "artifacts": [
    {
      "role": "packet-capture",
      "path": "capture.fcapng",
      "authority": "packet bytes, packet timestamps, interfaces, attribution comments, loss accounting",
      "sensitivity": "ordinary",
      "content_type": "application/vnd.tcpdump.pcapng",
      "required": true
    },
    {
      "role": "application-jsonl",
      "path": "application.jsonl",
      "authority": "application-layer event stream",
      "sensitivity": "sensitive",
      "content_type": "application/x-ndjson",
      "required": false
    },
    {
      "role": "tls-key-log",
      "path": "tls-keylog.log",
      "authority": "proxy-owned analyzer TLS key log",
      "sensitivity": "secret-adjacent",
      "content_type": "text/plain",
      "required": false
    }
  ],
  "omissions": [
    {
      "role": "har",
      "reason": "not-observable",
      "severity": "info"
    }
  ],
  "correlation": {
    "anchors": ["session_id", "target_id", "flow_id", "proxy_connection_id", "process_id", "role", "started_at", "ended_at"]
  },
  "cleanup": {
    "status": "partial",
    "report": "cleanup.json",
    "updated_at": "2026-08-25T00:05:05Z"
  }
}
```

The example uses placeholder values only. It contains no local paths, endpoints, account data, or real local title names.
