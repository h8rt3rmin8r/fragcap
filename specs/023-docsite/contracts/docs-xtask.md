# Contract: cargo xtask docs

Replaces the current stub (which exits 2). The single entry point for the
documentation site, local and in continuous integration (specification section
22.6).

## Invocation

```text
cargo xtask docs          # start the site locally with hot reload
cargo xtask docs build    # produce the static export
cargo xtask docs check    # run the documentation linter
```

## Subcommands

| Subcommand | Action | Exit |
| --- | --- | --- |
| `docs` (no arg) | `pnpm --dir site dev`: dev server, hot reload | passes through the dev server; 2 if pnpm absent |
| `docs build` | `pnpm --dir site build`: static export to `site/out/`, then assert `.nojekyll` and `CNAME` (fragcap.com) present | 0 built and asserted, 1 build failed or a marker missing, 2 pnpm absent |
| `docs check` | `scripts/lint-docs.sh check` | 0 clean, 1 failures, 2 bash absent |

## Exit contract

The 0/1/2 contract matches `lint`/`deps`/`wrappers`:

- 0: ran and passed.
- 1: ran and failed (build error, missing export marker, or linter failure).
- 2: could not run because a required tool (pnpm for build/dev, bash for check) is
  absent. Never a false pass, matching `neutral` and `msrv`.

## Export assertions (docs build)

After the pnpm build, `docs build` confirms the export root `site/out/` contains:

- `.nojekyll` (empty marker; without it the static host strips the asset dir).
- `CNAME` containing exactly `fragcap.com`.

A missing marker is exit 1, so the failure is caught at build time rather than at
deploy.

## Wiring

- `docs check` is added to the `cargo xtask ci` aggregate and as a named step in
  `ci.yml`.
- `docs build` is the build step `docs.yml` runs (the same entry point), so the
  local build and the continuous-integration build are identical.
- The pnpm build is not run in the Rust `ci` aggregate legs (they assume no Node);
  `docs.yml` owns it with a pinned Node.
