# Implementation Plan: Scoped SOCKS5 UDP Association

**Branch**: `codex/115-socks5-udp-associate` | **Date**: 2026-09-02 | **Spec**: [spec.md](spec.md)

## Summary

Extend the authenticated SOCKS5 path with one control-owned UDP association. Add bounded wire parsing, immutable client endpoint ownership, policy-checked proxy DNS, fixed family-specific upstream sockets, exact peer reply validation, metadata-only evidence, complete loss accounting, and deterministic cleanup.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Dependencies**: Existing standard library and Tokio; no new package

**Storage**: Existing runtime counters and `application.jsonl`; no new artifact

**Testing**: Unit wire tests, real loopback UDP protocol lab, security/tenancy tests, lifecycle and full xtask gates

**Platform**: Windows production; portable controlled loopback tests

**Constraints**: Loopback, authenticated TCP ownership, fixed sockets, finite datagram and peer bounds, no fragmentation, no payload retention, no Internet

**Scope**: Issue #311 only; #312 through #318 and #334 remain open

## Constitution Check

- **P-1**: PASS. The relay is explicit, loopback, capability-authenticated, child-scoped, reversible, and external to the target.
- **P-2/P-3**: PASS. Wire and socket ownership stay in `fragcap-proxy`; orchestration and CLI remain unchanged.
- **P-4/P-9**: PASS. Every ingress datagram has a named outcome; exact observed endpoints are retained and missing endpoints stay unavailable.
- **P-5/P-8**: PASS. Existing pcapng and sidecar authorities remain intact and mechanical gates cover all changes.
- **P-10/P-11**: PASS. One stored target and route remain authoritative, and the architecture records only the implemented #311 boundary.

Post-design check: PASS. Fixed sockets and a bounded peer set add no competing lifecycle, routing, or artifact authority.

## Architecture And Phases

1. Generalize the authenticated SOCKS request parser to distinguish CONNECT and UDP ASSOCIATE.
2. Add a bounded UDP frame parser/encoder and explicit frame failures.
3. Bind one client-facing loopback relay and fixed upstream IPv4/IPv6 sockets, then return success.
4. Run a cancellation-aware association loop governed by control EOF and idle timeout.
5. Pin the client endpoint, resolve and policy-check each destination, retain exact contacted peers under a cap, and validate every reply source.
6. Add typed metadata-only events and protocol accounting for association, transfer, every drop class, mappings, and terminal cleanup.
7. Add the offline functional/security matrix and update architecture documentation and changelog.

## Project Structure

```text
specs/115-socks5-udp-associate/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   ├── socks5-udp-wire.md
│   └── socks5-udp-evidence.md
├── checklists/
│   ├── requirements.md
│   └── security.md
└── tasks.md

crates/fragcap-proxy/src/{application.rs,lib.rs,model.rs,socks5.rs,upstream.rs}
crates/fragcap-proxy/tests/socks5_udp.rs
crates/fragcap/src/deep_capture/application.rs
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
docs/glossary/{capture-and-networking.md,index.md}
crates/fragcap-proxy/README.md
AGENTS.md
```

## Complexity Tracking

No constitution exception is required.
