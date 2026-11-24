# Data Model: Withdrawn Site Advisory Cleanup

S213 adds no product data structure or persistent user state. It removes one finite policy record and tightens the existing site audit configuration boundary.

## Site workspace configuration

- **Authority**: site/pnpm-workspace.yaml.
- **Relevant fields**: `packages`, `allowBuilds`, and `overrides` remain. Any top-level `auditConfig` or `audit` key is refused by both the offline supply-chain gate and the hosted site job. LF and CRLF are accepted; bare CR is refused. Root keys must use canonical block YAML with ASCII plain names matching `[A-Za-z][A-Za-z0-9_-]*`, optionally followed by whitespace before the colon. An indentless `- ` list item after a root mapping key is accepted; a standalone root sequence and other unsupported root YAML forms are refused.
- **Invariant**: A green site advisory job represents the unchanged full-graph pnpm audit command with no workspace advisory ignore.

## Site exception record

- **Previous authority**: supply-chain/site-audit-exception-v1.json for one GHSA and locked picomatch version.
- **S213 state**: Removed. No replacement record or active ignore is created.
- **Boundary**: The unrelated Rust policy records in supply-chain/policy-v1.json and deny.toml remain.

## Advisory and release records

- **Current advisory**: GitHub's GHSA record, withdrawn 2026-11-18 because the report is not a security issue.
- **Current tracker**: #630 remains open until the code cleanup and unfiltered gate pass, then closes on merge.
- **Historical release**: v0.11.2 accurately records its exception-qualified site audit at publication time. A dated clarification distinguishes that past result from the current withdrawn status.
