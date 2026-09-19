# Reliability Checklist: S158 Windows gate determinism

**Purpose**: Protect the two post-S157 failure boundaries during design and implementation\
**Created**: 2026-09-19

## Application Evidence

- [x] Consumer readiness precedes sink publication
- [x] Queue capacity and nonblocking forwarding remain unchanged
- [x] Ordered output and all loss classes remain exact
- [x] Startup failure has finite ownership and no orphan worker
- [x] Deterministic injection covers the historical race class

## Session Lifecycle

- [x] Capture, shutdown and drain authorities are distinct
- [x] Complete structured evidence outranks a later irrelevant clock sample
- [x] Incomplete and timed-out work still fails specifically
- [x] Cancellation and cleanup remain independently visible
- [x] Scripted clocks cover every deadline boundary

## Test Isolation

- [x] Poison recovery does not hide the original assertion
- [x] Environment restoration is unwind-safe
- [x] A prior environment value is preserved exactly
- [x] The original Windows case and later tests remain independently reportable

## Acceptance

- [x] Hosted first-attempt green evidence is required
- [x] A rerun remains diagnostic only
- [x] No local installed-product or real-game execution is permitted
