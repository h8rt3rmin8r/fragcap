# Feature Specification: Local Steam launch-data accumulation

**Feature Branch**: `038-launch-accumulation`

**Created**: 2026-08-14

**Status**: Draft

**Input**: User description: "S038 local appinfo launch-data accumulation. Each
end user's copy of fragcap accumulates its own Steam game launch data locally
and privately, reading the user's own local Steam appinfo cache, so that the
launch executable of a game becomes available for target resolution without any
of that data ever being shipped or shared."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Learn a game's launch executable from the local Steam cache (Priority: P1)

A user starts a capture against a Steam game they own. Before the capture arms,
fragcap consults the user's own machine to learn which executable Steam launches
for that game, and records that fact in a private local store so the resolver
can name the running client. Nothing about the user's library leaves the
machine.

**Why this priority**: This is the whole point of the slice. Without it the
local store holds no launch executables, and a game whose client is not
name-similar to its title and not resolvable by install layout cannot be
attributed. It is the smallest change that delivers standalone value: one game
learned is one game the resolver can now name.

**Independent Test**: Point the accumulator at a fixture that stands in for the
local Steam cache and a fresh local store, run it, and confirm the store now
holds the launch executable(s) for the app the fixture describes, with no
network access and no new dependency.

**Acceptance Scenarios**:

1. **Given** a local store with no launch data for app X and a local Steam cache
   describing app X with one launch executable, **When** accumulation runs,
   **Then** the store afterwards holds that launch executable for app X and the
   run reports it as written.
2. **Given** the same inputs but app X has several launch entries filtered by
   operating system, **When** accumulation runs, **Then** every entry Steam
   records is stored verbatim (the record is not reduced or normalized), and the
   resolver's existing Windows-client selection applies later at resolution time.
3. **Given** a local store that already holds public catalog and engine data for
   app X (its name, popularity, and engine), **When** accumulation writes launch
   data for app X, **Then** the catalog and engine columns are unchanged.

---

### User Story 2 - Skip games already known, refresh only what Steam changed (Priority: P2)

On every later capture the walk runs again, but a game whose launch data the
store already holds at the version Steam currently records is skipped, so repeat
runs stay fast. A game Steam has updated since it was learned is re-read.

**Why this priority**: Accumulation is only tolerable at capture start if the
steady state is cheap. This is what turns a slow first run into fast later runs
and lets a personal collection grow without repeated work. It depends on P1
existing but is independently testable.

**Independent Test**: Run accumulation twice against an unchanged fixture and
confirm the second run reads no app's launch config and reports every app as
skipped; then advance one app's recorded change-number in the fixture and
confirm only that app is re-read on the third run.

**Acceptance Scenarios**:

1. **Given** a store holding launch data for app X recorded at change-number N
   and a cache still at change-number N for app X, **When** accumulation runs,
   **Then** app X's launch config is not parsed again and app X is reported as
   skipped.
2. **Given** the same store but the cache now records change-number N+1 for app
   X, **When** accumulation runs, **Then** app X's launch config is re-read and
   the stored launch data is replaced with the newer version.
3. **Given** a store holding launch data for app X and a cache that no longer
   lists app X, **When** accumulation runs, **Then** app X's existing launch
   data is left in place (accumulation never prunes).

---

### User Story 3 - An honest account of the walk (Priority: P3)

When accumulation finishes it reports how many games it considered and how each
one turned out, so a walk that was cut short or that skipped unreadable games can
never be mistaken for one that covered everything.

**Why this priority**: A capture tool that silently under-collects produces
conclusions the user cannot detect are incomplete (P-4, P-9). The account is what
makes the collection trustworthy. It layers on P1 and P2 and is independently
verifiable.

**Independent Test**: Run accumulation against a fixture mixing writable,
already-current, and deliberately malformed app entries, and confirm the reported
counts sum to the number of apps considered and name each outcome.

**Acceptance Scenarios**:

1. **Given** a library of apps, some new, some current, some with a malformed
   launch config, **When** accumulation runs, **Then** every app lands in exactly
   one of written, skipped-as-current, or failed, and the three counts sum to the
   number of apps considered.
2. **Given** an app whose entry cannot be parsed, **When** accumulation runs,
   **Then** that app is counted as failed and the walk continues to the remaining
   apps rather than aborting.
3. **Given** a first run over a large library, **When** accumulation runs, **Then**
   progress is surfaced so a slow first run is visibly working rather than
   appearing hung.

---

### Edge Cases

- **No Steam, or no appinfo cache**: accumulation finds nothing to walk, records
  no launch data, and reports zero apps considered rather than failing the
  capture. The capture proceeds without it.
- **The appinfo cache is malformed or truncated at the top level**: accumulation
  surfaces the fault and records nothing from that file, rather than writing a
  half-parsed record; the capture still proceeds.
- **An app has no launch config, or an empty launch list**: it is not a parse
  failure. It is considered, produces no launch executable, and is accounted for
  as a considered app that yielded nothing to write (it is not counted as
  failed).
- **An app's launch entry names no executable**: the entry is not stored (an
  executable is the one required field); if that leaves the app with no storable
  entry it yields nothing to write.
- **The local store cannot be opened or written**: this is an error surfaced to
  the operator, not a silent skip, because the operator asked for accumulation.
- **A launch entry carries operating-system or beta-branch filters**: they are
  stored verbatim alongside the executable; this slice does not evaluate or drop
  entries by filter.

## Clarifications

### Session 2026-08-14

- Q: Where is the per-application change-number stored, given the store has no
  such column today? → A: This slice adds a nullable `appinfo_change_number`
  column to the games table via the store's first schema-version migration (v1
  to v2). It is a store-internal, additive migration; existing v1 stores are
  migrated forward with the column left null.
- Q: Which store does accumulation write to, and when does it run? → A: The same
  local hint database the resolver already reads (the one supplied by the hint-db
  option or its environment variable). Accumulation runs automatically at capture
  start only when such a database is configured; with no hint database configured
  there is no accumulation and no hint resolution, unchanged from today.
- Q: Which applications does the walk consider? → A: Only the installed library
  (applications carrying an install manifest), enumerated by the existing Steam
  library walk. The local appinfo cache is the source of launch configuration for
  those application ids; it is not itself walked for every application it happens
  to contain.
- Q: Does this slice populate launcher-mediated? → A: No. This source has no
  reliable field for it, and guessing would violate P-9. The slice writes launch
  entries only; the launcher-mediated flag and the token-required attribute are
  both left exactly as they were.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST read game launch data only from the user's own
  local Steam application-info cache on the machine fragcap runs on. It MUST NOT
  obtain launch data from any shipped data file or any remote source.
- **FR-002**: The set of applications the system considers MUST be the installed
  Steam library (applications carrying an install manifest), enumerated by the
  existing Steam library walk. The local appinfo cache is read as the launch-data
  source for those applications; it is not itself walked for every application it
  may contain. For each considered application the system MUST determine whether
  the local store already holds launch data for that application at the version
  the local cache currently records.
- **FR-003**: When the store lacks launch data for an application, or the local
  cache records a newer version than the store recorded, the system MUST read
  that application's launch configuration from the local cache and store its
  launch executable(s) and their metadata, together with the version marker used
  to decide staleness.
- **FR-004**: When the store already holds launch data for an application at the
  version the local cache currently records, the system MUST NOT re-read that
  application's launch configuration, and MUST account for it as skipped.
- **FR-005**: The system MUST store each launch entry as the cache records it,
  without reducing, reordering, normalizing, or filtering entries (P-9). Any
  reduction to a single client executable is the resolver's concern at
  resolution time, not the accumulator's.
- **FR-006**: Writing launch data for an application MUST leave that
  application's public catalog columns (name, popularity metrics) and engine
  columns untouched, so accumulation never clobbers shipped data.
- **FR-007**: The system MUST account for every application it considers in
  exactly one outcome, and those outcomes MUST reconcile to the number of
  applications considered, so a partial or interrupted walk cannot read as a
  complete one (P-4). The outcomes are: launch data written, skipped as current,
  and failed to parse.
- **FR-008**: A single application whose entry cannot be parsed MUST be counted
  as failed and MUST NOT abort the walk; the remaining applications MUST still be
  considered.
- **FR-009**: An application with no launch configuration, or with a launch
  configuration that yields no storable executable, MUST NOT be counted as
  failed. It is a considered application that produced nothing to write.
- **FR-010**: The system MUST surface progress during the walk so that a slow
  first run over a large library is visibly working rather than appearing hung.
- **FR-011**: The accumulation walk MUST run automatically when a capture run
  starts, without a separate operator command, whenever a local hint database is
  configured. It writes to that same database (the one the resolver reads); it
  introduces no second store. With no hint database configured there is no
  accumulation, matching today's behavior where hint resolution is likewise
  absent.
- **FR-011a**: Staleness MUST be decided per application by comparing the local
  cache's recorded change-number for the application against the change-number
  the store recorded when it last stored that application's launch data. The
  store therefore MUST record that change-number alongside the launch data.
- **FR-012**: The system MUST NOT open any network connection to accumulate
  launch data, and MUST NOT require or compile any networking capability for this
  feature.
- **FR-013**: The system MUST NOT open any process handle against any process to
  accumulate launch data. It reads only a file the platform already wrote
  (P-1).
- **FR-014**: The system MUST NOT add any new third-party dependency to the
  workspace to implement this feature.
- **FR-015**: The system MUST NOT populate the token-required attribute of a
  stored game from this source; that attribute is out of scope for this slice and
  is left as it was.
- **FR-015a**: The system MUST NOT populate the launcher-mediated flag from this
  source. It has no reliable field in the cache, and deriving it here would be a
  guess (P-9). The flag is left exactly as it was; this slice writes launch
  entries and the recorded change-number only.
- **FR-016**: A stored game that already carried launch data before this feature
  existed (for example from a hand-authored import) MUST be treated as any other
  game: refreshed when the local cache records a newer version, skipped when it
  does not.
- **FR-017**: Once launch data is stored, the existing Steam-application-id
  resolution path MUST be able to use it with no further change, so the benefit
  of accumulation appears at the next resolution automatically.

### Key Entities *(include if feature involves data)*

- **Local Steam application-info cache**: the file the user's Steam client
  maintains describing every application Steam has fetched metadata for, keyed by
  application id, each carrying a launch configuration and a version marker
  (change-number). Present only for applications the user's Steam knows about;
  its contents are private to the user's machine.
- **Launch configuration**: per application, an ordered set of launch entries.
  Each entry names an executable and may carry arguments, a launch type, and
  operating-system, architecture, and beta-branch filters.
- **Local hint store**: the user's private, writable database of game hints.
  This slice writes only its launch entries and the recorded per-application
  change-number; it does not touch the public catalog or engine columns, nor the
  launcher-mediated or token-required attributes. Recording the change-number
  requires one additive column, introduced by the store's first schema-version
  migration (v1 to v2).
- **Accumulation account**: the reconciled record of one walk: how many
  applications were considered and, for each, whether its launch data was
  written, skipped as current, or failed to parse.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After a first accumulation run against a library, every application
  whose local cache entry carries a parseable launch executable has that
  executable stored, and the run's account names every application considered.
- **SC-002**: A second accumulation run over an unchanged cache reads no
  application's launch configuration; every application is accounted for as
  skipped as current.
- **SC-003**: After one application's recorded version advances in the cache, the
  next run re-reads exactly that application and no other.
- **SC-004**: For every run, the counts of written, skipped, and failed
  applications sum exactly to the number of applications considered.
- **SC-005**: A malformed entry for one application does not prevent the launch
  data of the other applications in the same run from being stored.
- **SC-006**: Accumulation completes with no network connection opened at any
  point, verifiable by the feature compiling and passing its tests with no
  networking capability present.
- **SC-007**: The change adds no new entry to the workspace dependency lockfile.
- **SC-008**: The minimum supported toolchain build and the full repository check
  set both stay green.

## Assumptions

- The user's Steam client is the source of truth for what launch executable a
  game uses; fragcap records what Steam already decided rather than deciding
  itself. An application absent from the user's local cache simply yields no
  launch data, which is expected, not an error.
- The local store already provides the launch-entries table and the
  launcher-mediated column from an earlier slice, so the launch data itself needs
  no schema change. It has no per-application change-number column, so this slice
  adds one via the store's first schema-version migration (v1 to v2): an
  additive, store-internal migration that leaves existing rows' new column null.
  This revises the earlier expectation that no store migration would be needed;
  it is the cost of deciding staleness by change-number rather than by mere
  presence.
- Running the walk at capture start, rather than as a separate command, is the
  operator-chosen default. A future slice may add an explicit refresh command or
  make the walk opt-out; that is out of scope here.
- Deciding staleness by the application's recorded change-number, rather than a
  plain has-it or not check, is the operator-chosen default, so an application is
  re-read only when Steam actually updated it.
- Pooling accumulated launch data across users is explicitly out of scope and is
  tracked separately (issue #94). This slice keeps every learned fact on the
  machine that learned it.
- The launcher-mediated flag is not populated by this slice. The cache has no
  reliable field for it, so deriving it here would be a guess (P-9). The primary
  and only stored fact from this source is the launch entry set (plus the
  change-number used to decide staleness). Deriving launcher-mediated, if ever
  worthwhile, is a separate judgment for a later slice.
