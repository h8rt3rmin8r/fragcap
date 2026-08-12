### Decisions

**2026-08-11: documentation site foundation (slice S18 sub-slice C, part 1).
Pinned-artifact, specification, and design decisions.**

- **Pinned artifacts changed, recorded here.** This slice adds
  `scripts/lint-docs.sh` (under the pinned `scripts/**`) and a "Documentation
  check" step to `.github/workflows/ci.yml` (a pinned workflow), scoped to the
  ubuntu leg because it needs bash, mirroring the wrapper gate. The site build
  and deploy workflows (`docs.yml`, `links.yml`) are not touched here; they
  deploy the site and land with the site in sub-slice S18c-2.
- **S18c is delivered in two sub-slices under one roadmap id (operator decision,
  2026-08-11).** Part 1 (this change) is the deterministic, fully-verifiable
  foundation: the glossary split, the section 4.4 amendment, the documentation
  linter, and `cargo xtask docs`, all green under `cargo xtask ci`. Part 2
  (S18c-2) is the Fumadocs on Next.js site and the brand application, which need
  a network install and a static build. This keeps a verified, mergeable
  checkpoint out of a large slice; the single v0.2.0 release waits for S18c-2.
- **Specification section 4.4 amended in-slice, against the usual defer-to-release
  convention.** Section 22.4 binds the glossary split to "one page per category
  from section 4.4," and the authored glossary already carried an eighth category
  ("Command Line and Diagnostics", eight entries) not in section 4.4's seven.
  Splitting into eight pages while the spec said seven would be an internal
  contradiction the analyze gate must not ship, so section 4.4 gains the eighth
  category here (operator decision). Section 22.4 references section 4.4 without a
  hardcoded count, so no numeric edit was needed there.
- **Hosting stays GitHub Pages; Cloudflare serves DNS only (operator decision).**
  Sections 22.1 and 23.2 name GitHub Pages and reject a vendor hosting account.
  The domain living on Cloudflare is a DNS fact; no Cloudflare credential enters
  continuous integration and `wrangler` is not a dependency. The Cloudflare DNS
  records and the GitHub Pages settings are an operator runbook applied by hand
  after merge, out of scope for the code slice.
- **The completeness check enforces a definition body, not a references section
  on every entry.** Specification section 4.6 lists a references section among
  entry completeness, but the authored glossary carries references only where a
  primary source exists (14 of 125 entries), and fabricating a reference to
  satisfy the linter would violate P-9. The linter therefore requires a non-empty
  definition body per entry and validates references and the "why it matters
  here" callout where present, rather than mandating them on every entry.
- **The free-text term inventory of section 4.2 is not attempted; the glossary
  graph is.** The linter's `check` guards the glossary's own integrity (entry
  completeness, cross-link and see-also resolution, index reproducibility)
  deterministically. A full undefined-term scan over all prose would need a term
  list the project does not maintain and would false-positive on ordinary
  English; it is left out rather than shipped unreliable.
