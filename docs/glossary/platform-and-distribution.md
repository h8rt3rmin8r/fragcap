# Platform and Distribution

## Supply-chain gate

The combined offline and network-backed authority that decides whether fragcap's locked dependency graph remains eligible for release. The offline half compares complete all-feature Windows and Linux graphs plus the exact shipped Windows closure with one reviewed policy. The network half evaluates current advisories, yanks, maintenance notices, licenses, bans, and sources.

{: .matters }
> A graph cannot disappear from review because its feature is optional or its target is inactive on the current host. Unknown or stale authority fails closed, and no supply-chain exception may disable the whole gate.

**See also:** [Compatibility line](#compatibility-line), [Finite dependency exception](#finite-dependency-exception), [Software bill of materials](#software-bill-of-materials), [Third-party notices](#third-party-notices), [Capability feature](#capability-feature)

## Compatibility line

A set of Cargo versions expected to be semver-compatible: the major number at or above 1.0, and `0.minor` before 1.0. Simultaneous lines for one package represent independently maintained code lineages and require an exact finite policy record.

{: .matters }
> Two patch releases inside one line remain visible but are not mislabeled as a duplicate-major problem. Adding a new major or pre-1.0 minor line fails until its owner, rationale, expiry, and removal condition are reviewed.

**See also:** [Supply-chain gate](#supply-chain-gate), [Finite dependency exception](#finite-dependency-exception)

## Finite dependency exception

A temporary approval for one exact supply-chain rule and package version. It carries a unique identity, owner, rationale, creation date, expiry date, and observable removal condition.

{: .matters }
> Wildcards, global bypasses, missing owners, expired records, and unused records fail the gate. An unavailable advisory database is infrastructure failure and cannot be excused as though it were a package finding.

**See also:** [Supply-chain gate](#supply-chain-gate), [Compatibility line](#compatibility-line)

## Software bill of materials

**Also known as:** SBOM

The CycloneDX 1.5 machine-readable inventory generated from the exact locked Windows release graph. It records component identities and dependency relationships and is bound to the fragcap version, source revision, target, feature set, lockfile digest, and supply-chain policy digest.

{: .matters }
> Both the portable archive and MSI contain the validated SBOM. Build-only and development-only packages are not represented as distributed runtime libraries, and stale or missing components stop packaging.

**See also:** [Supply-chain gate](#supply-chain-gate), [Third-party notices](#third-party-notices), [MSI installer](#msi-installer)

## Third-party notices

The human-readable dependency identity, license expression, source, attribution, and legal text generated from the same exact release graph as the [software bill of materials](#software-bill-of-materials).

{: .matters }
> This file is dependency evidence, not the project's Apache `NOTICE` and not release notes. It is generated and reconciled before packaging, then installed beside fragcap in both release forms.

**See also:** [Software bill of materials](#software-bill-of-materials), [Supply-chain gate](#supply-chain-gate), [MSI installer](#msi-installer)

## npcap

A Windows packet capture driver and library, the current successor to WinPcap.

{: .matters }
> **npcap is not redistributable.** fragcap detects it rather than shipping it,
> and no distribution artifact contains it. npcap is by the Nmap Project. The
> installation option that matters is WinPcap API compatible mode, which fragcap
> links against; current npcap installs loopback capture support automatically,
> so it is no longer a separate option to enable. The mode is verifiable from the
> registry, which is how `fragcap doctor` names it when it is missing.

**See also:** [Dependency model](platform-and-distribution.md#dependency-model),
[Loopback](capture-and-networking.md#loopback)

**References:**

- npcap project documentation, https://npcap.com. Installation options and
  license terms.

## Dependency model

fragcap's external tools fall into three tiers:

- **Required: [npcap](platform-and-distribution.md#npcap).** The capture driver.
  Without it fragcap captures nothing; `fragcap doctor` fails its npcap check.
- **Recommended: Wireshark.** The analyzer captures are opened in. Its installer
  also bundles and installs npcap, so it is the simplest way to obtain both. This
  tier is documentation guidance rather than a check: `doctor` does not test for
  Wireshark itself.
- **Optional: the [extcap](windows-internals.md#extcap) integration.** It ships
  with Wireshark and lets fragcap feed it live; it needs only `fragcap extcap
  install` to register. `doctor` warns when it is not registered and labels the
  row optional, never a blocker.

{: .matters }
> This entry is the single source for the tiers. The README and the Getting
> Started page summarize and link here rather than restating them. Where
> `fragcap doctor` can enforce a tier it does (npcap as a failing check, the
> extcap registration as an optional warning), so the tool and the docs do not
> disagree; the recommended tier is an onboarding recommendation, not a doctor
> check. npcap is by the Nmap Project, and fragcap detects it rather than
> bundling it (specification section 20.2).

**See also:** [npcap](platform-and-distribution.md#npcap),
[extcap](windows-internals.md#extcap),
[Capability feature](platform-and-distribution.md#capability-feature),
[Readiness check](command-line-and-diagnostics.md#readiness-check)

## Game profile

A JSON file describing a game's process topology, stage match rules, and
capture defaults. It is the `profile` variant of the
[master target schema](process-and-attribution.md#master-target-schema); every profile declares a
[profile schema version](process-and-attribution.md#profile-schema-version), and a reference to one
resolves through the [profile resolution order](process-and-attribution.md#profile-resolution-order).
The format moved from TOML to JSON in the profile migration (#76).

{: .matters }
> Profiles are data, not code. They carry the same license as the repository,
> and a contributor can add support for a title without writing Rust. Validation
> reports every problem in a profile rather than stopping at the first, because
> the population writing these files is not the population that can debug a
> parser.

**See also:** [Stage](process-and-attribution.md#stage), [Lifecycle class](process-and-attribution.md#lifecycle-class),
[Terminal stage](process-and-attribution.md#terminal-stage), [Match predicate](process-and-attribution.md#match-predicate),
[Ambiguous image match](process-and-attribution.md#ambiguous-image-match),
[Profile schema version](process-and-attribution.md#profile-schema-version),
[Profile resolution order](process-and-attribution.md#profile-resolution-order),
[Duration literal](capture-and-networking.md#duration-literal)

## VDF

**Also known as:** Valve key-value format

Valve's key-value text format. Steam records library locations
(`libraryfolders.vdf`) and per-title metadata (`appmanifest_<app_id>.acf`) in
it: quoted keys, quoted-or-nested-block values, line comments, and backslash
escapes.

{: .matters }
> fragcap parses the subset these two manifest kinds use with a small hand-rolled
> parser rather than a dependency, because the format is small and stable
> (specification section 16.2). A malformed manifest is reported and skipped, not
> fatal, so one bad file does not hide every good one.

**See also:** [Library discovery](platform-and-distribution.md#library-discovery),
[Application-info cache](platform-and-distribution.md#application-info-cache)

## Application-info cache

Steam's local binary file (`appcache/appinfo.vdf`) recording, per application it
has fetched metadata for, a change-number and a launch configuration. fragcap
reads it during
[launch-data accumulation](process-and-attribution.md#launch-data-accumulation) as
the source of a title's launch executables. It is a distinct format from the text
[VDF](#vdf) of `libraryfolders.vdf` and the `.acf` manifests: a length-prefixed
binary key-values format (with a string table for keys in its current version)
that a hand-rolled parser reads by hand, framing each application's section by a
size field so a malformed section is isolated and the walk resyncs.

{: .matters }
> Reading it needs no Steam session and no network: it is a file Steam already
> wrote, so the read is passive and opens no process handle (P-1). A hand-rolled
> binary offset can be self-consistent with a synthetic fixture yet wrong against a
> real file, so the parser is validated offline and against a real cache manually,
> like live capture.

**See also:** [VDF](#vdf),
[Launch-data accumulation](process-and-attribution.md#launch-data-accumulation),
[Game profile](platform-and-distribution.md#game-profile)

## Library discovery

Reading Steam's local metadata to enumerate installed titles: fragcap locates
the Steam installation through its Windows registry entry, reads the
library-folders manifest to find every library, and reads every application
manifest across them, yielding each installed title with its application
identifier and install directory.

{: .matters }
> Discovery reads local files and the registry only. It installs nothing,
> downloads nothing, and runs no Steam component, the same detection-not-bundling
> posture the project holds toward [npcap](platform-and-distribution.md#npcap).

**See also:** [VDF](platform-and-distribution.md#vdf), [Profile scaffolding](platform-and-distribution.md#profile-scaffolding),
[Managed launch](platform-and-distribution.md#managed-launch)

## Profile scaffolding

Generating a [game profile](platform-and-distribution.md#game-profile) skeleton from an installed title.
fragcap scans the install directory for executable images, proposes
launcher-suggestive images as launcher stages and the largest remaining image as
the client, and emits a profile that passes section 15.4 validation unedited.

{: .matters }
> The output is a heuristic starting point, marked as such in a header comment,
> and must be verified against an observed capture session: image names alone
> cannot tell a launcher from a client, and a title may run several processes
> sharing one image name. The scaffold never infers process ancestry from a
> static scan.

**See also:** [Game profile](platform-and-distribution.md#game-profile), [Stage](process-and-attribution.md#stage),
[Ambiguous image match](process-and-attribution.md#ambiguous-image-match),
[Library discovery](platform-and-distribution.md#library-discovery)

## Managed launch

Starting a stored title after fragcap is already watching and its capture handle
is open, so every process in the launch chain produces a start event fragcap
observes. A Steam-anchored target uses the existing protocol handler. A direct
target uses one exact client path beneath its stored install root, an explicit
working directory, and an argument vector without a command shell.

{: .matters }
> Managed launch eliminates the acquisition race: a launcher whose whole lifetime
> is shorter than any poll interval is still observed, because the watcher is
> armed before the launch is issued. Direct launch creates the selected child
> but does not inspect it or retain a second process observer; ETW and the socket
> table remain authoritative (constitution P-1).

**See also:** [Launcher chain](process-and-attribution.md#launcher-chain),
[Acquisition timeout](process-and-attribution.md#acquisition-timeout), [Game profile](platform-and-distribution.md#game-profile)

## Capability feature

A compile-time Cargo feature that links one of fragcap's platform backends into
the binary: `live` (the npcap capture source), `socket-table` (the IP Helper
attribution backend), and `etw` (the process-event tracing source). `fragcap
doctor` reports which are present, and a release binary ships with all three.

{: .matters }
> Presence is a property of the built binary, not the machine around it, so
> `doctor` reports it as a first-class fact: a binary without `live` cannot
> capture at all, a blocking failure rather than a downstream "no interfaces"
> symptom. Shipping the binary without these features once made every capture
> fail while the readiness report still read "ready".

**See also:** [npcap](platform-and-distribution.md#npcap), [Readiness check](command-line-and-diagnostics.md#readiness-check), [Socket table](process-and-attribution.md#socket-table)

## MSI installer

The Windows Installer package (`.msi`) fragcap publishes for a release. It
installs the binary per-machine under the program-files directory, adds that
directory to the system path, ships the barebones targets hint database beside
the binary as the template the first-run bootstrap seeds from, registers an
uninstall entry, best-effort excludes its install directory from Windows
Defender, and links the npcap download page on completion.

{: .matters }
> The installer is a convenience over the portable archive, not a second product:
> it carries the same binary and the same hint database, and the release also
> publishes both on their own so a user can decline it. It bundles no npcap
> (specification section 20.2); it only links the download page. Its runtime
> behavior is verified by hand, like live capture, because the automated check
> set cannot install it.

**See also:** [Unsigned installer](platform-and-distribution.md#unsigned-installer),
[Windows Defender exclusion](platform-and-distribution.md#windows-defender-exclusion),
[npcap](platform-and-distribution.md#npcap)

## Unsigned installer

A distribution installer published without an Authenticode code signature.
fragcap's [MSI installer](platform-and-distribution.md#msi-installer) is unsigned
for the current release, so Windows SmartScreen shows an unrecognized-publisher
warning when it runs.

{: .matters }
> An unsigned installer is labeled as unsigned rather than implying a trust it
> cannot prove: the documentation states the SmartScreen warning is expected and
> that verifying the published SHA-256 checksum is the integrity check in place of
> a signature (P-9). Code signing is a separate, non-blocking track (issue #79).

**See also:** [MSI installer](platform-and-distribution.md#msi-installer)

## Windows Defender exclusion

A path added to Microsoft Defender's exclusion list so its scanning skips that
location. fragcap's [MSI installer](platform-and-distribution.md#msi-installer)
best-effort adds its own install directory on install and removes it on
uninstall.

{: .matters }
> The exclusion is scoped to fragcap's own install directory and is an
> installer and operating-system configuration action, not a capture technique: it
> opens no process handle and touches no target process, its memory, its traffic,
> or the network stack, so it is outside the technique denylist (constitution
> P-1). It is best-effort because Windows Tamper Protection can refuse it even for
> an elevated installer, and a refusal must not fail the install.

**See also:** [MSI installer](platform-and-distribution.md#msi-installer),
[Unsigned installer](platform-and-distribution.md#unsigned-installer)

## Artifact identity

The version, source revision, target, architecture, feature set, filename, content digest, checksum, signature state, and package metadata that jointly identify one final release download.

{: .matters }
> S131 requires every field to agree before publication, so a correctly named package containing stale, rebuilt, wrong-feature, or wrong-target bytes cannot inherit another artifact's certification.

**See also:** [Package certification](platform-and-distribution.md#package-certification), [Release build identity](platform-and-distribution.md#release-build-identity)

## Package contract

The versioned closed declaration of official release artifacts, exact entries, size ceilings, prohibited content, PE imports, signature policy, installer ownership, lifecycle cases, and publication order.

{: .matters }
> Unknown content and missing required rows fail rather than being treated as harmless packaging variation. Changes to the contract are release-policy changes and require review.

**See also:** [Package entry](platform-and-distribution.md#package-entry), [Package certification](platform-and-distribution.md#package-certification)

## Package entry

One canonical relative file inside a release package, identified by role, size ceiling, byte digest, ownership class, and every artifact surface on which it must appear.

{: .matters }
> Paths are case-unique, non-traversing, and byte-compared across ZIP, MSI, and standalone surfaces where applicable. A permitted basename does not excuse a wrong digest or undeclared native import.

**See also:** [Package contract](platform-and-distribution.md#package-contract), [Artifact identity](platform-and-distribution.md#artifact-identity)

## Release build identity

The machine-readable compile-time record of fragcap version, source revision, target triple, architecture, exact official feature set, native backend, and official-build status.

{: .matters }
> `--version` proves only semantic version. The fuller identity lets final-package certification reject a local rebuild, stale source, wrong target, or feature-incomplete executable without inferring from its filename.

**See also:** [Capability feature](platform-and-distribution.md#capability-feature), [Artifact identity](platform-and-distribution.md#artifact-identity)

## Installer effect

One exact installer-owned file or machine-state change with declared install, repair, upgrade, and removal expectations.

{: .matters }
> Ownership requires a stable product, component, path, or effect marker. A display name, filename, process identifier, or pre-existing administrator-owned Defender exclusion cannot authorize deletion.

**See also:** [MSI installer](platform-and-distribution.md#msi-installer), [Windows Defender exclusion](platform-and-distribution.md#windows-defender-exclusion)

## Lifecycle case

One finite package transition with exact preconditions, installer operation, accepted exit status, state observations, deadline, cleanup result, and terminal classification.

{: .matters }
> Clean install, repair, exact-byte reinstall, supported upgrade, downgrade refusal, and uninstall are required rows. A skipped, timed-out, warning-only, or unreconciled row cannot authorize a release.

**See also:** [Installer effect](platform-and-distribution.md#installer-effect), [Package certification](platform-and-distribution.md#package-certification)

## Package certification

The blocking reconciliation of final ZIP, MSI, catalog, and checksum bytes against the [package contract](platform-and-distribution.md#package-contract), including package contents, size, PE imports, build identity, unsigned state, packaged native smoke, installer lifecycle, exact cleanup, and publication ordering.

{: .matters }
> Certification is attached to the bytes built once and later published. Rebuilding after the check, continuing after a failed smoke, or publishing before post-transfer revalidation would break that authority.

**See also:** [Artifact identity](platform-and-distribution.md#artifact-identity), [Lifecycle case](platform-and-distribution.md#lifecycle-case), [Unsigned installer](platform-and-distribution.md#unsigned-installer)
