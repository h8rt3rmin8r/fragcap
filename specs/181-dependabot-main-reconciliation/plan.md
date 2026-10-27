# Implementation Plan: S181 Dependabot reconciliation and green-main recovery

**Branch**: `codex/s181-dependabot-main-reconciliation` | **Date**: 2026-10-27 | **Spec**: [spec.md](spec.md)

## Summary

Resolve #511, five open site alerts, PR #510, and stale advisory patches discovered in six isolated Rust locks. Prepare locked dependencies before offline documentation discovery, run root contracts on PRs and main, audit every committed dependency graph, and deliver one PR through verified green main and scanner reconciliation.

## Technical Context

**Language/Version**: Rust 1.96.0, MSRV 1.88; Node 24; pnpm 10.26.0.

**Primary Dependencies**: Zod 4.6.6, DOMPurify 3.4.17, Tokio 1.53.3, rustls-webpki 0.103.16; existing pinned cargo-deny 0.20.2.

**Storage**: Seven committed Cargo locks, site package metadata, dependency policy, workflows, and slice evidence. No runtime migration.

**Testing**: Focused workflow mutations, fresh seven-graph Rust and complete-site audits, repository gates, site tests/export, PR workflows, isolated-cache docs dispatch, final-main checks.

**Target Platform**: Windows development; hosted Windows/Linux; GitHub Pages.

**Project Type**: Dependency and CI maintenance.

**Performance Goals**: Preserve ordinary caches; execute one explicit cold-cache qualification.

**Constraints**: Preserve locked resolution, offline executable discovery, advisory/license policy, published identity, and hidden noninteractive Windows tooling.

**Scale/Scope**: Five kickoff alerts, one bot PR, seven Rust graphs, one site graph, one implementation PR.

## Constitution Check

P-1 through P-7 and P-10 preserve all target, capture, attribution, and runtime boundaries. P-8 requires the installed Spec-Kit sequence and all applicable gates; pinned workflows receive dated decisions. P-9 separates source checks, final-main checks, deployment, cold-cache proof, and scanner state. P-11 preserves release identity. No constitutional exception is needed. The current owner expressly authorizes S181 publication and integration through main, overriding generic pre-push stopping guidance for this run.

## Project Structure

```text
specs/181-dependabot-main-reconciliation/
  spec.md
  plan.md
  research.md
  data-model.md
  quickstart.md
  checklists/
  tasks.md
.github/workflows/docs.yml
.github/workflows/audit.yml
xtask/src/supply_chain.rs
site/package.json
site/pnpm-workspace.yaml
site/pnpm-lock.yaml
fuzz/Cargo.lock
performance/native-proxy/Cargo.lock
spikes/*/Cargo.lock
spikes/*/audit/Cargo.lock
docs/maintainers/supply-chain.md
docs/fragcap-specification.md
docs/plans/README.md
changelog.d/
```

**Structure Decision**: Extend existing workflows and their supply-chain wiring guard. No runtime crate, policy framework, or external product interface is introduced.

## Decisions

1. Patch only affected resolutions and coordinated transitive edges. Reuse #510's Zod update; close it as superseded after patched main exists.
2. Add unconditional `cargo fetch --locked` before root contracts and remove their push-only guard. Preserve offline compiled discovery. Add manual `cold_cache` dispatch that isolates Cargo home/target and skips cache restore; run root contracts offline after fetch.
3. Add Node 24/pnpm 10.26 site audit with frozen install and `pnpm audit --audit-level low` covering production, development, and optional dependencies. Unavailable evidence fails.
4. Retain the root full cargo-deny gate and audit all six isolated manifests with the same pinned tool and root policy.
5. Add workflow mutation tests for fetch ordering, PR/main parity, full audit scope, and failure propagation. Actual hosted cold-cache execution supplies behavioral proof.
6. Record final scanner reconciliation in #511. New findings remain in scope until final verification.

## Implementation Sequence

1. Complete specification, clarification, checklist, research, design, tasks, and blocking analysis.
2. Establish failing advisory/workflow baselines and focused regression cases.
3. Patch dependency resolutions, documentation preparation, and audit wiring.
4. Update maintenance/specification records and decisions; complete local qualification.
5. Publish one PR, resolve source CI/review findings, and integrate under owner delegation without changing protections.
6. Verify final-main workflows, cold-cache docs, fresh audit/links, Pages, bot reevaluation, and zero-outstanding inventory; close #511 with evidence.

## Post-Design Constitution Recheck

Original validations remain authoritative. Changes add dependency preparation and audit coverage; all product and release boundaries remain intact.
