# Quickstart: Validate Scoped QUIC And HTTP/3 Inspection

## Prerequisites

- Work on `codex/118-quic-http3-inspection`.
- `.specify/feature.json` points to `specs/118-quic-http3-inspection` and remains unstaged.
- Use synthetic loopback certificates, peers, routes, and payloads only.

## Focused Validation

```sh
cargo test -p fragcap-proxy --test quic_http3
cargo test -p fragcap --test application_stream
cargo test -p fragcap --test deep_capture_session
```

Expected results:

- A trusted client-facing QUIC connection and independently verified upstream connection form one scoped pair.
- Generic bidirectional and unidirectional streams plus QUIC datagrams forward and reconcile under bounds.
- HTTP/3 requests and responses retain exact semantic metadata and bounded bodies.
- 0-RTT, migration, endpoint changes, trust failures, and unrouted traffic produce stable refusals with no fallback.
- Cancellation joins all tasks and releases all endpoint state.

## Full Gate

```sh
cargo xtask ci
```

Review the final diff for issue #314 only. Confirm exact dependency pins and license coverage, no process access, no target key extraction, no pinning bypass, no global proxy change, no unbounded owner, no Unicode dash punctuation, no BOM, no mojibake, and no staged `.specify/feature.json`.
