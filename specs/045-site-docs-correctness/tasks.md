# Tasks: site/docs correctness

**Feature**: 045-site-docs-correctness | **Spec**: [spec.md](spec.md) | **Plan**: [plan.md](plan.md)

All edits are under `site/`. No Rust, schema, CLI, or fixture file is touched.
There is no unit-test harness for MDX content; verification is the site build
plus grep and visual checks (see [quickstart.md](quickstart.md)). The four user
stories are independent except that US1 and US2 both edit
`writing-a-profile.mdx`, so US2's edit to that file follows US1's.

## Phase 1: Setup

- [ ] T001 Install site dependencies for the build gate: run `pnpm install` in `site/` (fumadocs-ui 16.14.3, next 16). No code change; enables `pnpm build` verification.

## Phase 2: Foundational

No foundational/blocking tasks: the four user stories touch disjoint files (except the noted US1/US2 overlap) and each is independently testable.

## Phase 3: User Story 1 - profile docs are JSON, not TOML (#115, P1)

**Goal**: Both profile pages describe and demonstrate the format as JSON matching the published schema (schema, kind, fidelity, game, stage).

**Independent test**: `grep -rin "toml" site/content/docs/reference/profile-schema.mdx site/content/docs/guides/writing-a-profile.mdx` shows no format prose or code fences; every example validates against `docs/schema/target-schema.v1.json`.

- [ ] T002 [US1] Rewrite `site/content/docs/reference/profile-schema.mdx`: change the front-matter `description` and body prose from TOML to JSON; replace the ```toml minimal-profile block with a ```json block carrying `schema`, `kind`, `fidelity`, `game` (`id`, `name`), and a non-empty `stage` array (per research.md); reconcile the page's key tables (game/capture/stage/match) against the schema `$defs`, adding the top-level `kind` and `fidelity` rows the page omits today.
- [ ] T003 [US1] Convert the three ```toml example blocks in `site/content/docs/guides/writing-a-profile.mdx` to ```json (objects/arrays, inline `match` objects), each schema-valid (include `schema`/`kind`/`fidelity`); and change the two command examples' profile paths from `.toml` to `.json`.

## Phase 4: User Story 2 - typed placeholders, one concrete slug, no hedging (#116, P2)

**Goal**: Exactly one generic slug remains (the CLI `--profile` example); every other `eso` is a typed placeholder; the hedging sentence is gone.

**Independent test**: `grep -rn "eso" site/content/docs` shows exactly one intentional occurrence (cli.mdx `--profile` example); no `eso.exe`/`eso.toml`/`id = "eso"`/`--profile eso` elsewhere; the "illustrative, not a shipped profile" sentence is absent.

- [ ] T004 [P] [US2] In `site/content/docs/getting-started.mdx`: delete the hedging sentence ("The `eso` used below is illustrative, not a shipped profile."); change the `--profile eso` command to a `<game-id>` placeholder.
- [ ] T005 [P] [US2] In `site/content/docs/reference/cli.mdx`: keep the single concrete `fragcap run --profile eso ...` example under the `--profile <REF>` flag as the one retained slug (verify it reads clearly as an example value).
- [ ] T006 [P] [US2] In `site/content/docs/guides/capture-modes.mdx`: change the three `--profile eso` commands (file, stream, ring modes) to `<game-id>` placeholders, leaving the mode flags intact.
- [ ] T007 [US2] In `site/content/docs/guides/writing-a-profile.mdx` (after T003): replace `eso`-derived example values with typed placeholders (`id` -> `<game-id>`, `exe` -> `<client>.exe`, profile path -> `<profile>.json`); keep exactly the CLI page's single concrete slug as the only literal `eso`.

## Phase 5: User Story 3 - architecture diagrams fit the column (#120, P2)

**Goal**: The two horizontal diagrams render vertically and legibly; the third is unchanged.

**Independent test**: `/docs/architecture` shows both converted diagrams fitting the content column in light/dark at desktop and mobile widths.

- [ ] T008 [P] [US3] In `site/content/docs/architecture.mdx`: change `flowchart LR` to `flowchart TD` on the two horizontal diagrams (around lines 18 "The pieces" and 78 "The pipeline"); leave the already-`TD` diagram (around line 102) unchanged.

## Phase 6: User Story 4 - docs footer sits in flow (#112, P2)

**Goal**: The footer renders inside the docs content column with no full-viewport gap; home footer unchanged; exactly one footer per page.

**Independent test**: any `/docs/*` page shows the footer directly under content, sidebar/TOC sticky unchanged; `/` footer unchanged; one footer per page.

- [ ] T009 [P] [US4] In `site/app/layout.tsx`: remove the body-level `<Footer/>` render (and its import if now unused) so docs pages no longer inherit the detached sibling.
- [ ] T010 [P] [US4] In `site/app/(home)/layout.tsx`: render `<Footer/>` below `{children}` (import from `@/components/footer`) so the home group keeps its correct footer.
- [ ] T011 [P] [US4] In `site/app/docs/[[...slug]]/page.tsx`: render `<Footer/>` as the last child of `<DocsPage>`, after `</DocsBody>` (import from `@/components/footer`), placing it in the docs content flow while preserving the DocsPage pager slot.
- [ ] T012 [US4] In `site/components/footer.tsx`: update the stale comment (lines ~4-9) that claims the footer is "rendered once at the body level" and that "layouts carry no footer slot", to describe the new home + in-docs-flow placement. No visual/style change.

## Phase 7: Polish & Verification

- [ ] T013 Add a changelog fragment `changelog.d/045-site-docs-correctness.fixed.md` (and a `.changed.md` if the footer placement warrants a separate Changed entry) describing the four documentation/layout corrections, dash-free, UTF-8/LF.
- [ ] T014 Run the build gate in the foreground and watch to completion: `cd site && pnpm build`. Resolve any MDX/JSX/Mermaid build error.
- [ ] T015 Run the grep gates from quickstart.md (no TOML format prose/fences; exactly one `eso`; hedging sentence gone) and confirm each passes.
- [ ] T016 Visually verify all four stories via `pnpm dev` (profile pages JSON; one slug; both diagrams fit light/dark + desktop/mobile; docs footer in flow, home unchanged, one footer per page).
- [ ] T017 Confirm `git status`/`git diff --stat` touches only `site/`, the changelog fragment, and `specs/045-site-docs-correctness/` (no Rust, schema, CLI, or fixture files).

## Dependencies

- T001 before T014 (build needs deps).
- T003 (US1) before T007 (US2) - same file `writing-a-profile.mdx`.
- T002, T008, T009, T010, T011, T012 are mutually independent ([P]) - disjoint files.
- T004, T005, T006 are [P] among themselves and with US1/US3/US4.
- T013-T017 (polish/verify) after all content and layout tasks.

## Parallel execution example

After T001, these can proceed in parallel (distinct files):
`T002` (profile-schema.mdx), `T004` (getting-started.mdx), `T005` (cli.mdx),
`T006` (capture-modes.mdx), `T008` (architecture.mdx), `T009` (layout.tsx),
`T010` ((home)/layout.tsx), `T011` (docs page.tsx), `T012` (footer.tsx).
Then `T003` -> `T007` on writing-a-profile.mdx.

## MVP

User Story 1 (#115) alone is a viable increment: it removes the actively-wrong
TOML instructions. The other three stories layer clarity, legibility, and layout
fixes on top and can ship in the same slice or independently.
