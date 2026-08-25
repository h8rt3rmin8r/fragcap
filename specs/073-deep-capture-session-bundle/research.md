# Phase 0 Research: Deep Capture session bundle

## Decisions

### R-1: Manifest as required bundle index

**Decision**: Every Deep Capture bundle has a manifest. Sidecars without a manifest are incomplete output, not a bundle.

**Rationale**: Doctor cleanup, analyzer integration, and future status output need one stable file to read first. A convention based only on filenames would force every consumer to rediscover intent and omissions independently.

**Alternatives considered**:

- Directory convention only: rejected because omissions and cleanup results have no durable home.
- Embed everything in `.fcapng`: rejected because compatibility with unmodified pcapng readers outranks richness.

### R-2: `.fcapng` remains packet truth

**Decision**: `.fcapng` owns packets, packet timing, interfaces, attribution comments, and loss accounting. It does not carry full decrypted application objects.

**Rationale**: The project already relies on ordinary pcapng compatibility. Decrypted HTTP bodies, HAR objects, proxy logs, and key logs are different security and format surfaces.

**Alternatives considered**:

- Custom pcapng options for application events: deferred. It would require a larger compatibility discussion and does not remove the need for sidecars.
- Replace pcapng with a custom archive: rejected by P-5.

### R-3: Application JSONL is canonical for proxy events

**Decision**: Application JSONL is the canonical structured event stream for application-layer observations. HAR is a projection for HTTP workflows, not the only application truth.

**Rationale**: HAR is valuable but HTTP-specific. JSONL can represent HTTPS, WebSocket handshakes, metadata-only observations, proxy errors, and unsupported protocol notes using one streaming shape.

**Alternatives considered**:

- HAR-only application output: rejected because non-HTTP and partial-inspection cases need structured records too.

### R-4: HAR is utility-wide

**Decision**: HAR is allowed whenever HTTP semantics are observable, regardless of mode. Deep Capture makes HTTPS HTTP semantics observable when the target accepts the local CA and routes through the proxy.

**Rationale**: Plaintext HTTP in ordinary Capture should not be excluded from HAR merely because HAR is useful to Deep Capture.

**Alternatives considered**:

- Deep Capture-only HAR: rejected because it would make mode, rather than observability, the deciding criterion.

### R-5: TLS key logs are sensitive analyzer aids

**Decision**: TLS key logs are optional, sensitive, proxy-owned analyzer aids linked from the manifest. They are never emitted silently.

**Rationale**: A key log can decrypt captured proxy-owned TLS tunnels in analyzers. It is not ordinary log output and must be named, scoped, and cleaned accordingly.

**Alternatives considered**:

- Always emit a key log for Deep Capture: rejected because it creates sensitive material without a need.
- Treat key logs as application records: rejected because they are secrets for analyzers, not observations.

### R-6: Correlation anchors are explicit fields

**Decision**: Sidecars carry `session_id`, `flow_id`, proxy connection id, process id when known, role when known, and time bounds. Packet annotations carry the same `flow_id` whenever the packet has a parsed flow key.

**Rationale**: Parsing pcapng comments or human log lines to correlate decrypted records would make downstream analysis brittle.

**Alternatives considered**:

- Time-only correlation: rejected because concurrent connections can overlap.
- Process-only correlation: rejected because a process can own many flows.

### R-7: Cleanup report owns per-resource cleanup facts

**Decision**: The cleanup report sidecar is authoritative for per-resource
cleanup results. The manifest carries the cleanup report path and aggregate
status only.

**Rationale**: Cleanup can be updated after the session ends by doctor. Keeping
the resource facts in one artifact avoids split-brain status after partial
writes or later cleanup attempts.

**Alternatives considered**:

- Duplicate per-resource cleanup in the manifest and sidecar: rejected because
  consumers would need conflict resolution.
- Manifest-only cleanup facts: rejected because doctor needs an appendable,
  focused cleanup artifact without rewriting the whole bundle index for every
  resource update.
