# Implementation Plan: Session Causal Diagnostics

**Branch**: `codex/s190-session-causal-diagnostics` | **Date**: 2026-11-04 | **Spec**: [spec.md](spec.md)

## Summary

Extend the existing native connection diagnostic with bounded stage, failure-boundary, and timing facts for #551, then finish #553 by projecting actual elapsed and unused authorized budget for every lifecycle phase into terminal output and the manifest. Both changes use the session's existing identity, evidence, and finite-budget authorities.

## Technical Context

**Language/Version**: Rust, workspace toolchain in `rust-toolchain.toml`.
**Primary Dependencies**: Existing Tokio, rustls, serde_json, and Windows process/socket evidence; no new crate planned.
**Storage**: Additive versioned fields in retained `proxy.jsonl`, calibration diagnostics, terminal output, and manifest v2; no database migration.
**Testing**: Focused controlled Rust regressions, then repository formatting, lint, test, documentation, and workflow checks.
**Target Platform**: Windows product with platform-independent controlled proxy and lifecycle checks.
**Project Type**: Rust workspace with native proxy, facade, and CLI.
**Performance and scope**: Per-connection diagnostics remain bounded by current connection-detail retention; no per-packet work or unbounded phase history.

## Constitution Check

- P-1: The feature observes existing authorized proxy and lifecycle work; no target-process access or trust change is added.
- P-2/P-3: Platform-specific process evidence stays in the facade; native proxy retains protocol observations without claiming a PID from the credential.
- P-4/P-9: Unknown and dropped detail are explicit, and observed stage or phase facts are not silently synthesized.
- P-5/P-7/P-8: Packet format and shell wrappers remain unchanged; text and Rust follow repository conventions.
- P-11: Update the architecture of record and slice ordering for exactly the delivered contract.

## Project Structure

```text
specs/190-session-causal-diagnostics/
  spec.md
  checklists/requirements.md
  research.md
  plan.md
  data-model.md
  contracts/session-diagnostics.md
  quickstart.md
  tasks.md
crates/fragcap-proxy/src/
  model.rs
  runtime.rs
  http1.rs
  tls.rs
  upstream.rs
crates/fragcap/src/deep_capture/
  adapters.rs
  api.rs
  diagnostics.rs
  application.rs
  lifecycle.rs
  model.rs
  native.rs
  session.rs
crates/fragcap-cli/src/
  commands/calibrate/assessment.rs
  commands/deep_capture.rs
  events.rs
  orchestrator.rs
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/S190-session-causal-diagnostics.fixed.md
```

## Sequence

1. Preserve connection identity and implement bounded native stage and failure facts at the actual protocol boundaries. Keep client TLS completion if upstream TLS fails and classify post-response header timeout separately.
2. Project diagnostic fields through the facade, application correlation, proxy lifecycle, retained bundle, and calibration/terminal JSON with exact owner relation or unavailable/ambiguous status. Process ownership is bound after process collection; the proxy runtime reports no guessed PID.
3. Record each lifecycle phase's monotonic start and end at its owner, and project actual elapsed and unused authorized budget into terminal output and manifest. Keep authorization plans unchanged.
4. Update specification, documentation, and changelog; run the repository's automated checks; publish the S190 branch and official PR; address permitted review rounds and CI before owner merge.

## Complexity Tracking

No constitutional exception or new dependency is planned.

## Spec-Kit Analysis

All eight functional requirements map to T003-T013 and both user stories have independent controlled outcomes. The analysis found that proxy lifecycle detail and application correlation have different owners, and that terminal event serialization is in `events.rs`; the structure and tasks name those owners. Implementation analysis also caught a version-one compatibility break from adding a required field to the public `TerminalSnapshot` struct. The completed design carries timing through an additive lifecycle event and a defaulted artifact-sink method, preserving existing struct literals. Existing bounded records and named loss counters are retained.
