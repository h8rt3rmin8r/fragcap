# Implementation Plan: Generic TCP And Non-HTTP TLS Evidence

**Branch**: `codex/116-generic-tcp-tls` | **Date**: 2026-09-02 | **Spec**: [spec.md](spec.md)

## Summary

Add one bounded generic-stream observer and use it in approved SOCKS5 tunnels and trusted no-ALPN CONNECT TLS. Preserve existing HTTP selection, TLS verification/refusal, route, connection, application artifact, correlation, and cleanup authorities.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Dependencies**: Existing standard library, Tokio, rustls, bytes, and base64; no new package

**Storage**: Existing `application.jsonl`, `proxy.jsonl`, runtime accounting, and body retention budget

**Testing**: Unit bounded relay tests, loopback SOCKS/TLS tests, serialization and lifecycle tests, full xtask gate

**Platform**: Windows production; portable controlled loopback tests

**Constraints**: Target-scoped authenticated routes, independent TLS verification, fixed buffers, bounded retention, no semantic guessing, no downgrade fallback

**Scope**: Issue #312 only; #313 through #318 and #334 remain open

## Constitution Check

- **P-1**: PASS. Interception remains explicitly selected, target-scoped, proxy-owned, auditable, and external to the target. No prohibited technique is introduced.
- **P-2/P-3**: PASS. Transport/TLS mechanics stay in `fragcap-proxy`; packet capture and attribution remain untouched.
- **P-4/P-9**: PASS. Every observed byte is forwarded or failed explicitly, and retained versus omitted evidence plus encrypted/decrypted provenance is exact.
- **P-5/P-8**: PASS. Existing pcapng remains unchanged and application evidence extends its versioned JSONL contract.
- **P-10/P-11**: PASS. One stored target and shared route remain authoritative, and documentation records only the completed #312 boundary.

Post-design check: PASS. Reusing application and body authorities avoids a second artifact or storage policy.

## Architecture And Phases

1. Add typed generic direction, provenance, outcome, and chunk values plus protocol accounting.
2. Implement a fixed-buffer full-duplex observer that claims bounded retention independently from forwarding and emits monotonic chunks.
3. Replace SOCKS aggregate-only relay observation with generic chunks for plain and opaque TLS classifications.
4. Add buffered no-ALPN TLS discrimination to HTTP CONNECT, preserving bytes for either HTTP/1.1 or protocol-unknown relay.
5. Serialize generic chunks in application and lifecycle streams and reconcile queue loss through existing writer accounting.
6. Prove transport, TLS, refusal, omission, truncation, half-close, correlation, and cleanup cases.
7. Update architecture, glossary, status, README, AGENTS, and changelog.

## Project Structure

```text
specs/116-generic-tcp-tls/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/generic-stream-evidence.md
├── checklists/{requirements.md,security.md}
└── tasks.md

crates/fragcap-proxy/src/{application.rs,generic.rs,lib.rs,model.rs,runtime.rs,socks5.rs,tls.rs}
crates/fragcap-proxy/tests/{socks5_proxy.rs,https_proxy.rs}
crates/fragcap/src/deep_capture/{application.rs,lifecycle.rs}
crates/fragcap/tests/application_stream.rs
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
docs/glossary/{capture-and-networking.md,index.md}
crates/fragcap-proxy/README.md
AGENTS.md
```

## Complexity Tracking

No constitution exception is required.
