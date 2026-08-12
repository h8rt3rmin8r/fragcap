### Added

- **The documentation website exists** (specification sections 22 and 23,
  roadmap slice S18). It is a Fumadocs static export served from the domain root
  at fragcap.com: a landing page held to section 23.1, a first-run getting
  started guide, guides for writing a profile and choosing a capture mode, a
  reference set (the command line, the profile schema, the output formats), an
  architecture overview, a contributing page, and the glossary rendered as
  browsable pages. Unmodified analyzers were always the compatibility target for
  the output; this is the same principle for the documentation, which reads as
  ordinary static HTML with no server behind it.
- **The single-source glossary renders into the site** (specification section
  22.4). The authored glossary lives once, under `docs/glossary/`; a prebuild
  step renders each category page into the site's content tree, turning the
  kramdown "why it matters here" note into a distinct callout and rewriting each
  relative cross-link into a site route, so a link that resolves on GitHub also
  resolves as a page. The rendered tree is generated at build time and never
  committed, so the two copies cannot drift.
- **Static search over the whole documentation set** (specification section 22,
  FR-009). The search index is exported as a static file the browser downloads
  and searches with no server, indexed by heading so each glossary term is an
  independent result. A query splits on whitespace, underscores, and hyphens, so
  `path_regex` and `5-tuple` find their terms.
- **The brand identity is applied** (specification section 23.3): the vendored
  Space Grotesk, Geist, and Geist Mono faces are served locally with their OFL
  license texts, the Signal Cyan accent sits on a dark-first neutral ground,
  favicons and a web manifest and a social preview are wired into the page head,
  and the footer carries the "A ShruggieTech project" endorsement.
- **`cargo xtask docs build` produces the real export.** The sub-slice S18c-1
  command reported the site application absent and exited 2; the application now
  exists, so `docs build` builds the static export and asserts it carries the
  `.nojekyll` marker and `CNAME`, and `docs` starts the site's dev server.
