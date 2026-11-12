# Implementation Plan: Installed Direct Route Acceptance

**Branch**: codex/s205-installed-route-acceptance | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

**Input**: Owner-authorized S205 completion of #593, #594, and #595 after merged S204. Push and official PR are authorized. At most one explicit second Codex review request may be made; final merge remains with the owner.

## Summary

Make the existing direct-route implementation earn its installed-product claim through one cold controlled raw-client journey from candidate package bytes. Bind client-visible external exchange, actual passive Npcap packet ordering, calibration fact applicability, and native debug-report authority. Add exact regressions and fix demonstrated disagreements, while leaving v0.11.0 installed state and native evidence intact after comparison.

## Technical Context

**Language/Version**: Rust, repository MSRV 1.89; PowerShell 7 for existing Windows package harness.

**Primary Dependencies**: Existing Fragcap workspace, installed Npcap, verified signed WinDivert component, native proxy and route stream, Windows Installer, no new dependency planned.

**Storage**: Existing target store, generated session registry and bundles, protected route.jsonl and manifest; candidate package harness roots remain distinct from default installed native paths.

**Testing**: Driver-free Rust route, correlation, CLI and artifact fixtures; ignored signed-driver Windows matrix; controlled non-loopback cloudbase peer; candidate MSI and ZIP package certification; full cargo xtask ci and hosted CI.

**Target Platform**: Windows x64 for installed route and physical driver order; platform-neutral report and decision fixtures.

**Project Type**: Rust workspace library and CLI with Windows packaging.

**Performance Goals**: Existing paired unrelated-sentinel p99 bound, zero missing sentinel replies and unexplained post-receive loss, bounded selected-route event writer with no broad-path synchronous file or JSON work.

**Constraints**: Exact pre-effect authorization, cold causal client proof, no target instrumentation, no unrelated payload retention, finite queue and cleanup, no permanent cloudbase state, exact installed path and restoration readback.

**Scale/Scope**: One cold controlled selected client per route case, concurrent unrelated sentinels, finite route and report fixtures.

## Constitution Check

**Pre-design gate**: P-1 permits only explicit scoped signed routing and requires exact client/destination proof, unrelated continuity, finite queue, watchdog, and cleanup. P-4 requires surfaced loss. P-9 separates observed source from inference. P-11 binds released claims to published v0.11.0 while this feature remains unreleased. Installed-product rules require native defaults and actual canonical path inventory for a default-path claim. No target memory access, process instrumentation, global proxy change, Npcap redistribution, or new dependency is planned.

**Post-design gate**: The controlled client is an ordinary external process launched through the public target path, with no target inspection or memory rights. Package certification remains its own evidence tier. Physical Npcap output, route receipts, proxy transfer, calibration decisions, and report sources keep independent authority. Candidate MSI operations are conditional on exact published restoration bytes and clean Doctor state; otherwise the local installed journey cannot be claimed. Source-preserving report fixtures use existing protected artifact APIs. The design satisfies P-1, P-4, P-9, and P-11.

## Project Structure

The spec-kit artifacts live under specs/205-installed-route-acceptance/. The controlled client source and route matrix belong in integration/. Driver and correlation tests and any demonstrated fixes belong in crates/fragcap/src/deep_capture/ and crates/fragcap/tests/. Public candidate package tests belong in crates/fragcap-cli/tests/ and the existing Windows integration and package workflows when needed. Report and Doctor acceptance belong in crates/fragcap-cli/src/commands/, crates/fragcap-cli/src/doctor/, and their tests. The chronological slice index and master specification record only delivered behavior; a changelog fragment records user-visible changes.

## Implementation Sequence

1. Inventory the exact S203/S204 coverage and current issue criteria. Verify current installed v0.11.0 identity, native paths, active recovery, official restoration source, package features, and controlled-client launch capability without changing native state.
2. Add failing driver-free tests for remaining correlation negatives, report source/manifest/Doctor/access/summary gaps, and source preservation. Implement only demonstrated corrections.
3. Add a no-console controlled raw-TCP client and an ignored installed candidate acceptance harness that uses the public target and calibration path, exact peer bytes, active sentinels, the required native route stream, and finite cleanup. Run the same transport assertion through portable and installed candidate bytes; keep library diagnostics-on/off probes as a separate signed-driver tier. Do not count a redirected harness as default native placement.
4. Run controlled published v0.11.0 comparison and candidate external IPv4/IPv6 acceptance. Observe passive Npcap original-present and original-absent cases, or retain an explicit failed qualification rather than fabricating one. Reconcile compatibility, verdict, store, and later admission for TCP and supported UDP, including refusal and no-payload cases.
5. Reconcile protected route/report artifacts and actual installed default paths. Update documentation and dated fragments. Run foreground CI parity, signed-driver and package gates, exact-head hosted CI, and all review closeout. Close only issues with complete acceptance; hand the PR to the owner without merging.

## Decisions

- **One acceptance spine**: The controlled client and peer create selected route, passive packet, calibration, and report facts in one causally linked run, reducing inconsistent fixture-only claims.
- **Preserve existing installation**: Candidate MSI testing is contingent on exact published restoration bytes and native-state preflight; this is a technical prerequisite, not an unrelated approval gate.
- **No synthetic physical claim**: Driver-free fixtures can validate decision logic but cannot certify an observed Npcap packet order.
- **No standing field gate**: Once an issue's repository-controlled behavior passes, later private-title or published-release failures become new concrete defects under the constitution.
- **Pinned artifact decision**: If scripts or workflows change, add a dated changelog.d decision fragment; do not edit CHANGELOG.md on this feature branch.

## Complexity Tracking

No constitution violation or new architectural layer is planned. An installed candidate run may be refused by exact restoration or driver-order preflight; it cannot be replaced by a weaker success claim.
