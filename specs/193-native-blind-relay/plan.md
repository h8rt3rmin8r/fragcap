# Implementation Plan: Native Blind Relay

**Branch**: `codex/s193-native-blind-relay` | **Date**: 2026-11-04 | **Spec**: [spec.md](spec.md)

## Summary

Close #559 by consuming S191's immutable decision at every native ingress. Keep the current CLI launch-chain default until #561. Uninspected HTTP CONNECT and SOCKS paths forward opaque bytes, while plaintext HTTP/2 bridges protocol framing without exporting excluded content. Existing authentication, destination policy, finite resource ownership, and route-owner release remain authoritative.

## Technical Context

**Language**: Rust workspace, exact toolchain in `rust-toolchain.toml`.
**Dependencies**: Existing Tokio, h2, rustls, Quinn, and native proxy crates; no new runtime package planned.
**Storage**: Add typed relay accounting to existing proxy diagnostics and manifest projection, no database migration.
**Platform**: Windows product, platform-neutral runtime decision contract and controlled local origin fixtures.
**Verification**: Focused protocol fixtures, workspace checks, Windows/package checks where applicable, exact-head CI.

## Constitution Check

- P-1: Uninspected connections cannot touch TLS interception or key logging; explicit Deep Capture authorization and existing route policy remain.
- P-2/P-3: The proxy consumes S191's platform-neutral decision. Exact Windows owner resolution stays in the facade.
- P-4/P-9: Scope omission, transport failure, evidence loss, and packet loss remain separate; no silent downgrade or fabricated application observation.
- P-5/P-7/P-8: Ordinary pcapng and wrappers remain unchanged; source and docs remain UTF-8 without BOM.
- P-11: Master specification and changelog describe actual runtime support only. Post-release installed-product findings become issues.

## Design Sequence

1. Add explicit relay disposition to the immutable decision's runtime handling and trace. Authenticate first, then enter an uninspected protocol path before leaf creation.
2. For HTTP/1 CONNECT, retain the permitted upstream TCP socket after the 200 response and run a bounded opaque bidirectional relay with activity idle and session lifetime. Emit transfer totals and one terminal fact; never create a leaf or TLS event.
3. For plaintext HTTP/1 and HTTP/2, reuse framing engines with a non-exporting sink and payload retention disabled, then retain only bounded relay diagnostics. Add a cleartext HTTP/2 CONNECT stream path if the current bridge does not support it.
4. For SOCKS5 TCP, reuse byte-transparent forwarding with disabled generic chunk emission; for UDP, select the independent association decision before QUIC gateway creation and use the original datagram sockets for excluded owners.
5. Reconcile relay/inspection/uncertainty totals through the existing connection diagnostics and artifact projection. Keep route-owner release independent of observation end.
6. Run controlled same-hostname, protocol, address-family, timeout, failure, writer-loss, and lifecycle regressions. Update master specification, API/glossary, chronology, and changelog.

## Decision Record

- **D-1**: A single immutable per-connection or association action selects inspection or relay. Destination and capability admission remain separate. This avoids a hostname bypass and keeps the S191 owner proof authoritative.
- **D-2**: A relayed CONNECT uses raw origin TCP and never a client-facing or upstream TLS endpoint. The client's own TLS validation therefore remains intact.
- **D-3**: Plaintext HTTP must use a framing bridge, not a raw byte tunnel. Excluded application events and per-exchange proxy observations are suppressed while bounded connection diagnostics remain.
- **D-4**: SOCKS UDP has its own first-admitted-datagram decision. An excluded QUIC Initial uses the original upstream UDP sockets rather than the inspection gateway.
- **D-5**: Established relay uses activity-based idle and the authorized session's finite lifetime. HTTP header timeout applies only before the tunnel response; no hidden shortening of explicit session duration.
- **D-6**: #542 remains an independent observed launcher failure. #560 calibration and #561 operator selection follow this runtime work; neither is silently implemented here.

## Project Structure

`crates/fragcap-proxy/src/{runtime,http1,http2,socks5,generic,model}.rs` own transport and accounting changes. `crates/fragcap/src/deep_capture/{native,manifest}.rs` owns artifact projection and existing route lifecycle. Controlled regressions live in the owning crate tests. `docs/fragcap-specification.md`, `docs/plans/README.md`, API/glossary docs, and `changelog.d/` record the delivered behavior.
