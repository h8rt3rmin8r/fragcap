# Implementation Plan: S153 release publication

**Branch**: `codex/s153-release-publication` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md).

## Summary

Publish reviewed merged S152 source a7d24962999d38d7ff130722859d473543864862 as v0.10.1 using existing tag-triggered certification and owner-controlled registry publication. Reconcile verified current published records afterward through a separate human-merge PR. No product code, version bump, release workflow or policy change is planned.

## Technical Context

**Language/Version**: Existing Rust 1.96.0 task runner, workspace/MSRV 1.88; Markdown/JSON/MDX publication records.

**Primary Dependencies**: Existing git, gh, GitHub Actions, crates.io public metadata and repository xtask. No dependency changes.

**Storage**: Ignored target/release-verification-v0.10.1 downloads and public metadata; tracked scrubbed slice evidence and docs/published-release.json.

**Testing**: Existing release-guard/publication-order/identity regressions; cargo xtask ci, cargo xtask spec, cargo xtask notes 0.10.1, package-certification validate-report, site tests and hosted CI. Never local Test-PackageCertification.ps1 or installed product execution.

**Target Platform**: Hosted Windows builds/certification, hosted Linux publication, local headless read-only verification.

**Project Type**: Ten-crate Rust product and documentation site, release/evidence-only slice.

**Performance Goals**: Existing finite source/release jobs and metadata bounds, no polling process loop, no altered performance budgets.

**Constraints**: Preserve immutable existing tags/assets, fresh policy gates, bypass=true, no registry secrets, agent approval/bypass, agent merge or direct main push. Exact source and evidence identity are mandatory.

**Scale/Scope**: One patch, six assets, ten crate versions and fourteen current publication markers.

## Constitution Check

Pre-research and post-design: PASS. P-1 forbids new sensitive product effects; hosted controlled certification is retained. P-2/P-3/P-4/P-5 product contracts unchanged. P-6 uses established terms. P-7 no new shell product logic. P-8 UTF-8/no BOM/LF and soft-wrapped Markdown, hidden Windows launchers. P-9 distinguishes source CI, release publication, documentation PR and independent acceptance; any retries remain disclosed. P-10 unchanged target authority. P-11 existing validator binds candidate version separately from actual publication; markers change only after live verification. Analyze remains blocking.

## Project Structure

The slice contains spec.md, two checklists, plan.md, research.md, data-model.md, contracts/publication.md, quickstart.md, tasks.md, analysis.md and verification.md. Reconciliation owns docs/published-release.json, fourteen CURRENT_RELEASE_SURFACES declared by xtask/src/spec.rs, current applicability prose, docs/maintainers/v0.10.1-release-handoff.md and the chronological docs/plans/README.md entry. Historical v0.10.0 evidence is preserved; only its explicitly current baseline marker changes. Existing .github/workflows/release.yml and xtask validation/publish code are inspected, not modified.

## Phase 0: Research

The installed planning skill requires an independent read-only research agent. Its bounded audit confirms serial identity, package certification, release creation and registry publication; exact artifact names fragcap-windows-x86_64 and windows-package-certification-summary; dependency-ordered resumability and the separate owner-approval boundary. Primary research also reads source workflow, ten-crate ORDER, publication marker validator and S152 immutable source/handoff. See research.md.

## Phase 1: Design

Source selection requires every applicable merged-source workflow success, including final Windows package certification and CodeQL. An annotated v0.10.1 tag binds exactly a7d2496 and is pushed under this user's explicit release authorization. Existing tag pipeline produces fresh hosted-certified bytes once, rechecks their integrity before release creation and enforces the owner gate for registry upload. Preserve the independent package run/report identity and compare report version/source explicitly to the peeled tag, not merely its syntax.

Download the official six assets and certification summary to an ignored, exact version directory. Validate all sidecar SHA-256 values, sizes, canonical report, exact source/version, official=true, complete=true and no findings. Independently query each of the ten public registry version endpoints and require non-yanked 0.10.1. Report green only after all mandatory tag-release jobs and source checks succeed.

Only then update actual publication JSON and the fourteen baseline markers. Rewrite only current applicability paragraphs that otherwise imply S151 remains unreleased; preserve historical S150/S151/S152 narrative. Add a dated decision fragment for the release-handoff update. Validate before pushing a narrow records PR; user owns merge. All received bot feedback is handled without exceeding the existing two-round review budget. No independent acceptance checkboxes close without independent evidence.

## Decision Log

1. Reuse merged S152 release source rather than gate publication on a new S153 PR merge; source is already reviewed and carries the complete patch, while records require actual later publication facts.
2. Explicit release request authorizes tag/push/publication and the bounded reconciliation PR, superseding autopilot's default pre-push pause. It does not authorize owner deployment approval, bypass or merge.
3. Preserve owner review and administrator bypass exactly. If the run waits, ask once for the normal owner decision; never weaken the environment or use an alternative credential path.
4. Reuse existing validation and negative regressions before records edits; no product change means no new production test seam or gate waiver is necessary. Full local/hosted parity still runs for final records.
5. Preserve partial-publication evidence and use failed-job-only registry rerun when warranted. Successful creation is never rerun blindly and immutable tags/assets are never replaced.
6. Checklist prerequisites require plan.md before substantive planning, so only a blank resolved template was staged earlier. This does not skip ordered checklist review or design.

## Implementation and Verification

Tasks order source gates before tag, tag before hosted release, complete public verification before baseline edits, then local gates/commit/PR checks and handoff. The owner may provide one essential environment decision. Installed sensitive software, production Doctor, live capture, real games and real trust effects remain prohibited on the operator machine. The final handoff distinguishes live release completion from the still-human-owned reconciliation merge and independent acceptance.
