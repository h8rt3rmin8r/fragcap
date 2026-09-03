# Implementation Plan: Generic UDP Observations

**Branch**: `codex/117-generic-udp-observations` | **Date**: 2026-09-02 | **Spec**: [spec.md](spec.md)

## Summary

Extend S115's authenticated UDP association with one typed generic datagram record. Reuse the body retention budget, application writer, connection correlation, and protocol accounting. Preserve complete forwarding while retaining a bounded payload prefix, exact endpoints, direction, sequence, timing, and explicit loss or socket-error provenance.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Dependencies**: Existing standard library, bytes, serde_json, and Tokio; no new package

**Storage**: Existing `application.jsonl`, manifest, and lifecycle accounting; no new artifact

**Testing**: Unit retention and serialization tests, real loopback UDP matrix, injected error and queue tests, full xtask gates

**Platform**: Windows production; portable controlled loopback tests

**Constraints**: Authenticated S115 association only, exact boundaries, finite retention, no observation backpressure, no inferred semantics or ICMP

**Scope**: Issue #313 only; #314 through #318 and #334 remain open

## Constitution Check

- **P-1**: PASS. Evidence remains explicit, loopback, capability-authenticated, child-scoped, external to the target, and reversible.
- **P-2/P-3**: PASS. UDP transport and event types stay in `fragcap-proxy`; bundle serialization remains in the facade.
- **P-4/P-9**: PASS. Datagram and loss conservation are explicit, endpoints are observed, and platform limitations remain unavailable rather than inferred.
- **P-5/P-8**: PASS. Existing artifact authorities and mechanical gates cover the additive record.
- **P-10/P-11**: PASS. One target route remains authoritative and documentation claims only routed generic UDP evidence.

Post-design check: PASS. The slice adds no route, artifact, dependency, or lifecycle authority.

## Architecture And Phases

1. Add a generic datagram value with direction, sequence, endpoints, lengths, retained bytes, and outcome.
2. Add a per-association observer sharing the existing connection and session retention budgets.
3. Observe accepted client and upstream ingress at the S115 boundary before complete-payload forwarding.
4. Extend protocol and writer accounting for observation, omission, truncation, queue loss, storage failure, and visible socket errors.
5. Serialize one additive application JSON Lines record with base64 payload only when retained.
6. Prove exact boundaries, ordering, duplicates, bounds, forwarding independence, errors, and cleanup in the controlled lab.
7. Update architecture records and changelog, then run the full gate.

## Project Structure

```text
specs/117-generic-udp-observations/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── generic-udp-evidence.md
├── checklists/
│   ├── requirements.md
│   └── security.md
└── tasks.md

crates/fragcap-proxy/src/{application.rs,model.rs,runtime.rs,socks5.rs}
crates/fragcap-proxy/tests/socks5_udp.rs
crates/fragcap/src/deep_capture/application.rs
crates/fragcap/tests/deep_capture_application.rs
docs/{fragcap-specification.md,fragcap-spec-outline.md,plans/README.md}
docs/glossary/{capture-and-networking.md,index.md}
crates/fragcap-proxy/README.md
AGENTS.md
```

## Complexity Tracking

No constitution exception is required.
