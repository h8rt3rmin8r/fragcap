# Feature Specification: TargetSource discovery seam and discovery tiers

**Feature Branch**: `052-target-source-discovery`

**Created**: 2026-08-17

**Status**: Draft

**Input**: Slice S052 (issue #139, milestone v0.5.0). Implements constitution
principle P-10 (One Path To A Target). Depends on S051 (the target entry model).
Source: fragcap-v0.5.0-UX-Handoff-Plan.md sections 7.1-7.4 and 3.3.

## Overview

A capture target can come from many places: a Steam library, a game installed by
another launcher, a folder the user points at, or a single executable the user
names. Today only Steam is walked, and its accumulation logic is welded to the
Steam crate. This slice introduces one seam, `TargetSource`, through which every
origin produces the same kind of thing: a candidate target. Single-target
authoring and bulk platform walking become the same operation at different batch
sizes, which is exactly what principle P-10 asks for. Adding Epic, GOG, Xbox,
Battle.net, or an emulator ROM directory later becomes a new implementor of the
seam with no downstream change.

This slice delivers the seam, three discovery tiers that populate it, the
directory-shape descent discipline the tiers share, and the persistent volume
eligibility table that makes walking across every fixed drive safe. It does **not**
deliver the engine-signature detection matcher itself (that is slice S053); it
delivers the seam that matcher will plug into and the stop-on-hit descent
contract that keeps the walk fast.

## Clarifications

### Session 2026-08-17

- Q: On a fresh machine with an empty exclusion table, which fixed volumes does
  the known-roots walk enumerate? → A: Auto-allowlist the fixed volumes present at
  first run. On first discovery the system enumerates the currently present
  `DRIVE_FIXED` volumes and records each as eligible in the allowlist, so
  out-of-box discovery walks them (SC-001 holds). A volume that appears later, or a
  mount that misreports as fixed, is not auto-added and requires explicit opt-in.
  This honors both the allowlist framing (a new/misreporting volume is not walked
  by default) and the out-of-box listing requirement.
- Q: What does a bulk source (Steam, known-roots) do with its candidates in S052,
  surface only, or auto-persist into `local.db`? → A: Surface live; persist on
  first use. Discovery runs live each time the listing is shown (always current, no
  stale rows, no setup step), and a candidate becomes a durable `local.db` target
  entry automatically the instant the user acts on it (captures or selects it), via
  the S051 entry model. Bulk scan results are not written into the user-owned store
  at scan time. Only the volume exclusion/eligibility table is persisted by this
  slice.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Steam discovery runs through the shared seam (Priority: P1)

The existing Steam library walk is expressed as a `SteamSource` implementing the
`TargetSource` seam. It walks the Steam library folders and app metadata, emits
one candidate per installed title at the heuristic-unverified fidelity, and each
candidate's Steam appid joins to the shipped catalog so a known title carries its
catalog classification. Nothing about the observable Steam result changes; what
changes is that Steam is now one source among several behind a single interface.

**Why this priority**: This is the seam itself proven against the one real source
that already exists. Without it there is no P-10 seam and no place for tiers 2 and
3 to attach. Refactoring the existing behavior onto the seam with no regression is
the load-bearing first step; every later story is a new implementor of what this
story defines.

**Independent Test**: Drive discovery with a fixture Steam source (canned library
and app metadata, no filesystem) and assert it yields the expected candidates at
heuristic-unverified fidelity with appids that join to catalog fixtures. The real
`SteamSource` is exercised against committed metadata fixtures with no live Steam
install.

**Acceptance Scenarios**:

1. **Given** a Steam install with two library folders holding three installed
   titles, **When** `SteamSource` discovers, **Then** it returns three candidates,
   each at heuristic-unverified fidelity, each carrying its Steam appid.
2. **Given** a candidate whose appid matches a catalog row, **When** discovery
   joins against the catalog, **Then** the candidate carries that row's
   classification; **and** a candidate whose appid is absent from the catalog is
   still returned, classified unknown, never dropped.
3. **Given** an app-metadata entry that cannot be parsed, **When** `SteamSource`
   discovers, **Then** that entry is counted as a parse failure in the run account
   and the remaining titles are still returned (no silent loss).

---

### User Story 2 - A machine without Steam still shows games (Priority: P2)

On a machine with no Steam installed, discovery still produces a non-empty listing
whenever a game launcher's install directory exists. A known-roots source
enumerates a fixed, hard-coded list of directories that only ever contain games
(the Epic, GOG, Riot, Battle.net, Ubisoft, EA, Origin, Xbox, and generic
`Games` roots), and it does so across **every** fixed volume, because a second or
third drive holds games as often as the system drive. Exhaustive enumeration of
every executable on the machine is explicitly rejected: a normal machine carries
thousands of non-Windows executables that are overwhelmingly updaters,
uninstallers, and helpers, and listing them would bury the game the user came for.

**Why this priority**: This is the story that makes the tool useful to the
majority of users who install games outside Steam or on secondary drives. It is
second only to proving the seam because it is the first source the seam was built
to make possible.

**Independent Test**: Drive a fixture volume inventory (a set of named fixed
volumes) and a fixture directory tree, and assert `KnownRootsSource` returns one
candidate per game directory found under any known root on any listed volume,
with no filesystem access in the test.

**Acceptance Scenarios**:

1. **Given** no Steam install but an Epic Games directory containing two game
   folders on the system drive, **When** known-roots discovery runs, **Then** it
   returns two candidates.
2. **Given** a game directory present only on a second fixed volume, **When**
   known-roots discovery runs, **Then** the candidate on the second volume is
   returned (volumes beyond the system drive are enumerated).
3. **Given** a known root that does not exist on a volume, **When** known-roots
   discovery runs, **Then** that root contributes no candidates and no error; a
   missing root is normal, not a failure.

---

### User Story 3 - The user points discovery at a place they know (Priority: P3)

A user who knows exactly where their game is can point discovery straight at it. A
directory source takes one path and yields one candidate. An interactive source
wraps the directory source with a human confirmation step and stamps the accepted
candidate at the authored fidelity, because a human vouched for it. These back the
`targets scan <dir>` and `targets add <exe>` entry points. Authoring one target by
hand and walking a whole platform are, as P-10 requires, the same operation
producing the same kind of candidate.

**Why this priority**: It closes the discovery model to the case the automatic
tiers cannot cover (a standalone title in an unusual place) and gives the user
direct control. It depends on the seam (US1) but not on the automatic tiers.

**Independent Test**: Give the directory source a fixture path and assert one
candidate at the source's default fidelity; give the interactive source a fixture
path plus a scripted confirmation and assert the candidate is stamped authored on
acceptance and omitted on rejection.

**Acceptance Scenarios**:

1. **Given** a directory path that holds a game, **When** the directory source
   discovers, **Then** it returns exactly one candidate for that path.
2. **Given** a user confirmation of yes, **When** the interactive source
   discovers, **Then** the candidate is stamped at authored fidelity.
3. **Given** a user confirmation of no, **When** the interactive source
   discovers, **Then** no candidate is produced and the outcome is counted as
   declined by the user, not lost.

---

### User Story 4 - Walking across drives never touches a volume the user excluded (Priority: P2)

Because known-roots discovery enumerates across every fixed volume, the user needs
a way to keep a volume out of the walk: a network share that reports as fixed, a
userspace or FUSE mount presenting as a fixed drive, or simply a volume they never
want scanned. A persistent, user-editable volume eligibility table records which
volumes are eligible. The design is an allowlist, not a denylist, because a static
denylist cannot recognize a userspace mount that presents itself as an ordinary
fixed drive; eligibility is decided by what the user has affirmed, not by what a
fixed blocklist happens to name. An excluded volume is never enumerated by any
tier-2 walk.

**Why this priority**: It ships with US2 rather than after it, because the moment
known-roots walks across all fixed volumes, an unsafe or unwanted volume can be
touched. The eligibility machinery is the safety layer that makes the cross-volume
walk shippable, so it shares US2's priority.

**Independent Test**: Seed the eligibility table with a volume marked excluded,
present a fixture inventory that includes it, and assert the known-roots walk
never enumerates the excluded volume while still enumerating the eligible ones.

**Acceptance Scenarios**:

1. **Given** a volume recorded as excluded, **When** known-roots discovery runs
   across the inventory, **Then** that volume is not enumerated and none of its
   directories appear as candidates.
2. **Given** the eligibility table is edited to re-include a previously excluded
   volume, **When** discovery runs again, **Then** that volume is enumerated.
3. **Given** an empty eligibility table, **When** discovery runs, **Then** the
   default eligibility rule is applied consistently and the decision for each
   volume is recoverable (why a volume was or was not walked can be stated).

---

### Edge Cases

- A directory matches a game signature at its top level: descent stops there and
  that directory becomes one candidate; the walk does not descend into its
  subtree looking for more (stop-on-hit is what keeps the walk fast).
- A directory contains no signature and no known-root match: it contributes no
  candidate and is counted as considered-but-not-a-game, not silently skipped.
- Two sources surface the same underlying title (Steam appid and a known-root
  directory for the same game): both candidates are produced by their sources;
  reconciling them to one target is the authoring/entry concern of S051 and the
  hero command (not resolved inside a single source).
- A known root exists but the user lacks permission to read it: the failure is
  counted against that root and named; the rest of the walk continues.
- A volume disappears mid-walk (removable media, unmounted share): the walk
  counts the interruption for that volume and continues; no partial candidate is
  emitted for it.
- The interactive source runs in a non-interactive context (no console): it
  produces no candidate and says why, rather than blocking or guessing.

## Requirements *(mandatory)*

### Functional Requirements

**The seam**

- **FR-001**: The system MUST define a single `TargetSource` seam that every
  discovery origin implements, exposing a stable name, a discover operation that
  yields zero or more candidate targets, and a default fidelity the source stamps
  on the candidates it produces.
- **FR-002**: The discover operation MUST return a truthful account of the run in
  addition to the candidates: every item the source considered MUST land in
  exactly one counted outcome (produced, or one of the named non-produced
  outcomes such as parse-failure, declined, or not-a-game), and those counts MUST
  reconcile to the number of items considered (the P-4 conservation discipline the
  catalog seeder already follows).
- **FR-003**: A candidate target MUST carry enough to identify what was found (its
  path or platform identity), the fidelity its source stamped, and any
  classification joined from the catalog, without requiring the source to have
  written anything durable.

**Tier 1 - Steam**

- **FR-004**: The existing Steam library and app-metadata accumulation MUST be
  refactored to implement `TargetSource` as `SteamSource`, walking the library
  folders and app metadata and emitting one candidate per installed title.
- **FR-005**: `SteamSource` MUST stamp its candidates at heuristic-unverified
  fidelity and MUST join each candidate's Steam appid against the shipped catalog,
  carrying the catalog classification when the appid is known and classifying
  unknown (never dropping) when it is not.
- **FR-006**: Refactoring Steam onto the seam MUST NOT change the observable set
  of Steam candidates for an unchanged install (no regression against the prior
  behavior).

**Tier 2 - known roots**

- **FR-007**: The system MUST provide `KnownRootsSource`, which enumerates a
  hard-coded list of directories that only ever contain games. The list for
  v0.5.0 is exactly: the Steam library `steamapps\common`, the default Steam
  `steamapps\common`, Epic Games, GOG Galaxy Games, Riot Games, Battle.net,
  Ubisoft launcher games, EA Games, Origin Games, XboxGames, and the generic
  `Games` root.
- **FR-008**: `KnownRootsSource` MUST enumerate known roots across every eligible
  fixed volume, not only the system volume.
- **FR-009**: The system MUST NOT discover targets by enumerating every executable
  on the machine and asking of each whether it is a game; exhaustive
  executable enumeration is rejected.
- **FR-010**: A known root that does not exist on a given volume MUST contribute no
  candidate and MUST NOT be treated as an error.

**Tier 3 - user-pointed**

- **FR-011**: The system MUST provide `DirectorySource`, which takes one directory
  path and yields at most one candidate for it.
- **FR-012**: The system MUST provide `InteractiveSource`, which wraps
  `DirectorySource` with a human confirmation step and stamps an accepted
  candidate at authored fidelity; a rejected candidate MUST be counted as declined
  by the user, not lost.
- **FR-013**: `DirectorySource` and `InteractiveSource` MUST be the discovery
  mechanism behind the `targets scan <dir>` and `targets add <exe>` entry points.

**Shared descent discipline (seam for S053)**

- **FR-014**: Tiers 2 and 3 MUST classify a directory by its shape (the presence
  of engine-signature artifacts such as a Unity player library or an Unreal
  engine-binaries tree) rather than by a curated per-title list, so that a
  standalone or non-catalog title is recognized as a game.
- **FR-015**: The directory walk MUST test each directory for a signature and stop
  descending on a hit, and MUST NOT enumerate a directory's executables first and
  then ask whether each is a game. (S052 provides this descent-and-stop contract
  and the seam the matcher plugs into; the signature matcher itself is S053.)

**Volume safety (layer 3 only for v0.5.0)**

- **FR-016**: The system MUST persist a user-editable volume eligibility table in
  the user-owned store (`local.db`), expressed as an allowlist of eligible volumes
  rather than a denylist of blocked ones.
- **FR-016a**: On first discovery against an empty eligibility table, the system
  MUST enumerate the fixed volumes present at that moment and record each as
  eligible (seed the allowlist permissively), so out-of-box discovery walks them.
  A fixed volume that first appears after this seeding, or a mount that misreports
  as fixed, MUST NOT be auto-added to the allowlist and MUST require explicit user
  opt-in before it is walked.
- **FR-017**: A volume the eligibility table does not mark eligible MUST never be
  enumerated by any tier-2 walk, and the eligibility decision for each volume MUST
  be recoverable (statable) rather than silent.
- **FR-018**: The specification MUST record, without implementing, the volume
  hazards deferred to v0.6.0 so they are not rediscovered: cloud placeholder
  hydration (the recall-on-open and recall-on-data-access reparse behaviors),
  reparse-point loops, and the within-volume skip list for deep scanning. Deep
  filesystem scanning is out of scope for this slice.

**Testability**

- **FR-019**: Every source MUST be drivable in tests by a fixture with no
  filesystem, volume-inventory, or live-platform dependency, so the whole
  discovery model is testable on any machine.

**Persistence boundary**

- **FR-020**: A source MUST produce its candidates without writing them to
  `local.db`; the discovery listing is computed live from the sources each time it
  is requested (so it is always current and requires no setup step). The only
  durable write this slice makes is the volume eligibility table (FR-016/FR-016a).
- **FR-021**: A candidate MUST become a durable `local.db` target entry (via the
  S051 entry model) automatically at the moment the user acts on it (captures or
  selects it), with no separate manual authoring step. Bulk scan results for games
  the user never acts on are not persisted.

### Key Entities *(include if feature involves data)*

- **TargetSource**: The seam. An origin of candidate targets, identified by name,
  able to discover, carrying a default fidelity it stamps on what it produces.
  Steam, known roots, a directory, and an interactive prompt are all instances.
- **CandidateTarget**: What a source produces. Identifies what was found (a path or
  platform identity), the fidelity stamped by its source, and any catalog
  classification joined in. A candidate is not yet a stored target entry; turning a
  candidate into a durable entry is the S051 entry model's job.
- **Discovery account**: The truthful per-run tally (mirroring the seeder's
  conserved summary): items considered, items produced, and each named
  non-produced outcome, reconciling so nothing is silently dropped.
- **Volume eligibility table**: A persistent, user-editable record in `local.db`
  expressing which fixed volumes are eligible for the cross-volume walk (an
  allowlist), plus enough to state why a volume was or was not walked. It is what
  backs a user's exclusion of a volume.
- **Fixed volume inventory**: The set of fixed volumes the walk considers, supplied
  as a value so discovery is a pure decision over it in tests.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a machine with no Steam installed but at least one known game
  root present, discovery returns a non-empty listing (the empty-table failure the
  slice exists to remove).
- **SC-002**: 100% of the discovery model is exercised by tests that use fixture
  sources and fixture volume inventories with no filesystem, no live platform, and
  no specific machine required.
- **SC-003**: A volume marked excluded is enumerated zero times across every
  tier-2 walk in every test that includes it.
- **SC-004**: For every discovery run, the sum of the named outcome counts equals
  the number of candidates considered (conservation holds; no candidate is
  unaccounted for).
- **SC-005**: Refactoring Steam onto the seam produces the identical observable set
  of Steam candidates for an unchanged install versus the pre-refactor behavior
  (zero regression).
- **SC-006**: Adding a new platform origin requires implementing only the
  `TargetSource` seam, with no change to the discovery driver, the tiers, or the
  entry model (verified by the shape of the seam, e.g. a fixture source added in a
  test needs no other change).

## Assumptions

- **Fidelity vocabulary reused, not reinvented**: `default_fidelity` reuses the
  existing `FidelityTier` from the profile crate (the same vocabulary S051's entry
  model reuses), so authored, verified, and heuristic-unverified mean one thing
  across the project.
- **Discovery surfaces candidates live; it persists on first use** (settled, see
  Clarifications and FR-020/FR-021): a source produces candidates for the live
  listing and writes nothing to `local.db`; a candidate becomes a durable entry via
  the S051 entry model automatically the instant the user captures or selects it.
  The only durable write this slice makes is the volume eligibility table, because
  the cross-volume walk needs it to be safe now. This keeps P-10's "one path to a
  target" intact: candidates from every source flow through the same entry model to
  become targets.
- **The detection matcher is S053**: This slice defines the descent-and-stop
  contract and the seam the matcher attaches to (FR-014, FR-015). The actual
  signature table and the matching logic land in S053; here a fixture classifier
  stands in so the descent discipline is testable.
- **Catalog join is read-only against the shipped `catalog.db`**: `SteamSource`
  reads catalog classifications; it does not write the catalog. The two-store split
  from S050 (catalog.db shipped/disposable, local.db user-owned) is assumed in
  place.
- **Cross-source deduplication is out of scope**: When two sources surface the same
  underlying title, both candidates are produced; collapsing them to one target is
  the entry model's identity/merge concern (S051) and the hero command's listing
  concern (S055), not a per-source responsibility.
- **The eligibility default is permissive-seed, then allowlist** (settled, see
  Clarifications and FR-016a): the first discovery seeds the allowlist with the
  fixed volumes present at that moment, so the tool is useful out of the box; from
  then on the allowlist framing is what lets a user carve out a mount that
  misreports as fixed and stops a later-appearing volume from being walked without
  an explicit opt-in. Each per-volume eligibility decision is statable.

## Dependencies

- **S051 (target entry model)**: candidates become durable targets through the
  entry model's handle, identifier, and classification vocabulary. Merged.
- **S050 (two-store split)**: the volume eligibility table lives in `local.db`; the
  catalog join reads `catalog.db`. Merged.
- **Constitution P-10 (One Path To A Target)**: the governing principle this slice
  operationalizes; P-4 (No Silent Loss) governs the discovery account; P-9 (The
  Instrument Does Not Lie) governs that unknown candidates are classified unknown,
  never dropped or guessed.
- **Related issue #135**: the getting-started QA feedback requesting
  auto-discovery, which this slice addresses.
