# Implementation Plan: Direct Route Acceptance

**Branch**: `codex/s204-direct-route-acceptance` | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Owner-authorized S204 closure of the remaining controlled criteria for #593, #594, #595, and #598 after merged S203. The owner authorized push and official PR, allows at most one explicit second Codex review round, and reserves final merge.

## Summary

Audit the S203 implementation against each issue's acceptance contract, add precise failing regressions for gaps, run controlled signed-route and packaged entry-point tests, fix demonstrated failures, and reconcile native report, scope help, and issue status. Keep direct-route reachability separate from passive packet availability.

## Technical Context

**Language/Version**: Rust MSRV 1.89; repository-pinned Rust toolchain.
**Primary Dependencies**: Existing signed WinDivert, Npcap, private TCP bridge, native SOCKS proxy, serde_json, and bounded lifecycle writer; no new dependency planned.
**Storage**: Existing protected native bundle, route.jsonl, manifest, target store, and derived debug report; no migration planned.
**Testing**: Driver-free Rust regressions; ignored disposable Windows signed-driver probes; controlled non-loopback cloudbase peer; Windows native integration and MSI/ZIP package certification; full repository gate.
**Target Platform**: Windows x64 live route, platform-neutral report fixtures.
**Project Type**: Rust workspace library and CLI.
**Performance Goals**: Existing paired sentinel p99 bound, zero unexplained missing replies or post-receive packet loss; no synchronous broad packet-path file or JSON work.
**Constraints**: Exact owner and destination proof, finite work and output, no remote persistence, no unrelated payload retention, protected native data, source-preserving report.
**Scale/Scope**: One controlled cold selected client per test case, concurrent unrelated sentinels, finite selected flows.

## Constitution Check

**Pre-design**: P-1 requires deliberate selected routing, exact client proof, unrelated continuity, bounded failure, and cleanup. P-4 requires explicit loss. P-9 separates observation from inference. P-11 requires truthful shipped claims. The installed-product field-test rule requires default native placement and exact path inventory if an installed journey is claimed. Text, access, dependency, and pinned-artifact rules remain binding.

**Post-design**: Tests and any fixes preserve the route receipt, client-delivery, and passive packet authorities as distinct facts. Controlled package tests may use their harness directories; they do not establish default installed storage. A default installed journey must use documented native defaults. No target instrumentation, global route, new dependency, or permanent cloudbase service is planned.

## Project Structure

Slice artifacts live in `specs/204-direct-route-acceptance/`. Route tests and fixes belong in `crates/fragcap/src/deep_capture/` and `crates/fragcap/tests/`. Calibration and report CLI tests and fixes belong in `crates/fragcap-cli/src/` and `crates/fragcap-cli/tests/`. Matrix mapping belongs in `integration/direct-route-matrix-v1.json` and the existing Windows workflows if required. Active docs, `docs/plans/README.md`, and an unreleased changelog fragment record delivered behavior.

## Implementation Sequence

1. Inventory S203 route, correlation, report, help, and test coverage against four issue criteria. Add focused failing fixtures before changing application code.
2. Qualify signed selected TCP against controlled non-loopback IPv4 and IPv6 peers with exact two-way bytes, active unrelated sentinels, diagnostics on/off, refusal, cancellation, and no residual listener. Use another owned controlled peer if cloudbase lacks IPv6; never count an unavailable family as passing. Fix only reproduced faults.
3. Qualify original-present/absent and negative TCP/UDP correlation; reconcile compatibility, verdict, stored fact, and admission from the same decision. Preserve helper and passive packet distinctions.
4. Complete report/stream, manifest, Doctor, sharing, access, crash-prefix, source immutability, and human/JSON fixture coverage. Run candidate MSI and ZIP command help through the package tier and compare active docs. Preserve the existing v0.11.0 help as historical evidence, not a passing candidate.
5. Run the full local and hosted Windows/package gates, resolve all review findings, and hand a green PR to the owner without merging.

## Decisions

- **Acceptance before closure**: S203 implementation is a baseline, not proof that every issue criterion passed. Close an issue only with its own controlled evidence; a later active-release failure becomes a new concrete defect.
- **Live peer**: Use cloudbase only as an ephemeral unprivileged origin and verify its exit. Avoid remote files and services so sibling workloads are unaffected.
- **Package authority**: Distinguish a staged binary or certified archive from a default installed journey. Do not claim default storage, retention, or cleanup from a custom-path harness.
- **No speculative bridge change**: The prior suspected reply and checksum boundaries are already modified. Change them again only if a failing external regression identifies a remaining defect.
