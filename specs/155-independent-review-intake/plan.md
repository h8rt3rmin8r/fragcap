# Implementation Plan: S155 independent review execution and findings intake

**Branch**: `codex/s155-independent-review-intake` | **Date**: 2026-09-16 UTC | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/155-independent-review-intake/spec.md`

## Summary

Bind the published v0.10.1 review candidate to one machine-readable immutable asset registry, add a strict completed-review intake validator that never self-certifies reviewer independence, and replay the exact public ZIP and MSI through separate portable and installed controlled-smoke paths on disposable GitHub-hosted Windows. Preserve the existing not-started handoff record, keep #333 and #413 open without external evidence, and publish only bounded sanitized campaign evidence.

## Technical Context

**Language/Version**: Rust 2021 with MSRV 1.88; PowerShell 7 on GitHub-hosted Windows

**Primary Dependencies**: Rust standard library, existing `serde_json`, existing `xtask` infrastructure, Windows Installer, GitHub Actions

**Storage**: Versioned JSON contracts and bounded JSON reports in repository and workflow artifacts

**Testing**: `cargo test -p xtask`, full `cargo xtask ci`, PowerShell package-certification campaign, GitHub Actions hosted Windows replay

**Target Platform**: Repository tooling on supported development platforms; campaign execution only on disposable `windows-2025` hosted runners

**Project Type**: Rust workspace with repository-owned CI and packaging scripts

**Performance Goals**: Completed review records at or below 512 KiB; hosted package report below its existing 1 MiB ceiling; every child and campaign operation time-bounded

**Constraints**: No product execution on the owner's workstation; no locally rebuilt candidate bytes; no hidden or interactive child console; no uploaded secret, private path, host identifier, raw payload or private key; no self-certification of independent acceptance

**Scale/Scope**: Twelve closed review areas, six public release files, two controlled-smoke surfaces, one completed-record contract and one immutable published candidate

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: Pass. The campaign uses only the shipped explicit Deep Capture path and its existing controlled target. It adds no target instrumentation or prohibited process access.
- **P-2 and P-3**: Pass. Changes are isolated to `xtask`, packaging automation and documentation. No core, capture or attribution dependency changes occur.
- **P-4**: Pass. Both smoke paths require complete loss and cleanup accounting. Missing or indeterminate evidence is a failure.
- **P-5**: Pass. No capture format or analyzer compatibility changes occur.
- **P-6**: Pass. The design reuses existing glossary terms and introduces no new product-domain term.
- **P-7**: Pass. PowerShell orchestrates installation and calls existing Rust authorities. Review-record parsing and validation live in Rust.
- **P-8**: Pass. Existing repository linters and pinned-script decision requirements remain mandatory.
- **P-9**: Pass. Hosted replay is reported as reproducible candidate evidence, never as independent acceptance. Review findings and limitations remain explicit.
- **P-10**: Pass. No target storage or resolution path changes occur.
- **P-11**: Pass. The master specification receives the missing S154 lineage and the S155 evidence boundary without describing unshipped acceptance.

## Project Structure

### Documentation (this feature)

```text
specs/155-independent-review-intake/
├── checklists/
│   ├── requirements.md
│   └── security-review.md
├── contracts/
│   └── review-record.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
└── tasks.md
```

### Source Code (repository root)

```text
.github/workflows/
└── published-review-candidate.yml

docs/security/
├── native-product-review-candidate.v1.json
├── native-product-review-handoff.md
├── native-product-review-record.v1.json
└── native-product-review-scope.v1.json

scripts/
└── Test-PackageCertification.ps1

xtask/src/
├── main.rs
├── package_certification.rs
└── review_record.rs
```

**Structure Decision**: Keep review intake in `xtask`, extend the existing package-certification script rather than add a second installer authority, and give hosted replay a dedicated workflow because it consumes immutable public release bytes instead of building the pull-request source.

## Complexity Tracking

No constitution violation or architecture exception is required.
