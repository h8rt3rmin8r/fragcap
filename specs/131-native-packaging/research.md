# S131 Research: Native Windows Packaging Certification

## Decision 1: Close issue #329 at the final-byte release boundary

S129 proves staged Windows product behavior and S130 proves dependency plus legal evidence before package assembly. Neither inspects the ZIP or MSI that users download, exercises the installer lifecycle, independently revalidates checksums after artifact transfer, or measures signature state. S131 owns that remaining boundary and does not repeat the S129 protocol matrix or S130 dependency audit.

The current release workflow has a critical ordering defect: it can create a GitHub release before the tag-to-workspace-version identity check in the later crate-publication job. S131 moves identity validation ahead of packaging and requires certification before artifact upload, GitHub release creation, or crate publication.

## Decision 2: Use one closed package contract and one task-runner authority

`integration/windows-package-contract-v1.json` defines the three primary artifacts, three checksum sidecars, exact shared payload, size ceilings, build identity, PE machine and import allowlist, signature policy, installer ownership, lifecycle rows, predecessor identity, report bounds, and workflow order. Unknown fields, vocabulary values, artifact entries, imports, lifecycle rows, or outcomes fail closed.

`cargo xtask package-certification` validates that contract and all repository wiring offline. Its report-validation mode consumes the bounded Windows harness report and proves complete reconciliation. `cargo xtask ci` invokes the offline mode on every supported development platform without installing software or changing machine state.

This extends the existing S129 registry/report pattern. A second generic workflow engine or package framework was rejected because it would duplicate established authorities and delay the release.

## Decision 3: Certify bytes once and publish those exact bytes

The required Windows workflow builds the official release binary, validated S130 evidence, catalog, portable ZIP, MSI, and checksum sidecars once. Certification consumes those exact staged bytes, writes a public-safe summary, and uploads one certified artifact bundle. The release job downloads that bundle, independently rechecks its closed inventory and digests, and publishes only those files.

Building packages again after certification was rejected because it would sever the evidence from the bytes released. Leaving the current MSI smoke as `continue-on-error` was rejected because warning-only evidence cannot authorize publication.

## Decision 4: Expose exact native build identity

The packaged executable gains a machine-readable identity command containing fragcap version, source revision, target triple, architecture, exact official feature set, and native backend identity. Official packaging supplies the source revision and marks the build as the official closure; certification binds that output to the PE version resource, filename, package metadata, and executable digest.

`--version` and Doctor alone were rejected as sufficient proof because they expose only semver or selected readiness facts and cannot distinguish a locally rebuilt, wrong-target, wrong-feature, or stale-source executable. The identity command performs no effect and introduces no product dependency.

## Decision 5: Inspect final content, size, PE machine, and imports

The archive and MSI each expose exactly six shared files: `fragcap.exe`, `catalog.db`, `LICENSE`, `NOTICE`, `fragcap.cdx.json`, and `THIRD-PARTY-NOTICES.txt`. The validator rejects case-fold duplicates, traversal, absolute paths, links, alternate separators, unknown entries, and byte differences across package surfaces. The standalone catalog must match both packaged copies.

Reviewed per-entry and per-artifact ceilings block accidental runtime bundling while allowing ordinary compiler variance. Exact-byte size baselines were rejected because compiler and compression changes make them brittle. The final executable's PE machine and ordinary plus delayed import tables are matched against a closed allowlist, so a hidden Python, OpenSSL, external proxy, wrong-architecture, or unknown dynamic dependency cannot pass merely because the filename is allowed.

## Decision 6: Validate the current unsigned policy accurately

The master specification and current release documentation declare the executable and MSI unsigned. `Get-AuthenticodeSignature` must return determinate `NotSigned` with no signer for the MSI and every executable copy. ZIP, catalog, documents, evidence, and checksum sidecars carry explicit `not_applicable` states. Signed, invalid, unknown, inconsistent, or falsely described output blocks certification.

S131 does not reopen signing procurement or claim a signature. Adding a code-signing service was rejected as both outside issue #329 and inconsistent with the architecture of record.

## Decision 7: Exercise a finite real MSI lifecycle

Fresh hosted Windows cases cover clean install, repair after one deletion and one mutation, exact-byte same-version reinstall, upgrade from the exact pinned v0.8.0 MSI, downgrade refusal, and uninstall. Each invocation is non-interactive, hidden, finite, exit-accounted, and followed by exact reconciliation of program files, related-product registration, PATH, the best-effort Defender effect, and seeded user-owned state.

The v0.8.0 predecessor is acquired before the offline lifecycle phase from its official release URL and accepted only at SHA-256 `eaf2554b1da3721400c1b00f5ea0a298455f59454b0084e617ed2efcdcf83901`. S131 does not generate a fake public predecessor.

The initial specification draft named deterministic rollback as a standalone required case. Planning deliberately narrows that idea because issue #329 requires install, upgrade, repair, and uninstall correctness, while a transform-based rollback injector would create a new MSI mutation subsystem. Failed transactions remain bounded by cleanup and residue assertions when encountered, but standalone fault-injected rollback is not a release blocker in this slice.

## Decision 8: Preserve exact ownership boundaries

Installer-owned state is the fixed Program Files payload, one product registration, one exact system PATH element, and only the exact Defender exclusion created for the installed executable path. The exclusion is best effort on installation, so `present` and `unavailable` are distinct allowed outcomes; uninstall requires exact absence. Existing forward-only extcap integration, per-user databases, captures, bundles, and any independently managed analyzer state remain user-owned and byte-identical across lifecycle cases.

The current WiX package cannot prove whether fragcap created a pre-existing exact Defender exclusion, yet its uninstall removes that exclusion unconditionally. S131 corrects this ownership defect with an exact installer marker or removes automatic exclusion management if exact ownership cannot be made transactional. It does not preserve the unsafe behavior.

Ownership is never inferred from a display name, filename alone, certificate subject, process identifier, or broad directory. This preserves the same exact-effect rule used by Deep Capture cleanup.

## Decision 9: Use a constrained offline packaged-binary smoke

The extracted and installed final executable is run with a fresh temporary user-data root, a sanitized environment that excludes Python and package-manager paths, and network limited to controlled loopback origins. The smoke checks build identity, machine-readable Doctor output, and the existing controlled native calibration path. Child-process accounting permits only the tested executable and declared Windows installer processes.

GitHub-hosted Windows images may contain Python, so the report does not falsely claim the VM lacks it. Instead, the package, target-feature dependency closure, PE imports, child environment, child process inventory, and offline loopback behavior jointly prove that the official product does not require or invoke it.

## Decision 10: Correct package documentation without erasing history

Current production paths already use `fragcap-native`; historical specs, changelog entries, the excluded comparison spike, and the legacy compatibility-fact round-trip remain accurate history. S131 changes only current package/runtime guidance and one stale Doctor label.

The README's `hint.db` download is corrected to the actual `catalog.db`, and NOTICE wording is scoped to official packages so it does not contradict the separately authorized source-build `doctor --fix` path. Npcap remains a separately installed Capture prerequisite and is never bundled or silently installed.

## Decision 11: Pin package inputs and remove workflow drift

The package contract records exact WiX/cargo-wix input identities and the Npcap SDK checksum used only to compile the optional live backend. The SDK version is unified with the S129 Windows workflow. Package certification asserts that no SDK or Npcap payload survives staging.

Both release-preparation script twins update the same embedded Windows/package contract versions. Their existing divergence is corrected in the smallest shared fixture necessary; S131 does not redesign release preparation.

## Decision 12: Add no product runtime dependency or scheduled work

All new validation lives in `xtask`, one PowerShell harness, workflow definitions, package metadata, and documentation. The executable change exposes compile-time identity only. S131 creates no local automation, background soak, target instrumentation, system-wide proxy setting, trust effect, tag, release, crate publication, or final Deep Capture completion claim.
