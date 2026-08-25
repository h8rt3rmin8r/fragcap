<!-- spec-impact: 26.3 -->

Added Deep Capture readiness and cleanup checks to `fragcap doctor`: proxy backend availability, local CA trust state, analyzer key-log readiness, stale proxy ports/processes, stale manifests, TLS key logs, sensitive sidecars, session storage reporting, and confirmation-gated cleanup for unfinished manifests and known sensitive sidecars under fragcap-owned session storage.
