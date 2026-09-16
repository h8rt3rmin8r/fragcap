# Implementation Plan: S154 regression reliability and native documentation readiness

**Branch**: `codex/s154-regression-documentation` | **Date**: 2026-09-16 UTC | **Spec**: [spec.md](spec.md)

## Summary

Resolve #420 with explicitly armed and acknowledged test-only writer backpressure, twenty fresh isolation scenarios and exact offered = written + dropped accounting. Retain real TCP coverage as a healthy accepting consumer with an independent full file sink. Complete documentation engineering child #421 under open parent #331 using a closed coverage registry, confined authored-page references and existing non-ignored executable test authorities.

## Technical Context

**Language/Version**: Existing Rust workspace (MSRV 1.88), MDX/Next.js site and pinned Node/pnpm tools.\
**Primary Dependencies**: Existing std, tempfile, serde_json and repository test-source validator only; no new dependency.\
**Storage**: Test temporary pcapng files and versioned documentation inventory, no product storage change.\
**Testing**: Existing Linux/Windows cargo gates, focused streaming tests, xtask unit contracts, CLI reference default/net, schema readers, production site unit/accessibility checks.\
**Target Platform**: Existing Linux and Windows CI.\
**Project Type**: Windows observability library/CLI and static documentation site.\
**Performance Goals**: Existing five-second capture submission bound; registration and stall acknowledgement within five seconds.\
**Constraints**: No installed sensitive software, game, real trust mutation, product execution, runtime/dependency/version/release-policy change.\
**Scale/Scope**: Two sink integration files; one small documentation validator and registry; bounded existing-page corrections, slice and changelog records.

## Constitution Check

Pre-research PASS: P-1 explicit scope and host prohibition, P-3 no dependency edge, P-4/P-9 exact independent sink loss, P-8 UTF-8 without BOM and LF, P-11 truthful published v0.10.1 identity. Existing owner bypass remains untouched. #331/#333/#413/#334/#278 are not completed by engineering readiness.

Post-design PASS uses the same limits. No deviations or complexity exceptions are needed. Public product APIs and persistence remain unchanged.

## Phase 0: Research

The planning skill requires independent read-only research. The documentation agent audits current page-to-authority mappings while the primary agent inspects StreamSink and the existing stalled-writer seam. Consolidated decisions and alternatives are recorded in [research.md](research.md).

## Phase 1: Design

The fake writer accepts the header while its shared control is unarmed. After consumer registration, arm it, submit the first packet and wait for positive blocked-write acknowledgement before queue saturation. No magic header-byte budget or TCP capacity assumption remains. Controlled release drains exactly the in-flight packet and four accepted slots: 100 offered = 5 written + 95 refused. This specifically proves queue loss rather than only terminal discards. Existing timeout coverage separately checks the unwritten tail. A reading TCP client verifies full healthy stream and full file output.

A schema-1 native documentation inventory has eleven closed topics. Each row names a current site MDX page and precise Rust test authorities with explicit features. The small validator rejects unknown/duplicate/missing topics, unsafe/non-current paths, absent pages and missing/ignored test functions using the existing test-source parser. A narrow wrapper reuses review-handoff Cargo ownership and actual harness discovery, including enclosing configuration and invalid package feature refusal; supported integration targets avoid inferred nested module ownership. The existing docs check runs it before its linter and CLI parser checks. Its Windows child tooling uses hidden non-interactive launchers. The inventory is engineering traceability, not an independent audit verdict or a substitute for actual CI execution.

[Data model](data-model.md), [contracts](contracts/readiness.md) and [quickstart](quickstart.md) define validation boundaries.

## Project Structure

Feature records live in `specs/154-regression-documentation/`. Test changes live in `crates/fragcap-sink/tests/streaming_backpressure.rs` and `streaming_tcp.rs`. Documentation traceability lives in `docs/audits/native-documentation-coverage.v1.json` and a reader-facing site reference. The validator is `xtask/src/docs_coverage.rs`, invoked by `xtask/src/docs.rs` and ordinary unit tests.

## Execution and Verification

Generate tasks, then run the blocking read-only cross-artifact analysis before implementation. Verify focused positive/negative contracts first, then complete local checks and the production site using hidden redirected launchers. Push the slice branch and open the official PR under explicit user authorization. Reconcile every actual review and final-head hosted CI; request at most one second bot review round. Human merge is the handoff boundary.

## Complexity Tracking

No constitution violations or new product architecture.

## First-review correction

2026-09-16: The first Codex review correctly identifies that hidden dev descendants do not receive the invoking console's cancellation event. An attached-console exception would conflict with the repository's mandatory hidden-launch rule. Reuse the existing Windows integration runner's suspended child, non-inheritable kill-on-close Job Object and thread-resume helpers through narrow crate visibility instead. The dev shim cannot launch before assignment; normal shim exit closes the job, and abrupt xtask exit lets Windows close it. Assignment/resume failure kills and reaps the exact already-owned child. Finite build/check launch behavior remains unchanged. Two controlled Windows subprocess tests prove both normal shim exit and abrupt owner exit release shim and descendant listener ports within two seconds, without launching the actual site or sensitive product.

## Final evidence checkpoint

2026-09-16: Reviewed implementation db50c0159b0fe3dacfb19e1aef595b28018c4ae6 passes all 23 executed hosted checks. The second and final Codex review reports no major issues, and the original P2 thread is answered and resolved. Complete the task record at this verified implementation checkpoint, then push only these final Markdown records. Preserve the original final-head green-CI requirement: verify the evidence-only commit separately and record its exact head/check outcome on PR #422 before owner handoff. Using the external PR record for the last checkpoint avoids an endless series of self-referential evidence commits; it does not waive a check or authorize a third review or merge.
