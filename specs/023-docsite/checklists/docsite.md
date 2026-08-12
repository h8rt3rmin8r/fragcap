# Documentation site checklist

**Purpose**: Slice-specific quality gates for the fragcap documentation site
**Created**: 2026-08-11
**Feature**: [spec.md](../spec.md)

## Static export correctness (spec 22.2)

- [ ] Export mode enabled; the build emits a fully static site
- [ ] Image optimization disabled (no server-side image handler)
- [ ] No base path configured; links resolve at the apex domain root
- [ ] `.nojekyll` marker present in the export root
- [ ] `CNAME` file present in the export root naming fragcap.com
- [ ] The build asserts the marker and `CNAME` are present (fails otherwise)

## Information architecture (spec 22.3, 23.1)

- [ ] Landing, getting-started, guides, reference, architecture, glossary, and
      contributing sections all exist
- [ ] Landing page: one sentence, one worked invocation with output, prerequisite
      named, three links, nothing else
- [ ] Getting started orders the first run install, verify, capture, open
- [ ] The three expectations precede the first capture instruction
- [ ] The capture-driver prerequisite fronts every usage instruction

## Glossary split (spec 22.4, 4.4)

- [ ] One authored source page per section-4.4 category (eight after amendment)
- [ ] Generated alphabetical index at the glossary root
- [ ] Index reproducible: fix mode leaves the committed index unchanged
- [ ] Term headings re-leveled and every cross-link resolves
- [ ] Heading-level search; tokenizer splits on whitespace, underscore, hyphen
- [ ] "Why it matters here" note renders as a distinct element

## Documentation linter (spec 22.5, 4.6)

- [ ] `scripts/lint-docs.sh` passes the ShruggieTech Bash compliance checker
- [ ] Check mode enforces entry completeness, cross-links, and term inventory
- [ ] Fix mode regenerates the index in place and nothing else
- [ ] Link mode verifies external URLs (weekly, not per-commit)
- [ ] Check mode wired into `cargo xtask ci` and `ci.yml`

## Task and workflow wiring (spec 22.6)

- [ ] `cargo xtask docs` (dev), `docs build` (export), `docs check` (linter)
- [ ] All three under the 0/1/2 exit contract
- [ ] `docs.yml` builds and deploys to GitHub Pages (default branch), builds only
      on a pull request, with the Pages permissions and environment
- [ ] `links.yml` runs the linter's link mode weekly

## Brand and governance (spec 23.3, Q-7, Q-8)

- [ ] Space Grotesk / Geist / Geist Mono applied; OFL license texts shipped
- [ ] Color ratio roughly 80 neutral / 15 cyan / at most 5 orange; dark first
- [ ] Favicons, web manifest, and 1280x640 social preview applied
- [ ] "A ShruggieTech project" endorsement in Geist Mono, subordinate, no combined
      logo
- [ ] "Instrument, not weapon" acceptance test met (no excluded imagery or
      vocabulary; orange scarce; status never by color alone)

## Non-negotiables

- [ ] Any new term carries a glossary entry in the same change (P-6)
- [ ] Pinned-artifact changes (`.github/workflows/**`, `scripts/**`) and the
      section 4.4 amendment recorded as dated changelog decisions
- [ ] UTF-8 without BOM, LF, no em or en dashes, single trailing newline
- [ ] `cargo xtask ci` and `neutral` green in the foreground
