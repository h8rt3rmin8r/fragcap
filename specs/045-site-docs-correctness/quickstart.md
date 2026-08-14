# Quickstart / Verification Guide: site/docs correctness

This slice is verified by building the documentation site and visually checking
the affected pages. There is no unit-test harness for MDX content; the build is
the mechanical gate.

## Prerequisites

- Node with pnpm 9 available.
- Run from the site directory.

## Build gate (mechanical)

```bash
cd site && pnpm install && pnpm build
```

Expected: the Next.js production build completes with no errors. A malformed MDX
code fence, a broken JSX import in a layout file, or an invalid Mermaid block
fails the build here.

## Visual verification (`pnpm dev`)

```bash
cd site && pnpm dev
```

Then check each page. Toggle light/dark theme and check desktop and a narrow
(mobile) width where noted.

### #115 - profile format is JSON

- `/docs/reference/profile-schema`: the description and body say JSON, not TOML;
  the minimal profile example is a JSON block carrying `schema`, `kind`,
  `fidelity`, `game`, and `stage`.
- `/docs/guides/writing-a-profile`: every profile example is JSON; both command
  examples name a `.json` profile path (no `.toml`).
- Grep gate: `grep -rin "toml" site/content/docs` returns no profile-format
  prose or code fences (only unrelated matches, if any, are acceptable and should
  be reviewed).

### #116 - one slug, typed placeholders, no hedging

- `/docs/reference/cli`: exactly one concrete `eso` example remains, under the
  `--profile <REF>` flag.
- Grep gate: `grep -rn "eso" site/content/docs` shows exactly one intentional
  `eso` occurrence (the CLI example); no `eso.exe`, `eso.toml`, `id = "eso"`, or
  `--profile eso` elsewhere; the "illustrative, not a shipped profile" sentence
  is gone from getting-started.

### #120 - architecture diagrams fit the column

- `/docs/architecture`: the two formerly-horizontal diagrams render vertically
  and fit the content column; labels are legible in light and dark at desktop and
  mobile widths without horizontal scrolling. The third diagram is unchanged.

### #112 - docs footer in flow

- Any `/docs/*` page: the footer sits directly under the content with no
  full-viewport empty gap; the sidebar and table-of-contents sticky behavior is
  unchanged; exactly one footer renders.
- `/` (home): the footer is visually unchanged from before.

## Done signal

The build passes and every visual check above holds in both themes at both
widths.
