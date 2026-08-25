# Phase 1 Data Model: Deep Capture session bundle

This slice defines future file contracts. It adds no Rust type and no database table.

## SessionBundle

The logical bundle for one session.

| Field | Meaning |
| --- | --- |
| `manifest` | Required manifest path, always present. |
| `artifacts` | Ordered artifact declarations. |
| `omissions` | Expected artifacts that were not produced, each with a reason. |

Invariant: a directory with sidecars but no manifest is not a Deep Capture session bundle.

## SessionManifest

The manifest is the authoritative bundle index.

| Field | Meaning |
| --- | --- |
| `manifest_version` | Contract version for the manifest shape. |
| `session_id` | Stable id shared by every artifact in the bundle. |
| `mode` | `capture` or `deep-capture`. |
| `target` | Target row id, stable handle, display name if available, and compatibility fact references. |
| `started_at`, `stopped_at` | Session time bounds. |
| `proxy` | Backend, backend version, mode, listen endpoint token, and runtime status for Deep Capture. |
| `trust` | Local CA thumbprint, trust store location, user confirmation state, and cleanup state. |
| `artifacts` | Every produced artifact with role, path, authority, sensitivity, and content type. |
| `omissions` | Expected but absent artifact declarations with reason and severity. |
| `correlation` | Anchor names and versioned rules used by sidecars. |
| `cleanup` | Per-resource cleanup result. |

## ArtifactDeclaration

| Field | Meaning |
| --- | --- |
| `role` | One of `packet-capture`, `application-jsonl`, `har`, `tls-key-log`, `proxy-log`, `process-trace`, `compatibility-updates`, `cleanup-report`. |
| `path` | Relative path under the bundle root. |
| `authority` | Which facts this artifact owns. |
| `sensitivity` | `ordinary`, `sensitive`, or `secret-adjacent`. |
| `content_type` | Media type or stable internal type token. |
| `required` | Whether the artifact is required for this session shape. |

## ApplicationRecord

One JSONL record for an application-layer observation.

Required anchors:

- `type`
- `session_id`
- `target_id` or stable target handle
- `flow_id`
- `proxy_connection_id`
- `started_at`
- `ended_at` or explicit open-ended state
- `process_id` when known
- `role` when known
- attribution state when process/role are unavailable

HTTP records additionally carry method, scheme, authority, path, request headers if retained, response status if known, response headers if retained, body retention metadata, and HAR join id when a HAR entry exists.

## CleanupResult

Per-resource cleanup facts. Cleanup status is one of `not-needed`, `succeeded`, `partial`, `failed`, `deferred`, or `not-attempted`.

Resources include proxy process, proxy port, local CA trust, local CA material, TLS key log, proxy log, application records, packet capture, process trace, and manifest.

## Authority Rules

| Fact | Authority |
| --- | --- |
| Packet bytes, packet timestamps, interfaces, loss accounting | `.fcapng` |
| Packet attribution comments | `.fcapng` |
| Application transaction stream | application JSONL |
| HTTP archive view | HAR |
| Analyzer TLS secrets for proxy-owned tunnels | TLS key log |
| Proxy startup, shutdown, backend errors, connection ids | proxy log |
| Process launch and exit chronology | process trace |
| Bundle membership, omissions, sensitivity, cleanup status | manifest |
| Target compatibility changes | compatibility update sidecar and local target store |
