# Contract: Deep Capture Status Events

Deep Capture events extend the existing lifecycle event stream. Each line is JSON with the existing `ts` and `event` fields.

## Event kinds

- `deep_capture.preflight`: preflight completed with `status`, `blockers`, `warnings`, `target`, `proxy_backend`, and `trust_state`.
- `deep_capture.proxy_started`: local proxy started with `backend`, `version`, `listen_addr`, `listen_port`, and `session_id`.
- `deep_capture.trust`: CA/trust state changed or was confirmed with `state`, `thumbprint`, and `action`.
- `deep_capture.launch`: managed launch started with `launch_case`, `scoped_proxy`, and `target`.
- `deep_capture.application`: application observation summary with `flow_id`, `proxy_connection_id`, `protocol`, and `inspectability`.
- `deep_capture.bundle`: bundle artifact written with `role`, `path`, `sensitivity`, and `required`.
- `deep_capture.cleanup`: cleanup resource status with `resource`, `status`, and `reason`.
- `deep_capture.complete`: terminal session summary with `session_id`, `manifest`, `inspectable`, `metadata_only`, `unsupported`, and `cleanup_status`.

## Required behavior

- Events must be emitted through the existing `Emitter`.
- Human mode may summarize the same facts, but JSON mode must not depend on parsing human prose.
- Non-terminal human output must not contain terminal control sequences.
- Sensitive contents are never included. Paths may be included only for bundle artifacts and cleanup resources that are already visible in the manifest.
