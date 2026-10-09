# Implementation Plan: S172 v0.10.6 release preparation

**Branch**: `release/0.10.6` | **Date**: 2026-10-09 | **Spec**: [spec.md](spec.md)

## Summary

Prepare a verified local v0.10.6 candidate from merged S171 and the exact two-file dependency patch from #470. Reuse version-only and release assembly tooling, reconcile generated and audited identities, preserve published v0.10.5 records, and commit before the autopilot pre-push handoff.

## Technical Context

**Language/Version**: Pinned Rust toolchain, MSRV 1.88; existing Node/pnpm site tooling.

**Dependencies**: Existing product graph; exact site Next.js 16.3.8 with existing overrides. No new product package.

**Storage**: Versioned source, lockfiles and controlled evidence. No operator data mutation.

**Testing**: Golden suites, HTTPS integration, full `cargo xtask ci`, MSRV/neutral, release/documentation gates and site unit/accessibility checks.

**Target Platform**: Windows product, portable protocol/source tests and static documentation site.

**Project Type**: Ten product crates and xtask plus documentation site.

**Performance Goals**: Preserve existing finite bounds and acceptance thresholds.

**Constraints**: Hidden noninteractive child tooling; no tag/push/publish; UTF-8 without BOM, LF and single final newline.

**Scale/Scope**: Twelve current fragments plus S172 fragments, first-party and generated release identities, controlled evidence and site dependency patch.

## Constitution Check

P-1: No instrumentation, game launch, real capture, trust or operator-history mutation. P-2/P-3: Neutrality and dependency gates unchanged. P-4/P-5/P-9: Loss, protocol and truth assertions unchanged. P-6/P-7: Existing terms and thin wrappers retained. P-8: Full analyzed spec-kit sequence, existing gates and dated release-document decisions. P-10: Target storage unchanged. P-11: Candidate Applies-To 0.10.6 with current publication 0.10.5. Governance: feature branch and local commits; later remote and publication actions retain their authorization boundaries. All pre-design gates pass.

## Project Structure

The slice contains spec, plan, tasks, research, data model, quickstart, contracts, checklists, analysis and verification. Candidate files include root and isolated Cargo lockfiles, sink version assertions, CLI Windows version assertion, generated goldens, native conformance matrix/report, master Applies-To, not-started review template, supply-chain digests, site package/lockfile, release notes, candidate handoff and chronological plans index.

## Decisions

1. Incorporate #470's exact two-file patch locally; external PR stays open until authorized supersession or owner merge.
2. Stale #470 HTTPS test failure is already corrected by S169's TCP_NODELAY and complete pre-handshake request. Validate current source; do not change production from stale evidence.
3. Use the established `release/0.10.6` branch with existing cargo-release configuration and explicit `--no-publish --no-tag --no-push`; do not modify release.toml or run the clean-main wrapper after the spec commit.
4. Use the established release exception `cargo xtask changelog --release 0.10.6 2026-10-09`; inspect once-only chronological assembly. General feature PRs still use fragments.
5. Rebind supply-chain digests only after confirming unchanged third-party Rust versions/features/counts; retain review dates and expiry.
6. Preserve current publication, frozen review candidate, historical notes and measured physical Windows evidence at their exact identities.
7. Local certification contract validation is distinct from final hosted package certification. This kickoff finishes local preparation at the pre-push boundary.

## Implementation Sequence

1. Finish clarify/checklists/research/design/tasks, run blocking analysis and commit the specification gate.
2. Fetch exact #470 head and inspect then restore only site/package.json and site/pnpm-lock.yaml.
3. Preview and execute version-only bump; refresh isolated locks offline; update assertions, conformance, review template and Applies-To; regenerate goldens.
4. Add dated decision/security fragments, snapshot/rebind graph digests, assemble all fragments once and write bounded highlights and candidate handoff.
5. Run focused and full CI parity, MSRV/neutral, static docs and site/browser checks; correct demonstrated failures proportionally.
6. Record evidence, check hygiene, commit cleanly and present exact push command.

## Post-Design Constitution Recheck

All gates pass without exceptions. Candidate and published authorities remain separate; release branch permits only version preparation.

## Complexity Tracking

No new architecture or constitutional exception.
