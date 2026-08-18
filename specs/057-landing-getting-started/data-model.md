# Phase 1 Data Model: Landing page and getting-started rewrite

This slice is documentation plus one reporting-removal code change; it defines no
new persistent data. The "entities" are the documentation surfaces and the doctor
report structure they mirror.

## Entity: Doctor report (modified)

The environment readiness report `fragcap doctor` emits. Structure after the
change (see [contracts/doctor-report.md](contracts/doctor-report.md)):

- **Identity**: rows `version`, `binary`, `catalog db`, `local db`. (Removed:
  `profile dir`.)
- **Platform**: `os`, `subsystem`, `privilege`.
- **Capture driver**: `npcap`, `loopback adapter`, `winpcap api mode`, `live
  backend`, `socket-table backend`.
- **Tracing**: `process events`.
- **Interfaces**: one `adapter` row per interface.
- **Integration**: `analyzer extcap`.
- **Preparation** (S056, absent-only): `catalog store`, `target entries`.
- Removed entirely: the **Profiles** section (`profiles: bundled, user`).

`Inputs` fields removed: `profile_dir`, `bundled_count`, `user_count`.

## Entity: Getting-started guide

The first-run narrative. Ordered sections after the rewrite:

1. **Before you begin** - the dependency model (npcap required, Wireshark
   recommended, extcap optional) as diagram + prose; the conditional
   prerequisite-install walkthrough (skip if present); the three expectations
   (payloads encrypted, launcher handoff not captured, loopback is self-talk); the
   extcap integration named descriptively (no command yet).
2. **1. Install fragcap** - download affordance (releases link; `.msi`/`.zip`/
   `.sha256` named); SmartScreen/checksum guidance; MSI's optional extcap step.
3. **2. Verify the install** - run the terminal as Administrator; `fragcap doctor`
   sample (matching the corrected binary); elevation turns privilege/capture green;
   the optional `fragcap extcap install` home.
4. **3. Find a target** - `fragcap targets` (discovery is automatic and the happy
   path); Steam App ID defined inline; the non-Steam `targets add`/`scan` route;
   the numbered row is what `fragcap capture <n>` honors.
5. **4. Capture** - `fragcap capture <n>` (or `--target <handle>`); it arms, waits,
   attributes, writes `capture.fcapng`.
6. **5. Open the result in Wireshark** - the endpoint; attribution in packet
   comments; unmodified analyzers read it as ordinary pcapng.

Endpoint invariant: following the guide literally produces a `.fcapng` file
(SC-001).

## Entity: Landing page

`site/app/(home)/page.tsx`. Ordered blocks:

1. Masthead (logo + wordmark + version).
2. Settled opening: the "40,000 packets" sentence + the two explanatory paragraphs
   (FR-001).
3. Prerequisite callout (npcap required, Wireshark to read).
4. Worked example: the `fragcap targets` hero listing in the monospace face
   (FR-002).
5. Dependency-model diagram (FR-003).
6. A small number of capability statements, each linking to a page that still
   exists (no `writing-a-profile` link).
7. One primary call to action: Get started (FR-003); plus secondary nav
   (Repository, Glossary, Changelog).

Prohibited (FR-004): testimonials, feature grids, badges, pricing, sponsorship.

## Entity: CLI reference

`site/content/docs/reference/cli.mdx`. Mirrors
[contracts/cli-reference-surface.md](contracts/cli-reference-surface.md): nine
commands grouped Capture / Targets / Environment / Data, global options, no retired
commands.

## Entity: Retired-token inventory

The forbidden-token set and its carrier files:
[contracts/retired-token-inventory.md](contracts/retired-token-inventory.md). The
SC-002 gate is a grep over `site/` returning no retired usages.

## Relationships

- Getting-started's doctor sample DEPENDS ON the doctor report entity (must match
  after the code change).
- Landing, getting-started, CLI reference, capture-modes ALL DEPEND ON the retired-
  token inventory being emptied.
- index.mdx / architecture.mdx / meta.json / landing DEPEND ON the two deleted
  pages being unreferenced.
