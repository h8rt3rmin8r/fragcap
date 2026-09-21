# Implementation Plan: S159 v0.10.2 release

**Branch**: `release/0.10.2` | **Date**: 2026-09-20 | **Spec**: [spec.md](spec.md)

## Summary

Prepare and publish v0.10.2 using the existing reviewed release machinery. Align candidate versions, generated outputs, changelog and highlights, pass local and hosted gates, merge through the owner, push the exact merged tag, wait for protected publication, verify every public file and crate, then reconcile actual publication in a separate records-only pull request.

## Technical Context

**Language/Version**: Rust 2021, development toolchain 1.96.0, MSRV 1.88

**Primary Dependencies**: Existing cargo-release, repository xtask, GitHub Actions, GitHub CLI and crates.io API surfaces

**Storage**: Git release records, six public release files and ten registry version records

**Testing**: Source gates, generated golden checks, package certification, hosted Windows controlled smoke, checksums and public registry verification

**Target Platform**: Portable source validation and GitHub-hosted Windows x86_64 package certification

**Project Type**: Existing ten-crate Rust workspace and documentation site

**Performance Goals**: No runtime performance change; preserve the S128 native performance authority

**Constraints**: No local installed-product execution, real game, real trust mutation, direct main push, agent PR merge, agent deployment approval or unverified publication claim

**Scale/Scope**: One patch candidate, one immutable tag, six files, four release jobs, ten crates and one records-only reconciliation PR

## Constitution Check

*GATE: Passed before research and re-checked after design.*

- **P-1**: Pass. Local work uses source validation only; packaged effects remain on disposable hosted Windows infrastructure.
- **P-2 through P-5**: Pass. No product architecture, dependency direction, loss accounting or output compatibility rule changes.
- **P-6**: Pass. Existing release vocabulary is reused.
- **P-7**: Pass. Existing thin scripts and Rust-owned validators remain unchanged.
- **P-8**: Pass. Full formatting, lint, test, documentation, MSRV, neutral and hosted gates remain required.
- **P-9**: Pass. Candidate, public assets, registry publication and current baseline records remain distinct observed facts.
- **P-10**: Pass. Target ownership is unchanged.
- **P-11**: Pass. Candidate Applies-To moves with 0.10.2, while current published markers move only after verified publication.

## Project Structure

### Documentation (this feature)

```text
specs/159-v0-10-2-release/
├── checklists/
│   ├── release.md
│   └── requirements.md
├── contracts/
│   └── release-publication.md
├── analyze.md
├── data-model.md
├── plan.md
├── quickstart.md
├── research.md
├── spec.md
├── tasks.md
└── verification.md
```

### Candidate surfaces

```text
Cargo.toml
Cargo.lock
CHANGELOG.md
release-notes/v0.10.2.md
docs/fragcap-specification.md
conformance/native-http-tls/
crates/fragcap-sink/
crates/fragcap-cli/tests/
fixtures/golden/
specs/159-v0-10-2-release/
```

### Post-publication record surfaces

```text
docs/published-release.json
docs/maintainers/v0.10.2-release-handoff.md
docs/security/
docs/fragcap-specification.md
docs/fragcap-spec-outline.md
docs/plans/README.md
README.md
CONTRIBUTING.md
specs/159-v0-10-2-release/verification.md
```

**Structure Decision**: Reuse the established v0.10.1 two-phase release pattern. Do not modify release tooling during the release cut unless a verified defect blocks publication.

## Implementation Sequence

1. Commit complete spec-kit artifacts before candidate implementation.
2. Execute the established version-only bump and generation steps on `release/0.10.2`.
3. Author and validate v0.10.2 highlights, candidate documentation and full gates.
4. Push the official release PR, address at most two review rounds and wait for final-head green CI.
5. Ask the owner to merge the release PR.
6. Sync exact merged main, create and push v0.10.2, then monitor all release jobs.
7. Ask the owner for crates.io environment approval only if the workflow waits for it.
8. Verify the public release, six files, checksum sidecars, certification report and ten crates.
9. Create a records-only branch and PR, address reviews, and ask the owner to merge it.

## Complexity Tracking

No constitutional violation or architecture exception is required.
