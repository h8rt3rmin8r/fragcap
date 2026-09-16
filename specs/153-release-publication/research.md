# S153 Publication Research

## Release source and pipeline

Decision: tag merged S152 a7d24962999d38d7ff130722859d473543864862 after its source workflows pass. Rationale: the operator already merged the candidate and explicitly requests actual publication; another preparation-only merge adds no required product content. Alternative: tag an unmerged records branch, rejected because it lacks human review and cannot truthfully contain future publication facts.

Authority: [release workflow](../../.github/workflows/release.yml), [S152 handoff](../../docs/maintainers/v0.10.1-release-handoff.md), master specification sections 24.4/24.5, [GitHub release REST interface](https://docs.github.com/en/rest/releases/releases).

## Package evidence

Decision: consume the existing fragcap-windows-x86_64 artifact and windows-package-certification-summary/report.json, then verify the public downloaded bytes against checksums, sizes and report. Rationale: hosted lifecycle certification installs only on an isolated runner; local verification needs no sensitive product execution. Alternative: local rebuild/install, rejected by operator constraints and certified-byte identity.

The independent research pass notes that canonical report validation checks source syntax but S153 must additionally compare report.build_identity.version and source_revision to 0.10.1 and the exact peeled tag. Require official=true, complete=true, findings=[] and six complete reconciled artifact records. Authority: [package workflow](../../.github/workflows/package-certification.yml) and xtask/src/package_certification.rs.

## Registry and recovery

Decision: use the existing protected publish job, then query all ten exact public version endpoints and require non-yanked records. The dependency order is fragcap-core, fragcap-profile, fragcap-capture, fragcap-attr, fragcap-sink, fragcap-steam, fragcap-targets, fragcap-proxy, fragcap, fragcap-cli. Existing xtask publication skips duplicate versions and stops on actual errors. A warranted failed-job-only rerun resumes a partial registry upload; it must not repeat successful release creation or replace published bytes.

Authority: [publisher](../../xtask/src/publish.rs), [registry API](https://crates.io/data-access), [GitHub workflow reruns](https://docs.github.com/en/rest/actions/workflow-runs#re-run-failed-jobs-from-a-workflow-run).

## Authority and reconciliation

Decision: retain sole-owner review, self-review permitted, administrator bypass enabled and one custom v* tag allowance. Configuration readback is not deployment approval. The owner must take any pending approval/bypass decision; the agent neither changes protections nor reads registry secrets. Alternative: automatic deployment approval or weakening protections, rejected by the established operator-controlled release contract.

Only after complete publication do docs/published-release.json and all fourteen CURRENT_RELEASE_SURFACES move to actual v0.10.1. Historical per-release narrative is retained. Independent #333/#413 and final #334/#278 remain unresolved, and optional #372 measurements use published bytes. No research unknowns remain.
