# Research: Documentation site

Phase 0 decisions, rationale, and rejected alternatives for slice S18c. The
decisions themselves are summarized in [plan.md](plan.md); this file carries the
reasoning.

## Hosting: GitHub Pages, not Cloudflare Pages

**Decision**: static export deployed to GitHub Pages at fragcap.com; Cloudflare
serves DNS only.

**Why**: specification sections 22.1 and 23.2 name GitHub Pages and state the
hosting choice introduces "no vendor account, no billing relationship, and no
service terms beyond those already accepted for source hosting." The operator
confirmed staying on GitHub Pages (2026-08-11). The domain living on Cloudflare
is a DNS fact, not a hosting one: apex address records point at GitHub's Pages
addresses and a `www` alias points at the repository site host.

**Rejected**: Cloudflare Pages via `wrangler`. It would be the natural tool if
hosting moved to Cloudflare, but it contradicts the pinned spec decision, would
require amending sections 22.1 and 23.2 plus `docs/brand/README.md`, and would put
a Cloudflare deploy token into continuous integration. `wrangler` also manages
Workers and Pages, not DNS records, so it is the wrong tool for the DNS-only role
this project actually needs, and it is not installed on the build machine.

## Cloudflare configuration: documented runbook, not automated

**Decision**: the DNS records and Pages settings are relayed to the operator as a
runbook and applied by hand after merge.

**Why**: continuous integration holds no Cloudflare credential (by the hosting
decision), and the DNS records are set once and change rarely. The apex records
must be DNS-only (grey cloud) so GitHub Pages can issue its own certificate and
"Enforce HTTPS" works; proxying the apex interferes with that. This is exactly the
kind of one-time infrastructure step that belongs to the operator, not to a build.

## Glossary source of truth: category Markdown under docs/, index generated

**Decision**: `docs/glossary/<category>.md` (one per section-4.4 category) is the
authored source; `docs/glossary/index.md` is generated; the site copies
`docs/glossary/` into its content tree at build.

**Why**: keeping the glossary as Markdown under `docs/` means the conventions
linter's existing `.md` walk already covers it for encoding and dashes, and a
reader browsing the repository still has the glossary without building the site.
Generating the index (section 22.4) from the category files removes index drift,
and check mode fails on any difference. Copying into the site tree at build avoids
a second authored copy, so P-6's single-source discipline holds.

**Rejected**: authoring the glossary natively as MDX inside `site/content` (a
second copy the repository reader cannot see and the `.md` linter would miss);
and pointing Fumadocs directly at `../docs/glossary` (couples the site config to a
sibling path and complicates the content source). A build-time copy is the
simplest arrangement that keeps one authored source.

## The eighth glossary category

**Decision**: amend section 4.4 to add "Command Line and Diagnostics" as an eighth
category and reconcile section 22.4's count, in-slice.

**Why**: the interim glossary already carries that category with eight authored
entries (CLI and `doctor` surface terms) that do not fit the other seven. The
operator chose to legitimize it rather than force the entries into ill-fitting
buckets. It is amended in-slice, against the usual defer-to-release convention,
because section 22.4 binds "one page per category" to section 4.4, so a split into
eight pages while the spec says seven is an internal contradiction the analyze
gate must not ship. The amendment is two edits (the section 4.4 list and the
section 22.4 count) and is recorded as a dated decision.

## The linter is Bash, sharing the wrappers' standard and checker

**Decision**: `scripts/lint-docs.sh` to the ShruggieTech Bash standard; the
existing `cargo xtask wrappers` checker's file list is extended to cover it.

**Why**: specification section 22.5 names `scripts/lint-docs.sh` built to the
ShruggieTech Bash standard. S18b already authored a Bash structural checker in
`xtask/src/wrappers.rs` (`check_bash`) and hard-coded the two wrapper paths; the
linter is a third `scripts/*.sh` under the same standard, so extending that list
is the whole integration. The linter's own logic (the four section-4.6 checks and
index generation) is text processing over Markdown, which Bash with standard
utilities does without a new dependency.

**Rejected**: writing the linter in Rust (xtask) or Node. The spec names a Bash
script at a specific path, and a wrapper that needs a compiled binary to lint
Markdown is heavier than the job. The three-mode interface (check, fix, link) is
the section-22.5 contract.

## The docs task shells out; the pnpm build is not in the ci aggregate

**Decision**: `cargo xtask docs` shells to pnpm (dev, build) and to the linter
(check); `docs check` is in the `ci` aggregate; the pnpm build is owned by
`docs.yml`, not the `ci` aggregate.

**Why**: `cargo xtask docs` is the single entry point (section 22.6), and the
linter check is cheap and dependency-light, so it belongs in the `ci` aggregate
that runs on every push. The full pnpm build needs Node and network access to the
npm registry, which the Rust `ci` legs do not assume; `docs.yml` runs it with a
pinned Node and is the workflow that watches it to completion. `docs build` exits
2 when pnpm is absent, never a false pass.

## Node version pinning and the lint walk

**Decision**: pin the Node version in `docs.yml`; exclude `.next` and `out` from
the conventions-linter walk.

**Why**: a reproducible build needs a pinned Node (`setup-node` `node-version`),
recorded as a dated decision because the workflow is pinned. The build outputs
carry minified vendored code with em dashes and CRLF that would fail the encoding
checks, so `.next` and `out` join `node_modules` in the excluded set; `xtask` is
not pinned, so this needs no decision fragment of its own.

## Alternatives considered and dropped

- A hosted documentation service (readthedocs, a docs SaaS): rejected by section
  22.1, which requires source, site, and docs in one repository under one license
  with no vendor dependency.
- Bundling the fonts from a CDN: rejected by the static-export and offline
  constraints and by the brand kit shipping the fonts locally under OFL 1.1; the
  site loads them as local `@font-face`.
- A base path (serving from a repository subpath): rejected by section 22.2; the
  site serves from the apex domain root, so no `basePath`.
