# Implementation Plan: S160 QUIC observation headroom

**Branch**: `codex/s160-quic-observation-headroom` | **Date**: 2026-09-21 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/160-quic-observation-headroom/spec.md`

## Summary

Correct the third observed Windows QUIC queue-saturation recurrence by replacing
the disproven readiness-only assumption with one shared 16,384-event product and
harness queue authority, deterministic post-readiness stall coverage, and a
truthful payload projection that adds pre-writer queue loss while partitioning
post-queue storage loss exactly once. Preserve nonblocking forwarding, finite
ownership, exact overload loss, the 256 MiB worker ceiling, artifact content, cleanup, protocol
behavior, and first-attempt hosted acceptance.

## Technical Context

**Language/Version**: Rust 2021 with MSRV 1.88 and pinned product toolchain 1.96.0

**Primary Dependencies**: Rust standard library synchronization plus the existing exact-pinned workspace graph; no dependency change

**Storage**: Existing application JSON Lines version 2, unchanged; new performance JSON Lines version 2 with required attempted-event and queue-capacity fields plus corrected payload meaning; historical performance version 1 remains readable

**Testing**: Rust unit and integration tests, performance authority validation, controlled synthetic loopback hosted campaign, and full workspace gates

**Target Platform**: Windows x86-64 is the failing authority; portable queue and accounting regressions run on all supported build hosts

**Project Type**: Rust workspace, facade library, native proxy library, CLI, and isolated performance harness

**Performance Goals**: Fourteen short-profile cases with zero queue or storage loss; 16,384-event finite queue; at most 256 MiB worker memory; at most 32 MiB artifact; shutdown within 5 seconds

**Constraints**: Nonblocking producers, forwarding independent from evidence storage, exact loss beyond capacity, no event suppression, no protocol or artifact-content change, no installed local product, no real game, no real trust mutation, no sensitive live capture

**Scale/Scope**: One shared capacity constant, three product or harness consumers, one private test seam, one payload projection, one performance report schema bump with two required fields, one registry field, focused documentation and regression coverage

## Constitution Check

*GATE: Passed before Phase 0 research and re-checked after Phase 1 design.*

- **P-1**: Pass. Work changes bounded observation storage and controlled tests
  only. No target process, external route, trust mutation, driver, game, or
  denylisted technique is introduced.
- **P-2 and P-3**: Pass. The capacity authority remains in the Deep Capture
  facade and the isolated harness consumes it downward. Core, packet capture,
  and attribution are unchanged.
- **P-4**: Pass. Producers remain nonblocking. Every event beyond the finite
  bound retains its exact event and byte counters. The short gate still rejects
  any actual loss.
- **P-5**: Pass. Capture and application artifact formats and contents are
  unchanged. The isolated performance report advances to schema version 2
  because field requirements and payload meaning change; version 1 remains
  readable and unknown versions remain refused.
- **P-6**: Pass. No new product-domain vocabulary requires a glossary entry.
  Slice-local terms are defined in the data model.
- **P-7 and P-8**: Pass. No wrapper behavior changes, and all Rust, JSON, and
  Markdown remain under repository mechanical gates.
- **P-9**: Pass. The performance projection corrects a misleading field meaning
  so observed bytes include each accepted or lost disposition exactly once.
- **P-10 and P-11**: Pass. Target behavior is unchanged. The master
  specification records the superseding queue authority and retained overload
  truth without changing the published v0.10.2 baseline.
- **Pinned artifacts**: Pass with a dated decision. The reviewed performance
  registry changes its queue ceiling; the workflow file itself remains
  unchanged.
- **Development workflow**: Pass. Issue #435 records the concrete recurrence;
  S160 runs the full spec-kit sequence before implementation and uses fresh
  first-attempt hosted evidence.

Post-design check: PASS. The design changes one bounded capacity and one report
projection, adds deterministic tests at their owning boundaries, and leaves
security, forwarding, protocols, artifact schemas, and cleanup intact.

## Project Structure

### Documentation (this feature)

```text
specs/160-quic-observation-headroom/
├── checklists/
│   ├── reliability.md
│   └── requirements.md
├── contracts/
│   └── queue-and-report.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
changelog.d/
├── S160-quic-observation-headroom.decisions.md
└── S160-quic-observation-headroom.fixed.md

crates/fragcap/src/deep_capture/
├── application.rs
└── native.rs

crates/fragcap-proxy/src/
└── application.rs

performance/
├── native-proxy-budgets-v1.json
└── native-proxy/src/
    ├── main.rs
    └── workloads.rs

docs/
├── fragcap-specification.md
├── plans/README.md
└── security/deep-capture-performance.md
```

**Structure Decision**: Correct the existing facade queue and isolated harness
in place. The facade owns the one exported default capacity because production
and performance both construct its artifact lease. The existing proxy sink
accounting contract gains one additive attempt counter because accepted plus
dropped is not a unique total when storage can drop an already accepted event.
The private writer stall seam stays test-only. No new crate, dependency,
workflow, or operator setting is warranted.

## Complexity Tracking

No constitution violation requires an exception.
