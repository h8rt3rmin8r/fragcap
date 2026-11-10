# Implementation Plan: Signed Direct Route Foundation

**Branch**: `codex/s199-signed-direct-route-foundation` | **Date**: 2026-11-09 | **Spec**: [spec.md](spec.md)

**Input**: Owner-authorized S199 for #577 and #579 under full Spec-Kit autopilot, with push and official PR authorization, exact-head CI and up to two review rounds.

## Summary

Amend the approved Deep Capture policy before code changes, then pin, stage and certify official signed WinDivert 2.2.2-A x64 bytes and corresponding source in both Windows packages. Extend read-only Doctor and package evidence without opening a route handle. Preserve passive Capture, Npcap non-bundling, the installation CA, and exact current-release claims.

## Technical Context

**Language/Version**: Rust MSRV 1.88, pinned development toolchain 1.96.0; PowerShell 7 Windows certification; Markdown, WiX and JSON contracts.

**Primary Dependencies**: Official WinDivert 2.2.2-A external component, existing Windows APIs and package tooling. No new Rust crate in S199.

**Storage**: Existing package contract and certification report; no target or session schema change.

**Testing**: Spec-Kit analysis, static package contract tests, injected Doctor cases, negative archive/PE/signature checks, `cargo xtask ci`, hosted Windows package certification and exact-head CI.

**Target Platform**: Windows 11 x64 distribution, platform-neutral static validation.

**Project Type**: Multi-crate Rust CLI and native Windows packages.

**Performance Goals**: No additional work in packet path or ordinary Capture; read-only Doctor remains bounded to a fixed set of small package files.

**Constraints**: No driver handle in S199, no private-title or workstation traffic, no general copyleft allowlist, no new route authority, UTF-8 without BOM, hidden/noninteractive child console tools.

**Scale/Scope**: Two issues; constitution, dependency policy, package pipeline, Doctor, tests, docs and dated changelog.

## Constitution Check

**Pre-design gate**: P-1 and the license section conflict with approved #572. The owner explicitly authorized #577 to reconcile them. This is the first planned governance deliverable and will be committed before implementation. A 2.0.0 amendment removes the blanket driver/key-export bans only within the approved Deep Capture boundary. No prohibited target memory, game-file edit, silent global proxy or unrelated process permission is added. The governing decision is checked against the master specification before any package code.

**Post-design gate**: P-2/P-3 keep future platform route work outside core; P-4/P-9 require exact package and readiness truth; P-5 keeps established artifact formats; P-6 updates vocabulary; P-7/P-8 require UTF-8, hidden execution and complete gates; P-10 target resolution is unchanged; P-11 distinguishes package readiness from routed traffic and unreleased source from published v0.10.14. The owner authorized push and PR, superseding autopilot's ordinary pre-push halt for this slice only. Merge and release remain with the owner.

## Project Structure

### Documentation

`specs/199-signed-direct-route-foundation/` holds spec, checklist, research, plan, data model, package contract, quickstart, tasks and analysis.

### Source

`integration/windows-package-contract-v1.json`, `.github/workflows/package-certification.yml`, `scripts/Test-PackageCertification.ps1`, `xtask/src/package_certification.rs`, `crates/fragcap-cli/wix/main.wxs`, `crates/fragcap-cli/src/doctor/`, `supply-chain/`, governance and site docs, `changelog.d/`.

## Implementation Sequence

1. Amend constitution and all categorical prohibitions, exact exception and historical-source wording, with a dated governing decision. Commit this governance gate before package implementation.
2. Add failing contract and Doctor tests for exact component identity and absence/mismatch behavior. Add a Windows negative mutation matrix for source archive, DLL, SYS, PE architecture and signature.
3. Pin the upstream archives and hashes, download only in the package workflow, verify before staging, and add the shared component entries to ZIP and MSI. Include license, source, notice and SBOM inventory.
4. Extend read-only Doctor and package certification for exact component state, file equality and lifecycle, retaining the existing CA and Npcap paths.
5. Reconcile operator and maintainer docs with package reality, including current limits. Re-run Spec-Kit analysis and quality checks.
6. Run local CI parity and exact Windows hosted certification; commit, push, publish official PR with `Closes #577` and `Closes #579`, resolve CI and every arriving review. Request no more than one second `@Codex` round, then hand the green PR to the owner.

## Complexity Tracking

The governance amendment is required by the owner's approved product decision and precedes component implementation. No new Rust crate, driver route, target effect, or storage schema is introduced.
