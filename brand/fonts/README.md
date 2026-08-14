# fragcap Fonts

fragcap uses Space Grotesk for display text, Geist for body copy, and Geist
Mono for packet data, code, metadata, and interface labels. WOFF2 files are
provided for web use. Static TTF instances are provided for desktop and design
tools.

## Available weights

| Family | Weights shipped |
| --- | --- |
| Space Grotesk | 500, 700 |
| Geist | 400, 500 |
| Geist Mono | 400 |

Nothing else exists. Asking CSS for a weight that is not in this table makes
the browser synthesise it, which produces a faux bold that prints poorly and
forces outlined glyphs into exported PDFs. Emphasis in body copy resolves to
Geist Medium (500). In mono contexts, carry emphasis with colour.

## Glyph coverage

Geist and Geist Mono have no `U+25A0` BLACK SQUARE and no `U+25C6` BLACK
DIAMOND. Do not type geometric shapes as glyphs in brand material - the
renderer will silently substitute a system serif. Draw them in CSS instead, as
`components.css` does for the status badges.

Geist and Space Grotesk are licensed under the SIL Open Font License 1.1. The
complete license texts are included in `licenses/`.
