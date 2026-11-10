# Windows package certification

S131 makes the final Windows downloads a release authority. `cargo xtask package-certification` is the offline contract and wiring check. `.github/workflows/package-certification.yml` builds the official feature closure once, assembles the ZIP, MSI, catalog, and checksum sidecars, runs the destructive lifecycle only on its fresh Windows runner, validates the bounded report, and uploads the exact certified bytes. `.github/workflows/release.yml` can publish only that uploaded bundle after rehashing every transferred artifact and sidecar against its certified report row.

## Local static validation

Run `cargo xtask package-certification` after changing the release workflow, WiX source, build identity, package contract, certification harness, package-facing documentation, or size/import policy. The command performs no install or machine effect and is part of `cargo xtask ci`.

Run `.agents/skills/shruggie-powershell/scripts/Test-ScriptCompliance.ps1 -Path scripts/Test-PackageCertification.ps1` after changing the Windows harness. The harness targets PowerShell 7, uses hidden non-interactive child processes, keeps raw MSI logs in disposable scratch, and writes only the closed public-safe report.

## Contract updates

Treat `integration/windows-package-contract-v1.json` as reviewed release policy. A new package entry, import, artifact, lifecycle result, tool version, predecessor, or size ceiling requires a dated decision fragment and the matching specification change. Do not widen a ceiling or import list solely to make an unexplained candidate pass. Inspect the actual final bytes and explain the reason.

The current primary artifacts are the versioned portable ZIP, versioned MSI, and standalone `catalog.db`. Each gets one `.sha256` sidecar. The ZIP and MSI carry the same six payload files. Checksum sidecars are not recursively checksummed, and the schema-version-4 certification report is CI evidence rather than another public release download. The validator retains schema 2 and schema 3 as read-only historical contracts; current certification emits only schema 4.

## S185 installation CA

S185 adds one installation CA to the real MSI lifecycle. The first install discloses and performs the clean break for canonical pre-S185 Fragcap state, then provisions a machine-specific root in `LocalMachine/Root` and a separate machine-DPAPI-protected signer under ProgramData. The package gate asserts an exact public fingerprint on clean install, unchanged identity on repair and reinstall, retained identity after ordinary uninstall, and exact removal only through elevated `fragcap ca remove`. Its first-install cutover fixture lives in the disposable Windows runner profile; later synthetic user fixtures are created after cutover and remain preserved through ordinary maintenance. Portable ZIP use requires an explicit elevated `fragcap ca install`. The MSI, ZIP and session bundle must not contain private signer bytes. The installation CA is deliberately persistent and is not an MSI-owned uninstall effect.

## Installer ownership

The MSI owns its fixed Program Files payload, related-product registration, exact system PATH entry, and only a Defender exclusion whose exact normalized path is recorded by its private ownership marker after successful creation. The marker is cleared only after the exclusion is observed absent, so a failed removal retains exact cleanup authority. A pre-existing administrator-owned exclusion has no marker and survives uninstall. The matrix seeds and hashes the isolated `%APPDATA%` catalog, local database, session bundle, and personal Wireshark extcap paths plus an isolated `%LOCALAPPDATA%` capture path because those are the user-owned locations the installer must preserve.

The real matrix covers clean install, repair after controlled file damage, exact-byte same-version reinstall, upgrade from the digest-pinned v0.8.0 MSI, refusal of that predecessor as a downgrade after the candidate is current, and uninstall. Every Windows Installer invocation has a 600-second ceiling and local verbose log. A skipped, timed-out, warning-only, or unreconciled row fails.

S137 extends the report with a closed fresh-start proof. The real ordinary uninstall still hashes and preserves every seeded user fixture. The packaged executable's synthetic command cases use an explicit scratch `User\.fragcap\roaming` and `User\.fragcap\local` pair, proving current-user cleanup, Doctor recovery, custom database and bundle exclusion, independent Wireshark registration preservation, and clean reinstall. The MSI's own optional fresh-start case operates only in the disposable runner account's real profile and verifies the impersonated user context before removing the exact new roots. The ordinary installed default-path case clears the scratch override and reads back its real database and session bundle. S192 also checks independent ordinary processes under one account with different AppData variables and compares opened-file destinations; this does not emulate an arbitrary MSIX host. Actual packaged-host behavior is observed after release, and any defect returns as a GitHub issue. Private owner data never participates in certification.

Published v0.10.14 package replay uses the released executable's `.fragcap` fixture paths and records a fresh-start transition suppression marker beside them, so retained legacy AppData state cannot restore a cleaned store. Historical v0.10.13 replay continues to use that release's AppData fixture paths.

## Current integrity policy

S194 adds the public inspection-scope parser to both packaged controlled smoke
surfaces. The candidate executable runs the exact-game reachability case with
`--inspect-scope game`; the bounded structured-event reader requires the
authorization plan to report requested and effective `exact-game` scope,
`blind-relay` middleware handling, and policy `exact-game-v1` before accepting
the existing reached-client result. This certifies the exact package bytes and
parser contract. Installed-product and private-title behavior is assessed only
after release, and any defect returns as a GitHub issue.

S197 source advances the same controlled policy identity to `exact-game-v2`.
Version 1 remains the published v0.10.14 case and cannot qualify a version 2
candidate. The controlled smoke must compare the candidate's exact version 2
policy while tagged replay keeps its released contract.

The package contract carries those three controlled-smoke values explicitly.
Published-package replay uses the contract and smoke-event validator from the
published source tree. A historical contract without `controlled_smoke` keeps
its original invocation and evidence schema, while every current candidate is
required to run and report the S194 exact-game case.

The MSI and fragcap executable are deliberately unsigned. Certification requires a determinate Authenticode `NotSigned` result with no signer. The bundled WinDivert64.sys is a separate official signed upstream component, with exact Authenticode validity and signer thumbprint checked on the Windows runner. Its companion WinDivert.dll is determinately unsigned. The portable ZIP, catalog, documents, dependency evidence, source archive and checksum sidecars have no Authenticode signature requirement.

## S199 signed component foundation

The package contract pins the official WinDivert 2.2.2-A binary and tag-source archive URLs, SHA-256 hashes, x64 PE machine and driver signer thumbprint. The Windows workflow verifies both downloaded archives and all selected members before staging. MSI and ZIP carry the same four files beside `fragcap.exe`: `WinDivert.dll`, `WinDivert64.sys`, `WinDivert-LICENSE.txt`, and `WinDivert-source.zip`. Exact shared-file comparisons run after clean install, repair, same-version reinstall, upgrade and downgrade refusal. Uninstall must remove the MSI-owned files; S199 never opens the driver or creates its service, so the package test makes no claim about a service installed later by a direct-route session. The portable package has no MSI installation-CA effect.

The generated CycloneDX evidence and third-party notices list WinDivert as an external component under its LGPLv3 option. The upstream license and tag source archive are packaged for offline access. The selected future binding is a session-scoped dynamic load of the DLL from the exact application directory, followed by a driver handle only after direct-route authorization; S199 introduces no runtime binding or handle open. The DLL is separate and replaceable in that directory. `fragcap doctor` hashes all four files read-only and reports exact packaged-byte readiness, not a successful driver load or route. It also reports a disabled WinDivert service when that read-only registry state is observable. Missing or altered files warn without changing ordinary Capture readiness. The release gate refuses wrong archive or member hashes, x64 or signature mismatch, and a stale component contract.

The [upstream WinDivert lifecycle](https://reqrypt.org/windivert-doc.html#uninstalling) says `WinDivertOpen` installs the driver on demand with Administrator rights. If an active handle exists when the package files are removed, file removal does not prove the service has stopped; upstream says an already running driver is automatically uninstalled at the next reboot. A later route adapter must close its own handles and report any still-running service or pending reboot honestly. S199 package certification cannot create an active handle because direct routing is not implemented here; its lifecycle proof covers exact owned file removal and static Doctor readiness, not active-driver teardown.

The executable's hidden `__build-identity` command binds version, source revision, target triple, architecture, exact official features, native backend, and official-build marker. Package certification also checks PE machine, version resources, ordinary imports, delayed imports, final sizes, shared-file digests, and two constrained loopback native smoke surfaces. One runs the portable ZIP executable and one runs the exact installed MSI executable immediately after clean installation. Both smoke rows must carry the certified `fragcap.exe` entry digest, and their session identity digests must differ so one execution cannot certify both surfaces. Each receives separate synthetic data roots, database, bundle, target declaration and exact-program outbound firewall rule for non-loopback destinations. Each samples the descendant process tree and TCP/UDP endpoints, rejects undeclared executables and any non-loopback observation, and records process and endpoint counts. The exact `conhost.exe` under the current Windows system directory is the sole permitted system descendant because Windows can attach its console host to hidden console-subsystem execution. Positive authority is exactly one structured native proxy-start event followed by one terminal reached-client calibration event from the same session, with exact loopback family, positive port, controlled case and route dimensions. Rust owns the bounded event parsing and emits a closed session summary; PowerShell only orchestrates the process, transports bounded Rust predicate codes and records host observations. Socket sampling remains negative and diagnostic authority: any observed non-loopback endpoint fails, while an empty sample or absence of an exact loopback socket does not. Predicate failures use a closed bounded vocabulary and never include raw paths, addresses, payloads or credentials.

## Failure handling

Do not publish around a failed package workflow. Keep the failed run, inspect its bounded finding and runner-local MSI log, reproduce through a pull request, and fix the smallest owning layer. If a tag run fails before GitHub release creation, cut a new semantic version after the fix rather than moving the tag. Crate publication remains after GitHub release creation and the `crates-io` environment approval.

Update the predecessor only when a newer supported public package becomes the intentional upgrade floor. Acquire it from the official release URL before the offline lifecycle phase, record its exact size and SHA-256, and verify both before any install. Never let an unpinned network download become installer evidence.
