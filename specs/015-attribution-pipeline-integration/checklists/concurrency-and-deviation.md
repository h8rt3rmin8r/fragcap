# Domain Checklist: Lock-Free Refresh and Trait Deviation

**Purpose**: Guard the two failure-prone surfaces of this slice - the lock-free
resolve path under a `&self` refresh, and the completeness of the
architecture-of-record trait deviation. Checked at `analyze` and again at
pre-push verification.
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Lock-free resolve (section 11.6)

- [x] The per-packet `resolve` read path acquires no lock introduced by this
      slice; the interior mutability added for `refresh(&self)` lives on the
      refresh (control-thread) side only.
- [x] A concurrency test exercises `resolve` on one thread across a `refresh`
      publication on another, asserting resolve never blocks (SC-003, FR-002).
- [x] The retention map's window origin is unchanged: retention measured from
      last-observed-present, not from the refresh that noticed absence (P-9 edge
      case).
- [x] A no-op refresh (no change) does not churn the endpoint set or the filter.

## Trait deviation completeness (section 29)

- [x] `FlowAttributor::refresh` signature is `&self` and the trait stays
      dyn-compatible and `Send + Sync` (existing compile-time assertions pass).
- [x] Every implementor updated: `SocketTableAttributor`, `PublishedResolver`,
      `ScriptedAttributor`, role-stamping attributor.
- [x] Every test double updated: `StubAttributor` (traits.rs and pipeline),
      `Fixed`, `PanicOnEndpoints`.
- [x] The deviation is recorded as a dated decision fragment and promoted to
      specification section 29 (and sections 11, 12.2 for the two resolutions).

## Narrowing restriction correctness (section 12.2)

- [x] The narrowing input is restricted to endpoints owned by profiled PIDs
      before `FilterManager::poll` compiles the program (FR-005).
- [x] The owning identifier survives the endpoint enumeration far enough to
      join; UDP joins by owning module (S10 rule), not by an address it lacks
      (FR-006).
- [x] An unprofiled process's endpoints in the same socket table are excluded
      from the compiled program (SC-002).
- [x] The "filter narrowed to N endpoints" message counts only profiled
      endpoints, in both offline and live command paths (FR-007).

## No regressions

- [x] No new discard class; existing named counters keep their meanings and the
      conservation invariant holds (FR-009, P-4).
- [x] No platform dependency enters `fragcap-core` (P-2); no process handle is
      opened (P-1); no new runtime dependency.
- [x] Offline goldens are byte-identical after `RefreshDriver` removal (offline
      drives no live refresh).
