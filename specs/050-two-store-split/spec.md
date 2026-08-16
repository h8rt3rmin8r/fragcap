# Feature Specification: The two-store split (catalog.db + local.db)

**Feature Branch**: `050-two-store-split`

**Created**: 2026-08-16

**Status**: Draft

**Slice**: S050 (GitHub issue #137, milestone v0.5.0). Depends on S049 (merged).

**Input**: fragcap v0.5.0 UX Handoff Plan sections 3.1 and 4.

## Clarifications

### Session 2026-08-16 (autopilot)

- Q: Does S050 redirect learned launch accumulation to `local.db` and read both
  stores now, or defer that to S051? -> A: Redirect now. Learned accumulation
  writes to `local.db` in this slice, and resolution consults both `catalog.db`
  (shipped) and `local.db` (learned), so there is no attribution regression and
  the "catalog refresh leaves `local.db` byte-identical" criterion is meaningful.
  The fidelity-ordered cascade collapse and the target-entry model remain S051.
- Q: One shared schema for both files, or divergent schemas per store? -> A: One
  shared store type and schema for both files in S050. The current schema already
  carries the tiers `catalog.db` needs and the learned-data columns `local.db`
  needs; reusing one store type on two files is the minimal split. Later slices
  add their own tables to `local.db`.
- Q: What becomes of the `--hint-db` command-line store-path override? -> A:
  Replace it with two optional overrides, `--catalog-db <path>` and
  `--local-db <path>`, each defaulting to its file in the AppData root. No alias
  for `--hint-db` is kept (v0.5.0 ships no deprecation shims).

## User Scenarios & Testing *(mandatory)*

The actors are the user, who installs fragcap, captures, and accumulates learned
data on their own machine and will later refresh the shipped data; ShruggieTech,
who ships a seed catalog and replaces it wholesale between releases; and the tool
itself, which bootstraps its stores on first run.

### User Story 1 - A refresh of shipped data cannot touch the user's data (Priority: P1)

Today a single `hint.db` in the user's AppData holds both the ShruggieTech seed
data and the launch data the user's own machine has learned by walking its local
Steam library. Replacing the seed is therefore either destructive to the learned
data or a three-way merge. After this slice, the shipped data lives in
`catalog.db` and the user's data lives in `local.db`, so a future
`fragcap catalog update` replaces `catalog.db` wholesale and never reads or
writes `local.db`.

**Why this priority**: This is the reason the split exists. Every later v0.5.0
slice (target entries, overrides, detection results, IGDB enrichment, volume
exclusions) writes user data that a catalog refresh must never endanger, and the
clean trust boundary is what makes those slices safe to build.

**Independent Test**: Populate `local.db` (capture a title so learned launch data
accumulates), record its bytes, replace `catalog.db` with a different file, and
confirm `local.db` is byte-identical afterward.

**Acceptance Scenarios**:

1. **Given** a `local.db` holding learned data, **When** `catalog.db` is replaced
   wholesale, **Then** `local.db` is byte-identical before and after.
2. **Given** a capture that accumulates learned launch data, **When** it runs,
   **Then** the learned data lands in `local.db` and `catalog.db` is byte-identical
   before and after.
3. **Given** the two stores, **When** a reader inspects the layout, **Then**
   `catalog.db` and `local.db` are separate files sharing no storage, so neither
   operation on one can alter the other.

---

### User Story 2 - A fresh install yields both stores without elevation (Priority: P1)

A user installs fragcap and runs it for the first time. The installer places a
seed `catalog.db` beside the binary in Program Files; first run copies it into the
per-user AppData root (writable, no elevation) if none is present, and creates an
empty `local.db` there. From then on both stores live in AppData, so a future
`catalog update` writing `catalog.db` never needs administrator rights.

**Why this priority**: The location choice is load-bearing. If `catalog.db` lived
beside the binary in Program Files, refreshing it would require elevation, which
is exactly what the v0.5.0 UX work is trying to remove. Both stores must be
per-user and writable without elevation.

**Independent Test**: On a machine with no prior fragcap data, install and run
`fragcap` once as an ordinary user; confirm both `catalog.db` and `local.db`
appear in the AppData root and no elevation was required.

**Acceptance Scenarios**:

1. **Given** a fresh install with a seed beside the binary, **When** fragcap runs
   for the first time, **Then** `catalog.db` and `local.db` both exist in the
   AppData root.
2. **Given** an AppData root missing `catalog.db` but with the seed present beside
   the binary, **When** fragcap runs, **Then** it copies the seed into AppData as a
   writable `catalog.db`.
3. **Given** an AppData root missing `local.db`, **When** fragcap runs, **Then** it
   creates an empty `local.db`, leaving any existing `catalog.db` untouched.
4. **Given** no seed beside the binary (a portable or developer build), **When**
   fragcap runs, **Then** it still starts, creating an empty `catalog.db` rather
   than failing.

---

### User Story 3 - Learned launch data accumulates into the user's store (Priority: P2)

As the user captures titles, fragcap learns launch executables by reading the
local Steam appinfo cache and accumulates them. After the split, that learned
data is written to `local.db`, and resolution still uses both the shipped catalog
data and the user's learned data, so which client a game resolves to does not
change because of the split.

**Why this priority**: The split must not silently regress attribution. It is P2
rather than P1 because it is the behavior-preservation guarantee around the
storage change in US1 and US2, not the storage change itself.

**Independent Test**: Capture a title whose client is known only from local
appinfo learning; confirm it resolves to the same client after the split as
before, and that the learned row is in `local.db`, not `catalog.db`.

**Acceptance Scenarios**:

1. **Given** a title resolvable only from learned launch data, **When** capture
   runs after the split, **Then** it resolves to the same client as before the
   split.
2. **Given** a capture that learns a new launch executable, **When** it completes,
   **Then** the new row is present in `local.db` and absent from `catalog.db`.

---

### Edge Cases

- **An old `hint.db` is present.** A user upgrading from a version that wrote
  `hint.db` has one in AppData. There is no migration: first run creates
  `catalog.db` and `local.db` fresh, and the old `hint.db` is left untouched and
  unused. (The entire user base is two people who can delete a folder.)
- **`catalog.db` deleted but `local.db` present.** First run re-seeds `catalog.db`
  from the template (or creates it empty if no template), leaving `local.db`
  untouched. This is also the steady state during a `catalog update`.
- **`local.db` deleted but `catalog.db` present.** First run recreates an empty
  `local.db`, leaving `catalog.db` untouched.
- **The seed template is read-only** (installed into Program Files). The copy into
  AppData must be writable, or the tool cannot open its own catalog for writing.
- **Both stores absent** (first run, no seed): both are created empty; the tool
  runs with no hints rather than failing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: fragcap MUST keep its stores as two separate files in the per-user
  AppData root: `catalog.db` (ShruggieTech-owned, disposable) and `local.db`
  (user-owned, durable). They MUST share no backing storage.
- **FR-002**: On first run, if `catalog.db` is absent, fragcap MUST create it by
  copying a seed installed beside the binary into a writable file in AppData; if
  no seed is present, it MUST create an empty `catalog.db`. This MUST require no
  elevation.
- **FR-003**: On first run, if `local.db` is absent, fragcap MUST create it empty
  in AppData, requiring no elevation, and MUST leave `catalog.db` untouched.
- **FR-004**: `catalog.db` MUST hold the ShruggieTech-shipped tiers: the public
  catalog tier (Steam appid, name, popularity metrics), the launch metadata tier,
  and the engine attribution tier. It MUST be the designated home for the
  detection signature table, whose contents a later slice seeds.
- **FR-005**: `local.db` MUST hold the user-owned data that exists today: the
  launch data learned from the local Steam appinfo cache, and user preferences
  and consent flags. It MUST be the designated home for the later-slice tables
  (target entries, per-target overrides, locally observed detection results, IGDB
  enrichment, the volume exclusion table, and the last-listing snapshot), each of
  which its owning slice creates.
- **FR-006**: Learned launch-data accumulation MUST write to `local.db` and MUST
  NOT write to `catalog.db`.
- **FR-007**: Replacing `catalog.db` wholesale MUST leave `local.db` byte-identical.
  No operation on either store may read or write the other.
- **FR-008**: After the split, resolution MUST continue to use both the shipped
  catalog data and the user's learned data, so no title resolves to a different
  client than it did before the split.
- **FR-009**: The Windows installer (MSI/WiX) MUST install the seed store beside
  the binary under the name `catalog.db`, not `hint.db`.
- **FR-010**: fragcap MUST NOT migrate data from an existing `hint.db`. A fresh
  install creates both stores fresh; an old `hint.db` is ignored.
- **FR-011**: Creating or opening either store MUST require no elevation beyond
  the installer itself.
- **FR-012**: The default path logic MUST yield `catalog.db` and `local.db` in the
  AppData root. The command line MUST expose two optional overrides,
  `--catalog-db <path>` and `--local-db <path>`, each defaulting to its AppData
  file; the former `--hint-db` flag MUST be removed with no alias.

### Key Entities

- **`catalog.db`**: the ShruggieTech-shipped store. Read for resolution, replaced
  wholesale by a future `catalog update`, never written by user actions. Every
  row is `heuristic-unverified` by construction, as today.
- **`local.db`**: the user-owned store. Accumulates learned and user-authored data,
  never replaced by an update.
- **Seed template**: the `catalog.db` copy the installer places beside the binary;
  the source the first-run bootstrap copies into AppData.
- **First-run bootstrap**: the step that ensures both stores exist in AppData,
  copying the seed for `catalog.db` and creating `local.db` empty.
- **Learned launch data**: the launch executables fragcap accumulates from the
  local Steam appinfo cache; user-owned, so it lives in `local.db`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After a fresh install and one run as an ordinary user, both
  `catalog.db` and `local.db` exist in the AppData root, with no elevation.
- **SC-002**: Replacing `catalog.db` wholesale leaves `local.db` byte-identical in
  100% of cases (verified by hashing before and after).
- **SC-003**: A capture that accumulates learned launch data leaves `catalog.db`
  byte-identical and changes only `local.db`.
- **SC-004**: A title that resolved to a given client before the split resolves to
  the same client after it (no attribution regression).
- **SC-005**: An old `hint.db` present in AppData is neither read nor written; the
  tool operates entirely from `catalog.db` and `local.db`.
- **SC-006**: The store's existing tests pass against the split.

## Assumptions

- **S050 redirects learned accumulation to `local.db` now, and resolution reads
  both stores.** (Resolved in the 2026-08-16 clarification.) The acceptance
  criterion that a catalog refresh leaves `local.db` byte-identical is only
  meaningful if `local.db` holds the learned data, so accumulation is redirected
  to `local.db` and resolution consults both stores. The fuller cascade collapse
  and the target-entry model are S051.
- **Both files use the existing single store type and schema in S050.** (Resolved
  in the 2026-08-16 clarification.) The current schema already contains the tiers
  `catalog.db` needs and the learned-data columns `local.db` needs; reusing one
  store type on two files is the minimal split. Later slices add their own tables
  to `local.db`.
- **The command line exposes `--catalog-db` and `--local-db`** overrides, each
  defaulting to its AppData file; `--hint-db` is removed with no alias. (Resolved
  in the 2026-08-16 clarification.)
- **The `fragcap-targets` crate keeps its name** (internal; nobody types it).
- **No new runtime dependency.** The embedded SQLite engine (behind the `targets`
  feature) already backs the single store and backs both.
- **No capture, attribution, or output-format behavior changes** beyond where the
  learned data is stored and read; the packet path is untouched.

## Constitution alignment

- **P-11 and the S049 gate**: this slice modifies specification section 24.5
  (release packaging references `hint.db`) and any section describing the store
  layout, so its changelog fragments carry the appropriate `spec-impact`, and the
  specification is edited in the same change (the S049 release gate).
- **Pinned artifacts**: the WiX/MSI source and any release-workflow or packaging
  step that names `hint.db` are pinned; each change lands with a dated
  `decisions` fragment under `changelog.d/`.
- **P-9 (The Instrument Does Not Lie)**: the split changes where data is stored,
  never what is observed or reported; no attribution is altered or dropped.
- **P-2**: no platform-specific dependency enters `fragcap-core`; the store work is
  confined to `fragcap-targets` and the CLI.
- **Text hygiene (P-8)**: all edited files stay UTF-8 without BOM, LF, no
  em-dashes or en-dashes.
