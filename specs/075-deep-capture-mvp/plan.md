# Implementation Plan: Deep Capture MVP

**Branch**: `codex/219-deep-capture-mvp`

**Issue**: #219

## Summary

Implement the first functional Deep Capture vertical slice: one stored target, one managed launch path whose facts show scoped proxy compatibility, one session-owned `mitmdump` proxy backend, explicit fragcap-owned CA/trust lifecycle, packet capture running beside application-layer observation, a #216 session bundle, #217 compatibility fact updates, and #218 doctor cleanup integration.

The implementation should be one substantial PR. The internal phases below are checkpoints for development and review, not separate PR boundaries.

## Technical Context

**Language**: Rust.

**Primary crates**: `fragcap-cli`, `fragcap`, `fragcap-targets`, and existing test support.

**Runtime backend**: external `mitmdump` child process, discovered the same way doctor reports proxy readiness.

**Dependencies**: No new Rust dependency for the MVP unless implementation proves the current process and JSON support cannot safely express the required behavior. A native Rust proxy backend remains a follow-on design issue.

**Testing**: Unit tests over injected adapters, CLI contract tests, controlled local target integration tests, bundle validation tests, compatibility store tests, and the usual workspace gates.

## Constitution Check

- **P-1 No covert target instrumentation**: Pass if the MVP uses only scoped proxy configuration, an owned proxy process, explicit current-user trust changes, and proxy-owned analyzer aids. The implementation must not add injection, hooks, target memory reads, target TLS key extraction, executable mutation, Winsock/LSP changes, interception drivers, or system-wide proxy fallback.
- **P-2 Core stays platform-neutral**: Pass if platform-specific CA/trust and process orchestration remain outside `fragcap-core`.
- **P-3 Capture and attribution stay decoupled**: Pass if proxy application observations are sidecars and do not become an attribution dependency.
- **P-4 No silent loss**: Pass if unsupported protocols, uninspectable TLS, proxy misses, backend failure, sink failure, and cleanup failure all have explicit counters or omission/status records.
- **P-5 Compatibility over richness**: Pass if `.fcapng` remains packet truth and HAR/application sidecars are additive.
- **P-9 The instrument does not lie**: Pass if the command distinguishes observed, unknown, metadata-only, unsupported, refused, and cleanup-not-attempted states.
- **P-10 One path to a target**: Pass if Deep Capture resolves stored targets through the existing selector/store path and writes facts to the existing `deep_capture_facts` table.

## Scope

In scope:

- First-class Deep Capture CLI surface for one stored target.
- Blocking preflight using existing doctor-compatible proxy and Deep Capture readiness facts.
- `mitmdump` backend adapter with owned process lifecycle, bound local port, structured event ingestion, and cleanup.
- Fragcap-owned CA material and explicit current-user trust workflow.
- Managed launch with scoped proxy environment variables or equivalent target-scoped configuration.
- Existing packet capture path running alongside proxy observation.
- Session bundle creation using the #216 contracts.
- Application JSONL and optional HAR when HTTP semantics are observable.
- Deep Capture lifecycle status events through the existing emitter.
- Compatibility fact updates through the #217 local store model.
- Controlled local target verification path with no third-party account or real title data.
- Privacy scan for committed fixtures and docs.

Out of scope:

- Universal game compatibility.
- Native Rust proxy backend implementation.
- Certificate pinning bypass or trust evasion.
- QUIC decryption.
- Non-HTTP TLS dissection beyond metadata-only records.
- System-wide proxy configuration.
- Community compatibility sync.

## Implementation Phases

### Phase 1: Command and Preflight

Add the Deep Capture command surface, parse options, resolve exactly one stored target, and refuse unsupported invocation shapes before any proxy, trust, or launch side effect. Preflight should consume the same raw facts doctor reports where possible and layer command-specific blocking rules on top.

### Phase 2: Proxy and Trust Adapters

Introduce narrow traits for proxy backend and trust management, plus fake implementations for tests. Implement `mitmdump` as the first backend through child process orchestration. Keep process cleanup and port cleanup auditable.

### Phase 3: Session Orchestration

Coordinate session id, bundle root, proxy start, trust confirmation, managed launch, packet capture, application observation ingestion, status output, interrupt handling, and cleanup. Avoid duplicating the existing capture pipeline; Deep Capture should call into the same orchestrator where feasible.

### Phase 4: Outputs and Compatibility Facts

Write manifest, application JSONL, HAR where observable, proxy/process sidecars, compatibility update sidecar, and cleanup report. Insert scrubbed compatibility facts into `deep_capture_facts` after the run.

### Phase 5: Verification and Hardening

Run deterministic fake-backend tests in CI, optional local `mitmdump` demonstration tests, privacy scans, and full workspace gates. Validate that ordinary Capture, doctor, and extcap behavior still hold.
