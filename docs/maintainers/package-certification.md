# Windows package certification

S131 makes the final Windows downloads a release authority. `cargo xtask package-certification` is the offline contract and wiring gate. `.github/workflows/package-certification.yml` builds the official feature closure once, assembles the ZIP, MSI, catalog, and checksum sidecars, runs the destructive lifecycle only on its fresh Windows runner, validates the bounded report, and uploads the exact certified bytes. `.github/workflows/release.yml` can publish only that uploaded bundle after rehashing every transferred artifact and sidecar against its certified report row.

## Local static validation

Run `cargo xtask package-certification` after changing the release workflow, WiX source, build identity, package contract, certification harness, package-facing documentation, or size/import policy. The command performs no install or machine effect and is part of `cargo xtask ci`.

Run `.agents/skills/shruggie-powershell/scripts/Test-ScriptCompliance.ps1 -Path scripts/Test-PackageCertification.ps1` after changing the Windows harness. The harness targets PowerShell 7, uses hidden non-interactive child processes, keeps raw MSI logs in disposable scratch, and writes only the closed public-safe report.

## Contract updates

Treat `integration/windows-package-contract-v1.json` as reviewed release policy. A new package entry, import, artifact, lifecycle result, tool version, predecessor, or size ceiling requires a dated decision fragment and the matching specification change. Do not widen a ceiling or import list solely to make an unexplained candidate pass. Inspect the actual final bytes and explain the reason.

The current primary artifacts are the versioned portable ZIP, versioned MSI, and standalone `catalog.db`. Each gets one `.sha256` sidecar. The ZIP and MSI carry the same six payload files. Checksum sidecars are not recursively checksummed, and the certification report is CI evidence rather than another public release download.

## Installer ownership

The MSI owns its fixed Program Files payload, related-product registration, exact system PATH entry, and only a Defender exclusion whose exact normalized path is recorded by its private ownership marker after successful creation. The marker is cleared only after the exclusion is observed absent, so a failed removal retains exact cleanup authority. A pre-existing administrator-owned exclusion has no marker and survives uninstall. The matrix seeds and hashes the isolated `%APPDATA%` catalog, local database, session bundle, and personal Wireshark extcap paths plus an isolated `%LOCALAPPDATA%` capture path because those are the user-owned locations the installer must preserve.

The real matrix covers clean install, repair after controlled file damage, exact-byte same-version reinstall, upgrade from the digest-pinned v0.8.0 MSI, refusal of that predecessor as a downgrade after the candidate is current, and uninstall. Every Windows Installer invocation has a 600-second ceiling and local verbose log. A skipped, timed-out, warning-only, or unreconciled row fails.

## Current integrity policy

The MSI and every packaged executable are deliberately unsigned. Certification requires a determinate Authenticode `NotSigned` result with no signer. The portable ZIP, catalog, documents, dependency evidence, and checksum sidecars are explicitly not applicable. Do not describe any artifact as signed until the master specification and contract change together.

The executable's hidden `__build-identity` command binds version, source revision, target triple, architecture, exact official features, native backend, and official-build marker. Package certification also checks PE machine, version resources, ordinary imports, delayed imports, final sizes, shared-file digests, and the constrained loopback native smoke. The smoke uses an exact-program outbound firewall rule for non-loopback destinations, samples the descendant process tree and TCP/UDP endpoints, rejects undeclared executables or observed non-loopback addresses, and requires an observed loopback endpoint before reporting completion.

## Failure handling

Do not publish around a failed package workflow. Keep the failed run, inspect its bounded finding and runner-local MSI log, reproduce through a pull request, and fix the smallest owning layer. If a tag run fails before GitHub release creation, cut a new semantic version after the fix rather than moving the tag. Crate publication remains after GitHub release creation and the `crates-io` environment approval.

Update the predecessor only when a newer supported public package becomes the intentional upgrade floor. Acquire it from the official release URL before the offline lifecycle phase, record its exact size and SHA-256, and verify both before any install. Never let an unpinned network download become installer evidence.
