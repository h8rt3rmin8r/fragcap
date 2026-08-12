### Added

- **The glossary is split into per-category pages with a generated index**
  (specification section 22.4, roadmap slice S18). The interim `docs/glossary.md`
  is now one authored page per section-4.4 category under `docs/glossary/` (eight
  pages, 125 terms), and `docs/glossary/index.md` is a generated alphabetical
  index of every term linking to its definition on the owning category page.
- **The documentation linter `scripts/lint-docs.sh` enforces P-6 mechanically**
  (specification sections 4.6 and 22.5). It has three modes: `check` validates
  entry completeness (every entry carries a definition body), cross-link
  resolution (every internal glossary link resolves to an existing term anchor),
  and index reproducibility (the committed index matches a fresh generation);
  `fix` regenerates the index in place; `link` verifies external reference URLs
  respond (for the weekly schedule). It is built to the ShruggieTech Bash
  standard and passes the repository's Bash compliance checker. Before this,
  constitution P-6 was satisfiable but kept by hand; it is now enforced on every
  push.
- **`cargo xtask docs` is a real command** (specification section 22.6),
  replacing the stub: `docs check` runs the linter, `docs build` produces the
  static export and asserts it carries the `.nojekyll` marker and `CNAME`, and
  `docs` (no argument) starts the site's dev server. Each returns the 0/1/2 exit
  contract. The documentation check is part of `cargo xtask ci` and a step in the
  `ci.yml` workflow. The `docs build` and `docs` subcommands report the site
  application absent and exit 2 until it lands with sub-slice S18c-2.
- **Specification section 4.4 gains an eighth glossary category**, "Command Line
  and Diagnostics", legitimizing the eight CLI and diagnostics terms the glossary
  had already accumulated. The generated index and the per-category split follow
  from it.
