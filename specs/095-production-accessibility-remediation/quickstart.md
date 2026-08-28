# Quickstart: Production Accessibility Remediation

## 1. Install the locked site dependencies

From `site/`:

```powershell
pnpm install --frozen-lockfile
pnpm exec playwright install chromium
```

The browser is test infrastructure and is not included in the static export.

## 2. Build the production export

From the repository root:

```powershell
cargo xtask docs build
```

Expected: the generated changelog completes, the static export builds, and `.nojekyll` plus `CNAME` are present.

## 3. Run the production accessibility regression

```powershell
pnpm --dir=site test:accessibility
```

The Playwright configuration starts `site/scripts/serve-export.mjs` on loopback, derives the route inventory from `site/out`, and checks the contract at 320, 768, and 1440 pixel widths.

Expected:

- every public route exposes one `main-content` landmark and a working first-focus skip link;
- every generated changelog route has no heading-level descent greater than one;
- affected light-theme muted and syntax text has at least 4.5:1 computed contrast;
- the two hydrated architecture diagrams expose distinct expected names;
- the test reports nonzero route, contrast, and diagram populations.

## 4. Run repository parity gates

```powershell
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --locked
cargo xtask docs check
cargo xtask ci
git diff --check
```

Review the final diff for issues #263, #264, #265, and #268 only, exact lockfile changes for the development test dependency, UTF-8 without BOM, LF endings, no Unicode dash characters, and exclusion of `.specify/feature.json` from staging.
