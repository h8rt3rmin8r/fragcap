# Implementation Plan: S167 dependency alert reconciliation

**Branch**: `codex/s167-dependency-alert-reconciliation` | **Date**: 2026-10-06 | **Spec**: [spec.md](spec.md)

## Summary

Update the four isolated historical proxy spike lockfiles and the documentation site dependency inventory to remove every version named by the eight current Dependabot alerts. Preserve the reviewed product workspace graph and release state, validate the affected projects, and present one official PR with final-head CI and review evidence.

## Technical Context

**Language/Version**: Existing Rust workspace and Node.js site using pnpm 10.26.0.

**Primary Dependencies**: Rustls in four isolated Cargo.lock files; source-map-js, KaTeX, and DOMPurify in the site pnpm lockfile. Mermaid owns the site KaTeX and DOMPurify edges; Tailwind and PostCSS own source-map-js edges.

**Storage**: Tracked lockfiles and site package metadata only. No runtime data migration.

**Testing**: Exact lockfile version assertions, frozen site install, site build and existing tests, supported locked spike checks, `cargo xtask ci`, hosted PR workflows.

**Target Platform**: Existing Windows development and hosted Linux/Windows gates.

**Project Type**: Dependency maintenance across historical research projects and public documentation site.

**Performance Goals**: No product runtime change.

**Constraints**: Preserve ten product crate versions, root Cargo.lock, release records, and existing dependency policy. Do not weaken advisory or license gates.

**Scale/Scope**: Eight alerts in five tracked lockfiles; one S167 PR. Existing PR #455 overlaps four lockfiles.

## Constitution Check

P-1 through P-7 and P-10 preserve passive/active capture boundaries because no product code or capture behavior changes. P-8 requires Spec Kit analysis and ordinary format, lint, test, docs, and supply-chain gates. P-9 requires exact evidence for the resolved dependency versions and truthful alert status. P-11 keeps the current released identity separate from this unshipped maintenance change. No principle needs an exception.

## Project Structure

```text
specs/167-dependency-alert-reconciliation/
  spec.md
  plan.md
  research.md
  data-model.md
  quickstart.md
  checklists/
  tasks.md
site/
  package.json
  pnpm-lock.yaml
spikes/native-proxy/Cargo.lock
spikes/native-proxy/audit/Cargo.lock
spikes/http-mitm-proxy/Cargo.lock
spikes/http-mitm-proxy/audit/Cargo.lock
changelog.d/
```

**Structure Decision**: Keep each dependency update in its owning manifest and lockfile. No shared product code or new policy layer is warranted for fixed-version maintenance.

## Decisions

1. Combine the eight findings in one S167 branch. The existing Dependabot PR #455 covers only Rustls, so merge-ready S167 will incorporate its exact useful lockfile changes and identify the bot PR as superseded. This avoids two independently mergeable paths for the same files.
2. Use the package managers to produce lockfile changes, then inspect the complete diff and lockfile graph. Mermaid 11.17.2 and 12.0.0 still declare KaTeX `^0.16.47`, so the site needs an explicit KaTeX override to reach the fixed line. Exact site overrides for all three affected packages keep future lock regeneration from reintroducing a vulnerable instance. The override belongs in `site/pnpm-workspace.yaml`, which pnpm 10 reads.
3. Keep GitHub alert closure distinct from branch verification because Dependabot scans the default branch after merge. Before merge, prove fixed versions in source and require green hosted gates.
4. Keep root Cargo.lock and released identities unchanged. Any newly required product graph change would be a documented scope deviation before implementation.

## Implementation Sequence

1. Complete Spec Kit research, tasks, and blocking analysis.
2. Establish the failing baseline for the eight package-version assertions.
3. Apply the minimal Rustls and site dependency resolutions, then rerun assertions.
4. Run affected project and repository checks, record exact results, and inspect the final diff.
5. Commit, push, open the S167 PR, resolve hosted CI and review findings within the allowed two rounds, then request owner review and merge.

## Post-Design Constitution Recheck

The design keeps dependency policy intact, adds no product effect, and separates proposed source evidence from default branch alert state. No constitutional exception is needed.
