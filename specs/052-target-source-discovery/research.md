# Phase 0 Research: TargetSource discovery seam and discovery tiers

All decisions below were resolvable from the spec, the constitution, the S050/S051
records, and the existing crate surfaces; no external unknowns remained after the
clarify phase. Each is stated as Decision / Rationale / Alternatives considered.

## D1. Where the seam and the sources live (crate placement, P-2/P-3)

**Decision**: The `TargetSource` trait, `CandidateTarget`, the discovery account,
the `DirectoryClassifier` seam, `KnownRootsSource`, `DirectorySource`,
`InteractiveSource`, and the volume eligibility store all live in
`fragcap-targets` behind the existing default-off `targets` feature. The two
platform-touching adapters, `SteamSource` (which wraps `fragcap-steam`'s
`library`/`appinfo` walk) and the real Win32 volume inventory, live in the
`fragcap` facade.

**Rationale**: `CandidateTarget` carries a catalog `Classification` and a
`FidelityTier`, both already reachable from `fragcap-targets`; the eligibility
table is a `fragcap-targets` `Store` migration. So the seam belongs with its data.
`SteamSource` needs both the trait (in `fragcap-targets`) and the Steam walk (in
`fragcap-steam`); the facade is the one crate that already depends on both leaf
crates (`xtask/src/deps.rs` lines for `fragcap -> fragcap-steam` and
`fragcap -> fragcap-targets`), exactly the composition-root pattern S051 used for
the end-to-end test. Placing `SteamSource` there adds no new inter-crate edge, and
keeps `fragcap-targets` free of the Windows API and of `fragcap-steam` so it goes
on building under the GNU host toolchain.

**Alternatives considered**:
- *`SteamSource` in `fragcap-targets`, adding a `fragcap-targets -> fragcap-steam`
  edge.* Rejected: it introduces a new allowlist edge (a decision plus a
  `deps.rs` change), couples the SQLite crate to the Windows Steam crate, and pulls
  `fragcap-steam` in whenever the `targets` feature is on. The facade already
  bridges them at no cost.
- *`SteamSource` in `fragcap-steam`, adding `fragcap-steam -> fragcap-targets`.*
  Rejected for the mirror reason: it drags the SQLite/`rusqlite` graph into the
  Steam crate.
- *A new `fragcap-discovery` crate depending on both.* Rejected as premature: the
  facade already is that crate, and a new crate adds an edge and a publish unit for
  two small adapters.

## D2. Where the real volume enumeration lives (P-2)

**Decision**: The pure `KnownRootsSource` takes an injected `VolumeInventory`
(a value/seam listing eligible fixed volumes) and an injected directory lister, so
it is filesystem- and platform-free. The real inventory is a `cfg(windows)`
adapter in the facade over `GetLogicalDrives` + `GetDriveTypeW` (fixed volumes)
plus a stable per-volume identity (volume GUID path via the
`FindFirstVolume`/`GetVolumePathNamesForVolumeName` family, or
`GetVolumeInformationW` serial as the simpler identity).

**Rationale**: FR-019 requires every source to be fixture-drivable with no
volume-inventory or live-platform dependency; injection is the only way to satisfy
that and still ship a real walker. `windows-sys` is already workspace-pinned at
0.36 and used by four crates, so the adapter adds no `Cargo.lock` package. Keeping
the adapter in the facade (not in `fragcap-targets`) preserves the targets crate's
portability.

**Alternatives considered**: putting the Win32 calls in `fragcap-targets` behind
`cfg(windows)`. Workable (P-2 only binds `fragcap-core`), but it would give the
portable targets crate its first platform dependency for no gain, since the facade
already hosts `SteamSource` and is the natural home for platform adapters.

## D3. Stable volume identity (safety, re-mount robustness)

**Decision**: The eligibility table keys each volume on a stable identity that
survives drive-letter reassignment (the volume GUID path, with the volume serial
as a fallback identity), and stores the current mount point (drive letter) as a
mutable display attribute plus the drive-type observed at record time.

**Rationale**: FR-016a must stop a *later-appearing or misreporting* volume from
being walked without opt-in; keying on the drive letter alone would let a
reassigned letter inherit a prior volume's eligibility, which is the reuse hazard
the allowlist exists to prevent. This mirrors the S010 reasoning that a reused
identifier must be distinguished from its predecessor.

**Alternatives considered**: keying on the drive letter (rejected: letters are
reassigned) or on the serial alone (accepted only as a fallback: a serial changes
on reformat, which is an acceptable "treat as new, require opt-in" outcome).

## D4. Eligibility semantics: permissive seed, then allowlist (clarified)

**Decision (from the clarify session)**: On first discovery against an empty
table, enumerate the fixed volumes then present and record each eligible. From
then on the table is an allowlist: a volume not recorded eligible is not walked,
and a newly appearing or misreporting fixed volume requires an explicit
user opt-in. Each per-volume decision is statable (recorded with its reason:
seeded-at-first-run, user-added, user-excluded, or unseen).

**Rationale**: reconciles the "allowlist, not denylist" intent (§7.4) with SC-001
(a fresh no-Steam machine must list games out of the box). A strict empty
allowlist would list nothing on a fresh machine; a pure denylist would silently
walk a misreporting FUSE/RustFS mount. Permissive-seed-then-allowlist is the only
option satisfying both.

**Alternatives considered**: default-eligible/exclude-on-demand (a denylist in
practice; rejected by the operator for the misreporting-mount hazard) and strict
empty allowlist (rejected: breaks out-of-box listing without a first-run prompt).

## D5. Candidate lifecycle: surface live, persist on first use (clarified)

**Decision (from the clarify session)**: A source produces candidates for a live
listing and writes nothing to `local.db`. A candidate becomes a durable
`TargetEntry` (via the S051 entry model) automatically when the user acts on it
(captures or selects it); bulk scan results for games the user never touches are
not persisted. The only durable write this slice makes is the eligibility table.

**Rationale**: keeps the listing always current (no staleness reconciliation),
keeps `local.db` meaning "targets the user vouched for" (the S050 boundary), and
keeps S052 scope minimal while remaining zero-friction (the listing shows games
without any setup step). Persistence-on-use is the natural P-10 trigger: acting on
a candidate is the "author" operation for every source.

**Alternatives considered**: auto-persisting every bulk find at scan time
(rejected by the operator: front-loads dedup/staleness work and writes low-fidelity
machine guesses into the user-owned store).

## D6. The descent stop-on-hit contract and the S053 seam

**Decision**: Tiers 2 and 3 consult a `DirectoryClassifier` seam per directory.
The descent tests a directory for a signature and, on a hit, emits one candidate
and stops descending into that subtree; on a miss it counts the directory as
considered-not-a-game and descends one level under a known root. S052 ships the
descent loop, the stop-on-hit contract, and a trivial/fixture classifier; S053
ships the real signature table implementing the same seam.

**Rationale**: FR-014/FR-015 make performance load-bearing: never enumerate a
directory's executables first (FR-009). Expressing classification as a seam lets
S053 drop in the real matcher with no change to the tiers, and lets S052's tests
drive a fixture classifier deterministically. It mirrors the existing
`CatalogSource`/`FixtureCatalog` seam pattern.

**Alternatives considered**: hard-coding a Unity/Unreal check inside the walker
now (rejected: it would duplicate and pre-empt S053's matcher and couple the walk
to a fixed signature set) and deferring the whole descent to S053 (rejected: the
known-roots walk that FR-007/FR-008 require needs the descent contract now).

## D7. The discovery account (P-4 conservation)

**Decision**: `discover()` returns candidates plus a `DiscoveryAccount` whose
named counts (produced, parse-failed, declined-by-user, considered-not-a-game,
volume-skipped, access-error) reconcile to the number of items considered, with an
`is_conserved()` invariant asserted in every source test, directly modeling
`SeedSummary`.

**Rationale**: P-4 forbids a discard path without a counter; the seeder already
established the conserved-summary shape and its `is_conserved` test. Reusing that
shape means a new non-produced outcome added later without a counter fails the
conservation test rather than passing silently.

**Alternatives considered**: returning only `Vec<CandidateTarget>` and logging
drops (rejected: a logged drop is a silent loss under P-4).

## D8. Steam refactor with zero observable change (FR-006)

**Decision**: `SteamSource` is a thin adapter that calls the existing
`fragcap-steam` `library`/`appinfo` walk unchanged and maps each installed title
to a `CandidateTarget` at heuristic-unverified fidelity, joining the appid to
`catalog.db`. A parity test asserts the candidate set equals what the prior walk
produced for the same fixture install; the existing `SteamWalkerProvider` continues
to exist until S054's capture rework consumes the source form.

**Rationale**: FR-006 forbids a regression in the Steam candidate set; a parity
test against committed metadata fixtures is the mechanical guard. Keeping
`SteamWalkerProvider` in place avoids touching the capture entry point, which is
S054's boundary (the same transitional-window discipline S051 used).

**Alternatives considered**: rewriting the Steam walk onto the seam in one step and
deleting `SteamWalkerProvider` now (rejected: it entangles the capture entry point
that S054 owns and risks the very regression FR-006 forbids).

## D9. No new dependency

**Decision**: introduce no new crate. Reuse `fragcap-steam`, `fragcap-targets`
(`rusqlite`, `catalog`, `Store`), and the already-pinned `windows-sys` 0.36.

**Rationale**: every capability the slice needs already exists in the graph; the
project's standing preference is to add nothing when arithmetic over existing
surfaces suffices (the S03/S06/S08 pattern). The `Cargo.lock` delta is zero.

**Alternatives considered**: a directory-walk crate such as `walkdir` (rejected:
the descent is a shallow, stop-on-hit, one-level walk under known roots, not a
general recursive traversal, and `std::fs::read_dir` expresses it directly) and a
volume-enumeration crate (rejected: `windows-sys` already carries the three calls
needed).
