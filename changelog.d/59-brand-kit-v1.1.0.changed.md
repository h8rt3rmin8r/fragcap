The vendored brand kit under `brand/` is refreshed to version 1.1.0, a
presentation and completeness pass over the resolved 1.0.0 identity rather than a
re-decision of it. Every brand immutable is unchanged: Geist Mono, Signal Cyan
`#27C7E7`, Capture Orange `#FF5300`, and the dark-first `#050708` ground. The
logo, wordmark, and favicon masters are redrawn as clean filled paths, and the
favicons and social preview are re-rendered from them.

What the refresh adds is deliverable rather than decorative: a Fault color
(`#E9505F` on dark, `#C0293A` on light) for the failure state the earlier palette
had no color for; a semantic token layer (`--fc-bg`, `--fc-fg`, and their kin)
plus a `.fc-light` class so the light reading mode is expressible in code;
`tokens/base.css` and `tokens/spacing.css`; a `components/` set,
`guidelines/index.html`, `styles.css`, a `SKILL.md`, and a measured `VERIFY.md`
whose numbers are re-derived from the shipped files. The version 1.0.0
`--fragcap-*` CSS variable names are retained as aliases of the canonical
`--fc-*` tokens, so nothing that consumed the old names breaks. The documentation
site single-sources both its palette (now including the Fault swatch) and its
logo, favicon, and guide assets from `brand/`, so the site and the kit cannot
drift.
