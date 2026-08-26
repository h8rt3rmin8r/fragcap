# Research: Doctor Single Enumeration

## Issue Evidence

Issue #203 identifies `crates/fragcap-cli/src/doctor/probe.rs:live_probe` as calling `fragcap::enumerate()` and then `fragcap::detect_driver()`. Both paths enumerate live capture devices through the npcap device-list API. Doctor only uses `detect_driver()` for loopback support, and the issue notes that the first inventory already contains the fields needed for that verdict.

## Prior Slice Context

S079 added interactive progress and hidden timings for `fragcap doctor` and explicitly deferred #203. Its clarification record says #203 should remain a separate follow-up optimization. Its report-surface contract is load-bearing for S080: progress and timings are stderr-only interactive surfaces, while final human and JSON reports remain unchanged.

## Constraints

- Do not fabricate `Some(false)` after failed enumeration.
- Do not remove `detect_driver()` for other callers.
- Do not change doctor human or JSON report contracts.
- Do not change live capture acquisition behavior.
- Do not add dependencies.

## Open Research Questions

No open research question blocks implementation. Code inspection will decide whether the shared loopback predicate belongs in the live-capture crate or inside the doctor probe path.
