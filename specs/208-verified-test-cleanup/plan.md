# Implementation Plan: Verified Test Cleanup

**Branch**: `codex/s208-test-cleanup` | **Date**: 2026-11-16 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification in `specs/208-verified-test-cleanup/spec.md`

## Summary

Deliver #605 and #606 as one test-ownership slice. Replace proven persistent fixture writers with unique owned roots and fallible, verified cleanup. Add a bounded run-level residue gate around repository test execution, so tests cannot pass while leaving newly owned output. Amend the constitution and contributor/reviewer guidance to require exact effective-root accounting after every application test. Keep product native bundles and unproven historic directories outside test-scratch deletion authority.

## Technical Context

**Language/Version**: Rust 2021, workspace MSRV 1.89; Markdown and existing workflow YAML

**Primary Dependencies**: Existing workspace `tempfile` 3.10; standard filesystem/process APIs; no new third-party package

**Storage**: Unique test-owned directories beneath the effective temp root; in-process CLI tests own a per-thread multi-call environment with a lock around each process-environment change; product session stores remain separate

**Testing**: Controlled Rust regressions for success, panic, cleanup failure, collisions, and exact ownership; Windows before/after residue inventory; repository `cargo xtask ci` and hosted checks

**Target Platform**: Windows primary, Linux CI parity

**Project Type**: Rust workspace with xtask runner, CLI, and integration tests

**Performance Goals**: Bounded inspection of the one run's exact root; no whole-machine temp sweep on ordinary gates

**Constraints**: No deletion by prefix alone; no live game, trust-store change, or existing historic-directory deletion; test and child processes remain hidden and noninteractive

**Scale/Scope**: #605 and #606, their proven test writers, and the test runner and guidance that govern them

## Constitution Check

*GATE: Pass before research and after design.*

- **P-1 and installed-product boundaries**: The slice changes test scratch and guidance only. It does not create a Deep Capture effect, touch a real game, or alter native bundle recovery.
- **P-4 and P-9**: Cleanup errors and unowned residue remain explicit failures, never silently discarded or reported as clean.
- **P-6 and P-8**: New terms receive glossary entries if introduced; Markdown, UTF-8/LF, source SPDX, and shell standards apply.
- **Development gates**: Spec-kit phases and analyze precede implementation. Controlled tests and full repository checks must pass before the PR claims completion.
- **Governance**: #605 is an owner-directed material expansion of installed-test guidance. Amend constitution 3.0.0 to 3.1.0 with a Sync Impact Report, then reconcile AGENTS, CONTRIBUTING, review guidance, and the master specification testing section. Do not change a principle to bypass a failed gate.
- **Pinned process files**: Any CI workflow change receives a dated `changelog.d/S208-verified-test-cleanup.decisions.md` entry and is reviewed as an effect of the approved #606 scope.

**Post-design re-check**: The design retains separate native evidence authority, exact recorded fixture ownership, bounded inventory, and no unsupervised historic deletion. No constitutional violation or unresolved clarification is present.

## Project Structure

### Documentation (this feature)

```text
specs/208-verified-test-cleanup/
  spec.md
  checklists/
  plan.md
  research.md
  data-model.md
  contracts/test-cleanup.md
  quickstart.md
  tasks.md
```

### Source Code (repository root)

```text
.specify/memory/constitution.md
AGENTS.md
CONTRIBUTING.md
docs/fragcap-specification.md
docs/plans/README.md
docs/security/native-product-review-handoff.md
crates/fragcap/tests/deep_capture_session.rs
crates/fragcap-cli/tests/common/mod.rs
crates/fragcap-cli/tests/cli_bundle.rs
crates/fragcap-cli/tests/cli_bootstrap.rs
crates/fragcap-cli/src/commands/technologies.rs
xtask/src/skills.rs
xtask/src/main.rs
xtask/src/test_cleanup.rs
.github/workflows/ci.yml
.github/workflows/platform.yml
.github/workflows/windows-integration.yml
supply-chain/policy-v1.json
changelog.d/S208-verified-test-cleanup.*.md
```

**Structure Decision**: Keep fixture ownership near the test writers. The shared xtask runner owns only the aggregate test invocation and its exact run root. Avoid a new production crate or product artifact schema for test-only behavior.

## Complexity Tracking

No constitution violations require an exception. The CI wrapper and workflow edit are within #606 because they provide the required bounded residue gate; the dated decision records their process impact.
