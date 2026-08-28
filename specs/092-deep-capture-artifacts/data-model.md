# Data Model: Deep Capture Bundle and Artifact Reference

This slice changes documentation rather than runtime data. The model records the shipped relationships the public reference must explain.

## Output Families

| Family | Producer | Root | Authority |
| --- | --- | --- | --- |
| Capture pcapng | Capture packet sink | One `.fcapng` file | Packet bytes, timestamps, interfaces, attribution comments, and loss accounting |
| Capture packet JSON Lines | Capture packet sink | One JSONL stream | Machine-readable packet records plus header and final trailer accounting |
| Deep Capture session bundle | Deep Capture coordinator | Bundle directory, indexed by `manifest.json` after successful finalization | A set of distinct packet, proxy-observation, projection, diagnostic, compatibility, and cleanup authorities |

## Manifest State

| State | Trigger | Packet truth implication | Inspection implication |
| --- | --- | --- | --- |
| `complete` | No recorded operation failure reached finalization | `capture.fcapng` is expected and required | No claim that the target reached the proxy or that optional artifacts were produced |
| `partial` | A later operation failed after the controlled path ran or packet truth exists | Packet truth or other useful evidence can remain | Collected observations are retained; omissions and cleanup must be read |
| `failed` | An operation failed and packet truth does not exist | `pcapng` is omitted with `writer-failed` when a final manifest can be written | No fabricated capture or universal statement about target traffic |

An initialization failure before final manifest writing can leave `cleanup.json` with `manifest-state = not-written`. That recovery path is not a finalized bundle state.

## Artifact Declaration

Each produced artifact entry contains:

| Field | Meaning |
| --- | --- |
| `role` | Stable role token used by events, omissions, and readers |
| `path` | Relative path within the bundle |
| `authority` | The fact family the artifact owns |
| `sensitivity` | `ordinary`, `sensitive`, or `secret-adjacent` |
| `content_type` | Media type of the artifact |
| `required` | Whether the role is part of the required bundle set or an optional requested projection or aid |

## Artifact Inventory

| Role | Path | Authority | Sensitivity | Required | Lifetime |
| --- | --- | --- | --- | --- | --- |
| `manifest` | `manifest.json` | `bundle-index` | `ordinary` | yes | Written after cleanup when finalization succeeds; a completed manifest is retained until operator removal, while an unfinished manifest can be removed by confirmed residue cleanup |
| `pcapng` | `capture.fcapng` | `packet-truth` | `ordinary` | yes when produced | Retained as packet evidence; absent rather than fabricated after writer failure |
| `application-jsonl` | `application.jsonl` | `application-events` | `sensitive` | yes | Written during finalization and retained until operator removal or confirmed residue cleanup |
| `har` | `http.har` | `http-projection` | `sensitive` | no | Written only when requested and HTTP method plus URL were observed; retained like other sensitive sidecars |
| `tls-key-log` | `tls-keylog.log` | `analyzer-aid` | `secret-adjacent` | no | Final path exists before traffic when requested; retained only when nonempty; removable by confirmed residue cleanup |
| `proxy-log` | `proxy.jsonl` | `proxy-events` | `sensitive` | yes | Written during finalization and retained like other sensitive sidecars |
| `process-trace` | `process-trace.jsonl` | `process-events` | `sensitive` | yes | Written during finalization, including an unavailable record when needed; retained like other sensitive sidecars |
| `compatibility` | `compatibility.json` | `compatibility-updates` | `ordinary` | yes | Retained run context; does not by itself prove later local-store persistence succeeded |
| `cleanup` | `cleanup.json` | `cleanup-report` | `ordinary` | yes | Written before and after final manifest writing; retained as the per-resource result |

## Omission

| Role | Reason | Severity | Condition |
| --- | --- | --- | --- |
| `pcapng` | `writer-failed` | `error` | Packet truth is absent at finalization |
| `har` | `no-http-semantics` | `info` | HAR was requested but no observation has both method and URL |
| `har` | `not-requested` | `info` | HAR was not requested |
| `tls-key-log` | `not-produced` | `warn` | Key logging was requested but the final file is absent or empty |
| `tls-key-log` | `not-requested` | `info` | Key logging was not requested |

## Correlation Relationships

| Anchor | Current locations | Limit |
| --- | --- | --- |
| `session_id` | Manifest, application JSON Lines, proxy log, process trace, compatibility record, cleanup report, status events | HAR and pcapng do not carry the session id as a field |
| Target id or handle | Manifest, application target id, compatibility record | Proxy and process sidecars do not carry target identity |
| `flow_id` | Application observations and pcapng packet comments when correlation succeeds; manifest has a summary list | Missing when proxy endpoints cannot be joined to packet truth |
| Proxy connection id | Application observations | Current proxy lifecycle sidecar does not repeat each observation id |
| Event time bounds | Application observations; HAR start time projection | Current proxy and process sidecars carry lifecycle facts without a shared complete timestamp contract |
| Process id, image, and role | Application observations when correlation succeeds; process trace where observed; pcapng attribution comments | Values can be null or unavailable and must not be invented |
| Attribution state | Application observations; packet comments carry packet-side fidelity | `packet-flow-only` and `proxy-only` describe available joins, not universal truth about the session |

## Cleanup Relationships

`cleanup.json` owns the ordered resource results for proxy process, proxy port, trust entry, proxy-private material, packet capture, TLS key log, bundle artifacts, and manifest state. The manifest repeats only aggregate cleanup status, report path, and update time. Session state and cleanup status are independent: a `complete` operation can still require the cleanup report to establish per-resource disposition, and a `partial` operation can have successful cleanup.
