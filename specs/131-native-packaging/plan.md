# Implementation Plan: Native Windows Packaging Certification

**Branch**: `codex/131-native-packaging` | **Date**: 2026-09-05 | **Spec**: `specs/131-native-packaging/spec.md`

**Input**: Feature specification from `specs/131-native-packaging/spec.md`

## Summary

Create one closed repository-owned contract for the three primary Windows release artifacts and their checksum sidecars, certify final ZIP and MSI bytes plus the real installer lifecycle on fresh hosted Windows runners, expose an exact native build identity, and make certification precede every GitHub release or crate publication path. The implementation extends S129's finite Windows evidence model and S130's release-input evidence without repeating their protocol or dependency campaigns.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88, pinned product toolchain 1.96.0; PowerShell 7 for Windows package lifecycle orchestration

**Primary Dependencies**: Existing `serde_json`, `ring`, and `windows-sys` dependencies in `xtask`; Windows Installer, PowerShell Authenticode and archive APIs, existing WiX v3 package definition; no product dependency change

**Storage**: Versioned JSON package contract, bounded JSON certification reports, and ignored runner-local MSI logs

**Testing**: Rust unit and mutation tests, offline static contract validation, packaged binary smoke, final-byte inspection, PE identity/import inspection, checksum reconciliation, and real MSI install/repair/reinstall/upgrade/downgrade-refusal/uninstall cases

**Target Platform**: Official `x86_64-pc-windows-msvc` ZIP and MSI on fresh GitHub-hosted Windows runners; static gate on Windows and Linux

**Project Type**: Rust workspace, repository task runner, PowerShell lifecycle harness, WiX installer, and GitHub Actions release pipeline

**Performance Goals**: Offline static validation under 60 seconds; each installer lifecycle case under 10 minutes; complete packaging workflow under 45 minutes

**Constraints**: Fail closed; no recurring local task; no product runtime dependency; no hidden download; no Npcap bundling; exact three-primary-artifact release contract; public reports contain no runner paths or account identity; current Authenticode policy remains explicitly unsigned

**Scale/Scope**: Three primary artifacts, three checksum sidecars, six shared payload entries, one closed PE import allowlist, six required lifecycle cases, one native packaged-binary smoke, and one blocking release dependency chain

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1/P-2/P-3**: PASS. Certification exercises only fragcap-owned packages in fresh CI roots, introduces no target-process access, and keeps package orchestration in `xtask` plus a Windows-specific harness.
- **P-4/P-9**: PASS. Every artifact entry, lifecycle case, checksum, signature state, PE import, machine effect, timeout, refusal, and residue has an explicit row and stable failure reason; warning-only success is removed.
- **P-5**: PASS. The portable archive and MSI retain the established six-file payload and unmodified analyzer compatibility; final-byte validation detects drift.
- **P-6/P-8**: PASS. Package contract, maintainer procedure, build identity, report schema, ownership vocabulary, and lifecycle rules receive primary documentation and executable checks.
- **P-7**: PASS. Package certification remains offline after its declared inputs are acquired and requires no user-supplied secret.
- **P-10/P-11**: PASS. Existing capture and Deep Capture artifact schemas remain unchanged. S131 claims only issue #329 and leaves final Deep Capture completion to #334.
- **Packaging and release**: PASS. The existing unsigned state is measured accurately; the three-download contract remains intact; tag/workspace identity and certification move ahead of publication.
- **Pinned artifacts**: PASS with a required dated decision. The package contract, workflow, WiX lifecycle settings, predecessor digest, tool pins, size ceilings, and PE import allowlist are explicit S131 deliverables.

Post-design check: PASS. The design reuses S129 process containment and reporting patterns, S130 validated package inputs, native Windows/MSI authorities, and the existing controlled loopback smoke. It does not add a general packaging framework, a signing service, or a synthetic rollback engine.

## Architecture and Phases

1. Freeze artifact, content, build-identity, PE, checksum, signature, installer ownership, lifecycle, report, and workflow contracts.
2. Add failing schema, mutation, workflow-order, WiX, build-identity, and report-reconciliation tests.
3. Implement the offline `cargo xtask package-certification` authority and include it in `cargo xtask ci`.
4. Expose exact machine-readable release build identity without changing runtime capability.
5. Add a compliant hidden-process PowerShell harness for final-byte checks, packaged smoke, and bounded MSI lifecycle cases.
6. Replace the best-effort release smoke with one required package-certification workflow whose certified bytes feed release publication.
7. Correct current package documentation and record S131 architecture, decisions, and changelog evidence.
8. Run spec-kit analysis, local convergence, hosted certification, full CI, and review closure.

## Project Structure

```text
specs/131-native-packaging/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
integration/
└── windows-package-contract-v1.json
xtask/src/
├── main.rs
└── package_certification.rs
scripts/
└── Test-PackageCertification.ps1
.github/workflows/
├── package-certification.yml
└── release.yml
crates/fragcap-cli/
├── build.rs
├── src/main.rs
└── wix/main.wxs
docs/maintainers/
└── package-certification.md
README.md
NOTICE
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
AGENTS.md
changelog.d/
```

**Structure Decision**: Keep the closed contract beside the existing Windows evidence registries, add static parsing and validation to the repository task runner, isolate destructive Windows lifecycle operations in one convention-compliant script, and make both pull-request and tag workflows consume the same contract and report authority.

## Complexity Tracking

No constitution violation requires an exception.
