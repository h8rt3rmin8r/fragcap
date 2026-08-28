# Quickstart: Site Discovery And Recovery

## 1. Install the locked site dependencies

From `site/`:

```powershell
pnpm install --frozen-lockfile
pnpm exec playwright install chromium
```

The dependency install reuses the existing lock graph; S096 adds no package version.

## 2. Build the production export

From the repository root:

```powershell
cargo xtask docs build
```

Expected: the static search index includes both exact promotion rules, the custom recovery body appears in `404.html`, and `.nojekyll` plus `CNAME` remain present.

## 3. Run the production browser regression

```powershell
pnpm --dir=site test:accessibility
```

Expected:

- `fragcap run` and `fragcap tap` lead to the current command reference and retain later v0.5.0 changelog results;
- the four established current queries retain their current leading routes;
- shallow and nested absent paths return HTTP 404 with one primary landmark and two recovery links;
- the recovery page remains contained at 320 and 1440 pixels;
- the shared skip link and both recovery links activate correctly;
- no browser or console error occurs beyond the asserted main-document 404 diagnostic.

## 4. Run repository parity gates

```powershell
pnpm --dir=site test:unit
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo xtask docs check
cargo xtask ci
git diff --check
```

Review the final diff for issues #266 and #267 only, no generated changelog changes, no new lockfile package, UTF-8 without BOM, LF endings, no Unicode dash characters, and exclusion of `.specify/feature.json` from staging.
