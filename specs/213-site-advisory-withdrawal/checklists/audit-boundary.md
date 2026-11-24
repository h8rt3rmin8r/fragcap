# Audit Boundary Checklist: Withdrawn Site Advisory Cleanup

**Purpose**: Keep the S213 security-audit requirements specific, testable, and historically accurate.
**Created**: 2026-11-24
**Feature**: [spec.md](../spec.md)

## Site Audit Authority

- [x] CHK001 The full-graph site audit command and its low advisory threshold are named exactly.
- [x] CHK002 The pinned pnpm and Node versions remain outside the cleanup scope.
- [x] CHK003 The withdrawn advisory's exception record and workspace CVE ignore are both removed.
- [x] CHK004 Both offline validation and the hosted site job refuse a top-level `auditConfig` or `audit` key, noncanonical root YAML syntax, and bare CR line breaks that could hide a key in the workspace YAML.
- [x] CHK005 A distinct advisory continues to fail the unchanged full-graph audit command.

## Negative and Completion Evidence

- [x] CHK006 Controlled bypass fixtures cover two `auditConfig` forms, `audit.ignore`, explicit audit disablement, unsupported root flow, explicit-key, merge or alias, tag, escaped-key and standalone-sequence forms, and bare CR; LF/CRLF and indentless `packages` baselines pass.
- [x] CHK007 The full audit, frozen-lockfile install, and static export are required on the changed revision.
- [x] CHK008 No passing result is claimed if registry data still reports the withdrawn GHSA or another advisory.
- [x] CHK009 The Rust advisory exception under #640 remains independently governed.

## Record Integrity

- [x] CHK010 Current guidance links GitHub's dated withdrawal and describes the present audit state.
- [x] CHK011 Historical v0.11.2 exception evidence remains accurate as a record of publication time.
- [x] CHK012 A dated decision records the pinned workflow change and why it removes a now-obsolete exception.
