# Implementation Plan: Scoped QUIC And HTTP/3 Inspection

**Branch**: `codex/118-quic-http3-inspection` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

## Summary

Extend the authenticated SOCKS5 UDP route with a finite native QUIC termination pair. Promote the already reviewed Quinn transport and add Hyperium HTTP/3 over the same rustls and ring stack. Bind each pair to one approved origin and session, refuse 0-RTT, migration, and unknown ALPN, forward HTTP/3 streams and negotiated QUIC datagrams under bounds, and reconcile all evidence through existing application and lifecycle artifacts.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Dependencies**: Existing Tokio, rustls, ring, bytes, rcgen, and Quinn 0.11.11; new exact-pinned h3 0.0.8 and h3-quinn 0.0.10

**Storage**: Existing application JSON Lines version 2, HAR, manifest, lifecycle, and cleanup authorities; no new artifact

**Testing**: Unit policy/accounting tests, real loopback Quinn and HTTP/3 peers, two independent peer lineages, injected failure and saturation tests, full xtask gates

**Platform**: Windows production with portable IPv4 loopback baseline; complete IPv6 parity remains #315

**Constraints**: Authenticated route only, immutable origin, TLS 1.3, no 0-RTT, no active migration, finite owners, no downgrade, packet truth unchanged

**Scope**: Issue #314 only; #315 through #318 and #334 remain open

## Constitution Check

- **P-1**: PASS. QUIC interception remains explicit, child-scoped, authenticated, session-owned, external to the target, and reversible. No target key extraction or pinning bypass is introduced.
- **P-2/P-3**: PASS. QUIC and HTTP/3 transport stays in leaf crate `fragcap-proxy`; core capture and attribution remain unchanged.
- **P-4/P-9**: PASS. Every stream, datagram, queue, storage, refusal, and retention loss has a named authority; unavailable facts remain absent.
- **P-5/P-8**: PASS. Existing packet and application artifacts remain consumable, and exact dependency, format, lint, license, spec, and analyzer gates apply.
- **P-6**: PASS. QUIC-specific terms receive primary-source glossary entries in the same slice.
- **P-10/P-11**: PASS. The existing target route remains authoritative and the master specification records only the completed scoped boundary.

Post-design check: PASS. The new runtime packages implement standards already required by issue #314 and add no new privileged or global effect.

## Architecture And Phases

1. Promote Quinn and add exact h3 dependencies with license and MSRV evidence.
2. Add typed admission, connection-pair, stream, datagram, refusal, and accounting values.
3. Add finite QUIC client/server configuration under the session CA, existing root store, and immutable destination policy.
4. Attach the QUIC gateway to authenticated UDP associations without changing generic UDP observation authority.
5. Add HTTP/3 stream and QUIC datagram forwarding with loss accounting; refuse unknown ALPN.
6. Add ALPN-selected HTTP/3 request/response forwarding and existing metadata/body observations.
7. Extend application JSON Lines, HAR, manifest, lifecycle, correlation, and cleanup reconciliation.
8. Prove two peer lineages, authenticated production routing, negative security behavior, limits, and cleanup in the controlled lab.
9. Update architecture records, glossary, plan status, AGENTS, README, and changelog, then run the full gate.

## Project Structure

```text
specs/118-quic-http3-inspection/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/quic-http3-evidence.md
|-- checklists/{requirements.md,security.md}
`-- tasks.md

crates/fragcap-proxy/src/{application.rs,certificate.rs,lib.rs,model.rs,quic.rs,runtime.rs,socks5.rs,upstream.rs}
crates/fragcap-proxy/tests/quic_http3.rs
crates/fragcap/src/deep_capture/{application.rs,har.rs,lifecycle.rs,manifest.rs}
crates/fragcap/tests/{application_stream.rs,deep_capture_session.rs}
Cargo.toml
Cargo.lock
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
docs/glossary/{capture-and-networking.md,index.md}
crates/fragcap-proxy/README.md
AGENTS.md
changelog.d/
```

## Complexity Tracking

No constitution exception is required. The dependency promotion is proportional to the standards implementation explicitly required by issue #314.
