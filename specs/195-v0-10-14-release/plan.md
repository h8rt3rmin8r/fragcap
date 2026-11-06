# Implementation Plan: v0.10.14 Release Stabilization and Candidate

**Branch**: `release/0.10.14` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/195-v0-10-14-release/spec.md`

## Summary

Correct the known hosted-Windows calibration regression race by giving that controlled end-to-end path the same finite seven-second launch allowance already used by adjacent controlled calibration cases, while preserving production defaults and exact operator deadlines. Then assemble S189 through S194 plus the correction as a coherent v0.10.14 source candidate, validate product and isolated dependency authorities, push the official PR, reconcile at most two hosted Codex review rounds, and stop before owner-controlled merge or publication.

## Technical Context

**Language/Version**: Rust 1.88 workspace, PowerShell 7 and Bash repository tooling, MDX documentation site

**Primary Dependencies**: Existing locked workspace dependencies, cargo-release, repository xtask, Windows package and conformance tooling; no new third-party package

**Storage**: No product storage change; release fragments, generated goldens, versioned conformance evidence, specification records, and dependency inventories only

**Testing**: Focused Rust integration regression, complete repository xtask checks, MSRV and neutral-platform checks, documentation and notes validation, all locked dependency graphs, Windows package certification, and exact-head hosted CI

**Target Platform**: Windows 11 product, Linux and Windows CI for platform-neutral and packaging contracts

**Project Type**: Multi-crate Rust CLI and libraries with Windows packaging and static documentation

**Performance Goals**: No production timing or packet-path change; deterministic hosted regression under ordinary scheduler variance; release checks stay within existing bounded workflow budgets

**Constraints**: No merge, tag, GitHub release, registry publication, current-public marker advance, installed private-title test, production timeout change, dependency waiver, or third Codex review round

**Scale/Scope**: Issues #566 and #567, six merged slices S189-S194, one controlled integration-test correction, one patch candidate, product plus isolated locked dependency graphs

## Constitution Requirements

**Status before research**: Passed.

- **P-1/P-2/P-3**: S195 changes no capture, attribution, routing, proxy, trust, process-access, or crate-ownership boundary. The controlled regression continues through existing public CLI and facade seams.
- **P-4/P-9**: The original failed hosted receipt remains evidence. The successful rerun does not erase the timing defect, and the correction does not convert a genuinely late launch into success.
- **P-5/P-6**: No packet, artifact, schema, glossary, or public protocol vocabulary changes.
- **P-7/P-8**: Release automation, fragments, generated files, encoding, prose, dependency inventories, and branch policy follow repository-owned authorities.
- **P-10**: No target-store or resolution behavior changes.
- **P-11**: Candidate version and specification applicability advance together, while published-release and frozen public-review records remain on verified v0.10.13.
- **Evidence boundary**: Source checks, exact-head hosted checks, package certification, published bytes, installed behavior, and independent review remain separate claims.
- **Workflow**: Full Spec-Kit sequence, chronological implementation, foreground verification, bounded hosted review, and owner-reserved merge.

**Post-design recheck**: Passed. The design changes one test-only launch allowance, leaves all production deadline authority intact, preserves the public v0.10.13 baseline, and records future publication as separate work under #567.

## Project Structure

### Documentation (this feature)

```text
specs/195-v0-10-14-release/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── release-candidate.md
├── checklists/
│   ├── requirements.md
│   └── release.md
└── tasks.md
```

### Source And Release Records (repository root)

```text
crates/fragcap-cli/tests/cli_calibrate.rs
Cargo.toml
Cargo.lock
crates/*/Cargo.toml
crates/fragcap-sink/src/{json,pcapng}/mod.rs
crates/fragcap-cli/tests/{windows_native_integration.rs,goldens/}
conformance/native-http-tls/{matrix-v1.json,report-v1.json}
fuzz/Cargo.lock
performance/*/Cargo.lock
changelog.d/
CHANGELOG.md
release-notes/v0.10.14.md
docs/fragcap-specification.md
docs/plans/README.md
docs/security/
CONTRIBUTING.md
```

**Structure Decision**: Change the existing controlled CLI regression in place, then use the repository's release authorities to advance candidate-bound version and generated records. Published-release records are deliberately excluded because S195 does not publish.

## Complexity Tracking

No constitution violation requires justification.
