# Phase 1 Data Model: TargetSource discovery seam and discovery tiers

Types below are the discovery model. Names in `code` are the intended Rust
identifiers; fields are described by role, not by exact type, where the type is an
implementation detail. All new types land in `fragcap-targets` unless marked
(facade).

## `TargetSource` (trait)

The seam. Every origin implements it.

| Member | Role |
| --- | --- |
| `name(&self) -> &str` | Stable, human-readable source name (e.g. `"steam"`, `"known-roots"`, `"directory"`). Used in the account and in listings. |
| `discover(&self) -> Result<Discovery, TargetsError>` | Produce candidates plus the truthful account. A hard failure (unreadable metadata store) is the `Err`; a per-item failure is counted in the account, never returned as `Err`. |
| `default_fidelity(&self) -> FidelityTier` | The fidelity this source stamps on its candidates unless a per-candidate override applies. |

Reuses `fragcap_profile::FidelityTier` (one fidelity vocabulary across the
project, as S051 established).

## `Discovery`

The return of `discover()`: the produced candidates and the account, together.

| Field | Role |
| --- | --- |
| `candidates: Vec<CandidateTarget>` | What the source produced this run. |
| `account: DiscoveryAccount` | The conserved tally (below). |

## `CandidateTarget`

What a source produces. Not yet a stored entry; becoming a durable `TargetEntry`
is the S051 entry model's job, triggered on first use (FR-021).

| Field | Role | Notes |
| --- | --- | --- |
| `identity` | What was found | One of: a filesystem path (tiers 2/3) or a platform identity such as a Steam appid (tier 1). Enough to later author an entry. |
| `display_name` | Best available human name | From Steam metadata, the directory name, or the executable stem. Never invented beyond these observations (P-9). |
| `fidelity: FidelityTier` | The stamped fidelity | Steam -> heuristic-unverified; interactive-accepted -> authored; directory -> its source default. |
| `classification` | Catalog join result | The catalog `Classification` when the identity joins a catalog row; `unknown` when it does not. Never dropped for being unknown. |
| `source_name` | Which source produced it | Mirrors `TargetSource::name`. |

## `DiscoveryAccount`

The P-4 conservation record, modeled on `SeedSummary::is_conserved`.

| Field | Role |
| --- | --- |
| `considered` | Total items the source examined. |
| `produced` | Items emitted as candidates. |
| `parse_failed` | Items whose metadata could not be parsed (counted, never dropped). |
| `declined_by_user` | Candidates a human rejected at the interactive step. |
| `considered_not_a_game` | Directories that matched no signature and no known-root rule. |
| `volume_skipped` | Items not examined because their volume was ineligible. |
| `access_error` | Items not examined because of a permission or I/O error, named per root/volume. |

Invariant `is_conserved()`: `produced + parse_failed + declined_by_user +
considered_not_a_game + volume_skipped + access_error == considered`. Asserted in
every source test.

## Sources (all implement `TargetSource`)

- **`SteamSource`** (facade): wraps `fragcap-steam` `library`/`appinfo`; one
  candidate per installed title at heuristic-unverified; appid joined to
  `catalog.db`. Parity with the pre-refactor walk is a test invariant (FR-006).
- **`KnownRootsSource`**: pure. Constructed with an injected `VolumeInventory` and
  a directory lister; walks the fixed known-root list (below) across every eligible
  volume, applying the descent stop-on-hit contract via a `DirectoryClassifier`.
- **`DirectorySource`**: one path in, at most one candidate out.
- **`InteractiveSource`**: wraps `DirectorySource`; an injected confirmation
  decides accept (stamp authored) or reject (count `declined_by_user`).
- **`FixtureSource`** (test-only): a canned candidate list + account, proving the
  driver needs no change to gain a new source (SC-006).

### Known-root list (v0.5.0, FR-007, fixed)

`SteamLibrary\steamapps\common`, `Program Files (x86)\Steam\steamapps\common`,
`Program Files\Epic Games`, GOG Galaxy `Games`, `Riot Games`, `Battle.net`,
Ubisoft launcher `games`, `EA Games`, `Origin Games`, `XboxGames`, `Games`.
Each is a path relative to a volume root; the walk applies all of them to every
eligible volume.

## `DirectoryClassifier` (trait, seam for S053)

| Member | Role |
| --- | --- |
| `classify(&self, dir) -> ClassifierVerdict` | Decide, from directory shape, whether this directory is a game (a hit), stamping a confidence/kind. |

`ClassifierVerdict`: `Hit { .. }` (emit one candidate, stop descending) or `Miss`
(count considered-not-a-game, descend one level under a known root). S052 ships a
trivial/fixture classifier; S053 ships the real signature matcher implementing this
trait.

## `Volume` and `VolumeInventory`

- **`Volume`**: stable `identity` (volume GUID path; serial as fallback), current
  `mount_point` (drive letter, mutable/display), and `drive_type` observed.
- **`VolumeInventory`** (trait): `fixed_volumes(&self) -> Vec<Volume>`. Real impl
  is a `cfg(windows)` facade adapter over `GetLogicalDrives`/`GetDriveTypeW`/volume
  GUID enumeration; a fixture impl returns a canned list for tests (FR-019).

## `volume_eligibility` (SQLite table, `local.db`)

Added by an additive migration bumping the shared schema from 3 (S051) to 4.

| Column | Role |
| --- | --- |
| `volume_id` | PRIMARY KEY. The stable volume identity (D3). |
| `mount_point` | Last-seen drive letter / mount path (display; mutable). |
| `drive_type` | Drive type observed when recorded. |
| `eligible` | Boolean: is this volume walked. |
| `reason` | Why: `seeded-first-run`, `user-added`, `user-excluded`. |
| `first_seen` | When the volume was first recorded. |

**Seeding (FR-016a)**: first discovery against an empty table inserts every
present fixed volume as `eligible = true, reason = seeded-first-run`. Thereafter a
volume absent from the table is treated as ineligible (unseen) and is not walked
until an explicit user opt-in inserts it `eligible = true, reason = user-added`.

**Store operations** (new on `Store`): `seed_volume_eligibility(&[Volume])`
(idempotent, first-run only), `eligible_volumes()`, `set_volume_eligibility(volume_id,
eligible, reason)`, `volume_eligibility(volume_id)`. Catalog store leaves the table
empty (local-only data), per the S050/S051 pattern.

## State & lifecycle

- A `CandidateTarget` is ephemeral: produced -> listed -> (on user action)
  authored into a `TargetEntry`. S052 owns produced and listed; the author step is
  the S051 model.
- A `volume_eligibility` row transitions `absent -> seeded-first-run` (once) and
  `absent|any -> user-added|user-excluded` (on user edit). No automatic transition
  re-includes a volume; that requires a user action (D4).
