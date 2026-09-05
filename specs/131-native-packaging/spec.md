# Feature Specification: Native Windows Packaging Certification

**Feature Branch**: `codex/131-native-packaging`

**Created**: 2026-09-05

**Status**: Draft

**Input**: Work slice S131 implementing issue #329 after S129 established staged Windows behavior and S130 established release supply-chain evidence.

## User Scenarios & Testing

### User Story 1 - Receive Complete Native Release Artifacts (Priority: P1)

As a Windows user, I need each official fragcap package to contain the complete native product and its required evidence so I can use Capture and eligible Deep Capture paths without installing Python, mitmdump, or another proxy package.

**Why this priority**: The release cannot claim a self-contained native backend unless the exact downloadable artifacts carry every fragcap-owned runtime component and no obsolete external proxy prerequisite.

**Independent Test**: Build the official portable archive and installer from one release identity, inspect their final contents, and run the packaged binary in an offline controlled environment that has no Python or external proxy package.

**Acceptance Scenarios**:

1. **Given** the official release inputs, **when** both Windows packages are assembled, **then** each contains the same required binary, catalog, project legal texts, software bill of materials, and third-party notices with no undeclared runtime component.
2. **Given** a clean supported Windows environment with no Python, mitmdump, or external proxy package, **when** the packaged binary starts and exercises the controlled native smoke path, **then** it does not request, download, probe for, or invoke an external proxy runtime.
3. **Given** a package containing Npcap, Python, an external proxy executable, an unexpected dynamic library, a shell wrapper, a game profile, a repository-only document, user-owned state, or an unknown file, **when** final-content validation runs, **then** release publication is blocked and the exact unexpected entry is named.

---

### User Story 2 - Install, Repair, Upgrade, and Uninstall Predictably (Priority: P2)

As a Windows administrator, I need the installer lifecycle to preserve user-owned state, repair owned program files, replace an older installation, and remove only installer-owned state so deployment and removal are safe and repeatable.

**Why this priority**: A complete payload is not releasable if install transitions duplicate files, preserve stale binaries, erase user evidence, or leave owned machine state behind.

**Independent Test**: Exercise clean install, repair, same-version reinstall, supported upgrade from a controlled predecessor, and uninstall in fresh Windows test roots, comparing the observed filesystem and machine effects with the declared ownership model after every transition.

**Acceptance Scenarios**:

1. **Given** no installed fragcap package, **when** the installer runs unattended, **then** every declared program file and owned machine effect is present exactly once and the installed binary reports the expected release identity.
2. **Given** a damaged or missing installer-owned program file, **when** repair runs, **then** the exact current file set is restored without modifying user-owned databases, captures, bundles, analyzer registration, or unrelated security configuration.
3. **Given** a supported prior fragcap installation, **when** the current installer upgrades it, **then** old program files and owned machine effects are replaced without a side-by-side duplicate and user-owned state remains unchanged.
4. **Given** an installed or partially repaired package, **when** uninstall runs, **then** installer-owned files and machine effects are removed, user-owned state is preserved, and a repeated uninstall cannot broaden deletion authority.

---

### User Story 3 - Verify Download Integrity and Published Truth (Priority: P3)

As a release consumer or maintainer, I need every download, checksum, signature claim, and content report to agree with the bytes that will be published so corruption, substitution, stale evidence, or a misleading signing claim blocks the release.

**Why this priority**: Checksums and signature state are the public integrity contract for the currently unsigned packages, and they must be derived from and checked against final artifacts rather than intermediate staging files.

**Independent Test**: Generate the release downloads and integrity evidence, independently recalculate every digest, inspect each package's signature state, alter one artifact or declaration at a time, and verify each mismatch prevents publication.

**Acceptance Scenarios**:

1. **Given** the final release downloads, **when** checksums are generated and independently validated, **then** every published file has exactly one correctly formatted digest and no stale or extra entry exists.
2. **Given** the declared unsigned-release policy, **when** executable and installer signature state is inspected, **then** the report records that state accurately and fails if a file is signed unexpectedly, malformed, or described inconsistently.
3. **Given** any altered package byte, missing required file, mismatched package version, changed binary identity, stale dependency evidence, incorrect checksum, or inconsistent signature claim, **when** the release gate runs, **then** no GitHub release or crate publication can begin.

### Edge Cases

- Archive entry names use alternate separators, case, duplicate names, absolute paths, parent traversal, links, or path forms that escape the package root.
- The archive and installer contain the same filename with different bytes or different versions.
- The staged binary is feature-incomplete, fails before command dispatch on a machine without Npcap, or reports a version different from the package identity.
- The package contains Npcap import libraries, driver files, installers, Python runtimes, proxy executables, OpenSSL components, or other prohibited prerequisites under misleading names.
- Repair encounters a missing file, modified file, locked file, cancelled operation, or interrupted transaction.
- Upgrade encounters an older supported package, the same version, a newer version, or an unrelated product with a similar name.
- Uninstall runs after repair, failed upgrade, partial rollback, or manual deletion of an installed file.
- User-owned catalog or local state exists beside or outside the installation directory and must not be treated as installer-owned residue.
- An analyzer integration was registered separately by the operator and must not be removed by package uninstall.
- Defender exclusion creation is unavailable, refused, or absent, and cleanup must remain bounded to the exact installer-owned path and declared state.
- A checksum file is missing, duplicated, malformed, differently cased, reordered, or includes a file not published.
- Signature inspection is unavailable, returns an indeterminate result, or disagrees with the declared release policy.
- A release workflow change permits publication to race ahead of package certification.

## Requirements

### Functional Requirements

- **FR-001**: S131 MUST define one versioned, closed package-certification contract for the official Windows portable archive, installer, standalone catalog, and checksum set.
- **FR-002**: The contract MUST define the exact required, optional, and prohibited final entries for each artifact, including identity, relative path, role, expected multiplicity, ownership class, and reviewed size ceiling.
- **FR-003**: Both the archive and installer MUST contain the same release binary, barebones catalog, project license, project notice, exact release software bill of materials, and complete third-party notices, with byte identity required wherever the contract declares shared content.
- **FR-004**: Final-content validation MUST reject undeclared entries and MUST specifically reject any Npcap component, Python runtime, external proxy runtime, OpenSSL runtime, shell wrapper, game profile, repository-only document, user-owned database, capture, session bundle, private key, credential, or machine-local path.
- **FR-005**: The packaged binary MUST carry the complete supported release capability set and MUST start on a supported Windows machine where Npcap, Python, and external proxy packages are absent; missing Npcap MUST remain a reported Capture readiness condition rather than a process-start failure.
- **FR-006**: An offline controlled smoke path MUST prove that packaged Deep Capture uses only the native backend and performs no hidden download, package-manager invocation, external proxy probe, or first-use runtime installation.
- **FR-007**: The installer MUST perform clean install, repair, same-version reinstall, supported major upgrade, downgrade refusal, and uninstall through one declared ownership model with bounded, deterministic outcomes.
- **FR-008**: Installer-owned files and machine effects MUST be enumerated before testing, and every lifecycle transition MUST reconcile observed state to that closed inventory without using filenames, product display names, subjects, issuers, friendly names, or process identifiers as sole ownership proof.
- **FR-009**: Repair MUST restore missing or modified installer-owned program files and MUST leave user-owned state and independently managed analyzer integration unchanged.
- **FR-010**: Upgrade MUST replace the prior installed product rather than create a side-by-side installation, MUST reject downgrade unless explicitly authorized by the package contract, and MUST preserve user-owned state.
- **FR-011**: Uninstall and any failed installer transaction MUST remove only exact installer-owned files and machine effects, MUST remove the exact installer-owned Defender exclusion when present, MUST preserve user-owned data and separately managed analyzer registration, and MUST report any residue or unverifiable cleanup.
- **FR-012**: Every lifecycle invocation MUST have a finite timeout, hidden non-interactive child-process execution, captured exit status, bounded public-safe diagnostics, and cleanup that runs after success, failure, timeout, or cancellation.
- **FR-013**: Package certification MUST run on supported Windows infrastructure using fresh test roots and MUST not depend on the operator's installed fragcap, user profile, trust store, Npcap installation, Python installation, external proxy installation, or desktop state.
- **FR-014**: Each primary release artifact, the portable archive, installer, and standalone catalog, MUST have exactly one independently verified SHA-256 sidecar over its final bytes, and the checksum inventory MUST reject missing, duplicate, unexpected, malformed, stale, or recursively checksummed sidecars.
- **FR-015**: The certification report MUST record the exact signature state of the installer and every executable package surface and enforce the declared release policy; the current unsigned policy MUST fail on an unexpected signature, an indeterminate inspection, or any claim that the artifact is signed, while non-executable artifacts and checksum sidecars MUST carry an explicit not-applicable state.
- **FR-016**: Package identity MUST bind artifact filename, package metadata, installed binary version, binary product identity, source revision, target triple, architecture, release feature set, content inventory digest, checksum, and signature state.
- **FR-017**: The package-certification report MUST be deterministic, versioned, bounded, public-safe, and complete only when every required artifact, content, lifecycle, integrity, and cleanup row reaches an allowed terminal outcome.
- **FR-018**: Release publication and crate publication MUST depend on successful certification of final artifacts; a best-effort or warning-only package test MUST not authorize publication.
- **FR-019**: The ordinary offline gate MUST validate the certification schema, closed vocabularies, complete matrix, workflow ordering, package contract, and test ownership without installing, repairing, upgrading, uninstalling, modifying Defender state, or executing release effects.
- **FR-020**: Changes to release workflows, installer definitions, packaging contracts, or certification scripts MUST carry a dated S131 decision fragment and trace to issue #329.
- **FR-021**: User-facing prerequisites, options, diagnostics, and release guidance MUST describe the native backend as the sole production Deep Capture path and MUST contain no instruction to install Python, mitmdump, mitmproxy, or another proxy package.
- **FR-022**: Historical specifications, changelog records, and explicitly non-shipping comparison spikes MAY retain accurate external-backend history, but package inputs, runtime paths, current user guidance, and certification authority MUST exclude them.
- **FR-023**: S131 MUST preserve Npcap as the separately installed Capture prerequisite and MUST never bundle, host, cache as fragcap-owned, or silently install any Npcap component.
- **FR-024**: S131 MUST add no target instrumentation, system-wide proxy configuration, silent certificate trust, product runtime dependency, code-signing claim, release tag, crate publication, final Deep Capture feature-completion claim, or recurring local task.
- **FR-025**: Certification MUST enforce reviewed per-artifact and per-entry size ceilings and MUST block any final artifact whose size exceeds its declared ceiling until the contract change is reviewed.
- **FR-026**: Certification MUST inspect the final Windows executable's PE machine and import tables against a closed target and native-library allowlist, including delayed imports, and MUST reject an unknown architecture or dependency even when the package filename is allowed.

### Key Entities

- **Package Contract**: The versioned closed declaration of official artifacts, exact contents, prohibited content, ownership, feature identity, integrity, signature policy, and lifecycle coverage.
- **Artifact Identity**: The release version, source revision, target, architecture, feature set, filename, content digest, checksum, and signature state that identify one final download.
- **Package Entry**: One normalized relative entry with role, size, digest, ownership class, and artifact membership.
- **Installer Effect**: One exact installer-owned file or machine effect with creation, repair, upgrade, rollback, and removal expectations.
- **Lifecycle Case**: One clean-install, repair, reinstall, upgrade, downgrade-refusal, failed-transaction, uninstall, or refusal scenario with preconditions, expected observations, deadline, and terminal result.
- **Certification Report**: The bounded public-safe reconciliation of the closed package contract against final bytes and lifecycle observations.

## Success Criteria

### Measurable Outcomes

- **SC-001**: Certification accounts for 100 percent of entries in every official release artifact, with zero missing, duplicate, unexpected, path-escaping, or unclassified entries.
- **SC-002**: The archive and installer expose one byte-identical release binary, catalog, license, notice, software bill of materials, and third-party notices, and every identity field agrees with the release version and source revision.
- **SC-003**: A clean offline Windows test environment with no Python or external proxy package completes the native package smoke path with zero network fetches, package-manager invocations, external proxy probes, or undeclared child processes.
- **SC-004**: Clean install, repair, reinstall, supported upgrade, downgrade refusal, and uninstall each complete within 10 minutes and reconcile 100 percent of declared installer-owned state while preserving 100 percent of seeded user-owned state; any failed transaction exercised by the matrix leaves no broader owned residue.
- **SC-005**: Each of the three primary release artifacts has exactly one matching SHA-256 sidecar, and every artifact has one determinate signature-state result consistent with the unsigned-release policy or an explicit not-applicable classification.
- **SC-006**: Controlled mutations covering missing, extra, altered, stale, mis-versioned, mis-featured, prohibited, unsigned-policy, checksum, traversal, lifecycle, timeout, and residue failures each block certification with the exact artifact, case, and remediation class.
- **SC-007**: The certification report contains every required matrix row, zero skipped required rows, zero unclassified outcomes, and no credentials, private material, host names, account identifiers, absolute operator paths, or captured payloads.
- **SC-008**: Release and crate publication have no path around successful final package certification, and S131 adds zero product runtime dependencies or hidden first-use installation behavior.

## Assumptions

- S129's staged-layout integration remains valid pre-package evidence; S131 adds authority over final ZIP and MSI bytes and their real lifecycle rather than duplicating S129's protocol matrix.
- S130's validated software bill of materials and third-party notices are required package inputs and are revalidated as final embedded bytes rather than regenerated by the certification layer.
- The supported predecessor for upgrade testing may be a controlled package built from the current installer contract with an older valid product version; downloading an old public release is not required for ordinary pull-request certification.
- The current product specification explicitly declares the MSI and executable unsigned. S131 validates and reports that state and does not reopen certificate procurement or claim Authenticode signing.
- Defender exclusion behavior is best effort at installation time, but any exact exclusion created by the installer remains owned cleanup state and must be absent after uninstall or rollback.
- User-owned state includes the per-user local target database, copied writable catalog, captures, Deep Capture bundles, and independently managed analyzer registration; none is installer-owned.
- Final Deep Capture completion language remains reserved for issue #334.
