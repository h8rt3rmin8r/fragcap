# Security Checklist: Generic TCP And Non-HTTP TLS Evidence

**Purpose**: Validate the interception and tenancy boundaries before implementation

## Authorization And Tenancy

- [x] Existing session capability remains mandatory
- [x] Existing target-scoped route and destination policy remain authoritative
- [x] No additional listener or system-wide effect is introduced
- [x] No target process access or target key extraction is introduced

## TLS Boundaries

- [x] Client-facing TLS uses only the session authority
- [x] Upstream TLS uses independent certificate verification
- [x] Pinning and client-auth failures remain explicit
- [x] Failed interception never silently downgrades

## Bounds And Loss

- [x] Forwarding buffers, retention, event chunks, queue capacity, and deadlines are finite
- [x] Omitted, truncated, and queue-dropped evidence is counted
- [x] Cleanup joins every stream task
- [x] No payload appears when capture is disabled
