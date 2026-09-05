# Implementation Plan: Native Supply-Chain and Compatibility Gate

**Branch**: `codex/130-native-supply-chain` | **Date**: 2026-09-05 | **Spec**: `specs/130-native-supply-chain/spec.md`

**Input**: Feature specification from `specs/130-native-supply-chain/spec.md`

## Summary

Create one closed repository-owned policy over the complete locked dependency graph, retain cargo-deny as the blocking network-backed advisory/license/source authority, validate finite exceptions and critical dependency maintenance, generate exact-pinned CycloneDX and third-party notices from the shipped Windows closure, and embed the validated evidence in both existing release packages without changing runtime behavior or absorbing S131.

## Technical Context

**Language/Version**: Rust 2021, minimum Rust 1.88, pinned product toolchain 1.96.0

**Primary Dependencies**: Existing `serde_json` and `ring` in `xtask`; exact CI tools cargo-deny 0.20.2, cargo-cyclonedx 0.5.9, and cargo-about 0.9.2; no product dependency change

**Storage**: Versioned JSON policy in the repository; generated SBOM and notices under release staging only

**Testing**: Pure policy and graph fixtures, live offline Cargo metadata reconciliation, workflow/WiX contract tests, evidence mutation tests, existing MSRV build, cargo-deny, and full `cargo xtask ci`

**Target Platform**: Static policy on Windows and Linux; shipped evidence for `x86_64-pc-windows-msvc`

**Project Type**: Rust workspace, repository task runner, and GitHub Actions release pipeline

**Performance Goals**: Offline policy validation under 60 seconds; network-backed audit under 15 minutes

**Constraints**: Fail closed; no product effects; no local schedule; no hand-written advisory, SPDX, or CycloneDX engine; no new release download; preserve S131 package-certification ownership

**Scale/Scope**: Three normalized graph views, one closed policy, one audit workflow, two generated evidence files, two existing packages

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: PASS. The policy strengthens the prohibited dependency boundary and adds no product capability, process access, interception library, trust effect, or route.
- **P-2/P-3**: PASS. Repository orchestration remains in `xtask`; product crate direction and platform ownership do not change.
- **P-4/P-9**: PASS. Unknown packages, edges, exceptions, advisory state, and evidence fail closed. Generated evidence never invents a complete result from an incomplete graph.
- **P-5**: PASS. Existing capture artifacts and analyzer compatibility remain unchanged.
- **P-6/P-8**: PASS. New vocabulary and maintainer procedures receive primary documentation and mechanical checks.
- **P-10/P-11**: PASS. Target data and output contracts are unchanged. S130 claims only issue #328 and leaves packaging completion plus final Deep Capture completion to #329 and #334.
- **Licensing**: PASS. cargo-deny evaluates the complete policy graph, cargo-about resolves distributable license text, and both shipped packages carry the resulting notices.
- **Pinned artifacts**: PASS with a required dated decision. `audit.yml`, `release.yml`, `deny.toml`, WiX payload declarations, and exact tool/action versions are explicit S130 deliverables.

Post-design check: PASS. Mature ecosystem tools own mutable external intelligence and standard generation, repository code owns closed graph and release reconciliation, and no runtime or package-certification boundary moved.

## Architecture and Phases

1. Freeze graph, policy, exception, report, SBOM, notices, and workflow contracts.
2. Add red tests for schema, normalized graph views, drift diagnostics, finite exceptions, evidence reconciliation, and release ordering.
3. Implement offline graph normalization and closed policy validation in `xtask`.
4. Tighten cargo-deny configuration and triggers with exact tool/action pins.
5. Add exact-pinned evidence configuration, generation, independent validation, and release package wiring.
6. Document update and emergency procedures, update specification records, and converge.
7. Run offline gates, network-backed audit, evidence generation, full CI, and release-wiring checks.

## Project Structure

```text
specs/130-native-supply-chain/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
├── checklists/
└── tasks.md
supply-chain/
├── policy-v1.json
├── about.toml
└── about.hbs
xtask/src/
├── main.rs
└── supply_chain.rs
.github/workflows/
├── audit.yml
└── release.yml
crates/fragcap-cli/wix/
└── main.wxs
docs/maintainers/
└── supply-chain.md
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
AGENTS.md
changelog.d/
```

**Structure Decision**: Keep policy and evidence configuration in one dedicated repository directory, keep orchestration and validation in the existing task runner, and modify only the existing audit/release/package inputs that consume that authority.

## Complexity Tracking

No constitution violation requires an exception.
