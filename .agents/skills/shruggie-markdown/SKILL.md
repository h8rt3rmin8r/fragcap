---
name: shruggie-markdown
description: Encodes the ShruggieTech house style for authoring Markdown documents (single-H1 structure, the labeled front-matter block, heading spacing, automatic anchors and manual tables of contents, prose-first density, opt-in hard wrapping with wrap-safe line breaks when the operator requests it, GFM footnotes, language-tagged code fences, base64 image embedding, and Mermaid or SVG diagrams). Applies whenever Claude is writing or refactoring a Markdown document to house style. Trigger on phrasings like "write this as a Markdown doc to house style", "format this README", "add a TOC", "embed this image in the Markdown", or "make a Mermaid diagram for this". This skill is the Markdown authoring house style, not the shruggie-markdown software product; do not fire it for software, build, packaging, or release requests about that product.
disable-model-invocation: false
user-invocable: true
when_to_use: Use when writing or cleaning up any Markdown document (README, report, spec, sprint plan, case study, agent context file) to the ShruggieTech standard, or when the operator says "fix the headings", "add footnotes", "make this self-contained", "wrap this at 80", "fix the line length", or "turn this into a Mermaid diagram". Do not use for the shruggie-markdown software product or for non-Markdown output.
---

# Shruggie Markdown

Author Markdown documents to the ShruggieTech house style so output is consistent
across GitHub, editor previews, and HTML or PDF render pipelines. This skill is
knowledge and formatting: once invoked it tells Claude how to structure, format,
and assemble Markdown. The concise standing rules are below; the worked examples,
the edge cases, and the "what renders where" matrix live under `assets/` and load
only when you need them.

This skill is named identically to an unrelated ShruggieTech software product, also
called `shruggie-markdown`. The skill is about authoring Markdown documents to house
style. It is not the software. Do not invoke it for software, build, packaging,
or release work about that product.

## When to Use

Invoke this skill when:

- The user asks to write, format, or clean up a Markdown document to house style
  (README, report, spec, sprint plan, case study, agent context file).
- The user asks for a specific Markdown construct: a table of contents, footnotes,
  an embedded image, a Mermaid diagram, a front-matter metadata block.
- The user asks to refactor an existing Markdown file so it follows the house
  conventions (heading spacing, single H1, language-tagged fences).

Do not invoke this skill for:

- The `shruggie-markdown` software product. Build, packaging, configuration, and
  release requests about the product are out of scope; if the operator means the
  software, say so and stop rather than formatting anything.
- Non-Markdown output (HTML pages, `.docx`, PDF generation). Those have their own
  skills (`shruggie-html`, `shruggie-docs`). This skill may note a render caveat
  for a Markdown document destined for PDF and stop there.
- Retroactively reformatting the existing document corpus. The skill governs new
  authoring, not a sweep of already-shipped files.

## Audience gate (read first)

Every rule below is gated on who reads the output document:

- AI-only documents (sprint plans, agent context files, session reports, spec
  updates consumed by agents) are pure Markdown. HTML is prohibited outright.
- Human-facing documents (READMEs, reports, case studies) are pure Markdown by
  default, and may use the three allowlisted HTML constructs only after confirming
  the document renders somewhere that honors the markup.

When in doubt about the audience, ask. When the document targets more than one
surface, format for the lowest common denominator (see
`assets/renderer-compatibility.md`).

## Wrapping gate (read second)

Hard wrapping is opt-in only. It is never the default and never assumed from
habit or prior sessions. The default for every document is soft wrap: one
logical line per paragraph and one per list item, with no inserted hard
breaks.

Do not hard-wrap a document, especially one destined for GitHub rendering (a
README, PR description, issue body, commit message, or any Markdown that
GitHub's web UI will display), unless the human operator explicitly asks for
it in the current conversation ("wrap at 80", "hard-wrap this", "wrap this for
the linter"). GitHub reflows Markdown prose dynamically at whatever width the
viewport allows; a hard-wrapped source file gains nothing there and instead
renders as short, jagged, hard-to-read lines. Only apply the hard-wrap
mechanics in "Line length and wrapping" below once the operator has opted in
for that specific document.

## House-style rules

### Document structure

- The first line of the rendered body is a single `#` H1, and it is the only `#`
  heading in the document. Everything below is `##` or deeper. A document with a
  machine-read YAML front matter block opens with the `---` block, and the single
  H1 is the first body line after the closing `---`.
- Every heading has exactly one blank line above and one blank line below. The
  only exception is a heading at the top of the file or immediately after a closing
  front matter `---`, which has no blank line above.
- Do not place a `---` horizontal rule directly above or below a heading; GitHub
  already draws a border under H1 and H2. A `---` on the line right after non-blank
  text becomes a setext H2, not a rule, so keep a blank line above any intentional
  rule. Use rules sparingly, only as a true section break in long human-facing
  documents.

### Front-matter field block

Directly under the H1, an optional labeled metadata block (Source, References,
Audience, Author, Date) is encouraged. Keys are bold, values regular weight,
inline backtick code allowed in a value that is a path or literal. Each field is
its own line, so a hard line break is required at the end of each line. Use a
trailing backslash (`\`) as the hard break; it survives whitespace trimming,
linters, and `.gitattributes` normalization. The two-trailing-spaces alternative
renders identically but the repo `CONVENTIONS.md` forbids trailing whitespace and
formatters strip it, which collapses the block. For committed documents, use the
backslash.

Halt-gate: if the operator wants committed documents to use two-trailing-space
breaks, that needs an explicit `CONVENTIONS.md` carve-out. Do not silently add
trailing whitespace to repo files; surface the decision.

### Prose density

Paragraphs are encouraged. Bullets are for genuinely enumerable items (options,
steps, parameters, rules). Explanatory and narrative content is prose, not a list
of fragments. This is a stated house preference.

### Line length and wrapping

Soft wrapping is the default for every document: one logical line per
paragraph and one per list item, with no inserted hard breaks. Hard wrapping
is opt-in only (see "Wrapping gate" above) and activates when the operator
explicitly asks for it in the current conversation ("wrap at 80", "hard-wrap
this", "wrap this for the linter"). Once hard wrap is active, the width is
80 columns by default, measured in source characters, not rendered width;
recognized operator overrides are 100 and 120 columns, selected in plain
language ("wrap at 100", "use a 120-column wrap"). Never hard-wrap Markdown
that will be read on GitHub (READMEs, PR or issue bodies, GitHub-rendered
docs) unless the operator has explicitly opted in for that document; GitHub
reflows prose dynamically and a hard-wrapped file renders as jagged short
lines there instead of the smooth paragraphs GitHub would otherwise produce.

Once hard wrap is active, hard-wrap body prose paragraphs and the text of
list items (bulleted, ordered, and footnote definitions) to the active
width. Never hard-wrap and never merge these constructs:

- ATX headings.
- Fenced code blocks and everything inside them.
- GFM pipe tables: never wrap or reflow a table row or a delimiter row.
- The machine-read YAML front-matter block, if present.
- The labeled front-matter field block (Source, References, Audience, Author,
  Date): each field stays on its own line ending with the trailing-backslash
  hard break.
- Reference-style link and image definitions, and base64 data-URI definition
  lines.
- Unbreakable tokens (URLs, file paths, inline code spans): do not split
  them. If one pushes a line past the width, let that line overflow rather
  than break the token.

Halt-gate: when a table row, fence line, or heading necessarily exceeds the
active width and the operator lints with default MD013, surface that the lint
config needs exceptions (`tables: false`, `code_blocks: false`, or a raised
`heading_line_length`). Do not mangle those constructs to dodge the linter.

When hard wrap is active, continuation-line safety has the highest priority
and overrides the width limit. Insert hard breaks only at spaces between
words; never break inside an inline code span, a `[text](url)` link, an
image, or a URL. Indent every
continuation line of a list item to the item's content column (the marker
width plus its trailing space): two spaces for a `-`, `*`, or `+` bullet, and
three for a single-digit ordered marker such as `1. ` (match the actual
marker width for wider ordered markers). Footnote definition continuations
align under the text after `[^label]: ` by the same rule. Plain paragraph
continuations start at column 0.

After its indentation, a continuation line MUST NOT begin with a sequence
that a CommonMark or GFM parser reads as the start of a block. The forbidden
leading sequences are:

- an unordered list marker: `-`, `*`, or `+` followed by a space or tab;
- an ordered list marker: one or more digits followed by `.` or `)` and then
  a space or tab (a year such as `2016.` is the classic trap);
- an ATX heading: one to six `#` characters followed by a space or tab;
- a blockquote marker: `>`;
- a code fence: three or more backticks or three or more tildes;
- a thematic break or setext underline: a run made only of the characters
  `-`, `*`, `_`, or `=`;
- a table row (`|`), an HTML block (`<`), a link reference definition
  (`[label]:`), or a footnote definition (`[^label]:`).

Leading indentation does NOT neutralize these. An indented `- ` under a
bullet is silently parsed as a nested bullet, and an indented `2016.` becomes
a nested ordered list; both are silent structural corruption, which is the
exact failure this rule exists to prevent. If the natural break at the active
width would place a forbidden token at the start of the continuation line,
move the break earlier so the token stays at the end of the current line,
even if the current line then ends a few characters short of the width.
Safety wins over width. If no safe earlier break exists on that line, leave
the token in place and allow the overflow. The two most common triggers are
the house style's spaced hyphen used as a dash (` - `) and a number or
ordinal followed by a period (`2016.`, `3.`); wrapping near either one
requires this check. Worked examples in `assets/authoring-reference.md`.

### Anchors, links, and tables of contents

GitHub auto-generates an anchor slug for every heading: lowercase, strip every
character that is not a letter, digit, space, or hyphen, then turn spaces into
hyphens (duplicates get `-1`, `-2`). An internal link is `[text](#slug)`. There
is no automatic in-body TOC; a TOC is a manual list of `[Heading](#slug)` links.
Explicit `<a id>` anchors are HTML, audience-gated, and usually unnecessary; for
GitHub-targeted documents rely on the automatic slug and do not hand-roll anchors.

### Footnotes

Use GFM footnotes: a `[^label]` reference in the prose and a `[^label]:`
definition at the bottom. The label is an identifier, not a display number; the
renderer numbers footnotes by order of first reference. Footnotes are a GFM
extension: they render on GitHub and in pandoc, but bare CommonMark shows the
literal `[^label]`, so confirm the target.

### Code fences

Every fenced block declares its language on the opening fence (`bash`,
`powershell`, `json`, `yaml`, `python`, `markdown`, `mermaid`, `text` for no
highlighting, and so on). To show a block that itself contains triple backticks,
fence the outer block with one more backtick than the longest inner run (four for
a normal nested block).

### Images

Prefer a committed image file with a relative path, `![alt](assets/diagram.png)`,
whenever the document must render on GitHub. For a single-file document read or
rendered outside GitHub, embed the image as a base64 `data:` URI using a
reference-style image in the body and the definition at the very bottom of the
file. Generate the definition with the bundled scripts, never by hand. Data-URI
images do not render on GitHub.

### Diagrams

Mermaid is the default; GitHub renders a fenced `mermaid` block natively. Fallbacks
are a committed `.svg` file referenced as an image (renders on GitHub) or inline
`<svg>` (HTML, audience-gated, unreliable on GitHub). Never draw diagrams as ASCII
or box-drawing characters: alignment drifts, agents reproduce them unreliably, and
they are inaccessible. Route every diagram to Mermaid or SVG.

## HTML policy

Default deny. A Markdown document uses pure Markdown unless a specific construct
genuinely requires HTML. Exactly three constructs in this skill require HTML, and
each is gated:

1. Justified text (`<div style="text-align: justify;">`)
2. Explicit heading anchors (`<a id="...">`)
3. Inline SVG (`<svg>...</svg>`)

The gate is audience. For AI-only documents, HTML is prohibited outright. For
human-facing documents, HTML is permitted only for those three constructs, and only
after confirming the document will be rendered somewhere that honors the markup
(see `assets/renderer-compatibility.md`). HTML used for pure decoration is never
permitted. Every HTML example in this skill lives inside a fenced code block; the
skill body itself is pure Markdown.

## Scripts

Generate base64 data-URI image definitions with the bundled scripts rather than
by hand. Both emit the same `[label]: data:<mime>;base64,<blob>` definition for
the bottom of the file and the same `![alt][label]` in-body usage hint.

- Bash: `${CLAUDE_SKILL_DIR}/scripts/encode-image-datauri.sh`
  `-i <image> [-l label] [-a alt] [-o file]`
- PowerShell: `${CLAUDE_SKILL_DIR}/scripts/Encode-ImageDataUri.ps1`
  `-Path <image> [-Label label] [-Alt alt] [-OutFile file]`

## Output hygiene

Every Markdown file the skill writes complies with `CONVENTIONS.md`:

- UTF-8 with no BOM, LF line endings, no trailing whitespace (the front-matter
  backslash break exists precisely so the block needs none), a single trailing
  newline.
- Zero em-dashes and zero en-dashes anywhere, including inside code comments and
  `alt` text. Use parentheses, commas, or standard hyphens.
- Body prose and list text use soft wrap by default (one logical line per
  paragraph or list item); hard-wrap only when the operator explicitly opts
  in for that document, and never for GitHub-bound content unless explicitly
  requested. See "Wrapping gate" and "Line length and wrapping". When hard
  wrap is active, a wrapped continuation line never begins with a Markdown
  block marker.
- No AI rhetorical tropes, especially the "not just X, it's Y" contrast.
- Every fenced code block declares a language.
- For plans, sprint documents, and update logs, sequence sessions chronologically.

## Additional resources

- `assets/authoring-reference.md`: long-form reference with worked examples
  for the single H1, front-matter block, heading spacing, horizontal-rule
  gotchas, anchors and TOCs, prose density, line length and wrapping (the
  opt-in default, the width override set, the spaced-hyphen and year wrap
  traps, and the exemptions), justified text, footnotes, and code fences.
- `assets/images-and-diagrams.md`: base64 data-URI image embedding, Mermaid, SVG
  (committed file and inline), the script invocations, and the no-ASCII rule.
- `assets/renderer-compatibility.md`: the "what renders where" matrix across
  GitHub.com, VS Code preview, pandoc or WeasyPrint, and Claude chat artifacts,
  with the fallback advice for Depends and Partial cells.
- `scripts/encode-image-datauri.sh`: Bash helper that encodes an image as a base64
  data URI and prints (or appends) the Markdown reference-style definition.
- `scripts/Encode-ImageDataUri.ps1`: PowerShell twin with identical output, shaped
  to the ShruggieTech PowerShell standard.
