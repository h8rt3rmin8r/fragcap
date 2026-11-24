# Data Model: Withdrawn Site Advisory Cleanup

S213 adds no product data structure or persistent user state. It removes one finite policy record and tightens the pinned pnpm site audit configuration boundary across the inputs pnpm reads.

## Site workspace configuration

- **Authority**: site/pnpm-workspace.yaml.
- **Relevant fields**: `packages`, `allowBuilds`, and `overrides` remain. Any top-level `auditConfig` or `audit` key is refused by both the offline supply-chain gate and the hosted site job. LF and CRLF are accepted; bare CR is refused. Root keys must use canonical block YAML with ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*`, optionally followed by whitespace before the colon. An indentless `- ` list item after a root mapping key is accepted; a standalone root sequence and other unsupported root YAML forms are refused.
- **Invariant**: The workspace supplies no advisory ignore or audit disablement. This is one part of the unfiltered audit boundary, not the complete set of pnpm configuration inputs.

## Site root package manifest

- **Authority**: `site/package.json`, parsed as JSON by both repository and hosted guards.
- **Relevant field**: Pinned pnpm 10.26.0 reads `pnpm.auditConfig` from this root manifest. The field is refused whether it names CVEs, GHSAs, or an empty configuration; unrelated package metadata and benign `pnpm` siblings remain permitted.
- **Invariant**: A JSON escape or whitespace change cannot make an effective `pnpm.auditConfig` key invisible to the guard.

## Other configuration inputs

- **Repository files**: `.npmrc` and `site/.npmrc` are absent and must remain absent for this audit boundary.
- **Hosted environment**: The audit step checks its own direct pnpm/npm configuration variables, then sets HOME and XDG_CONFIG_HOME to a fresh temporary directory before invoking the unchanged audit command. This prevents both direct settings and repository-controlled global `rc` or user `.npmrc` files from changing coverage or verdict.
- **Invariant**: A green hosted advisory job represents the exact pinned, low-threshold full-graph audit with no repository-controlled bypass in the workspace, root manifest, `.npmrc`, direct audit-step configuration variables, or global-config paths redirected by that step.

## Site exception record

- **Previous authority**: supply-chain/site-audit-exception-v1.json for one GHSA and locked picomatch version.
- **S213 state**: Removed. No replacement record or active ignore is created.
- **Boundary**: The unrelated Rust policy records in supply-chain/policy-v1.json and deny.toml remain.

## Advisory and release records

- **Current advisory**: GitHub's GHSA record, withdrawn 2026-11-18 because the report is not a security issue.
- **Current tracker**: #630 remains open until the code cleanup and unfiltered gate pass, then closes on merge.
- **Historical release**: v0.11.2 accurately records its exception-qualified site audit at publication time. A dated clarification distinguishes that past result from the current withdrawn status.
