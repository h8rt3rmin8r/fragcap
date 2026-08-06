# Authoring Reference

The long-form house-style reference for Markdown structure and prose. `SKILL.md`
states the standing rules; this file carries the worked examples and the edge
cases. Read it when you need the exact behavior of a rule or an example to copy.
Images and diagrams have their own reference in `images-and-diagrams.md`.

## Single H1 and document title

The first line of a rendered document is a level-one heading, and it is the only
`#` heading in the entire document. Everything below it is `##` or deeper.

The one standing exception: a document that carries a machine-read YAML front
matter block (skills, Jekyll, Hugo, and similar) opens with the `---` front
matter block, and the single H1 is the first line of the rendered body after the
closing `---`. The "only one H1" rule still holds for the body.

```markdown
---
title: Release Notes
layout: post
---

# Release Notes

## Summary
```

## Front-matter field list

Directly under the H1, an optional labeled metadata block is encouraged but not
required. Typical fields are Source, References, Audience, Author, and Date. Keys
are bold, values are regular weight, and inline backtick code is allowed in the
value when the value is a path, identifier, or literal.

Each field is its own rendered line. Plain consecutive lines collapse into one
paragraph in Markdown, so a hard line break is required at the end of each field
line. The recommended default is a trailing backslash (`\`), which is a GitHub
Flavored Markdown hard break. It survives trailing-whitespace trimming, linters,
and `.gitattributes` normalization.

```markdown
# Document Title

**Source:** `docs/spec/intake.md`\
**Audience:** Human-facing\
**Author:** ShruggieTech\
**Date:** 2026-06-29
```

The classic alternative is two or more trailing spaces at the end of each field
line, which renders identically. Document it, but know the conflict: the repo
`CONVENTIONS.md` forbids trailing whitespace, and most formatters strip it on
save, which silently collapses the block back into one paragraph. Use the
two-trailing-spaces method only for human-facing documents that are not run
through the repo's whitespace tooling. When the document is committed to this
repo, prefer the backslash.

Halt-gate: if the operator wants committed documents to use the two-trailing-
spaces method, that requires an explicit carve-out in `CONVENTIONS.md` for front-
matter blocks. Do not silently introduce trailing whitespace into repo files.
Surface the decision.

## Heading spacing

Every heading line has exactly one blank line above it and exactly one blank line
below it. The sole exception is a heading at the very top of the file, or
immediately after a closing front matter `---`, which has no blank line above.

Correct:

```markdown
Some closing sentence of the previous section.

## Next Section

The first paragraph of the next section.
```

Incorrect (no blank lines bracketing the heading):

```markdown
Some closing sentence of the previous section.
## Next Section
The first paragraph of the next section.
```

## No manual horizontal rules next to headings

Do not place a `---` horizontal rule directly above or below a heading. GitHub
renders H1 and H2 with a native bottom border, so a manual rule next to them is
redundant and visually doubles the divider. Two gotchas:

- A `---` on the line immediately after a line of non-blank text does not render
  as a horizontal rule. It promotes the text above it into a setext H2. This is
  a frequent source of accidental headings. Keep a blank line above any
  intentional `---`.
- Use horizontal rules sparingly, only as a true section break in long human-
  facing documents, and always with a blank line above and below.

This renders as a setext H2 reading "Not a horizontal rule", not as a rule:

```markdown
Not a horizontal rule
---
```

This renders as an actual horizontal rule:

```markdown
A normal paragraph.

---

The next paragraph.
```

## Heading anchors and internal links

### The automatic slug

GitHub generates an anchor id for every heading automatically. The algorithm:

1. Lowercase the heading text.
2. Strip every character that is not a letter, digit, space, or hyphen. This
   removes punctuation such as periods, colons, parentheses, and ampersands.
3. Replace each remaining space with a hyphen.

Duplicate slugs in the same document get `-1`, `-2`, and so on appended in
document order.

Worked example. The heading `Set Up & Go (v2)` yields the slug `set-up--go-v2`.
The ampersand and the parentheses are removed, and the two spaces that surrounded
the ampersand both become hyphens, which produces the double hyphen.

### Linking to a slug

An internal link is `[visible text](#slug)`:

```markdown
See [the setup section](#set-up--go-v2) for the steps.
```

GitHub does not insert a table of contents into the file body automatically; the
in-UI sidebar TOC is separate. An in-document TOC is a manual list of
`[Heading](#slug)` links:

```markdown
## Contents

- [Overview](#overview)
- [Set Up & Go (v2)](#set-up--go-v2)
- [Troubleshooting](#troubleshooting)
```

### Explicit anchors, and when they are even needed

Explicit anchors are usually unnecessary. They are only worth adding when a table
of contents or internal cross-linking is in play and either the renderer does not
auto-slug, or the heading text is expected to change while the anchor must stay
stable. The explicit-anchor technique is HTML, so it is audience-gated (see the
HTML policy in `SKILL.md`). Place an empty anchor immediately above the heading:

```markdown
<a id="stable-anchor"></a>

## A Heading Whose Text May Change Later
```

The `id` attribute is stripped by some Markdown sanitizers, while `name` is more
widely preserved on `<a>`. On GitHub the automatic slug still works alongside the
manual anchor. The standing recommendation for GitHub-targeted documents is to
rely on the automatic slug and not hand-roll anchors at all.

## Prose density

Paragraphs are encouraged. Do not reduce everything to bullet lists. Bullets are
for genuinely enumerable items (options, steps, parameters, rules). Explanatory
and narrative content is prose. This is a house preference, stated as such: when
the content reads as a sentence or two of explanation, write it as a sentence or
two, not as a fragment with a bullet in front of it.

## Line length and wrap-safe continuations

Hard-wrap prose at 80 characters. List-item continuations indent to the
content column: two spaces after the `-` marker, three after `1.`.

The wrap point matters as much as the wrap width. A continuation line that
begins with `-`, `+`, `*`, `>`, `#`, a digit run followed by `.` or `)`, or a
leading `|` is parsed as block syntax at that position, not as prose. This
bites hardest inside list items, where the continuation indent is exactly
where a nested list marker would sit.

Incorrect. The wrap lands so the continuation begins with a `+` marker, and
the renderer produces a nested bullet reading "grantable flags ...":

```markdown
- Grants are keyed by tenant (`tenant_type` is personal or organization;
  + grantable flags control whether a grant can be delegated onward).
```

Correct. Re-break the line by pulling the previous word down, so the
continuation starts with a plain word:

```markdown
- Grants are keyed by tenant (`tenant_type` is personal or
  organization; + grantable flags control whether a grant can be delegated
  onward).
```

The alternative repair is pushing the offending token up to end the previous
line, and the last resort is a backslash escape (`\+ grantable`), which
renders as a literal `+` but leaves an escape in the source. Prefer the
re-break.

Long unbreakable tokens use the MD013 escape hatch: the linter only flags a
line when whitespace occurs past the limit. Give the token its own
continuation line with nothing after it.

Incorrect. The URL sits mid-line, and the words after it put a space past
column 80 (the example line below is deliberately over-length, so this file
fences it behind an inline markdownlint disable):

<!-- markdownlint-disable MD013 -->

```markdown
The upstream discussion at https://github.com/example/repo/issues/4821#issuecomment-99182 settled the format.
```

<!-- markdownlint-enable MD013 -->

Correct. The URL starts its own line and ends it; the line exceeds 80 but
carries no whitespace past the limit, so MD013 stays quiet:

```markdown
The upstream discussion at
https://github.com/example/repo/issues/4821#issuecomment-99182
settled the format.
```

Never wrap table rows, code fence contents, headings, or link reference
definitions. When one of those must exceed 80 under a default MD013 config,
the fix is lint configuration (`tables: false`, `code_blocks: false`,
`heading_line_length`), not reflowing the construct.

## Justified text

Markdown has no native text justification. Justification requires HTML and is
therefore audience-gated. The blank line after the opening `<div>` and before the
closing `</div>` is required, or the renderer treats the inner content as raw HTML
and does not parse the Markdown inside it:

```markdown
<div style="text-align: justify;">

This paragraph is justified. The blank line after the opening div and before
the closing div is required, or the renderer treats the inner content as raw
HTML and does not parse the Markdown inside it.

</div>
```

Where it renders: the `style` attribute is stripped by GitHub.com and by most
Markdown sanitizers, so justified text does not render on GitHub. It renders in
full HTML pipelines that you control: pandoc to HTML or PDF, WeasyPrint, a
browser opening the converted HTML, and VS Code preview configured to allow it.
The deprecated `<div align="justify">` attribute form is slightly more likely to
survive some sanitizers but is unreliable and is not recommended. Bottom line:
use justified text only in human-facing documents that you render to HTML or PDF
yourself, and never rely on it for GitHub-rendered Markdown.

## Footnotes

GitHub Flavored Markdown footnotes attach a reference in the prose to a
definition collected at the bottom of the document:

```markdown
A claim that needs a source.[^src]

Another point with a longer note.[^longnote]

[^src]: The supporting detail for the first claim.
[^longnote]: A footnote can carry multiple paragraphs. Indent continuation
    lines by four spaces so they stay attached to the footnote.
```

Mechanics: the label (`src`, `longnote`, or a number like `1`) is an identifier,
not a display position. The renderer numbers footnotes sequentially in order of
first reference and collects them in a footnotes section at the bottom with
back-links. Definitions conventionally live at the bottom of the document but may
appear anywhere.

Portability: footnotes are a GFM extension. They render on GitHub and in pandoc.
Core CommonMark renderers without the extension show the literal `[^src]` text,
so footnotes are a "know your target" feature.

## Code fences and language tags

Every fenced code block declares its language on the opening fence. This drives
syntax highlighting and downstream tooling. Common identifiers:

- Shells and config: `bash`, `sh`, `powershell`, `json`, `yaml`, `toml`, `ini`,
  `dockerfile`
- Languages: `python`, `javascript` (alias `js`), `typescript` (alias `ts`),
  `sql`, `css`, `html`
- Markup and meta: `markdown` (alias `md`), `diff`, `text` or `plaintext` for no
  highlighting, `mermaid` for native diagram rendering

GitHub resolves language identifiers through Linguist; the full list lives in
Linguist's `languages.yml`.

The nested-fence rule matters constantly when documenting Markdown about Markdown.
To show a code block that itself contains triple backticks, fence the outer block
with four backticks (or more, always one more than the longest inner run). The
following five-backtick fence wraps a four-backtick block that in turn wraps a
three-backtick block:

`````text
````markdown
```bash
echo "the inner fence is three backticks"
```
````
`````
