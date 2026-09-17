# Implementation Plan: S156 deterministic package certification

**Branch**: `codex/s156-deterministic-package-certification` | **Date**: 2026-09-17 UTC | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/156-deterministic-package-certification/spec.md`

## Summary

Replace the package campaign's timing-sensitive positive socket assertion with strict parsing of the product's structured `deep_capture.proxy_started` and terminal `deep_capture.calibration_phase` evidence. Preserve firewall containment and all sampled negative evidence, emit package report schema 4 with schema-3 read compatibility, validate every authority independently in Rust, and produce closed bounded predicate diagnostics from PowerShell. Local work remains static and unit-level; exact package execution occurs only on disposable hosted Windows.

## Technical Context

**Language/Version**: Rust 2021 with MSRV 1.88; PowerShell 7 on GitHub-hosted Windows

**Primary Dependencies**: Rust standard library, existing `serde_json`, existing `xtask`, PowerShell JSON conversion and Windows networking/firewall cmdlets

**Storage**: Versioned bounded package-certification JSON report and existing transient hosted scratch data

**Testing**: `cargo test -p xtask package_certification`, static PowerShell parsing and compliance, `cargo xtask ci`, two disposable hosted Windows certification workflows

**Target Platform**: Repository tooling on supported development platforms; product execution only on GitHub-hosted `windows-2025`

**Project Type**: Rust workspace with repository-owned PowerShell package certification and GitHub Actions

**Performance Goals**: No new polling or product delay; at most 32 parsed structured events and 16 predicate diagnostics per smoke; preserve the existing 1 MiB report ceiling

**Constraints**: No installed product, real game, live capture or trust mutation on the owner workstation; no raw host values in diagnostics; no weakened firewall, ownership, non-loopback or cleanup checks; no retry-based acceptance

**Scale/Scope**: Two smoke surfaces, two deterministic event authorities, one current report schema, one legacy schema reader and two hosted workflows

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: Pass. The change observes existing explicit controlled Deep Capture events and retains target-scoped firewall containment. It adds no instrumentation or product capability.
- **P-2 and P-3**: Pass. Work is isolated to repository certification tooling, reports and documentation. Core, capture and attribution architecture do not change.
- **P-4**: Pass. Every observed non-loopback endpoint and unexpected process remains failing evidence. Absence of a polling sample is no longer mislabeled as loss.
- **P-5**: Pass. Capture formats and analyzer behavior do not change.
- **P-6**: Pass. The slice reuses existing product terms. `predicate diagnostic` is confined to the slice contract and does not enter product documentation as a new domain term.
- **P-7**: Pass. PowerShell performs bounded package and host orchestration plus strict structured event extraction. The reusable report contract and exhaustive mutation validation remain in Rust.
- **P-8**: Pass. PowerShell and Markdown follow repository standards, with both wrapper compliance authorities required.
- **P-9**: Pass. Structured events are validated as observed. Socket samples remain reported exactly as diagnostic evidence, including zero observations, rather than being fabricated or required.
- **P-10**: Pass. Target storage and resolution are unchanged.
- **P-11**: Pass. The master specification records the post-S155 correction without changing the published product baseline.

## Project Structure

### Documentation (this feature)

```text
specs/156-deterministic-package-certification/
├── checklists/
│   ├── certification.md
│   └── requirements.md
├── contracts/
│   └── package-certification-report.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
changelog.d/
├── S156-deterministic-package-certification.decisions.md
└── S156-deterministic-package-certification.fixed.md

docs/maintainers/
└── package-certification.md

scripts/
└── Test-PackageCertification.ps1

xtask/src/
└── package_certification.rs

docs/
├── fragcap-specification.md
└── plans/README.md
```

**Structure Decision**: Extend the single existing package-certification script and Rust validator. Do not add a second campaign, parser or report authority.

## Complexity Tracking

No constitution violation or architecture exception is required.
