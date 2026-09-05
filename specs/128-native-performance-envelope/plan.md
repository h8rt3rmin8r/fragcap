# Implementation Plan: Native Deep Capture Performance Envelope

**Branch**: `codex/128-native-performance-envelope` | **Date**: 2026-09-04 | **Spec**: `specs/128-native-performance-envelope/spec.md`

**Input**: Feature specification from `specs/128-native-performance-envelope/spec.md`

## Summary

Publish a closed 14-case native proxy performance registry, execute real loopback protocol workloads in fresh isolated workers, measure paired timing and exact resource/loss/shutdown authorities, add the missing bounded runtime gauges discovered during planning, and enforce short and multi-hour profiles through separate automation.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88, pinned product toolchain 1.96.0

**Primary Dependencies**: Existing exact-pinned native proxy graph only; existing `windows-sys` line gains process-status and threading API feature gates

**Storage**: Versioned JSON budget registry and newline-framed campaign reports under ignored build output; reports uploaded by automation

**Testing**: Task-runner registry tests, runtime accounting tests, deterministic harness self-tests, release-mode short campaigns, full `cargo xtask ci`

**Target Platform**: Windows 11 x86-64 product and performance authority; Ubuntu x86-64 portable comparison gate

**Project Type**: Rust workspace, isolated performance-tool workspace, and CLI task runner

**Performance Goals**: Fourteen required protocol/retention rows; at least 1 MiB/s useful loopback throughput; protocol-specific added p95 latency at or below the 25 to 750 millisecond registry ceilings; short matrix below 15 minutes; manual two-hour default soak or explicit project-owner acceptance of sufficient preserved zero-failure evidence

**Constraints**: Real production proxy paths, synthetic loopback only, no external network or trust mutation, no product fault/performance switch, no target handle or memory read, exact loss conservation, fresh worker per row, hidden Windows child processes

**Scale/Scope**: Seven protocols times two retention modes, seven windows after warmup, deterministic product overload/cache/task tests, two CI platforms for short runs, one manually dispatched Windows soak authority

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. Workers self-measure and use only harness-owned loopback endpoints. No target process, system trust, ambient routing, interception driver, or denylisted technique is involved.
- **P-2/P-3**: PASS. Runtime gauges remain in `fragcap-proxy`; the isolated harness depends downward on public product APIs. Capture and attribution are untouched.
- **P-4/P-9**: PASS. Every loss disposition reconciles, unsupported metrics refuse success, whole-harness CPU and memory are labeled honestly, and the previously unbounded failure detail is capped with an explicit eviction counter.
- **P-5**: PASS. Capture and Deep Capture artifact formats are unchanged; performance reports are separate test evidence.
- **P-6/P-8**: PASS. New vocabulary receives glossary entries and every artifact follows repository mechanics.
- **P-10/P-11**: PASS. Target storage is unchanged. The specification and roadmap record only the S128 performance boundary and preserve #334 as the completion gate.
- **Pinned artifacts**: PASS with recorded decision. The new performance workflow is required by issue #326 and receives a dated decision fragment.

Post-design check: PASS. The harness is isolated from the shipped graph, every required measurement has a production authority or explicit self-measurement source, and no new runtime capability is exposed to operators.

## Architecture and Phases

1. Freeze registry vocabulary, workloads, hard budgets, comparison rules, and short/soak profiles before recording measurements.
2. Add source-derived validator coverage for the complete protocol/retention matrix, report schema, evidence references, workflow profiles, and immutable budget digest.
3. Bound runtime failure details and expose leaf-cache, task, and application-queue gauges through existing observations.
4. Build one isolated parent/worker harness with four reusable real-protocol drivers and exact self-measurement.
5. Run paired direct/proxied windows and repeated full-matrix lifecycle churn; retain deterministic product overload/cache tests; emit bounded crash-readable reports.
6. Add short pull-request automation and a separate manual two-hour soak entry point.
7. Publish limits and interpretation guidance, update architectural records, run convergence, and execute all local gates.

## Project Structure

```text
specs/128-native-performance-envelope/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
performance/native-proxy/
├── Cargo.toml
├── Cargo.lock
└── src/
    ├── main.rs
    ├── metrics.rs
    └── workloads.rs
performance/native-proxy-reference-v1.json
performance/native-proxy-soak-v1.json
performance/native-proxy-budgets-v1.json
docs/security/deep-capture-performance.md
crates/fragcap-proxy/src/
├── application.rs
├── model.rs
└── runtime.rs
xtask/src/
├── main.rs
└── performance.rs
.github/workflows/performance.yml
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
AGENTS.md
changelog.d/
```

**Structure Decision**: Keep measurement clients, origins, process sampling, and report emission in a non-published isolated workspace. Add only truthful product-owned gauges to `fragcap-proxy`, and keep repository contract enforcement in `xtask`. This avoids making benchmark machinery part of the shipped API or duplicating proxy behavior.

## Complexity Tracking

No constitution violation requires an exception.
