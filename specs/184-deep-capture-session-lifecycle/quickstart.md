# S184 validation quickstart

1. Verify branch `codex/s184-deep-capture-session-lifecycle` and `.specify/feature.json` selecting `specs/184-deep-capture-session-lifecycle`.
2. Run the focused Windows owner-release test with hidden controlled children. Inspect exact process identities, control requests, exit verification, unresolved event, and cleanup resource statuses.
3. Run a controlled installed ordinary Deep Capture for more than one minute using exact plan authorization. Compare plan observation, wall-clock arm-to-stop, captured frame span, terminal deadline, and cleanup. Run calibration separately to confirm its 60-second ceiling.
4. Run Spec-Kit analysis, `cargo xtask ci`, applicable Windows/package gates, and exact-head PR checks. Record any skipped platform or physical evidence as unverified.
5. Resolve review threads, record review-round count, and hand the final SHA and remaining limits to the owner before merge.
