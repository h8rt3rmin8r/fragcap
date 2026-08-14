# Phase 0 Research: site/docs correctness

Two design decisions needed grounding before implementation; the rest of the
slice is direct content correction. Read-only references were consulted; no
source outside `site/` is edited.

## Decision 1: profile example shape (#115)

**Decision**: Rewritten profile examples are JSON that conforms to the published
schema `docs/schema/target-schema.v1.json`. Every example carries the schema's
required top-level keys `schema`, `kind`, and `fidelity`, and for a profile also
`game` (with `id` and `name`) and a non-empty `stage` array. The canonical
fuller example mirrors the committed fixture
`crates/fragcap-profile/tests/fixtures/schema/profile-valid.json`.

**Rationale**: The schema's top-level `required` is `["schema", "kind",
"fidelity"]`, and the `kind: profile` branch additionally requires `game` and
`stage` (`game.id`, `game.name`, `stage` minItems 1). The current doc examples
are TOML AND under-specified: the "minimal profile" omits `kind` and `fidelity`,
which are mandatory. Converting to JSON without adding those keys would produce a
schema-invalid example, so the reconciliation (adding `kind`/`fidelity` and any
other omitted keys) is part of the same correction, satisfying spec FR-002.

**Minimal valid profile (JSON)** used as the reference-page example (values are
typed placeholders per #116, except where a concrete example is intended):

```json
{
  "schema": 1,
  "kind": "profile",
  "fidelity": "authored",
  "game": { "id": "<game-id>", "name": "<Game Name>" },
  "stage": [
    {
      "role": "client",
      "lifecycle": "session",
      "terminal": true,
      "match": { "exe": "<client>.exe" }
    }
  ]
}
```

The writing-a-profile guide's fuller launcher+client example follows the fixture
structure (transient launcher stage with `path_contains`, terminal session
client stage), with placeholder values. The reference page's key tables are
reconciled against the schema `$defs` (game, capture, stage, match) during
implementation; where the current table and the schema disagree, the schema is
authoritative.

**Alternatives considered**: (a) Convert TOML to JSON verbatim without adding
`kind`/`fidelity` - rejected, produces schema-invalid examples. (b) Show a
maximal example with every optional key - rejected as noisier than a reader
needs; the minimal-valid plus one fuller example is clearer.

## Decision 2: docs footer placement (#112)

**Decision**: Remove the body-level `<Footer/>` from `site/app/layout.tsx`.
Render `<Footer/>` in `site/app/(home)/layout.tsx` (below `{children}`) for the
home group, and as the last child of `<DocsPage>` after `</DocsBody>` in
`site/app/docs/[[...slug]]/page.tsx` for docs pages.

**Rationale**: The detachment is caused by the body-level footer being a flex
sibling of fumadocs' `DocsLayout`, whose container forces `min-height: 100dvh`;
the footer's `marginTop: auto` then parks it a full viewport below the content.
Rendering the footer inside the docs content column (as DocsPage children, after
the body) puts it in the docs scroll flow so it terminates the column directly
under the content. Home pages use `HomeLayout`, which has no forced height and
already renders the footer correctly, so scoping the footer to `(home)` for the
home group keeps that appearance. Splitting the single body-level render into two
in-flow renders keeps exactly one footer per page (spec FR-007).

fumadocs-ui 16.14.3 was inspected: `DocsPage`'s `footer` prop is the built-in
next/previous **pager** slot (type `FooterOptions` with `items`/`component`/
`enabled`), not a general site-footer slot. Adding the site `<Footer/>` as
DocsPage content (rather than overriding that slot) preserves the pager and keeps
the change minimal and reversible.

**Alternatives considered**: (a) Override the DocsPage `footer` slot component
with the site footer - rejected, it replaces the prev/next pager. (b) CSS
override of `--fd-docs-height` / `min-height` on the docs grid - rejected as
fragile against fumadocs internals and upgrades (the issue's least-preferred
option). (c) Keep the body-level footer and neutralize the grid height - same
fragility. In-flow placement is the fumadocs-idiomatic fix.

## Non-decisions (direct corrections, no research)

- **#116 placeholders**: mechanical substitution of `eso` to typed placeholders,
  keeping one concrete example under the CLI `--profile <REF>` reference and
  deleting the hedging sentence. No design choice.
- **#120 Mermaid**: change `flowchart LR` to `flowchart TD` on the two wide
  diagrams (architecture.mdx lines 18, 78); the line-102 diagram is already `TD`.
  `TD` is chosen over `TB` to match the page's existing vertical diagram; both
  render top-down.
