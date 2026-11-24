# Contract: Unfiltered Site Advisory Audit

## Accepted configuration

`site/pnpm-workspace.yaml` retains its package, build, and override configuration with no top-level `auditConfig` or `audit` key. The guard accepts LF or CRLF line breaks and refuses bare CR. Root mapping keys use canonical block YAML with ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*`, optionally followed by whitespace before the colon. An indentless `- ` sequence item is accepted after a preceding root mapping key, as in `packages`; a standalone root sequence is refused. Nested override values and comments remain accepted. The audit workflow installs from the frozen lockfile with pinned Node 24 and pnpm 10.26.0, rejects the prohibited workspace settings, and runs the exact command `pnpm audit --audit-level low`. The job fails on that command's nonzero result.

## Refused configuration

| Controlled input in site/pnpm-workspace.yaml | Offline gate | Hosted pre-audit guard |
| --- | --- | --- |
| `auditConfig.ignoreCves` with any CVE | `site-audit-bypass: top-level auditConfig` | `Site audit bypass configuration is prohibited` before audit |
| `auditConfig.ignoreGhsas` with any GHSA | `site-audit-bypass: top-level auditConfig` | `Site audit bypass configuration is prohibited` before audit |
| `audit.ignore` with any advisory | `site-audit-bypass: top-level audit` | `Site audit bypass configuration is prohibited` before audit |
| `audit: false` | `site-audit-bypass: top-level audit` | `Site audit bypass configuration is prohibited` before audit |
| Root flow mapping, explicit key, merge or alias, tag, or quoted or escaped key | `site-audit-bypass: noncanonical root mapping syntax` | `Noncanonical site workspace root mapping` before audit |
| Bare CR line break, including one placed before a hidden audit key | `site-audit-bypass: bare CR line ending is prohibited` | `Bare CR line ending in site workspace` before audit |
| Standalone root sequence with no preceding mapping key | `site-audit-bypass: noncanonical root mapping syntax` | `Noncanonical site workspace root mapping` before audit |

The guard is limited to this workspace YAML. It refuses noncanonical root syntax rather than claiming to parse every YAML form or enumerate every possible external pnpm configuration. The hosted command must remain unconditional, unfiltered, and unchanged.

## Removed contract

The withdrawn GHSA's finite exception JSON, exact picomatch version guard, CVE ignore, expiry gate, and exception-qualified success message are removed. The locked picomatch package itself is not deleted or upgraded solely by this withdrawal.

## Evidence

The acceptance bundle consists of a clean offline supply-chain run, negative fixtures for the four bypass forms, unsupported root syntax, bare CR, and a standalone root sequence, plus CRLF and indentless `packages` positive fixtures, frozen-lockfile site install, the unfiltered audit output, static export, local repository gate, and final-head hosted CI. A stale registry response or another advisory is a red outcome to resolve, not a reason to add an ignore.
