# Implementation Plan: S173 Bundle access stabilization

**Branch**: `codex/s173-bundle-access-stabilization` | **Date**: 2026-10-19 | **Spec**: [spec.md](spec.md)

## Summary

Deliver #476 and #477 together: automatically retain genuine ordinary same-account output authority from elevated desktop sessions, retire historical repair, and preserve maintained bundle operations. The production fix uses the exact session desktop shell token rather than promoting an identification-only linked token. See [research.md](research.md).

## Technical Context

**Language/Version**: Rust, pinned toolchain, workspace MSRV 1.88.

**Primary Dependencies**: Existing windows-sys 0.36; additive Windows messaging bindings only. No package/version addition.

**Storage**: Existing private bundles and journals; no schema migration or owner-data change.

**Testing**: Token-stage regressions, actual Windows shell token validation and synthetic writer/read fixtures, CLI parser/help retirement, legacy cleanup/collection/Doctor regressions, full cargo xtask ci and hosted gates. Required genuine elevated evidence uses an unattended same-account desktop, including Windows Sandbox where available. Missing prerequisites never count as passed required rows.

**Target Platform**: Windows product, portable parser/library and static documentation checks.

**Project Type**: Existing Rust facade and CLI with xtask mechanical gates.

**Performance Goals**: Finite local recipient calls without helper wait for same-account operation; existing artifact bounds and different-account timeout remain.

**Constraints**: No target instrumentation or memory rights, hidden/noninteractive child tools, UTF-8 without BOM/LF, exact recipient proof and post-reconciliation verification.

**Scale/Scope**: Two high-priority issues, facade access, CLI guidance, maintained docs, Windows tests and narrowly scoped linter exception.

## Constitution Check

P-1 permits explicitly stated process rights and forbids memory-right target handles. The historical implementation chose a broader no-process-handle rule. S173 records a proportional exception for the exact desktop shell with PROCESS_QUERY_LIMITED_INFORMATION, token query/duplicate only; it never scans arbitrary process candidates. No principle amendment is needed. P-2/P-3 keep platform logic in facade. P-4/P-5/P-9 preserve artifact truth, independent access/cleanup and exact evidence. P-6 uses existing terms. P-7 keeps wrappers thin. P-8 requires complete spec-kit analysis and dated decisions. P-10 target storage is unchanged. P-11 preserves published records and labels source changes. Pre-design gate passes with the explicit implementation-choice deviation above.

## Project Structure

Feature artifacts live in specs/173-bundle-access-stabilization/. Facade changes belong to crates/fragcap/src/deep_capture/access.rs, access/windows.rs and mod.rs; access_repair.rs is retired. CLI changes belong to cli.rs, commands/bundle.rs, session_ux.rs, workflow_help.rs and their tests. crates/fragcap/tests/bundle_access.rs retains new-output/access-denial fixtures and removes repair-only fixtures. xtask/src/lint.rs admits only exact shell-query operations and rejects other process opens and memory rights. Maintained docs and docs/plans/README.md state the source transition; historical specs/release records remain.

## Decisions

1. Keep exact WTS account association. Ordinary producers retain their actual current token. Elevated same-account desktop producers resolve the exact shell window owner, open with literal query-limited process rights, query/duplicate its token, validate account/session and ordinary elevation/integrity, recheck shell identity, and retain only the token. No guessed account, extra terminal or elevated-to-restricted production substitution.
2. Label acquisition/validation/restoration errors by stage with original platform evidence. Failed final access never claims readable output or recommends retired commands.
3. Retain explicit authenticated different-account handoff and access-authorize. Controlled fallback stays labeled and cannot satisfy the genuine elevated regression.
4. Remove historical repair and only exclusive support. Shared descriptor pinning required by legacy cleanup/collection remains.
5. Current-source docs label removal separately from v0.10.6 history. Unresolved access guidance directs normal Windows permission administration without automatic repair promises.
6. Narrow mechanical exceptions validate source location and literal process/token rights; negative regressions cover broadened rights and transplanted calls. Document the exception in AGENTS/specification.
7. Use synthetic temporary output only. The owner offered Windows Sandbox; its unattended native CLI may prepare a disposable elevated desktop without changing host UAC/accounts/trust or owner bundles. Inspect actual token prerequisites.
8. Owner authorizes push/official PR and at most a second review round, superseding the default pre-push checkpoint. Stop for human merge after satisfied reviews and final-head green CI. No release publication.

## Implementation Sequence

1. Finish design/tasks/analysis and commit the specification gate.
2. Add failing token-stage and retired-command regressions; establish genuine elevated evidence early.
3. Correct acquisition with retained proofs and the scoped mechanical exception.
4. Remove historical CLI/facade workflow and exclusive support; retain legacy regressions.
5. Reconcile maintained guidance, architecture and dated decisions.
6. Run focused/full gates, publish official PR, reconcile every review and final-head CI, return for human merge.

## Post-Design Constitution Recheck

Design passes without principle amendment. Exact shell token acquisition is a documented implementation-choice deviation reviewed with the PR. Required elevated evidence remains required; equivalent-only proof and future owner testing do not close #476.

## Complexity Tracking

One exact desktop shell token adapter replaces unusable linked-token promotion. No service, broker, arbitrary process scan, package dependency or storage shape is added.
