# Contract: Unfiltered Site Advisory Audit

## Accepted configuration

`site/pnpm-workspace.yaml` retains its package, build, and override configuration with no top-level `auditConfig` or `audit` key. The guard accepts LF or CRLF line breaks and refuses bare CR. Root mapping keys use canonical block YAML with ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*`, optionally followed by whitespace before the colon. An indentless `- ` sequence item is accepted after a preceding root mapping key, as in `packages`; a standalone root sequence is refused. Nested override values and comments remain accepted.

`site/package.json` retains its normal package metadata and may have benign `pnpm` siblings, but the parsed root `pnpm.auditConfig` field is prohibited. `.npmrc` at the repository root and `site/.npmrc` are prohibited. The audit workflow installs from the frozen lockfile with pinned Node 24 and pnpm 10.26.0. In the audit step itself, it rejects direct pnpm/npm configuration overrides, sets HOME and XDG_CONFIG_HOME to a fresh temporary directory, and then runs the exact command `pnpm audit --audit-level low`. The job fails on that command's nonzero result.

## Refused configuration

| Controlled input | Offline gate | Hosted guard before audit |
| --- | --- | --- |
| `auditConfig.ignoreCves` with any CVE | `site-audit-bypass: top-level auditConfig` | `Site audit bypass configuration is prohibited` before audit |
| `auditConfig.ignoreGhsas` with any GHSA | `site-audit-bypass: top-level auditConfig` | `Site audit bypass configuration is prohibited` before audit |
| `audit.ignore` with any advisory | `site-audit-bypass: top-level audit` | `Site audit bypass configuration is prohibited` before audit |
| `audit: false` | `site-audit-bypass: top-level audit` | `Site audit bypass configuration is prohibited` before audit |
| Root flow mapping, explicit key, merge or alias, tag, or quoted or escaped key | `site-audit-bypass: noncanonical root mapping syntax` | `Noncanonical site workspace root mapping` before audit |
| Bare CR line break, including one placed before a hidden audit key | `site-audit-bypass: bare CR line ending is prohibited` | `Bare CR line ending in site workspace` before audit |
| Standalone root sequence with no preceding mapping key | `site-audit-bypass: noncanonical root mapping syntax` | `Noncanonical site workspace root mapping` before audit |
| `site/package.json#pnpm.auditConfig.ignoreCves` or `.ignoreGhsas` | Named site-audit-bypass finding after JSON parsing | Refusal after JSON parsing before audit |
| Root `.npmrc` or `site/.npmrc` | Named site-audit-bypass finding | Refusal before audit |
| Audit-step-only pnpm/npm configuration environment override | Not a repository-file fixture | Refusal in the same hosted step that invokes audit |
| Audit-step HOME or XDG_CONFIG_HOME points to a committed global `rc` or user `.npmrc` | Not a repository-file fixture | Fresh HOME and XDG_CONFIG_HOME replace the supplied paths before audit |

The workspace guard refuses noncanonical root syntax rather than claiming to parse every YAML form. The expanded boundary covers the repository-controlled pnpm inputs and the hosted audit step's environment identified in [the pinned pnpm source review](../research.md#decision-2a-close-the-pinned-pnpm-configuration-gap-found-in-pr-review) and [global-config path review](../research.md#decision-2b-isolate-hosted-global-configuration-paths). It does not claim to vet arbitrary external global configuration on a developer machine. The hosted command must remain unconditional, unfiltered, and unchanged.

## Removed contract

The withdrawn GHSA's finite exception JSON, exact picomatch version guard, CVE ignore, expiry gate, and exception-qualified success message are removed. The locked picomatch package itself is not deleted or upgraded solely by this withdrawal.

## Evidence

The acceptance bundle consists of a clean offline supply-chain run; negative fixtures for workspace bypass forms, unsupported root syntax, bare CR, a standalone root sequence, root-manifest CVE and GHSA ignores, both `.npmrc` paths, direct audit-step environment overrides, and workflow-wiring mutations for the fresh-home boundary; positive CRLF, indentless `packages`, and benign manifest fixtures; frozen-lockfile site install; the unfiltered audit output; static export; local repository gate; and final-head hosted CI and review readback. A stale registry response or another advisory is a red outcome to resolve, not a reason to add an ignore.
