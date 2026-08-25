# Contract: Deep Capture session manifest

The manifest is a JSON object written as `manifest.json` at the bundle root.

## Required top-level fields

```json
{
  "manifest_version": 1,
  "session_id": "fcap-session-00000001",
  "mode": "deep-capture",
  "target": {},
  "started_at": "2026-08-25T00:00:00Z",
  "stopped_at": "2026-08-25T00:05:00Z",
  "proxy": {},
  "trust": {},
  "artifacts": [],
  "omissions": [],
  "correlation": {},
  "cleanup": {
    "status": "succeeded",
    "report": "cleanup.json",
    "updated_at": "2026-08-25T00:05:05Z"
  }
}
```

## Artifact declaration

Each artifact declaration has:

- `role`
- `path`
- `authority`
- `sensitivity`
- `content_type`
- `required`

Allowed `sensitivity` values:

- `ordinary`: attribution metadata or status data whose sensitivity is comparable to existing Capture output.
- `sensitive`: decrypted application data, proxy logs, process traces, or artifact paths that should be handled carefully.
- `secret-adjacent`: analyzer key logs or certificate material references that can enable decryption of session traffic.

## Omission declaration

Each omission declaration has:

- `role`
- `reason`
- `severity`

Example reasons include `not-requested`, `not-observable`, `unsupported-protocol`, `proxy-not-reached`, `certificate-pinned`, `backend-unavailable`, and `writer-failed`.

## Cleanup declaration

The manifest cleanup object has:

- `status`
- `report`
- `updated_at`

The manifest cleanup status is an aggregate summary. The cleanup report sidecar
is authoritative for per-resource cleanup facts. If the manifest and cleanup
report differ, consumers MUST treat the cleanup report as the source of truth
and the manifest as stale until rewritten.

Allowed cleanup statuses are `not-needed`, `succeeded`, `partial`, `failed`, `deferred`, and `not-attempted`.
