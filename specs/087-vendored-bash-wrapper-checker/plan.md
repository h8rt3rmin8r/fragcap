# Implementation Plan: Vendored Bash Wrapper Checker

**Branch**: `codex/087-vendored-bash-wrapper-checker` | **Date**: 2026-08-27 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/087-vendored-bash-wrapper-checker/spec.md`

## Summary

Fix issue #199 by replacing `xtask/src/wrappers.rs`'s hand-authored Bash structural checker with calls to the vendored `shruggie-bash` compliance checker for `fragcap.sh`, `lint-docs.sh`, and `cut-release.sh`. Because that checker only warns when ShellCheck is absent, `cargo xtask wrappers` adds a Bash-runnable ShellCheck preflight so CI exits 2 when static analysis cannot run. Existing syntax, help, dry-run, and PowerShell checks stay intact.

## Technical Context

**Language/Version**: Rust, workspace MSRV 1.82

**Primary Dependencies**: Existing standard library process execution only

**Storage**: No storage changes

**Testing**: Focused xtask tests, direct vendored checker measurements, `cargo xtask wrappers`, then repository gate

**Target Platform**: Cross-shell repository gate on Windows and Linux CI, using relative paths from the repository root

**Project Type**: Rust workspace xtask compliance gate

**Performance Goals**: Negligible. Three extra checker invocations replace local byte scans and run only in compliance gates.

**Constraints**: No wrapper script behavior changes, no vendored checker modifications, no new dependency, no workflow edits, preserve 0/1/2 exit contract

**Scale/Scope**: One xtask module, master-spec wording, S087 traceability artifacts, changelog fragments

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 No Covert Target Instrumentation**: Pass. This slice changes only repository wrapper compliance checks.
- **P-2 Core Stays Platform-Neutral**: Pass. No `fragcap-core` changes.
- **P-3 Capture And Attribution Stay Separate**: Pass. No capture or attribution changes.
- **P-4 No Silent Loss**: Pass. No packet or discovery accounting changes.
- **P-5 Compatibility Outranks Richness**: Pass. No output format changes.
- **P-6 Glossary First**: Pass. Existing wrapper, Bash, ShellCheck, and CI terms are sufficient.
- **P-7 Wrappers Stay Thin**: Pass. Wrapper script behavior remains unchanged; only the gate changes.
- **P-8 House Standards Apply**: Pass. The gate now uses the vendored Bash standard authority.
- **P-9 The Instrument Does Not Lie**: Pass. A skipped ShellCheck cannot report as a clean compliance result.
- **P-10 One Path To A Target**: Pass. No target path changes.
- **P-11 The Specification Describes What Shipped**: Pass. The master specification will record the checker delegation and ShellCheck precondition.

## Project Structure

### Documentation (this feature)

```text
specs/087-vendored-bash-wrapper-checker/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── wrapper-gate.md
├── checklists/
│   ├── gate-authority.md
│   └── requirements.md
└── tasks.md
```

### Source Code (repository root)

```text
xtask/src/wrappers.rs
docs/fragcap-specification.md
changelog.d/199-vendored-bash-wrapper-checker.fixed.md
changelog.d/199-vendored-bash-wrapper-checker.decisions.md
```

**Structure Decision**: Keep the orchestration in `xtask/src/wrappers.rs` because the repository gate already owns wrapper compliance composition. Delegate Bash rule interpretation to the vendored checker instead of duplicating it in Rust.

## Complexity Tracking

No constitution violations or complexity exceptions are needed.

## Phase 0: Research

See [research.md](research.md).

## Phase 1: Design

See [data-model.md](data-model.md), [contracts/wrapper-gate.md](contracts/wrapper-gate.md), and [quickstart.md](quickstart.md).

## Post-Design Constitution Check

The design still passes all constitution checks. It strengthens P-8 enforcement by removing a duplicate Bash compliance implementation and strengthens P-9 by refusing to call a skipped ShellCheck run compliant.
