# Implementation Plan: S169 packet-flow acquisition and correlation

**Branch**: `codex/s169-packet-flow-acquisition-and-correlation` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Close #468 by correcting demonstrated loopback acquisition and owner-selection defects, preserving conservative native correlation. Research found explicit-interface loopback omission, inconsistent Npcap description recognition, absent local addresses preventing flow identity, and dynamic filters excluding new client-to-listener tuples. Canonical loopback endpoint order can also resolve the proxy server instead of the bound client. These are demonstrated source defects; no machine-specific root cause is claimed.

## Technical Context

**Language/Version**: Rust workspace, MSRV 1.88.

**Primary Dependencies**: Existing capture, attribution, profile, native proxy and target store; no new package.

**Storage**: Existing bounded FlowRegistry, process trace, application JSONL, compatibility rows and manifests. No storage/ACL migration.

**Testing**: Pure inventory/filter tests, declared socket tables, finite real loopback sockets with synthesized frames, shared pipeline/native join regressions and full `cargo xtask ci`.

**Target Platform**: Windows product; controlled tests on Linux and Windows without driver, game, elevation or trust mutation.

**Project Type**: Rust workspace library, facade and CLI.

**Performance Goals**: Existing finite bounds, at most two exact TCP loopback ownership lookups, bounded filters and unchanged refresh cadence.

**Constraints**: Preserve public struct literals using additive builders/private fields. Exact authorized listener only, existing user-space target/acquisition gate, no inferred owners.

**Scale/Scope**: #468 acquisition through terminal correlation. Separate storage and dependency work remains deferred.

## Constitution Check

P-1 preserves passive acquisition and explicit proxy authority. P-2/P-3 retain platform-neutral core and separate socket-table attribution. P-4/P-9 preserve gate/history accounting and exact observed ownership. P-5 keeps canonical flow IDs and analyzer format. P-6/P-8 require existing glossary terms and house standards. P-7 leaves wrappers thin. P-10 reuses target bindings. P-11 updates source behavior without changing published baseline. No violation or exception.

## Project Structure

- `crates/fragcap-core/src/filter.rs`, `pipeline/mod.rs`: additive per-handle fixed endpoint configuration.
- `crates/fragcap-cli/src/assemble.rs`, `orchestrator.rs`, `commands/capture.rs`, `commands/deep_capture.rs`: mandatory loopback selection, exact listener locality/filter handoff.
- `crates/fragcap/src/session.rs`: both-orientation stage-bound TCP loopback ownership and unit regressions.
- `crates/fragcap/tests/loopback_correlation.rs`, `crates/fragcap/src/deep_capture/native.rs`: controlled real endpoint identities, pipeline/process trace and final native join.
- `crates/fragcap-cli/src/commands/calibrate/assessment.rs`, `session_ux.rs`: distinct evidence diagnosis and supported Doctor inspection action.
- Slice artifacts, master specification, site troubleshooting, plan index and changelog fragments: acceptance and public behavior.

## Implementation Sequence

1. Specify, clarify and checklist; dispatch independent acquisition/correlation research under Spec-Kit plan Phase 0.
2. Resolve research, design data model/contracts/quickstart, generate tasks and pass read-only consistency analysis.
3. Parallel implementation is explicitly directed for non-overlapping [P] tasks: acquisition agent owns filter/pipeline and CLI assembly/orchestrator/capture/deep_capture configuration; correlation agent owns role stamping and controlled loopback integration; native evidence agent owns native.rs join and artifact/fact regressions. Root owns CLI diagnosis and documentation. Agree signatures first; do not edit another owner's file concurrently.
4. Establish failing regressions before owning-layer corrections. Preserve ordinary Capture semantics and calibration authority.
5. Reconcile both families, both port orders, early/late traffic, filter refresh, refusals, history bounds, packet/process/application/fact/manifest outcomes and every original criterion.
6. Run focused and full gates through verified hidden foreground execution; inspect diff/text hygiene and commit only S169 files.
7. Push/publish under explicit owner authorization, resolve every review and exact-head CI failure, request at most one second review round, hand off for human merge.

## Design Decisions

Deep Capture requires loopback even with explicit physical interfaces. Ordinary Capture precedence stays unchanged. Reuse flag-or-description recognition. Supply the exact listener IP as local only on a recognized loopback source. Do not loosen global NoLocalEndpoint parsing. Preserve the exact immutable TCP listener in each loopback filter alongside dynamic attributed endpoints, keeping physical filters and user-space gates unchanged.

Resolve both exact TCP loopback orientations in the facade with one binding snapshot. Select a single distinct bound owner, deduplicating consistent same-PID observations and preserving weaker retained fidelity. Two different bound owners or conflicting process identity remain unresolved under optional attribution. Preserve ordinary unrelated attribution when neither is bound. Core total ranking, creation-time exclusion, retention origin and canonical keys remain unchanged. Existing conflicting timestamped owners remain native ambiguous.

Early parsed gate-rejected flows already produce a withheld summary, whereas absent local addresses produce no flow key. Preserve the distinction and existing watching retention. Windows DLT_NULL IPv4/IPv6 parsing already works. Mapped-address behavior is not demonstrated as the incident cause.

Real sockets establish endpoint identities; synthesized frames through production parser/pipeline establish controlled packet semantics. No live Npcap or real-title behavior is claimed.

## Post-Design Constitution Recheck

All acquisition configuration derives from exact session and interface authority. Visibility cannot replace ownership. Finite bounds, stable APIs, losses and human consent remain intact; no conflict remains.
