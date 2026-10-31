# Implementation Plan: S184 Deep Capture session lifecycle

**Branch**: `codex/s184-deep-capture-session-lifecycle` | **Date**: 2026-10-31 | **Spec**: [spec.md](spec.md)

## Summary

Repair the v0.10.9 Deep Capture observation and shutdown path in one reviewable slice for #526 and #527. Release proven session owners despite a separate unproven declared-image event, preserve truthful route and trust obligations, and honor the ordinary requested duration through the plan, facade, Capture adapter, and terminal record.

## Technical Context

**Language/Version**: Rust workspace, Windows production adapters, MSRV 1.88.
**Dependencies**: Existing crates and Windows APIs only. No new product dependency or process access right.
**Storage**: Existing versioned session, cleanup, resource-journal, Doctor, and artifact authorities. Any new field or status must preserve reader compatibility and exact recovery.
**Verification**: Spec-Kit analysis, focused controlled regressions, `cargo xtask ci`, applicable Windows and package checks, installed controlled duration path, exact-head PR CI, and bounded external review.

## Constitution Check

P-1 requires exact proof before process control and prohibits memory access. Proven owners and unresolved observations are kept separate. Deep Capture remains explicit and target scoped. Retained trust after unresolved release is an auditable, recoverable partial obligation, not an unannounced new trust change. P-4 and P-10 require truthful loss and cleanup records. The full Spec-Kit sequence and foreground gates apply. Pre-design check passes.

## Structure

- `crates/fragcap-cli/src/commands/deep_capture/shutdown.rs`: exact owner selection, controlled Windows regressions.
- `crates/fragcap-cli/src/commands/deep_capture.rs`: mode-specific timing, production Capture adapter, plan and terminal presentation, trust manager boundary.
- `crates/fragcap/src/deep_capture/session.rs` and `routing.rs`: deadline authority and cleanup reconciliation.
- `crates/fragcap-cli/src/doctor/`: recoverable incomplete route/trust presentation as needed.
- `specs/184-deep-capture-session-lifecycle/`, `docs/fragcap-specification.md`, affected CLI documentation, `docs/plans/README.md`, and a `changelog.d/` fragment: traceability and operator contract.

## Design

1. Produce a typed owner-selection result with a proven set and unresolved observations. Treat only global watcher/receipt authority failures as whole-selection refusals. Keep per-process ownership checks strict and do not control the unresolved set.
2. Release proven owners within the existing finite grace and force windows. Continue observing late events and helpers. A successful route-owner result requires verified exit of every plausible routed owner; uncertainty is reported separately from proven-owner exit success.
3. Carry the route-owner result into route/target-route cleanup reporting. A local route-lease cleanup may succeed without implying inherited child routing ended. Retain an exact recoverable trust obligation when plausible owners remain live and state proxy continuity loss at the finite shutdown boundary.
4. Select deadlines by mode before effects. Calibration retains its shorter cap; ordinary Deep Capture accepts positive durations through the existing five-minute ceiling and rejects higher or zero values. Preserve the full accepted Capture duration with a separately accounted finite return allowance. Ensure authorization and terminal output show the exact effective observation and allowance.
5. Use hidden controlled process tests and one installed controlled run beyond one minute. Compare the same plan identity, actual arm-to-stop timing, packet span, route release, trust, and terminal states.

## Implementation Sequence

1. Specify, clarify, checklist, plan, tasks, and blocking analysis on literal S184 identity.
2. Write focused failing regressions for #526 and #527, then implement owner selection and deadline authority.
3. Reconcile route and trust cleanup, Doctor, terminal, and documentation; complete all tasks.
4. Run the full gate set in the foreground, commit the exact slice, push, and open an official PR.
5. Watch exact-head CI and external reviews, resolve every actionable comment, request at most one second review round if useful, then hand the reviewed head to the owner before merge.

## Post-Design Constitution Recheck

Pass. The design uses only existing permitted process rights, changes no Capture mode process control, and makes unresolved resource obligations explicit. Permanent installation trust remains #529.
