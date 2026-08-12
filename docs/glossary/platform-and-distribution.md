# Platform and Distribution

## npcap

A Windows packet capture driver and library, the current successor to WinPcap.

{: .matters }
> **npcap is not redistributable.** fragcap detects it rather than shipping it,
> and no distribution artifact contains it. Two non-default installation
> options are required: loopback traffic capture support and WinPcap API
> compatible mode. Both are verifiable from the registry, which is how
> `fragcap doctor` names the specific missing option.

**See also:** [Loopback](capture-and-networking.md#loopback)

**References:**

- npcap project documentation, https://npcap.com. Installation options and
  license terms.

## Game profile

A TOML file describing a game's process topology, stage match rules, and
capture defaults. Versioned: every profile declares a
[profile schema version](process-and-attribution.md#profile-schema-version), and a reference to one
resolves through the [profile resolution order](process-and-attribution.md#profile-resolution-order).

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

**See also:** [Library discovery](platform-and-distribution.md#library-discovery)

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

Starting a title through Steam's protocol handler after fragcap is already
watching and its capture handle is open, so every process in the launch chain
produces a start event fragcap observes.

{: .matters }
> Managed launch eliminates the acquisition race: a launcher whose whole lifetime
> is shorter than any poll interval is still observed, because the watcher is
> armed before the launch is issued. It requires `game.platform` and
> `game.app_id` in the profile, and it opens no process handle (constitution
> P-1).

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
