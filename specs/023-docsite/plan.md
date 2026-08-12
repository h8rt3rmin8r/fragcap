# Implementation Plan: Documentation site

**Branch**: `023-docsite` | **Date**: 2026-08-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/023-docsite/spec.md` (roadmap slice
S18 sub-slice C, specification sections 22, 23, and 4.6).

## Summary

Build the fragcap documentation website and the machinery that keeps it honest.
This slice adds:

1. A Fumadocs on Next.js application in a top-level `site/` directory, managed
   with pnpm, statically exported (export mode, image optimization off, no base
   path) and emitting a `.nojekyll` marker and a `CNAME` file (fragcap.com) into
   the export root. It carries the section 22.3 information architecture: a
   minimal landing page (section 23.1) and documentation for getting started,
   guides, reference, architecture, glossary, and contributing.
2. The glossary split: the interim `docs/glossary.md` becomes one authored source
   page per section-4.4 category under `docs/glossary/`, and the alphabetical
   index at the glossary root is generated, never hand-edited. Specification
   section 4.4 gains an eighth category, "Command Line and Diagnostics", and
   section 22.4's count is reconciled.
3. `scripts/lint-docs.sh`, the documentation linter, built to the ShruggieTech
   Bash standard, with three modes: check (the four section-4.6 checks, run by
   continuous integration), fix (regenerate the index in place), and link
   (external URL liveness, weekly). This turns constitution P-6 from hand-kept
   into mechanically enforced.
4. `cargo xtask docs` replacing the current stub: `docs` (dev server with hot
   reload), `docs build` (the static export continuous integration uses), and
   `docs check` (the linter), all under the 0/1/2 exit contract; the check is
   added to the `ci` aggregate and to `ci.yml`.
5. Real `docs.yml` (build and deploy to GitHub Pages) and `links.yml` (weekly
   link mode) workflows, replacing the failing skeletons; the vendored `brand/`
   kit applied to the site; glossary entries for any new term (P-6); and the
   pinned-artifact and spec-amendment decision fragments.

The load-bearing constraints are P-5 and P-6: the site is authored documentation
that links the master specification rather than mirroring it, and the glossary is
the one body the site and the linter share as a single source, so the linter's
term inventory and cross-link checks are what make the site's navigation aid
trustworthy. Hosting stays GitHub Pages per sections 22.1 and 23.2 (operator
decision); Cloudflare serves DNS only, configured by hand from a runbook after
merge, so no vendor credential enters continuous integration.

## Technical Context

**Language/Version**: TypeScript on Next.js (Fumadocs) for the `site/` app, built
with pnpm on Node (pinned in `docs.yml`); Bash 5.x via `#!/usr/bin/env bash` for
`scripts/lint-docs.sh`; Rust for the `cargo xtask docs` command (workspace MSRV
1.82).

**Primary Dependencies**: none new in Cargo. The `site/` app depends on Fumadocs,
Next.js, and React from the npm registry, isolated under `site/` with a committed
`pnpm-lock.yaml`; `node_modules` is gitignored (and already excluded from the
conventions-linter walk). The xtask command shells to pnpm and to
`scripts/lint-docs.sh`.

**Storage**: none. The site is a static export; the glossary source is Markdown
under `docs/glossary/`.

**Testing**: `scripts/lint-docs.sh check` validates the glossary (entry
completeness, cross-link resolution, term inventory) and index reproducibility;
`cargo xtask docs build` asserts the static export carries `.nojekyll` and
`CNAME` with no base path; `cargo xtask docs check` runs the linter; the linter
passes the repository's Bash compliance checker. The site's live behavior at the
apex domain is tier 2, verified by the operator post-merge from the deployment
runbook, as live capture has been since S09.

**Target Platform**: the built site is static HTML served by GitHub Pages at
fragcap.com. The build runs on the continuous-integration Linux leg; the linter
and xtask command run anywhere bash and the toolchain are present.

**Project Type**: Rust workspace plus a self-contained Node documentation app
under `site/`, plus committed Markdown docs under `docs/`.

**Performance Goals**: not applicable; a static export served from a CDN edge.

**Constraints**: P-5 (compatibility and clarity over richness; authored docs, not
a spec mirror); P-6 (glossary first, now mechanically enforced); P-8 (house
standards: the linter to the ShruggieTech Bash standard, UTF-8/LF, no em or en
dashes, SPDX); the 0/1/2 exit contract; the static-export requirements of section
22.2; the brand discipline of section 23.3.

**Scale/Scope**: one Node app, one Bash linter, one xtask command replacement, two
workflow rewrites plus one ci.yml step, the glossary split (8 category pages plus
a generated index), and a two-line master-spec amendment.

## Constitution Check

*GATE: evaluated before Phase 0 and re-evaluated after Phase 1 design.*

| Principle | Assessment |
| --- | --- |
| P-1 Passive Observation | N/A. The site observes nothing and touches no traffic; it is documentation. `cargo xtask lint` is unaffected. |
| P-2 Core Stays Platform-Neutral | PASS. No crate changes except `xtask` (the `docs` command) and the lint walk excluding the site build outputs; `fragcap-core` is untouched. |
| P-3 Capture And Attribution Separate | N/A. No source and no attributor. |
| P-4 No Silent Loss | N/A. The site counts nothing. |
| P-5 Compatibility Outranks Richness | PASS, and central. The site is authored documentation linking the specification, and the getting-started ordering exists precisely so a reader does not get an empty capture. |
| P-6 Glossary First | PASS, and central. The linter enforces the Undefined Term Rule mechanically; any term this slice introduces gets a `docs/glossary/` entry in the same change. |
| P-7 Wrappers Stay Thin | N/A. No wrapper here; `cargo xtask docs` shells to pnpm and the linter and parses no capture output. |
| P-8 House Standards Apply | PASS, and central. The linter is built to the ShruggieTech Bash standard and held there by the compliance checker; the site source and docs are UTF-8/LF, no em or en dashes. |
| P-9 The Instrument Does Not Lie | N/A. The site alters no observation. |
| Licensing | PASS. npcap is documented as a prerequisite; the site installs, downloads, and bundles nothing. The npm dependencies are the site's own build tooling, not fragcap runtime, and stay under `site/`. |
| Pinned artifacts | ACTION. `.github/workflows/docs.yml`, `links.yml`, and `ci.yml` change, and `scripts/lint-docs.sh` is added under `scripts/**`; recorded as a dated decision. The specification section 4.4 amendment is recorded as a dated decision too. `xtask/**` and `site/**` are not pinned. |

No principle is violated; the Complexity Tracking table is empty.

## Project Structure

### Documentation (this feature)

```text
specs/023-docsite/
├── plan.md              # This file
├── research.md          # Phase 0: decisions, rationale, alternatives
├── data-model.md        # Phase 1: glossary entry model + linter check model
├── quickstart.md        # Phase 1: runnable validation scenarios
├── contracts/
│   ├── docs-linter.md   # lint-docs.sh three-mode contract + the 4.6 checks
│   └── docs-xtask.md     # cargo xtask docs subcommand + export contract
├── checklists/
│   ├── requirements.md  # spec quality (from /speckit-specify)
│   └── docsite.md       # requirements-quality checklist (from /speckit-checklist)
└── tasks.md             # Phase 2 (/speckit-tasks)
```

### Source Code (repository root)

```text
site/                          # NEW: Fumadocs on Next.js app (pnpm), gitignored node_modules
├── package.json               # pinned deps; scripts dev/build
├── pnpm-lock.yaml             # committed lockfile
├── next.config.*             # output export, images unoptimized, no basePath
├── source.config.*           # Fumadocs MDX source config
├── app/                       # landing + docs routes, brand layout
├── content/docs/              # authored MDX (getting-started, guides, reference,
│                              #   architecture, contributing); glossary copied in
│                              #   from docs/glossary at build (single source)
├── public/                    # favicons, web manifest, social preview, CNAME, .nojekyll
└── lib/                       # search tokenizer (whitespace/underscore/hyphen)

docs/
├── glossary/                  # NEW: one .md per section-4.4 category (8 files)
│   └── index.md               # GENERATED alphabetical index (by lint-docs.sh fix)
└── glossary.md                # REMOVED (content migrated into docs/glossary/)

scripts/
└── lint-docs.sh               # NEW: doc linter (ShruggieTech Bash standard), 3 modes

xtask/src/
├── main.rs                    # replace the `docs` stub arm with docs::run; USAGE; ci step
├── docs.rs                    # NEW: dev/build/check dispatch, 0/1/2, shells pnpm + linter
├── wrappers.rs                # extend the checked-file list to cover lint-docs.sh
└── lint.rs                    # exclude site build outputs (.next, out) from the walk

.github/workflows/
├── docs.yml                   # REWRITE: build static export + deploy to Pages (pinned)
├── links.yml                  # REWRITE: weekly link-mode verification (pinned)
└── ci.yml                     # NEW step: cargo run -p xtask -- docs check (pinned)

docs/fragcap-specification.md  # section 4.4 add 8th category; section 22.4 count
changelog.d/S18c-docsite.added.md       # user-facing capability
changelog.d/S18c-docsite.decisions.md   # pinned-artifact + spec-amendment decisions
```

**Structure Decision**: The Node app is quarantined under `site/` so the Rust
workspace is unchanged and `node_modules` (already excluded from the lint walk)
stays out of the diff. The glossary's canonical source is Markdown under `docs/`
(so the conventions linter's existing `.md` walk covers it for encoding and
dashes, and non-site readers still have it); the site copies it into its content
tree at build, holding no second authored copy. The linter is a `scripts/*.sh`
under the same ShruggieTech Bash standard the wrappers use, so the existing
`cargo xtask wrappers` checker covers it once its file list is extended. The
`docs` command follows the `lint`/`deps`/`wrappers` xtask pattern.

## Key design decisions (recorded per autopilot decision policy)

Decided from the constitution, the architecture of record (specification sections
4.2 through 4.6, 22, 23), the vendored brand kit, and the three operator
decisions; reasoning and alternatives are in [research.md](research.md).

- **D1. The site lives in a quarantined `site/` app built with pnpm.** Fumadocs
  on Next.js is the mandated stack (section 22.1). Keeping it under `site/` with a
  committed `pnpm-lock.yaml` and gitignored `node_modules` isolates the JS
  toolchain from the Rust workspace; pnpm is the package manager already present
  on the machine.
- **D2. The static export is configured once (section 22.2).** Next.js `output:
  'export'`, `images.unoptimized: true`, no `basePath`. A postbuild step writes
  `.nojekyll` and `CNAME` (fragcap.com) into the export root, and `cargo xtask
  docs build` asserts both are present, so a missing marker fails the build rather
  than shipping an unstyled site.
- **D3. The glossary source is `docs/glossary/<category>.md`; the index is
  generated.** The interim `docs/glossary.md` is split into one authored file per
  section-4.4 category. The alphabetical index (`docs/glossary/index.md`) is
  generated by `scripts/lint-docs.sh fix` from the category files and is never
  hand-edited; check mode fails on any drift. The site build copies
  `docs/glossary/` into its content tree, so there is exactly one authored copy.
- **D4. Specification section 4.4 is amended to eight categories.** The interim
  glossary already carries "Command Line and Diagnostics" (eight entries), which
  is not among the seven. Rather than force those entries into ill-fitting
  buckets, section 4.4 gains it as a legitimate eighth category and section 22.4's
  count is reconciled in the same edit (operator decision, 2026-08-11). Prior
  slices deferred master-spec edits to release; this one is made in-slice because
  section 22.4 binds the split to section 4.4 and the two must agree for the
  analyze gate to pass. Recorded as a dated decision.
- **D5. The linter re-levels headings and rewrites cross-links during the split.**
  The interim glossary nests terms as H3 under H2 categories; each category page
  makes its terms H2, and the in-file `#anchor` cross-links become cross-page
  links. The linter's cross-link check (section 4.6 check 2) is the guard on this
  mechanical step, so a dangling link fails rather than shipping.
- **D6. `scripts/lint-docs.sh` is the P-6 gate, three modes.** check runs the four
  section-4.6 checks (entry completeness, cross-link resolution, term inventory,
  and, in link mode, external URL liveness) and exits non-zero on failure; fix
  regenerates the index; link verifies external URLs on the weekly schedule. It is
  built to the ShruggieTech Bash standard (shebang line 1, SPDX line 2, strict
  mode, the four-section layout, the `print_help`/`has_cmd`/`safe_run`/`log_*`
  fixtures, UTF-8/LF, no emoji) and passes the repository's Bash checker, whose
  file list is extended to include it.
- **D7. `cargo xtask docs` replaces the stub.** `docs` runs `pnpm --dir site dev`
  (hot reload); `docs build` runs the pnpm build and asserts the export markers;
  `docs check` runs `scripts/lint-docs.sh check`. It returns the 0/1/2 contract
  and exits 2 (could not run) when pnpm or bash is absent, never a false pass,
  matching `neutral`/`msrv`. `docs check` is added to the `ci` aggregate and as a
  named step in `ci.yml`, so P-6 is enforced on every push. The pnpm build itself
  is not run in the `ci` aggregate leg that lacks Node; `docs.yml` owns the build.
- **D8. `docs.yml` and `links.yml` become real.** `docs.yml` uses `setup-node`
  (pinned version) plus pnpm, builds the export, asserts the markers, and on the
  default branch uploads the Pages artifact and deploys with `pages: write` and
  `id-token: write` and a `github-pages` environment; on a pull request it builds
  without deploying. `links.yml` carries a weekly cron running the linter's link
  mode. Both are pinned; the Node version pin is recorded as a dated decision.
- **D9. The lint walk excludes the site build outputs.** `cargo xtask lint`
  already skips `node_modules`; it is extended to skip `.next` and `out` so the
  minified build artifacts (which carry em dashes and CRLF from vendored code) do
  not fail the encoding checks. `xtask` is not pinned. The site's own authored
  source (TypeScript, MDX, CSS) stays under the walk and is held to the encoding
  rules.
- **D10. The brand kit is applied, not re-derived.** Fonts are loaded from
  `brand/fonts/` as local `@font-face` (Space Grotesk display, Geist body, Geist
  Mono code and interface labels), with the OFL 1.1 license texts shipped in the
  site; colors from `brand/tokens/colors.css` with the 80/15/5 ratio, dark first;
  favicons and web manifest from `brand/favicons/`; the 1280x640 social preview;
  and the "A ShruggieTech project" endorsement in Geist Mono, subordinate, in the
  footer only. The landing page holds to section 23.1 minimalism and the whole
  site to the "instrument, not weapon" test (section 23.3).

## Open honesty note (surfaced at the pre-push halt)

The site's real proof is serving at fragcap.com, styled and interactive, over
HTTPS, with deep links resolving, and that cannot happen in continuous
integration: enabling GitHub Pages, setting the custom domain, and editing
Cloudflare DNS are operator actions performed once after merge from the
deployment runbook, and the first deploy only exercises after Pages is enabled.
What this slice proves at tier 1 is that `cargo xtask docs build` produces a
static export carrying the `.nojekyll` marker and `CNAME` with no base path, that
`scripts/lint-docs.sh check` passes on the split glossary and fails on a missing
reference, a dangling cross-link, and an undefined term, that the index is
reproducible, and that `cargo xtask ci` is green with the documentation check
included. The live-domain behavior is verified by hand and the changelog says so,
rather than implying a green deploy. Like `platform.yml`, `docs.yml` is watched to
completion once before being reported as passing.
