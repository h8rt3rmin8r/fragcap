# Quickstart: S213 Validation

## Preconditions

Work on the clean S213 branch. Read [the specification](spec.md), [research decisions](research.md), and [site audit contract](contracts/site-audit-boundary.md). Confirm [GitHub's GHSA record](https://github.com/advisories/GHSA-fc5q-39xc-vjpf) still identifies the advisory as withdrawn. No game, installed product, driver, elevation, or custom test path is needed.

## Negative fixtures first

Run the focused xtask site-audit policy tests before and after implementation. Before the change, the proposed no-exception contract should fail. After the change, baseline workspace YAML passes, while injected `auditConfig.ignoreCves`, `auditConfig.ignoreGhsas`, `audit.ignore`, and `audit: false` each yield a named site-audit-bypass failure. Root flow-map, explicit-key, merge or alias, tag, escaped-key, and standalone-root-sequence canaries must fail as unsupported syntax; bare-CR lines must fail while CRLF and an indentless `packages` list must pass. A harmless comment and nested override must pass. Inject `pnpm.auditConfig` with both CVE and GHSA ignores into a parsed `site/package.json` fixture, and create each prohibited `.npmrc` file in a disposable repository fixture; both gates must refuse them while benign manifest siblings pass. Confirm an audit-step-only pnpm/npm environment override is refused. In the same hosted step, HOME and XDG_CONFIG_HOME must be set to a fresh temporary directory before the unchanged audit command, even if the step initially points them at a repository-controlled config path.

## Offline and site readback

Run `cargo xtask supply-chain` with the final source. Read `site/pnpm-workspace.yaml`, `site/package.json`, and the Git diff: no audit bypass in either source, no root `.npmrc` or `site/.npmrc`, no finite site exception JSON, and no unrelated Rust policy or lockfile change.

From site/, use pinned pnpm 10.26.0 for a frozen-lockfile installation, then run pnpm audit --audit-level low and pnpm run build. On a machine without a preinstalled pinned pnpm, the equivalent commands are:

```powershell
npm exec --yes --package=pnpm@10.26.0 -- pnpm install --frozen-lockfile
npm exec --yes --package=pnpm@10.26.0 -- pnpm audit --audit-level low
npm exec --yes --package=pnpm@10.26.0 -- pnpm run build
```

A stale registry response or distinct advisory is a real red outcome. Do not add a replacement ignore to make the gate green. Local manual commands are a separate evidence tier from the hosted same-step environment and temporary-home boundary; final-head hosted CI must pass after the review fix.

## Repository and publication gates

Run cargo xtask ci in the foreground and watch it to completion. Check final diff, encoding, line endings, and absence of private identifiers. Read current guidance and historical v0.11.2 evidence as separate time-bound claims. PR #642 is already published; resolve its security finding within the owner's two-round Codex cap, push the corrected candidate, and wait for green final-head CI and resolved reviews. Stop for the owner's merge ritual.
