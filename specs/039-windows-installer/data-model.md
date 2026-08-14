# Phase 1 Data Model: Windows installer and hint-database default

This slice introduces no database schema change (the hint store is already at
schema v2 from S038). The "entities" here are the distribution and path concepts
the requirements govern.

## Default hint-database location

- **What**: the per-user, writable path fragcap uses for its hint database when
  the operator supplies no explicit `--hint-db` flag and no `FRAGCAP_HINT_DB`
  override.
- **Value**: `%APPDATA%\fragcap\hint.db`, a sibling of the existing
  `%APPDATA%\fragcap\profiles` profile directory.
- **Resolution**: `Some(path)` when the application-data base is resolvable,
  `None` otherwise (the run then proceeds with no default, exactly as the profile
  directory already degrades).
- **Precedence**: explicit flag > `FRAGCAP_HINT_DB` > this default. The explicit
  sources retain their current semantics (absent is non-fatal and never created;
  unopenable is a loud error). The default is the only source the bootstrap ever
  creates.

## Hint-database template

- **What**: a read-only copy of the barebones database shipped beside the
  executable by both distribution forms (under the install directory for the MSI;
  beside the unzipped exe for the portable archive).
- **Location**: `<directory of current executable>\hint.db`.
- **Use**: the first-run bootstrap copies it to the default location when the
  default is absent; if no template is present, an empty store is created instead.
- **State**: read-only at rest; never written in place. The writable copy is the
  per-user default.

## Barebones hint database

- **What**: an empty, current-schema (v2) hint store.
- **Source**: `assets/hint-seed.json`, a `kind:"export"` document with an empty
  `records` array, imported offline via `fragcap targets import`.
- **Invariant**: valid current-schema; the existing export path round-trips it to
  a valid, empty `kind:"export"` document (no rows, no hand-massaging).
- **Growth**: filled by S038 local accumulation from the user's own machine and,
  later, opt-in community sync (#94). The full curated corpus is an out-of-band
  maintainer artifact, not this file.

## Installer

- **What**: the per-machine MSI artifact.
- **Fields / attributes**:
  - Install location: the platform program-files (64-bit) directory,
    subdirectory `fragcap`.
  - Payload: `fragcap.exe`, the `hint.db` template (read-only), `LICENSE`,
    `NOTICE`.
  - System path entry: the install directory, added to the machine path
    (`Part=last`, non-permanent), effective in newly opened terminals.
  - Upgrade identity: a stable, frozen `UpgradeCode` GUID; `ProductVersion`
    derived from the crate version at build time; `MajorUpgrade` so a later
    version replaces rather than duplicates.
  - Add/Remove entry: standard, with an About URL.
  - Defender exclusion: the install directory, added on install and removed on
    uninstall, best-effort (a refusal does not fail the install).
  - npcap link: surfaced on the exit dialog (opens the download page when the
    user opts in); the driver itself is never bundled, downloaded, or installed.
- **Signing**: none this release (unsigned by design; #79).

## Release artifact set

For a tagged build, the release exposes exactly:

- **Portable archive**: `fragcap-<version>-x86_64-pc-windows-msvc.zip` containing
  `fragcap.exe`, `LICENSE`, `NOTICE`, and `hint.db` (the database beside the exe
  so the portable user gets the same first-run bootstrap).
- **Installer**: the unsigned `.msi`.
- **Loose database**: a standalone copy of `hint.db`.
- **Checksums**: a `.sha256` for each of the three above.

## State transitions (first-run bootstrap)

```text
default path resolved (no explicit flag/env)
  ├─ default file exists ............................ no-op (leave as-is)
  └─ default file absent
       ├─ parent dir created
       ├─ template beside exe present ............... copy template -> default
       └─ template absent ........................... Store::open(default) -> empty schema
  (any failure of the above) ........................ warn, proceed without a default DB
```

Explicit `--hint-db`/`FRAGCAP_HINT_DB` never enters this transition: an absent
explicit path stays absent (non-fatal, no provider), a present one is used, a
present-but-unopenable one is a loud error in `build_resolver` (unchanged).
