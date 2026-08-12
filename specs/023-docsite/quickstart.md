# Quickstart: Documentation site

Runnable validation scenarios for slice S18c. Tier 1 is checkable without a
capture driver, a Cloudflare account, or a live deploy; tier 2 is the operator's
post-merge domain verification.

## Prerequisites

- The Rust toolchain (workspace MSRV 1.82) and `bash`.
- Node and pnpm for the site build (`cargo xtask docs build` and the dev server).

## Tier 1: local, no deploy

### Build the static export

```sh
cargo xtask docs build
```

Expect exit 0 and, in the export root (`site/out/`):

- `.nojekyll` present (empty marker).
- `CNAME` present, containing `fragcap.com`.
- `index.html` for the landing page and `docs/` routes.
- no base-path prefix on internal links; images not routed through an optimizer.

### Run the documentation linter

The scripts stay non-executable (mode 100644, like `scripts/fragcap.sh`) and are
invoked through `bash`, which is how continuous integration and the `wrappers`
gate call them.

```sh
bash scripts/lint-docs.sh check
```

Expect exit 0 on the split glossary. Then introduce, one at a time:

- an entry with no prose blurb or detail (only metadata markers),
- an empty `**References:**` section,
- a `See also` link to a non-existent entry,
- a glossary reference in a canonical document naming an undefined term,

and confirm `check` exits non-zero naming each. Restore, then:

```sh
bash scripts/lint-docs.sh fix
git diff --exit-code docs/glossary/index.md
```

Expect `fix` to regenerate the index and the `git diff` to be empty on a clean
tree (index reproducible).

### Run the docs check and the full gate

```sh
cargo xtask docs check
cargo xtask ci
cargo xtask neutral
```

Expect `docs check` to run the linter and exit 0; `cargo xtask ci` green with the
documentation check included; `neutral` green (or exit 2 if unrunnable).

### Serve locally

```sh
cargo xtask docs
```

Expect the site to start with hot reload; the landing page shows one sentence,
one worked invocation with output, the npcap prerequisite, and the three links,
and nothing else; the glossary shows eight category pages plus the index; search
finds a term by a part of a compound identifier.

## Tier 2: operator, post-merge (from the deployment runbook)

- Enable GitHub Pages (source: GitHub Actions), set the custom domain fragcap.com,
  enable Enforce HTTPS.
- Set the Cloudflare DNS records (apex address records DNS-only, `www` alias
  DNS-only) per the runbook.
- Verify `https://fragcap.com` serves the landing page styled and interactive
  (proves the `.nojekyll` marker worked), `https://www.fragcap.com` redirects to
  the apex, and a deep link under `/docs/...` loads directly.
