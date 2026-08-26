# Contract: MVP Bundle Validation

The MVP bundle must conform to the #216 manifest and application JSONL contracts.

## Required artifacts

- `manifest.json`: required bundle index.
- `capture.fcapng`: packet truth artifact.
- `application.jsonl`: canonical application observation stream.
- `proxy.jsonl`: proxy lifecycle and backend event sidecar.
- `process-trace.jsonl`: process and launch handoff sidecar when process tracing is available.
- `compatibility.json`: scrubbed compatibility facts produced by the run.
- `cleanup.json`: per-resource cleanup report.

## Optional artifacts

- `http.har`: produced only when HTTP semantics are observable and HAR output is requested or enabled by default for the MVP.
- `tls-keylog.log`: created at its final bundle path before proxy traffic only when explicitly requested, populated incrementally for live analyzer integration, and declared as produced only when nonempty at finalization.

## Required validation

- Every artifact is declared in the manifest with role, path, authority, sensitivity, content type, and required state.
- Every missing expected artifact appears in `omissions` with reason and severity.
- `session_id` is identical across manifest, sidecars, and status events.
- Application records include `flow_id` or an explicit reason that flow correlation was unavailable.
- Sensitive and secret-adjacent artifacts are never printed in full in status output.
