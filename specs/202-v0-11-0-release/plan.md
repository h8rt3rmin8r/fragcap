# Implementation Plan: v0.11.0 Direct Route Release

**Branch**: `release/0.11.0` | **Date**: 2026-11-10 | **Spec**: [spec.md](spec.md)

**Input**: Accepted S197 through S201, public v0.10.14, release issue #589 and milestone v0.11.0, and the owner's S202 candidate push and PR authorization with final merge and publication controls retained.

## Summary

Prepare one v0.11.0 source candidate from merged S197 through S201. Align every versioned product and generated authority, assemble all pending release fragments, write short release highlights, and reconcile site and first-use guidance while v0.10.14 remains public. Qualify the candidate locally and on the official PR, then hand it to the owner for merge. Continue S202 after the owner accepts the candidate: tag the exact merge under separate authorization, verify GitHub and registry publication, and update current-public records and Pages.

## Technical Context

**Language/Version**: Rust 1.89 workspace, PowerShell 7 and Bash repository tooling, MDX documentation site.

**Primary Dependencies**: Existing locked product and isolated workspace graphs, cargo-release, repository xtask, official Windows package certification, GitHub release and registry workflow; no planned third-party package addition.

**Storage**: Release fragments, generated goldens and conformance records, release notes, publication metadata, and exact public-file verification receipts; no product-store migration beyond the already merged S201 schema.

**Testing**: Release-preparation dry run, full local repository gates, notes and site validation, supply-chain and dependency policy, Windows final package certification, applicable PR checks, tagged publication checks, and independent public-file and registry readback.

**Target Platform**: Windows 11 product and packages; Windows and Linux source gates; static documentation site.

**Performance Goals**: No production packet-path change in release preparation; preserve S201's already qualified direct-route bounds.

**Constraints**: No candidate-authorized merge, tag, public release, registry approval, current-public marker advance, private-title requirement, third Codex round, or invented security-bot verdict.

**Scale/Scope**: Five merged slices S197 through S201, ten product crates, Windows ZIP/MSI/catalog and three sidecars, existing static documentation, one candidate PR, one later publication and records reconciliation.

## Constitution Check

**Before design**: Passed. P-1 through P-6 keep the merged Capture and Deep Capture behavior unchanged. P-7/P-8 require locked dependency, package, encoding, and documentation checks. P-11 requires the specification and release claims to agree with actual source and public identity. Development workflow requires exact evidence, owner merge, and separately authorized tag.

**After design**: Passed. Candidate, accepted merge, public release, and current markers have separate identities. The release plan neither adds product policy nor treats absent bot output or future field observations as a new gate.

## Project Structure

### Slice records

```text
specs/202-v0-11-0-release/
  spec.md
  plan.md
  research.md
  data-model.md
  contracts/publication.md
  quickstart.md
  checklists/requirements.md
  checklists/release.md
  tasks.md
  verification.md
```

### Existing release authorities

```text
Cargo.toml
Cargo.lock
release.toml
changelog.d/
CHANGELOG.md
release-notes/v0.11.0.md
conformance/
fuzz/Cargo.lock
performance/*/Cargo.lock
docs/fragcap-specification.md
docs/plans/README.md
site/content/docs/
docs/published-release.json
.github/workflows/release.yml
```

**Structure decision**: Use the existing release version tool and generated authorities. Do not rewrite runtime code or introduce a new publication workflow without a concrete failing release check.

## Execution Order

1. Confirm accepted source and public baseline; create tracking for the one public release.
2. Run the release dry run, then build the coherent v0.11.0 candidate and release notes without changing current-public records.
3. Complete local validation, Spec-Kit consistency and convergence, commit, push, official PR, CI, and actual review reconciliation.
4. Hand the exact candidate to the owner for final merge. This is an owner-controlled pause within S202, not a claim that public release is complete.
5. After accepted merge and separate publication authorization, tag and verify the official GitHub and registry results.
6. Reconcile current public records and site through a reviewed change, then perform branch and tracking housekeeping.

## Complexity Tracking

No constitution exception or new runtime dependency is planned.
