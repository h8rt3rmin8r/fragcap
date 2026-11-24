# Implementation Plan: Withdrawn Site Advisory Cleanup

**Branch**: `codex/s213-site-advisory-withdrawal` | **Date**: 2026-11-24 | **Spec**: [spec.md](spec.md)

**Input**: The S213 feature specification and [issue #630](https://github.com/h8rt3rmin8r/fragcap/issues/630).

## Summary

Remove the finite site-audit exception for GHSA-fc5q-39xc-vjpf after GitHub withdrew it. Restore an unqualified full site dependency audit without changing the lockfile merely to remove a withdrawn report. Replace the exception-specific validators with small offline and hosted guards for workspace YAML, the root package manifest, and repository `.npmrc` files; inspect the hosted audit step's own environment before invoking pnpm. Correct current guidance, preserve historical v0.11.2 release evidence, and leave the separate Rust advisory work under #640 untouched.

## Technical Context

**Language/Version**: Rust workspace toolchain 1.96.0 with declared MSRV 1.88; Node 24; pnpm 10.26.0.

**Primary Dependencies**: Existing Rust xtask and serde_json validation, GitHub Actions, pnpm's full-graph audit. No new dependency.

**Storage**: Committed YAML, JSON, Markdown, and Rust source only; no new product or user storage.

**Testing**: Focused xtask negative fixtures for workspace, `site/package.json#pnpm.auditConfig`, both repository `.npmrc` locations, unsupported root YAML syntax, bare CR, direct audit-step environment overrides, and mutations that remove or reorder fresh HOME/XDG_CONFIG_HOME isolation; positive LF/CRLF, indentless `packages`, and benign manifest baselines; cargo xtask supply-chain; pinned pnpm frozen-lockfile install, full audit, and build; cargo xtask ci; final-head hosted audit, docs, and repository checks.

**Target Platform**: GitHub hosted Linux site job and local/hosted repository gates. No Windows product behavior change.

**Project Type**: Repository supply-chain policy and documentation maintenance.

**Performance Goals**: Preserve the existing finite site audit command and avoid a new dependency scan or product runtime cost.

**Constraints**: Keep the unconditional pnpm audit --audit-level low command, Node/pnpm pins, lockfile, and Rust exceptions unchanged. A distinct advisory remains blocking. Historical release identity stays accurate.

**Scale/Scope**: One withdrawn GHSA and its site-only exception, with two independently testable stories. The exact affected files are listed below.

## Constitution Check

| Principle or gate | S213 approach | Result |
| --- | --- | --- |
| P-8, house standards | Use existing validators, pinned tools, UTF-8 Markdown and Rust conventions. | Pass |
| P-9, evidence truth | A green result requires an unfiltered audit; stale registry or independent findings stay red. | Pass |
| P-11, shipped-spec truth | Preserve the published v0.11.2 account, then date the withdrawal and current policy change. | Pass |
| Supply-chain authority | Remove only the withdrawn site exception; retain all unrelated Rust advisory blocking and #640. | Pass |
| Development workflow | Run specify, clarify, checklist, plan, tasks, analyze, implement, gates, PR review, and hosted CI. | Pass |
| Verified test cleanup | Controlled fixture data remains in test-local memory or exact disposable roots and is cleaned after the gate. | Pass |

No constitution deviation is proposed. Recheck this table after source changes and before publication.

## Project Structure

### Specification and verification

```text
specs/213-site-advisory-withdrawal/
  spec.md
  checklists/requirements.md
  checklists/audit-boundary.md
  plan.md
  research.md
  data-model.md
  contracts/site-audit-boundary.md
  quickstart.md
  tasks.md
```

### Source and current documentation

```text
site/pnpm-workspace.yaml
site/package.json  (read and validate; no production dependency change)
.npmrc and site/.npmrc  (must be absent)
supply-chain/site-audit-exception-v1.json  (remove)
.github/workflows/audit.yml
xtask/src/supply_chain.rs
docs/maintainers/supply-chain.md
docs/maintainers/v0.11.2-release-handoff.md  (read-only historical comparison)
docs/plans/README.md
docs/fragcap-specification.md
changelog.d/S213-site-advisory-withdrawal.changed.md
changelog.d/S213-site-advisory-withdrawal.decisions.md
```

**Structure Decision**: Keep the existing site job and offline xtask as independent audit authorities. Replace their exception-specific checks in place. Accept LF/CRLF and canonical root block-mapping keys in the workspace YAML, including indentless list items after a mapping key; reject bare CR, standalone root sequences, and unsupported root forms instead of trusting a partial parse to find every audit key. Parse the root package manifest as JSON to reject `pnpm.auditConfig`, reject repository `.npmrc` sources, and put the direct configuration environment assertion in the same hosted step as `pnpm audit`. In that step, set HOME and XDG_CONFIG_HOME to a fresh temporary directory before invoking the unchanged command, preventing step-scoped path overrides from reaching repository-controlled global config files. A prior step cannot see variables scoped only to the audit step. Do not add a new policy file, new dependency, or general-purpose YAML policy engine. The site lockfile and Rust policy remain outside the diff unless a separate red gate demonstrates a concrete need.

## Delivery Boundary

The PR closes #630 only after the expanded configuration guard, full unfiltered site audit, frozen install, static export, local repository gate, and final-head hosted checks pass and review comments are resolved. The owner retains merge authority. S213 does not claim a product runtime, installed-product, or private client compatibility result. #640's temporary Rust exception and replacement remain separate.
