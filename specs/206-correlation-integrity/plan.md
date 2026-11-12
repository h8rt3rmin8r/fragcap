# Implementation Plan: Packet Correlation Integrity

**Branch**: codex/s206-correlation-integrity | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

## Summary

Reconcile the direct-route and launch-chain ownership evidence around one exact flow and process-generation contract. Prove or preserve uncertainty for both reported failure classes. Use controlled source tests and the existing disposable hosted Windows matrix; keep #594 and #596 independently closable.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.89; existing PowerShell 7 hosted package scripts.

**Dependencies**: Existing fragcap-core flow registry, facade session gate, native correlation, process trace, calibration and target store. No new package is planned.

**Storage**: Existing protected native bundle and default target store. Reuse current artifact versions unless a demonstrated contract defect requires a versioned additive change.

**Platforms**: Driver-free tests on all supported build hosts; signed WinDivert and Npcap matrix on disposable Windows; installed package bytes only for a claimed physical driver-order result.

**Checks**: Focused regression, `cargo xtask ci`, Windows integration, final package certification where package behavior changes, exact-head hosted checks, bounded review closeout.

**Performance and privacy**: No broad packet-path synchronous JSON or filesystem work; finite pending proof; existing sentinel latency and zero-loss bound; no unrelated payload retention or target instrumentation.

## Constitution Check

**Pre-design**: P-1 permits explicit exact-client direct routing and requires unrelated continuity. P-4 requires observed packet conservation and named loss. P-9 forbids promoting inference to packet truth. Installed-product rules require defaults unless exact paths were approved; this slice uses disposable hosted package paths for new physical tests. No constitution change is needed.

**Post-design**: Keep route receipt, passive packet, and causal process evidence independent. Preserve exact positive fact admission and no-effect refusals. Unknown packet delivery or process ancestry cannot become positive. No private target or new trusted material is required.

## Project Structure

- `crates/fragcap/src/session.rs`: causal gate, pending proof, and rejection accounting.
- `crates/fragcap/src/deep_capture/native.rs`: per-flow proxy window, route receipt, packet omission, and process-generation reconciliation.
- `crates/fragcap-core/src/flow.rs`: bounded per-flow observation and omission history only if a demonstrated defect requires it.
- `crates/fragcap-cli/src/commands/calibrate/` and tests: verdict/store/admission agreement.
- `integration/` and hosted Windows workflow: controlled chain and physical order matrix.
- `specs/206-correlation-integrity/`, `docs/fragcap-specification.md`, `docs/plans/README.md`, `changelog.d/`: contracts, evidence, and dated decisions.

## Sequence

1. Reproduce #596 with a deterministic cold chain and per-flow proxy windows. Identify actual cause among late binding, pending timeout/cooldown, bounded history, ambiguous ancestry, and unrelated omission. Add failing regression before correction.
2. Audit #594 current route proof, positive and negative calibration, target-store persistence, and later admission. Add missing independent tests. Use a packaged hosted path to seek original-present and original-absent ordering without synthetic substitution.
3. Fix only demonstrated defects, retain exact source provenance, and check conservation and output parity.
4. Run focused, CI-parity, driver, and package checks. Record exact physical outcomes, update the master and chronological plan only for delivered behavior, commit, push, open PR, and resolve no more than two review rounds.

## Decisions

- **Independent gates**: #596 closes from a complete controlled launch-chain fix even if #594 lacks an observed physical ordering. This prevents an environmental condition from stalling a separate defect.
- **No speculative timeout increase**: Pending proof waits or cooldowns change only after a failing timing test identifies them as the cause. Longer waits could stall the packet writer and harm unrelated traffic.
- **No global-loss promotion**: Capture-wide unlocated loss remains a declared uncertainty; exact flow-local omissions retain their own interval and cause. Do not invent a per-flow packet from a global counter.
- **Additive process-trace event**: The current version-one process trace permits typed records and already reports aggregate `packet-evidence-unretained`. Add bounded `packet.omission` records using the registry's existing flow-local interval and stable cause, while retaining the aggregate limitation and old-bundle readback. No schema migration or new packet authority is required.
- **One hosted physical batch first**: Use the existing disposable Windows matrix and package artifacts. Local installed defaults are claimed only after actual local installed readback under the constitution.

## Complexity Tracking

No new architecture or dependency is planned. A physical order that cannot be induced is recorded as unverified, not replaced by a fixture.
