# Implementation Plan: Complete IPv6 Parity

**Branch**: `codex/119-ipv6-parity` | **Date**: 2026-09-03 | **Spec**: [spec.md](spec.md)

## Summary

Promote the Deep Capture endpoint from an IPv4 port to an exact family-bearing loopback socket, add explicit IPv6 CLI selection and separate Doctor readiness, preserve scoped and mapped IPv6 identity, replace sequential TCP connection attempts with one finite staggered dual-stack race, and prove IPv6 parity across HTTP, HTTPS, SOCKS, TCP, UDP, QUIC, artifacts, and correlation.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Dependencies**: Existing standard library, Tokio, rustls, Quinn, h3, and workspace stack; no new dependency planned

**Storage**: Existing application JSON Lines version 2, HAR, manifest version 2, proxy and cleanup lifecycle streams; additive exact endpoint facts only

**Testing**: Endpoint and authority unit tests, deterministic connector-race tests through an injectable attempt seam, IPv6 protocol lab rows, facade and CLI integration, Doctor classifier and golden updates, full xtask gates

**Platform**: Windows production with portable loopback tests; exact IPv4 and IPv6 readiness observed independently

**Constraints**: One exact loopback bind, no wildcard, no external interface, one race winner, finite owners, no hidden family fallback, packet truth unchanged

**Scope**: Issue #315 only; #316 through #318 and #334 remain open

## Constitution Check

- **P-1**: PASS. The listener remains explicit, loopback-only, target-scoped, authenticated, and reversible. No target process access or global route mutation is added.
- **P-2/P-3**: PASS. Address parsing and connection racing stay in `fragcap-proxy`; facade owns session planning and CLI owns presentation and readiness probing.
- **P-4/P-9**: PASS. One selected peer or one stable failure is recorded. Mapped aliases cannot duplicate ownership, and unavailable facts remain absent.
- **P-5/P-8**: PASS. Existing packet and application formats remain readable; full analyzer, dependency, lint, format, schema, and conformance gates apply.
- **P-6**: PASS. Address-family, scoped-address, mapped-address, and connection-race terms receive glossary entries in this slice.
- **P-10/P-11**: PASS. Stored target routing remains authoritative and the master specification records only completed family parity.

Post-design check: PASS. No new dependency, privileged capability, interception mechanism, wildcard listener, or artifact authority is introduced.

## Architecture And Phases

1. Make the facade endpoint an exact loopback socket and add listener-family selection.
2. Carry the endpoint unchanged through plan, bind, routes, lifecycle, resource journal, rendering, and controlled harness.
3. Parse bracketed IPv6 and bounded numeric scopes into a typed authority without leaking the scope into TLS identity.
4. Canonicalize mapped peers for policy, deduplication, ownership, flow identity, and correlation while retaining observed sockets.
5. Add a bounded candidate planner and staggered one-winner TCP connection race under existing deadlines and cancellation.
6. Expose selected upstream local and peer sockets to protocol evidence.
7. Exercise IPv6 HTTP, HTTPS, SOCKS, TCP, UDP, QUIC, and HTTP/3 through controlled loopback rows.
8. Add independent IPv4 and IPv6 exact-bind probes and Doctor report rows.
9. Update master architecture, outline, plans, glossary, proxy README, AGENTS, issue note, and changelog, then run the full gate.

## Project Structure

```text
specs/119-ipv6-parity/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/ipv6-parity.md
|-- checklists/{requirements.md,security.md}
`-- tasks.md

crates/fragcap-proxy/src/{auth.rs,lib.rs,model.rs,quic.rs,runtime.rs,socks5.rs,tls.rs,upstream.rs}
crates/fragcap-proxy/tests/{ipv6_parity.rs,protocol_lab_support/*,upstream.rs}
crates/fragcap/src/deep_capture/{adapters.rs,application.rs,correlation.rs,model.rs,native.rs,routing.rs,session.rs}
crates/fragcap/tests/{deep_capture_session.rs,native_proxy.rs}
crates/fragcap-cli/src/{cli.rs,commands/deep_capture.rs,doctor/{checks.rs,mod.rs,probe.rs}}
crates/fragcap-cli/tests/{cli_args.rs,cli_doctor.rs}
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
docs/glossary/{capture-and-networking.md,index.md}
crates/fragcap-proxy/README.md
AGENTS.md
changelog.d/
```

## Complexity Tracking

No constitution exception is required. An injectable connector seam is justified only to prove deterministic race ownership without depending on host DNS timing; production remains a direct Tokio socket path.
