# Implementation Plan: Native Protocol Conformance

**Branch**: `codex/110-protocol-conformance` | **Date**: 2026-09-01 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/110-protocol-conformance/spec.md`

## Summary

Add a data-driven native HTTP and TLS conformance harness that executes independent loopback client and origin implementations, validates the complete Deep Capture artifact set, commits normalized synthetic results, and requires TShark consumption on a dedicated CI tier. Required rows are a closed set and any skip, duplicate, missing result, or stale evidence fails.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88

**Primary Dependencies**: Existing standard library, tokio, hyper, h2, rustls, serde_json, fragcap-proxy, and fragcap; no new product package

**Storage**: Versioned JSON conformance matrix and normalized JSON report plus committed synthetic pcapng and TLS key-log analyzer fixtures

**Testing**: Data contract, independent loopback interoperability, production artifact readers, deterministic evidence drift, external TShark, full workspace and xtask gates

**Target Platform**: Windows production runtime; portable loopback harness on Windows and Linux; TShark analyzer tier on Ubuntu

**Project Type**: Rust workspace with test harness, xtask gate, and CI integration

**Performance Goals**: Complete portable conformance run within ordinary CI timeout; every row bounded to finite connections, streams, body bytes, and deadline

**Constraints**: Offline loopback only, zero required skips, no secret-bearing committed evidence, no product feature addition, no generic transport claim, no new product dependency

**Scale/Scope**: Issue #305, six standard protocol families, two client and two origin implementations per family, TLS and failure boundaries, nine integrated artifact roles, one external analyzer gate

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. All generated traffic is bounded loopback traffic under explicit test control. No target process access, global proxy, interception driver, pinning bypass, or remote destination is introduced.
- **P-2/P-3**: PASS. Product boundaries do not move. Proxy interoperability lives beside `fragcap-proxy`; complete bundle conformance lives at the `fragcap` facade; CI orchestration lives in xtask and workflows.
- **P-4/P-9**: PASS. Missing, skipped, lossy, ambiguous, unavailable, stale, and contradictory evidence remain explicit failures or named expected failure outcomes.
- **P-5**: PASS. Analyzer proof uses ordinary pcapng and the standard TLS key-log preference with unmodified TShark.
- **P-6/P-8**: PASS. New evidence vocabulary is documented and all committed text remains UTF-8 without BOM or mojibake.
- **P-10**: PASS. No target identity or storage path is introduced.
- **P-11**: PASS. This is the milestone 2 conformance exit gate and explicitly preserves the incomplete Deep Capture claim.

Post-design check: PASS. The matrix is evidence about existing behavior, not a second runtime truth model. Production readers remain the authority for each artifact.

## Architecture and Phases

1. Define the conformance matrix and normalized result contract, including implementation lineage, standards, tiers, row states, artifact assertions, and computed coverage.
2. Add a validator that rejects missing, duplicate, skipped, stale, same-lineage, or unreconciled required rows and checks deterministic committed results.
3. Add the missing bounded raw HTTP/2 and Hyper peers, inventory existing wire and library peers, and reject aliases through explicit driver lineage.
4. Add integrated facade scenarios that generate and verify application, HAR, key-log, correlation, lifecycle, cleanup, journal, and manifest truth.
5. Commit sanitized synthetic evidence and analyzer fixtures, then add an xtask command that validates portable evidence and invokes TShark when required.
6. Add Windows and Linux portable execution plus a dedicated Ubuntu analyzer job with no skip path.
7. Correct stale slice prose, document reproduction and review, add changelog fragments, and run all gates.

## Project Structure

```text
specs/110-protocol-conformance/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── tasks.md
├── contracts/
│   └── conformance-evidence.md
└── checklists/
    └── requirements.md

conformance/native-http-tls/
├── matrix-v1.json
├── report-v1.json
├── analyzer.pcapng
└── tls-keylog.log

crates/fragcap/tests/
└── native_conformance.rs

crates/fragcap-proxy/tests/
└── conformance.rs

xtask/src/
├── conformance.rs
└── main.rs

.github/workflows/ci.yml
docs/plans/README.md
docs/fragcap-specification.md
AGENTS.md
```

**Structure Decision**: Protocol peers and wire assertions live with the native proxy. Cross-artifact reconciliation lives in the facade, the only crate that legitimately owns the complete Deep Capture bundle. Versioned evidence is repository-level because xtask and CI consume it without depending on test build output.

## Explicit Deviation

S109 left two stale statements assigning "generic transports (#305)" to S110. This plan rejects that assignment. Issue #305 is the conformance gate, while master specification section 28 assigns generic TCP, SOCKS, UDP, QUIC, HTTP/3, and related transport work to milestone 3 issues #310 through #318. S110 corrects the prose rather than implementing unrelated transport features.
