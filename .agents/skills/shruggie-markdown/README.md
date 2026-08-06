# shruggie-markdown

Human-facing overview of the `shruggie-markdown` skill. The skill encodes the
ShruggieTech house style for authoring Markdown documents: single-H1 structure,
the labeled front-matter block, heading spacing, automatic anchors and manual
tables of contents, prose-first density, GFM footnotes, language-tagged code
fences, base64 image embedding, and Mermaid or SVG diagrams. It also encodes a
default-deny HTML policy gated on audience, so the rare HTML construct that
Markdown cannot express (justified text, explicit anchors, inline SVG) is used
only in human-facing documents rendered through a pipeline that honors it. The
standing rules live in `SKILL.md`; the worked examples and the "what renders
where" matrix live under `assets/`; two helper scripts under `scripts/` generate
base64 data-URI image definitions.

This skill is about writing Markdown documents to house style. It is not the
`shruggie-markdown` software product, which shares the name but is unrelated. The
skill does not build, package, or configure that product.
