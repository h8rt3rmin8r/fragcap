# Contract: Status output

Status output is a projection over the manifest and sidecars. It has one
machine-readable fact shape and one human-readable rendering of the same facts.
The renderer may change wording, but it must not add or omit facts relative to
the machine status object for the same status point.

## Machine status object

```json
{
  "type": "session.status",
  "session_id": "fcap-session-00000001",
  "mode": "deep-capture",
  "phase": "cleanup",
  "state": "partial",
  "manifest": "manifest.json",
  "artifacts": {
    "produced": ["packet-capture", "application-jsonl", "tls-key-log", "proxy-log", "process-trace", "compatibility-updates", "cleanup-report"],
    "omitted": [{"role": "har", "reason": "not-observable", "severity": "info"}]
  },
  "proxy": {
    "backend": "external-mitmproxy",
    "status": "completed"
  },
  "trust": {
    "store": "current-user",
    "ca_thumbprint": "sha256:example"
  },
  "cleanup": {
    "status": "partial",
    "report": "cleanup.json"
  },
  "sensitive_artifacts": ["application-jsonl", "tls-key-log", "proxy-log", "process-trace"]
}
```

Required fields:

- `type`
- `session_id`
- `mode`
- `phase`
- `state`
- `manifest`
- `artifacts.produced`
- `artifacts.omitted`
- `proxy.status`
- `trust.store`
- `trust.ca_thumbprint`
- `cleanup.status`
- `cleanup.report`
- `sensitive_artifacts`

Allowed `phase` values are `starting`, `capturing`, `stopping`, `cleanup`, and
`complete`.

Allowed `state` values are `running`, `complete`, `partial`, `failed`, and
`metadata-only`.

## Human status

Human status output reports the same fact set in readable prose or a table. It
must include the session id, mode, phase, state, manifest path, produced artifact
roles, omitted artifact roles with reasons, proxy state, trust state, cleanup
summary, and sensitive artifact roles.

Human status must not print decrypted payloads, TLS key material, certificate
private material, request headers, response headers, bodies, local account names,
or absolute local paths. Paths are displayed relative to the bundle root unless
the user explicitly requests a full path.

## Failure semantics

A failed writer, failed proxy startup, failed cleanup resource, or unavailable
application artifact changes `state` to `partial` when packet capture completed
and to `failed` when the session cannot produce packet truth. A metadata-only
Deep Capture session reports `metadata-only` when no application objects are
observable but process, proxy, compatibility, or cleanup facts remain useful.
