### Decisions

Dated 2026-08-11. Sub-slice S18c-2 of roadmap slice S18 (the documentation
website). Records the pinned-artifact changes, which the constitution requires be
made only with a dated decision.

- **`.github/workflows/docs.yml` is rewritten from the skeleton to a real build
  and deploy.** The skeleton dispatched manually and exited 1 by design, because
  a workflow that runs automatically and fails trains readers to ignore it. There
  is now a site to build: on a pull request it builds the static export and
  asserts the `.nojekyll` and `CNAME` markers without deploying; on the default
  branch it builds and deploys to GitHub Pages through `upload-pages-artifact` and
  `deploy-pages`, with `pages: write` and `id-token: write` and a `github-pages`
  environment. Hosting stays GitHub Pages behind Cloudflare DNS (the runbook is
  operator-run, documented, and uses no Cloudflare token in continuous
  integration), unchanged from the sub-slice S18c-1 decision.

- **`.github/workflows/links.yml` is rewritten to a weekly schedule.** External
  link liveness is a property of the outside world, not of a change, so it runs on
  a Monday cron rather than per commit; `workflow_dispatch` is kept for an
  on-demand run. It runs `bash scripts/lint-docs.sh link`, which exits 2 (could
  not run) when curl is absent and 1 on a dead link.

- **Node and pnpm are pinned in `docs.yml`: Node 24, pnpm 9.15.** These match the
  toolchain the committed `pnpm-lock.yaml` and every local build were produced
  with, so a Pages build cannot diverge from what was verified locally. `pnpm
  install --frozen-lockfile` fails rather than resolving a different graph.

- **`ci.yml` is unchanged.** Its `docs check` step landed with sub-slice S18c-1
  and already gates the glossary linter on every push; the website build is not a
  continuous-integration gate but a deploy pipeline, so it lives in `docs.yml`.

- **The glossary is rendered into the content tree at build time, not committed.**
  `docs/glossary/` is the single source; `site/scripts/prebuild.mjs` renders it
  into `site/content/docs/glossary/`, which is gitignored and excluded from the
  conventions linter (it is linted at its source). A committed second copy would
  be a drift surface for no benefit.

- **Static search uses the framework's built-in engine, no custom tokenizer.**
  The default engine already indexes by heading and splits queries on
  underscores and hyphens, which is what FR-009 asks for; this was verified
  against `path_regex` and `5-tuple` rather than assumed. A hand-rolled tokenizer
  would add a maintenance surface to reproduce behavior that already holds.

- **`docs.yml` and `links.yml` are watched to completion once before being
  reported as passing**, like `platform.yml`. Neither has run against a real site
  before this slice.
