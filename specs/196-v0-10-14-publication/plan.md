# Implementation Plan: v0.10.14 Publication

**Branch**: `codex/s196-v01014-publication` | **Date**: 2026-11-06 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/196-v0-10-14-publication/spec.md`

## Summary

Publish the already reviewed v0.10.14 source from exact merge `5efb38ee93daff140e1ba79f57eea0a93d5b572f`, preserve normal protected registry approval, independently verify every GitHub and crates.io byte, exercise the public installed product at documented defaults, then advance current-public records through a fully checked, reviewed, and merged pull request. Publication and record reconciliation remain distinct evidence phases but complete in one S196 work slice.

## Technical Context

**Language/Version**: Rust workspace with MSRV 1.88; PowerShell 7 for Windows package and installed-product operations; GitHub Actions YAML and JSON/Markdown release records

**Primary Dependencies**: Existing `xtask` release, notes, package-certification, review-record, and publication validators; GitHub CLI and API; crates.io sparse API and crate archive endpoints; existing signed MSI and ZIP package workflow

**Storage**: Public GitHub release assets, crates.io archives, repository JSON/Markdown current records, and Fragcap's default `C:\Users\<account>\.fragcap` installed state

**Testing**: Exact tag/source inspection, tagged workflow jobs, independent artifact hashes and sidecars, `cargo xtask package-certification validate-report`, registry checksum readback, installed public binary identity and default-path observations, `cargo xtask ci`, isolated advisory checks, site build, PR CI, hosted review, post-merge Pages readback

**Target Platform**: GitHub-hosted Ubuntu and Windows runners plus the current Windows workstation for installed public-package observation

**Project Type**: Windows CLI/desktop package and ten-crate Rust workspace with public documentation site

**Performance Goals**: No new runtime performance behavior; publication operations remain finite and registry propagation retries are bounded

**Constraints**: Tag only the accepted merge; no force tag; no protection bypass; no republishing an accepted crate; no current-marker advance before public readback; no custom installed-test storage; no private target identity or traffic in public records; no more than two hosted review rounds

**Scale/Scope**: One annotated tag, four release jobs, six GitHub files, three checksum sidecars, ten crate archives, one public installed-product observation, one records PR and merge, one Pages deployment, one release issue and milestone

## Constitution Check

*Constitution check: Passed before research and re-checked after design.*

- **P-1**: No target instrumentation or product behavior changes. Installed observation uses only the shipped product and its ordinary explicit modes.
- **P-2/P-3**: No core or dependency-direction changes.
- **P-4/P-9**: Public, installed, failed, skipped, and unavailable observations remain literal and separate. No missing evidence is inferred.
- **P-5**: Published `.fcapng` compatibility remains qualified by existing unmodified TShark checks; S196 does not change formats.
- **P-6**: No new product term is introduced.
- **P-7/P-8**: Existing release wrappers stay thin; all new records follow repository Markdown, PowerShell, encoding, and text-hygiene rules.
- **P-10**: No target creation or storage shape changes.
- **P-11**: The master specification and current-public records advance only after v0.10.14 is demonstrably public.
- **Licensing**: Public packages retain license, notice, SBOM, and third-party notice authorities. Npcap is not redistributed.
- **Workflow**: Full Spec-Kit sequence precedes records implementation. Existing pinned release workflows remain unchanged unless an observed defect requires a separately recorded dated decision.
- **Installed tests**: The public MSI uses default target/session/bundle locations and supported cleanup. Any unavailable exact location is reported as unverified.

Post-design re-check: passed. The publication contract, evidence entities, and validation guide preserve each constitutional boundary without a deviation.

## Project Structure

### Documentation (this feature)

```text
specs/196-v0-10-14-publication/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── publication.md
├── checklists/
│   ├── requirements.md
│   └── publication.md
├── tasks.md
└── verification.md
```

### Repository Records And Existing Automation

```text
.github/workflows/
├── release.yml
└── published-review-candidate.yml

docs/
├── published-release.json
├── fragcap-specification.md
├── maintainers/
│   └── v0.10.14-release-handoff.md
└── security/
    ├── native-product-review-candidate.v1.json
    ├── native-product-review-handoff.md
    └── native-product-review-record.v1.json

site/content/docs/
README.md
CONTRIBUTING.md
CHANGELOG.md
changelog.d/
```

**Structure Decision**: Use the repository's existing tag-driven release workflow and current-record surfaces. S196 adds only its Spec-Kit and verified publication records; it does not introduce another publication mechanism or runtime path.

## Execution Phases

1. Freeze the exact source and create the complete S196 specification, checklists, plan, tasks, and analysis.
2. Create and push the annotated v0.10.14 tag from the accepted merge, then follow the tagged workflow through normal protected approval.
3. Independently download and validate all GitHub release files and crates.io archives.
4. Install and observe the public MSI using default product storage and supported cleanup, filing concrete defects if any.
5. Derive current-public records solely from verified public evidence, validate locally, push the records branch, and open the official PR.
6. Reconcile up to two hosted review rounds and terminal CI, merge the PR under the owner's explicit authorization, and verify main plus Pages.
7. Close #567, complete project and milestone state, remove local slice state, and finish on clean synchronized main.

## Complexity Tracking

No constitutional violation or exceptional architecture is required.
