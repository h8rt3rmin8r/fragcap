# Implementation Plan: S152 registry approval and patch release

**Branch**: `release/0.10.1` | **Date**: 2026-09-15 | **Spec**: [spec.md](spec.md)

## Summary

Close #416's verified registry-approval gap using the existing `crates-io` environment and a small repository-owned fail-closed metadata validator, then prepare the v0.10.1 Doctor patch without tagging or publishing. Fresh public GitHub metadata is checked before package certification/release creation and again inside the approval-protected publish job before registry execution. Verification never approves a deployment. Preserve actual published v0.10.0 and all independent acceptance gates.

## Technical Context

**Language/Version**: Rust 1.96.0, MSRV 1.88; existing GitHub Actions Bash release steps.

**Primary Dependencies**: Existing xtask `serde_json`, standard library and existing `gh` on hosted runners. No product dependency or lockfile package change.

**Storage**: Ignored temporary public API JSON, reviewed scrubbed before/after configuration evidence under this slice; existing release notes, changelog and publication record.

**Testing**: Pure protection JSON fixtures and workflow regression tests, `cargo test -p xtask --locked`, `cargo xtask ci`, `cargo xtask msrv`, docs build and hosted package certification. Version-only preparation uses installed cargo-release and existing synthetic golden generators.

**Target Platform**: Portable validator; Linux hosted release jobs; Windows candidate certification on isolated hosted CI.

**Project Type**: Existing ten-crate Rust workspace with repository task runner and documentation site.

**Performance Goals**: At most 1 MiB per supplied protection document, one allowance page of at most 100 entries; an incomplete inventory refuses instead of silently omitting rules.

**Constraints**: No product effects or installed-sensitive-software execution. No new auth credential, environment secret access, automatic approval, tag, publication, direct main push or agent merge. Exact tag/version binding remains the identity gate.

**Scale/Scope**: One existing environment, one exact owner reviewer, one tag `v*` allowance, one patch version; no new deployment platform or generic policy engine.

## Constitution Check

Pre-research and post-design: PASS. P-1 through P-5 retain passive/explicit product boundaries, dependencies, accounting and compatibility. P-6 uses established release/approval vocabulary; add a glossary entry only if new vocabulary is required. P-7 keeps shell orchestration thin and validation in Rust. P-8 applies UTF-8/no-BOM/LF, soft-wrapped Markdown and existing gates. P-9 distinguishes configured protection from deployment approval and candidate source from published bytes. P-10 leaves target authority unchanged. P-11 binds candidate Applies-To to 0.10.1 while actual publication stays 0.10.0. Full spec-kit analyze blocks implementation. Workflow and release-instruction changes get a dated decision fragment assembled by the established release-only changelog procedure.

## Project Structure

The slice carries `spec.md`, both requirements checklists, this plan, `research.md`, `data-model.md`, `contracts/release-guard.md`, `quickstart.md`, `tasks.md`, `analysis.md` and later `verification.md`. Source changes are bounded to `xtask/src/release_guard.rs`, xtask dispatch/ordinary CI, `.github/workflows/release.yml`, release records and candidate identity/golden surfaces already owned by release preparation. No new product crate or runtime effect is introduced.

## Phase 0: Research

The installed plan skill explicitly requests independent research agents. Two read-only agents audit environment API capability and exact version-preparation surfaces while the primary agent authors artifacts. [research.md](research.md) records their findings and primary sources. Required-reviewer and tag-policy mutations use supported GitHub REST operations. Administrator bypass is not in the documented REST or GraphQL mutation schema; any bounded API attempt must be verified by readback and cannot count as success if ignored. A supported settings action, if required, stays operator-owned rather than being silently skipped or allowing a weaker guard.

## Phase 1: Design

The validator reads bounded JSON supplied by commands that fetch fresh public API metadata with `gh api --method GET`. It requires environment name, positive immutable environment identity, exact owner reviewer id/login and user type, `prevent_self_review=false`, explicit `can_admins_bypass=true`, custom-only policy and exactly one `type=tag,name=v*` allowance. It rejects absent or unknown protection classes and partial allowance pages; harmless wait timers remain allowed. Additional service-owned JSON fields do not change the policy. No secrets or metadata body are printed on errors.

Release steps fetch both documents in one shell step and immediately invoke the validator under `set -euo pipefail`; failed reads or checks prevent downstream effects. The identity job gates certification and release creation; the publish job independently re-fetches after normal owner approval or a deliberate owner bypass and immediately before `publish --execute`. Ordinary CI validates workflow wiring and pure negative tests without network policy or deployment approval. See the [contract](contracts/release-guard.md).

## Decision Log

1. Keep owner-only required review but allow self-review because the owner also initiates release workflows; manual approval is still a second action. Reject an unapprovable self-review ban and invented reviewers.
2. Retain administrator bypass and allow only tags matching `v*`; exact `vX.Y.Z` and workspace equality remain independently enforced. Do not allow main branches or broaden the environment.
3. Fail closed on unavailable settings, unsupported bypass configuration and partial inventories. Do not claim configuration from environment name, documentation or a successful request alone.
4. Follow pinned `release/*` branch policy rather than default `codex/`; use version-only cargo-release instead of running either whole release orchestrator. Existing Windows launcher concerns and the Bash orchestrator's missing portable identity updates make direct hidden bounded steps more reliable.
5. Assemble candidate changelog through existing release-only generator, preserving prior history and published markers. Preparation date is not publication proof. Actual publication and field/audit acceptance remain separate.
6. Explicit user push/PR authorization overrides autopilot's default pre-push halt. It does not authorize tag/publication, deployment approval or merge.
7. At 20:00 UTC the operator explicitly rejected disabling administrator bypass. This supersedes the initial autonomous false choice: restore `can_admins_bypass=true`, verify it, revise the exact policy tests and retain normal reviewer gating plus tag-only allowance. Owner bypass remains a deliberate available override and must never be misreported as required-reviewer approval. No agent deployment approval or bypass is authorized.

## Implementation and Verification

First add failing fixture and wiring tests, observe red, then implement minimal validator and release guards. Configure only the existing approved environment surface, read back immutable identity and all effective settings, preserve unrelated settings and record actual results. Prepare versions and embedded assertions, regenerate only owned synthetic output goldens, adjust portable conformance/staged-binary candidate identities and the not-started independent-review template. Add bounded patch highlights and dated decisions, assemble release records, run all local gates watched to completion and then hosted CI/reviews. Do not execute installed Doctor or real games. A remaining settings action is a concrete blocker, not an approval exception.
