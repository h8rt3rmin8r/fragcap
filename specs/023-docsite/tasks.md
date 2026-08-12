# Tasks: Documentation site

**Feature**: `specs/023-docsite` | **Branch**: `023-docsite` | Slice S18c

Dependency-ordered, test-driven where the logic is testable. Tier-1 checks (the
linter, the export markers, the xtask contract) land before the site polish.
`[P]` marks tasks that may run in parallel with their siblings.

## Phase A: Glossary split (single source)

- **T001**: Split `docs/glossary.md` into `docs/glossary/<category>.md`, one file
  per section-4.4 category (eight files). Re-level each term from H3 to H2, keep
  the section-4.3 entry structure, and rewrite in-file `#anchor` cross-links into
  cross-page links. Remove `docs/glossary.md`. Update the stale live pointer in
  `docs/plans/README.md` (the "interim home ... S18 splits it" sentence) to name
  the new `docs/glossary/` location. The historical references under `specs/*/`
  and `changelog.d/*` are records of past changes and are left untouched.
- **T002**: Amend `docs/fragcap-specification.md`: add "Command Line and
  Diagnostics" to the section 4.4 category list, and reconcile section 22.4's
  "one page per category" count to eight.
- **T003** [P]: Add glossary entries for any term this slice introduces (static
  export, base path, generated index, and any site term), in the correct category
  page (P-6).

## Phase B: Documentation linter (TDD)

- **T004**: Author `scripts/lint-docs.sh` skeleton to the ShruggieTech Bash
  standard (shebang line 1, SPDX line 2, strict mode + IFS, man-page help, the
  four-section layout, the `print_help`/`has_cmd`/`safe_run`/`log_*` fixtures),
  with `check`, `fix`, `link`, and `--help` mode dispatch and the 0/1/2 contract.
- **T005**: Implement fix mode: generate `docs/glossary/index.md` (alphabetical,
  every term linking to its entry anchor) from the category pages.
- **T006**: Implement check mode: entry completeness, cross-link resolution, term
  inventory, and index reproducibility (fail if the committed index differs from a
  fresh generation). Report every failure, exit 1 on any.
- **T007**: Implement link mode: verify external reference URLs respond; exit 1 on
  dead links; guard so it is skippable when offline (exit 2 could-not-run).
- **T008**: Run `scripts/lint-docs.sh fix` to write the committed index, then
  `scripts/lint-docs.sh check` and confirm exit 0 on the split glossary.
- **T009**: Negative checks: introduce a missing-references entry, a dangling
  cross-link, and an undefined term; confirm check exits non-zero naming each;
  revert.

## Phase C: xtask docs + linter gate wiring

- **T010**: Extend the checked-file list in `xtask/src/wrappers.rs` to include
  `scripts/lint-docs.sh`; confirm `cargo xtask wrappers` reports it compliant.
- **T011**: Add `mod docs;` and `xtask/src/docs.rs`: `docs` (pnpm dev), `docs
  build` (pnpm build + assert `.nojekyll` and `CNAME`), `docs check`
  (`lint-docs.sh check`). Replace the stub arm in `xtask/src/main.rs`, add the
  USAGE line, and return the 0/1/2 contract (exit 2 when pnpm or bash absent).
- **T012**: Add `docs check` to the `cargo xtask ci` aggregate (`xtask/src/main.rs`
  ci sequence).
- **T013**: Exclude `.next` and `out` from the conventions-linter walk in
  `xtask/src/lint.rs` so site build outputs are not linted; add a unit test.

## Phase D: Site application (Fumadocs on Next.js)

- **T014**: Scaffold `site/` (package.json, pnpm-lock.yaml, next config with
  `output: 'export'`, `images.unoptimized: true`, no `basePath`; Fumadocs source
  config). Gitignore `site/node_modules`, `site/.next`, `site/out`.
- **T015**: Postbuild step writing `.nojekyll` and `CNAME` (fragcap.com) into the
  export root; a prebuild step copying `docs/glossary/` into the content tree.
- **T016**: Apply the brand kit: local `@font-face` for Space Grotesk / Geist /
  Geist Mono (ship the OFL 1.1 license texts), color tokens with the 80/15/5
  ratio dark-first, favicons + web manifest + 1280x640 social preview, and the
  "A ShruggieTech project" footer endorsement.
- **T017**: Search tokenizer splitting on whitespace, underscores, and hyphens;
  heading-level indexing; the "why it matters here" callout as a distinct element.
- **T018** [P]: Landing page (section 23.1): one sentence, one worked invocation
  with output, the npcap prerequisite, and the three links, nothing else.
- **T019** [P]: Getting started (section 22.3): first-run ordering (install,
  verify, capture, open), the three expectations before the first capture step,
  the prerequisite fronted.
- **T020** [P]: Guides, reference (CLI, profile schema, output formats),
  architecture, and contributing pages; glossary route rendering the eight
  category pages plus the generated index. Each usage instruction fronts the
  prerequisite.

## Phase E: Workflows (pinned)

- **T021**: Rewrite `.github/workflows/docs.yml`: setup-node (pinned) + pnpm,
  `cargo xtask docs build`, assert markers, upload-pages-artifact + deploy-pages
  on the default branch with `pages: write` + `id-token: write` and a
  `github-pages` environment, build-without-deploy on pull requests.
- **T022**: Rewrite `.github/workflows/links.yml`: weekly cron running the
  linter's link mode; keep `workflow_dispatch`.
- **T023**: Add the `docs check` named step to `.github/workflows/ci.yml`
  (mirroring the `wrappers` step), gated `if: matrix.os == 'ubuntu-latest'`
  because it needs bash. The linter's term-inventory scans the canonical doc set
  (section 4.2: spec, README, architecture docs, website copy), not the process
  artifacts under `specs/` and `changelog.d/`.

## Phase F: Verification and records

- **T024**: Run the foreground gate: `cargo fmt --all -- --check`, `cargo clippy
  --all-targets --all-features -- -D warnings`, `cargo test --workspace --locked`,
  `cargo xtask ci` (with docs check), `cargo xtask neutral`, `cargo xtask msrv`,
  `cargo xtask docs build`, `scripts/lint-docs.sh check`, and a no-em/en-dash scan
  over new files. Read every result.
- **T025**: `changelog.d/S18c-docsite.added.md` (user-facing capability).
- **T026**: `changelog.d/S18c-docsite.decisions.md` (dated): the pinned-artifact
  changes (`docs.yml`, `links.yml`, `ci.yml`, `scripts/lint-docs.sh`), the Node
  version pin, and the section 4.4 amendment.
- **T027**: Commit onto `023-docsite` (conventional message + co-author trailer);
  never stage `.specify/feature.json`. HALT before push with the breakdown.

## Notes

- Tier 2 (the live deploy at fragcap.com, HTTPS, deep-link routing, Cloudflare
  DNS) is operator-verified post-merge from the deployment runbook; it is not a
  continuous-integration gate.
- `docs.yml` and `links.yml` are watched to completion once before being reported
  as passing, like `platform.yml`.
