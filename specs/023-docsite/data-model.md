# Data model: Documentation site

This slice's "data" is the glossary entry structure the linter validates and the
export markers the build asserts. There is no runtime data model.

## Glossary entry (specification section 4.3)

Each entry in a `docs/glossary/<category>.md` page is a term with this shape:

| Field | Required | Notes |
| --- | --- | --- |
| Term heading | yes | H2 on the category page (re-leveled from the interim H3) |
| Also known as | optional | aliases line, when the term has them |
| Blurb | yes | one sentence defining the term |
| Detail paragraph | yes | the explanation |
| Why it matters here | when the entry influenced a design decision | renders as a distinct visual element (a callout) |
| See also | optional | cross-links to other entries, within and across category pages |
| References | yes | primary-source links |

The linter's entry-completeness check (section 4.6 check 1) requires the blurb,
the detail paragraph, and the references on every entry, and the "why it matters
here" note on entries that influence a decision.

## Category (specification section 4.4, amended to eight)

1. Capture and Networking
2. Windows Internals
3. Process and Attribution
4. Anti-Cheat and Security
5. Rust and Tooling
6. Platform and Distribution
7. File and Wire Formats
8. Command Line and Diagnostics (added this slice)

One authored source page per category under `docs/glossary/`, plus the generated
`docs/glossary/index.md`.

## Generated alphabetical index

The index lists every term across all categories, alphabetically, each linking to
its entry anchor on the owning category page. It is generated from the category
pages by `scripts/lint-docs.sh fix` and asserted unchanged by check mode. It has
no authored content of its own; regenerating it must be a no-op on a clean tree.

## Linter check model (specification section 4.6)

| Check | Mode | Failure |
| --- | --- | --- |
| Entry completeness | check | an entry missing blurb, detail, references, or (when decision-influencing) the "why it matters here" note |
| Cross-link resolution | check | a `See also` or in-body link to a non-existent entry anchor |
| Term inventory | check | a term appearing in project documentation with no glossary entry (the Undefined Term Rule) |
| Index reproducibility | check | the committed `index.md` differs from the freshly generated one |
| External URL liveness | link | a references URL that does not respond (weekly, not per-commit) |
| Index regeneration | fix | writes `index.md` from the category pages |

## Search tokenization

The site search indexes at the heading level so each term is an independent
result, and the tokenizer splits on whitespace, underscores, and hyphens, so a
compound identifier (`socket-table`, `path_regex`) is findable by its parts.

## Static export markers (specification section 22.2)

| Marker | Location | Asserted by |
| --- | --- | --- |
| `.nojekyll` | export root | `cargo xtask docs build`, `docs.yml` |
| `CNAME` (fragcap.com) | export root | `cargo xtask docs build`, `docs.yml` |
| no base path | next config | build config; links resolve at apex root |
| images unoptimized | next config | build config |

## Exit contract (all xtask subcommands)

| Code | Meaning |
| --- | --- |
| 0 | ran and passed |
| 1 | ran and failed |
| 2 | could not run (pnpm or bash absent) |
