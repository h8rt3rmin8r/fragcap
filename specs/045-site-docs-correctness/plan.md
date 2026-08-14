# Implementation Plan: site/docs correctness

**Branch**: `045-site-docs-correctness` | **Date**: 2026-08-14 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/045-site-docs-correctness/spec.md`

## Summary

Fix four site-only defects on the documentation website, all under `site/`:
correct the profile-format docs from TOML to JSON (#115), replace the verbatim
`eso` slug and its hedging sentence with typed placeholders keeping one concrete
example (#116), flip the two horizontal architecture Mermaid diagrams to vertical
so they fit the content column (#120), and render the site footer inside the docs
content flow so it stops being detached below the full-viewport docs layout
(#112). No Rust, schema, CLI, or fixture change. Verification is the site
production build plus a visual check of the affected pages.

## Technical Context

**Language/Version**: TypeScript / TSX and MDX content. Node via pnpm 9.

**Primary Dependencies**: Next.js 16.3.0, fumadocs-ui 16.14.3 (DocsLayout,
DocsPage, HomeLayout), fumadocs-mdx, mermaid 11 (rendered by fumadocs). No
dependency is added or changed.

**Storage**: N/A (static content site).

**Testing**: `cd site && pnpm install && pnpm build` (Next.js production build)
plus `pnpm dev` visual verification of the affected pages. There is no unit-test
harness for the docs content; the build is the mechanical gate.

**Target Platform**: Static export served at fragcap.com (GitHub Pages).

**Project Type**: Documentation website (Next.js app router + fumadocs).

**Performance Goals**: N/A (content correctness and layout, not performance).

**Constraints**: All edits confined to `site/`. UTF-8, LF, no em or en dashes
(P-8). No new glossary term is introduced (P-6). Footer visual treatment is
preserved (positioning fix, not restyle).

**Scale/Scope**: Two MDX reference/guide pages rewritten to JSON, roughly ten
placeholder substitutions plus one deleted sentence across four MDX files, two
Mermaid direction tokens flipped on one page, and three layout files plus one
component comment touched for the footer. One changelog fragment.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- **P-1 Passive Observation**: No capture or process behavior changes. PASS.
- **P-2 Core Platform-Neutral**: `fragcap-core` (and all Rust) untouched. PASS.
- **P-5 Compatibility Outranks Richness**: Not engaged; docs-only. N/A.
- **P-6 Glossary First**: No new term; existing terms (profile, stage, extcap)
  are used as already defined. PASS.
- **P-8 House Standards**: UTF-8, LF, no em/en dashes in all edited content and
  code. Enforced during implementation. PASS.
- **P-9 The Instrument Does Not Lie**: This slice is the correctness fix for
  documentation that currently describes a format the tool no longer accepts and
  labels an example ambiguously; it brings the docs back into truth. PASS.

No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/045-site-docs-correctness/
├── spec.md              # /speckit-specify output
├── plan.md              # this file
├── research.md          # Phase 0: the two design decisions
├── quickstart.md        # Phase 1: the verification guide
├── checklists/
│   └── requirements.md  # spec-quality checklist (all passing)
└── tasks.md             # /speckit-tasks output (next command)
```

data-model.md and contracts/ are intentionally omitted: a documentation and
layout slice defines no new data entities and exposes no machine interface. The
"entities" in the spec are documentation artifacts, not a data model.

### Source (paths touched, all under site/)

```text
site/content/docs/reference/profile-schema.mdx     # #115: TOML -> JSON prose + example
site/content/docs/guides/writing-a-profile.mdx     # #115 + #116: JSON examples, .toml->.json, eso->placeholders
site/content/docs/getting-started.mdx              # #116: delete hedging sentence, eso->placeholder
site/content/docs/reference/cli.mdx                # #116: keep the ONE concrete eso example under --profile
site/content/docs/guides/capture-modes.mdx         # #116: eso->placeholder (3 commands)
site/content/docs/architecture.mdx                 # #120: two flowchart LR -> TD (lines 18, 78)
site/app/layout.tsx                                # #112: remove body-level <Footer/>
site/app/(home)/layout.tsx                         # #112: render <Footer/> below home children
site/app/docs/[[...slug]]/page.tsx                 # #112: render <Footer/> inside DocsPage, after DocsBody
site/components/footer.tsx                          # #112: update the now-stale placement comment
changelog.d/045-site-docs-correctness.*.md         # changelog fragment
```

**Structure Decision**: The four issues share the `site/` tree and one build, so
they ship as one slice. #115 and #116 co-edit `writing-a-profile.mdx`, so those
edits are made together. The footer fix is layout-file-local and independent of
the content edits.

## Design decisions (see research.md for rationale)

- **Mermaid direction**: change `flowchart LR` to `flowchart TD` on the two wide
  diagrams (architecture.mdx lines 18 and 78); leave the already-`TD` diagram
  (line 102) untouched. `TD` matches the page's existing vertical diagram.
- **Footer placement**: remove the body-level `<Footer/>` from `app/layout.tsx`;
  render it inside `app/(home)/layout.tsx` for the home group (unchanged
  appearance) and as the last child of `<DocsPage>` after `</DocsBody>` in the
  docs route (in the docs content column, in scroll flow). This keeps exactly one
  footer per page and preserves fumadocs' built-in prev/next pager slot. The
  DocsPage `footer` prop is the pager slot (a `FooterOptions` for next/previous
  items), so the site footer is added as content rather than by overriding that
  slot.
- **JSON reference**: the rewritten profile examples follow
  `docs/schema/target-schema.v1.json` and the fixture
  `crates/fragcap-profile/tests/fixtures/schema/profile-valid.json` (read-only
  references; not edited), including the top-level `kind` key the current doc
  omits. Doc key tables are reconciled to the schema while converting.

## Complexity Tracking

No constitution violations; no entries.
