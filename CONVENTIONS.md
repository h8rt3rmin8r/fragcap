# Conventions

Mechanical rules that apply to every file in this repository, including
generated ones. They exist so that formatting is never a review topic and
never a source of spurious diffs.

These are enforced by the repository linter in continuous integration, not by
review attention. Constitution principle P-8 makes this file binding.

The governing rules live in `.specify/memory/constitution.md`. This file is
the mechanical layer beneath them.

## File encoding

- UTF-8 without byte order mark.
- LF line endings, on every platform, including Windows.
- No trailing whitespace on any line.
- A single trailing newline at end of file, and exactly one.

`.gitattributes` normalizes line endings on commit and checkout, and
`.editorconfig` applies the rest at edit time. Neither replaces the linter;
they just mean the linter rarely has anything to say.

A byte order mark in a shell script breaks the shebang. Mixed encodings
produce mojibake in captured output that is painful to trace back after the
fact. Both failures are cheap to prevent and expensive to diagnose.

## Prose

**No em-dashes and no en-dashes anywhere.** This includes Markdown prose, code
comments, doc comments, commit messages, string literals, log messages, and
generated output. Use commas, parentheses, or standard hyphens.

Never introduce hard wrapping in Markdown. Keep each prose paragraph and each list item you author or modify on one source line, regardless of length. Display wrapping belongs to the renderer. Existing wrapped prose is migration debt and must not be copied into new work or mechanically rewritten outside the owning change.

When hard line breaks are intentional, use a trailing backslash, never two trailing spaces, because trailing whitespace is stripped. Do not wrap inside a table row, URL, heading, link definition, or fenced code block; let those run long.

Avoid the contrasting rhetorical device ("it's not just X, it's Y"). Avoid
hedging and unsolicited restructuring. State the thing.

Sequence plans, sprint documents, and update logs chronologically.

## Markdown

- One H1 per document, and it is the first line of content.
- One blank line before and after every heading, list, table, and fence.
- Every fenced code block carries a language tag. Use `text` when no language
  applies, never a bare fence.
- Reference-style links for anything used more than once.
- Relative links between repository documents, so they resolve on disk and on
  the site.

The house Markdown authoring standard applies in full for documents that ship
to the documentation site. This section is the subset that binds every
Markdown file regardless.

## Source files

Every source file carries an SPDX identifier as its first line, in the
comment syntax of that language.

```rust
// SPDX-License-Identifier: Apache-2.0
```

```sh
# SPDX-License-Identifier: Apache-2.0
```

Rust follows `rustfmt` with the repository configuration. Clippy runs with
`-D warnings`; a lint is fixed or explicitly allowed at the narrowest possible
scope with a comment explaining why. A blanket `allow` at crate level is a
review failure.

Shell scripts follow the ShruggieTech scripting standards: Bash for `.sh`,
PowerShell 7 for `.ps1`. Both are covered by their own compliance checkers in
continuous integration.

## Naming

- Directories and files: lowercase with hyphens (`flow-attribution.md`).
- Rust follows standard Rust naming; `rustfmt` and Clippy enforce it.
- Feature slice directories: `specs/NNN-slug/`, three digits, zero padded.
- Changelog fragments: `changelog.d/<key>.<section>.md`.

## What is never committed

- Build output of any kind. The documentation site is built by continuous
  integration and deployed as an artifact; `docs/` holds source only.
- The npcap Software Development Kit, or any npcap component. Constitution,
  licensing section.
- Capture files produced by a live run. Captures can carry addresses and, in
  some titles, session identifiers. Test fixtures under `fixtures/` are the
  deliberate exception and are reviewed before they land.
- Machine-local state: per-agent skill symlinks, the spec-kit active-feature
  pointer, editor directories.
- `CHANGELOG.md` edits from a feature branch. Add a `changelog.d/` fragment
  instead; see `changelog.d/README.md`.

## Line length

| Content | Limit |
| --- | --- |
| New or modified Markdown prose | One source line per paragraph or list item |
| Rust | 100 columns (`rustfmt`) |
| Shell | 80 columns |
| Tables, URLs, code fences | no limit |

## Before you commit

- Encoding, line endings, trailing whitespace, final newline.
- No em-dashes or en-dashes, anywhere, including comments.
- SPDX header present on every new source file.
- Every fenced code block has a language tag.
- No mojibake or encoding artifacts in any changed file.
- A `changelog.d/` fragment exists if the change is user-visible.
