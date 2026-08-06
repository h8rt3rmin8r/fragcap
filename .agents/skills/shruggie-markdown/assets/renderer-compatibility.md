# Renderer Compatibility Matrix

This is the honest "know your target" reference behind every audience-gating
decision in this skill. A feature is only safe to use when the target renderer
honors it. When the document targets more than one surface, the safe choice is
the lowest common denominator across all of them.

The columns are the four surfaces ShruggieTech Markdown commonly lands on:
GitHub.com (the README and repo-file reading surface), VS Code's built-in
Markdown preview, a full HTML or PDF pipeline you control (pandoc or WeasyPrint),
and a Claude chat artifact.

| Feature | GitHub.com | VS Code preview | pandoc / WeasyPrint to HTML or PDF | Claude chat artifact |
| --- | --- | --- | --- | --- |
| Two-trailing-space hard break | Yes | Yes | Yes | Usually |
| Backslash hard break | Yes | Yes | Yes | Usually |
| Automatic heading anchor slugs | Yes | No by default | Depends on tool | No |
| Explicit `<a id>` anchor | Partial (id may be stripped) | Depends | Depends on sanitizer | No |
| GFM footnotes | Yes | Depends on extension | Yes (pandoc) | Usually not |
| Mermaid fenced block | Yes | With extension | With filter or plugin | Often not |
| Referenced `.svg` or `.png` file | Yes | Yes | Yes | Depends |
| Inline `<svg>` | Sanitized, unreliable | Often yes | Yes | Depends |
| Base64 data-URI image | No | Yes | Yes | Depends |
| Justified `<div>` (style attr) | No (stripped) | Configurable | Yes | Depends |

## How to read the matrix

Where a cell says Depends or Partial, the standing advice is to fall back to the
GitHub-safe option unless the operator has named a specific non-GitHub target:

- For diagrams, prefer Mermaid; it renders on GitHub and degrades to readable
  source elsewhere.
- For images that must show on GitHub, commit the file and reference it with a
  relative path rather than embedding a `data:` URI.
- For internal links, rely on the automatic heading slug rather than a
  hand-rolled `<a>` anchor.
- For everything cosmetic, prefer plain Markdown.

Reach for the HTML-only constructs (justified text, explicit anchors, inline
SVG) only when the document is human-facing and you have confirmed it renders
through a pipeline that honors the markup. See the HTML policy in `SKILL.md`.
