# Implementation Plan: Managed Launch Observation Continuity

**Slice**: S189

**Spec**: [spec.md](spec.md)
**Date**: 2026-11-04

## Sequence

1. Use a managed Deep Capture policy distinct from ordinary Capture. Bind and retire exact process instances, keep a proven handoff provisional, and allow only a causally owned exact successor to reuse the terminal role.
2. Publish process lifetime intervals to the packet role stamper. Keep the original ETW event and S188's separately verified path evidence. A historical PID alone cannot stamp later traffic.
3. Start the managed observation clock at terminal client acquisition. Translate wall-clock ETW events onto the monotonic session clock for phase decisions while leaving packet and process evidence timestamps intact. Do not open the managed packet window for an intermediate launcher.
4. Resolve simple or multi-stage defaults after target preparation and before authorization. The facade and CLI reserve acquisition, observation, capture preparation, return, owner release, proxy shutdown, and cleanup time. Refuse a remaining runtime budget that cannot provide the authorized intervals.
5. Present the effective bounds and origins in the canonical authorization plan, human summary, terminal result, manifest, and command help. Update the architecture of record and changelog for the behavior delivered.
6. Commit the S189 change, push the authorized branch, open an official pull request, and handle the external review conversation. Release and later installed-product observation are separate owner activities; a later observed defect returns as a GitHub issue.

## Code ownership

`CaptureSession` owns binding, provisional handoff, and observation timing. `RoleStampingAttributor` owns packet-time role validity. The CLI orchestrator connects live events, packet gating, and process evidence. The Deep Capture facade and CLI own phase budgets and presentation. No new dependency or storage migration is planned.
