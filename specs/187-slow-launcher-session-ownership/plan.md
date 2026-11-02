# Implementation Plan: S187 Slow launcher session ownership

**Branch**: `codex/s187-slow-launcher-session-ownership` | **Date**: 2026-11-01 | **Spec**: [spec.md](spec.md)

## Summary

Close #540 and #541 together at the slow managed-launch boundary. Accept explicit longer finite launch and observation windows without clamping, expose requested and effective timing in authorization and terminal evidence, and preserve the full requested calibration span with a separate return allowance. Allow an exact cold Steam title to declare an ordered publisher chain, retain relevant descendant events, correlate proxy connections to exact process instances where evidence exists from first observation, and reconcile every route owner through bounded release and recovery.

## Technical Context

**Language/Version**: Rust workspace, MSRV 1.88.
**Primary Dependencies**: Existing workspace, Windows ETW, IP Helper socket table, native proxy, and process-object receipts; no new runtime package expected.
**Storage**: Existing target launch entries, session authorization, process trace, application JSONL, cleanup, and manifest. No target-store migration anticipated; artifact schema extensions require readers to tolerate additive fields or an explicit version bump.
**Testing**: Focused Rust unit and integration tests, controlled slow launcher and local proxy fixtures, `cargo xtask ci`, applicable isolated Windows/package gates, exact-head hosted CI.
**Target Platform**: Windows product; platform-neutral deadline, topology, and trace units where possible.
**Project Type**: CLI, session facade, managed launch, capture orchestrator, process evidence, native proxy.
**Performance Goals**: Finite five-minute explicit launch and ten-minute observation maxima; bounded event, instance, and connection evidence with counted overflow.
**Constraints**: No memory rights, image-name control, silent duration clamp, same-name attribution, unbounded release, default-path installed-field claim from scratch fixtures, live game execution, or active desktop focus.
**Scale/Scope**: #540 and #541 only. #542 protocol failure diagnosis remains separate.

## Constitution Check

P-1 permits exact preauthorized managed process control with minimal rights and prohibits target memory access. P-4 and P-9 require no silent loss or altered requested bounds and exact distinction between observed processes and controlled owners. P-5 preserves analyzer-compatible packet output. P-7 keeps CLI presentation thin and lifecycle policy in the facade. P-10 retains one stored target path. Constitution 1.8.0 governs any installed-product field test: documented default storage and actual canonical paths unless the owner agrees to a path exception. The design uses controlled fixtures for implementation acceptance and passes the pre-design gate.

## Project Structure

### Documentation

```text
specs/187-slow-launcher-session-ownership/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/session-and-process.md
  quickstart.md
  tasks.md
  checklists/
```

### Source Code

```text
crates/fragcap/src/deep_capture/model.rs
crates/fragcap/src/deep_capture/session.rs
crates/fragcap/src/deep_capture/proposal.rs
crates/fragcap/src/deep_capture/process.rs
crates/fragcap/src/deep_capture/native.rs
crates/fragcap/src/managed_launch.rs
crates/fragcap-cli/src/commands/deep_capture.rs
crates/fragcap-cli/src/commands/deep_capture/shutdown.rs
crates/fragcap-cli/src/commands/target_resolve.rs
crates/fragcap-cli/src/events.rs
crates/fragcap-cli/src/orchestrator.rs
crates/fragcap-core/src/pipeline/mod.rs
crates/fragcap-cli/tests/
crates/fragcap/tests/
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Reuse stored launch roles, exact shutdown receipts, event-limited process evidence, and native application correlation. Keep process observation and control as distinct contracts. Put deadline validation in the facade before adapter preparation; CLI maps arguments and presents requested/effective values.

## Design and Execution Order

1. Add failing deadline tests for 120-second calibration observation, 600-second ordinary observation, 300-second explicit launch wait, over-cap no-effect refusal, and exact requested/effective JSON and terminal values. Preserve 30/60-second defaults.
2. Validate all deadline fields before preflight adapters. Remove silent clamping, raise published ceilings, and give calibration capture a separate return allowance so a requested 120-second observation is actually available. Keep release, shutdown, and cleanup independent.
3. Add failing Steam chain preparation and profile synthesis tests. Accept ordered launcher/intermediate/client entries, canonicalize every declared executable, bind path and role into immutable authorization and shutdown scope, and require ancestry under the cold created Steam root.
4. Add trace tests for all relevant descendant birth/exit events, bounded overflow, incomplete watcher evidence, and same-name unrelated processes. Extend exact-root descendant closure without turning observations into process-control receipts.
5. Add first-connection correlation tests. Use an independent exact socket/process-instance join at connection inception where possible; preserve explicit ambiguous/unavailable state when the join cannot prove one instance. Never relabel ScopeWithheld packet observations or infer final-client proof from proxy authentication.
6. Add controlled two-process slow launcher, absent/late client, helper-born-during-shutdown, cancellation, proxy continuity, artifact, and Doctor recovery checks across the production entry points. Extend isolated Windows package certification to invoke the installed binary with the controlled slow launcher under that runner account's documented default target and session storage, read back the actual canonical paths, and keep private field identity out of artifacts. Ensure every remaining descendant becomes a counted obligation and only exact preauthorized processes can be closed.
7. Update architecture of record and changelog decisions, complete tasks, run analysis and full gates, then commit S187. Push and open the official PR under the owner's explicit authorization, handle exact-head CI and at most two review rounds, and hand the green PR to the owner for final review and merge.

## Post-Design Constitution Recheck

Pass, subject to implementation and tests. Explicit bounds and separate return allowance preserve requested timing truth. Canonical stage paths expand preauthorized scope only through the existing exact identity checks. Descendant trace inclusion and socket correlation add evidence without process-control authority. Incomplete or ambiguous evidence remains a refusal or explicit limitation, never a guessed attribution.
