# Implementation Plan: Direct Route Qualification

**Branch**: `codex/s201-direct-route-qualification` | **Date**: 2026-11-10 | **Spec**: [spec.md](spec.md)

**Input**: #587 reliability, #583 public CLI/calibration and package qualification, #584 operator documentation, and parent #572. The owner authorized push, official PR, CI, and up to two external review rounds, reserving final merge.

## Summary

First establish measured and exact candidate-owner behavior in the S200 route. Then connect its existing native routing adapter to one pre-effect public CLI plan and route-specific compatibility fact authority. Finally reconcile help and onboarding against the executable and package, and qualify the integrated Windows and package journey.

## Technical Context

**Language/Version**: Rust MSRV 1.89, pinned development Rust 1.96; Windows 11 x64 for live route.

**Dependencies**: Existing official signed WinDivert 2.2.2-A x64 package, Npcap for passive capture, native proxy and target store. No new driver or target instrumentation.

**Storage**: Existing append-only compatibility and native session artifacts; additive route identity/version and diagnostic fields where required.

**Testing**: Driver-free candidate tests, disposable Windows cold-chain and active-sentinel matrix, public CLI/Doctor and package certification, full `cargo xtask ci`, exact-head hosted CI and bot reviews.

**Performance**: Choose the smallest qualified candidate wait from measured selected success and unrelated continuity. Retain finite capacity and independent worker-stall fail-open behavior.

## Constitution Check

**Pre-design**: P-1 permits the signed, prearmed direct route only under explicit Deep Capture authorization and exact owner/destination proof. Unrelated packets must be promptly reinjected after header-only candidacy with no application-payload inspection or retention. P-4/P-9 require delivered-packet conservation and honest pre-receive uncertainty. P-10/P-11 require one stored target path and truthful shipped claims. The installed-product field-test rule applies to any actual installed-product acceptance run.

**Post-design**: The composite plan reuses S200's exact adapter and one authorization. Route kind/version separates new facts from legacy environment-only evidence. A candidate is never selected from missing proof. The worker watchdog, pass-through, finite bounds, and cleanup remain independent of a measured candidate wait. No constitution exception is requested.

## Project Structure

`specs/201-direct-route-qualification/` owns the spec-kit evidence. Candidate and runtime changes belong in `crates/fragcap/src/deep_capture/direct_route.rs` and `direct_runtime.rs`. Routing and fact applicability belong in the facade and target store; public selection, Doctor and result presentation belong in `crates/fragcap-cli`. Controlled matrix, package certification, embedded help, site, master specification, glossary and changelog follow their existing owners.

## Implementation Sequence

1. Add failing driver-free candidate tests for proof at deadline, proven unrelated pass-through, uncertainty, and finite limits.
2. Add controlled timing checkpoints and paired Windows chain/sentinel probes. Select the smallest qualified production wait and scale socket-event history and dependent deadlines. Update S200 FR-003 explicitly.
3. Add route-kind/version identity to compatibility applicability with additive migration or exact existing schema mechanism. Require original final-client receipt plus passive packet and proxy evidence for positive direct facts.
4. Bind CLI calibration, ordinary Deep Capture, authorization/resume, Doctor, and artifacts to one composite route plan and exact fact authority. Test mixed routes, legacy refusal, faults, cancellation and recovery.
5. Reconcile MSI/ZIP, Npcap/WinDivert/extcap and privilege documentation with actual source and package output.
6. Run analyze, full local gates, disposable Windows and package matrices, exact-head hosted CI and review closeout. Commit, push, and open the official PR. Stop before owner merge.

## Decisions

- **Candidate wait**: The #587 100 ms sweep is the starting qualification, not an approved default. Do not choose a production value until timing and sentinel gates pass. A longer arbitrary wait would hold unrelated candidates unnecessarily.
- **Ownership**: Use exact current socket owner for prompt known-unrelated pass-through; missing or ambiguous proof stays uncertain. Destination matching alone cannot establish client identity.
- **Route integration**: Reuse `RoutingPlan.direct_socket` and `NativeDirectRouting` rather than creating a second effect coordinator.
- **Fact migration**: Preserve append-only historical facts and require a new exact route identity for direct-route admission; old positive facts do not silently qualify.
- **Validation**: Repository-controlled synthetic and package gates determine issue closure. Later private-title failures are new concrete defects, not standing gates.
