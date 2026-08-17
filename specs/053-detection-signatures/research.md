# Research: Data-driven detection signatures

Phase 0 decision record. Each decision names what was chosen, why, and the
alternatives rejected. The two spec-level clarifications (match kinds, embedded
ruleset fate) were resolved with the operator and are recorded in the spec's
Clarifications section; this file resolves the implementation-level unknowns the
plan surfaced.

## D1. Crate placement of the matcher (the load-bearing decision)

**Decision**: The pure generic matcher and the `Signature` value type live in
`fragcap-profile`. The signature table, the `load_signatures` store method, the
Appendix B seed, and the `DirectoryClassifier` implementation live in
`fragcap-targets`. `fragcap-steam`'s scaffold receives an injected signature set
(a `&[Signature]`) from its facade caller. The facade loads signatures from
`catalog.db` (via `fragcap-targets`) and hands them to both the `technologies`
command and the scaffold.

**Rationale**: The dependency gate (`xtask/src/deps.rs`) fails closed on any edge
not in its expected set. The expected set has `fragcap-steam -> fragcap-profile`
and `fragcap-targets -> fragcap-profile`, and no `fragcap-steam -> fragcap-targets`
edge. Detection has two consumers today: the `technologies` CLI command and
`fragcap-steam`'s profile scaffold (`scaffold.rs` calls
`CompiledRuleset::embedded()`). The signature data lives in `catalog.db`, owned by
`fragcap-targets`. If the matcher moved wholesale to `fragcap-targets`, the scaffold
would need a new `fragcap-steam -> fragcap-targets` edge, which the gate rejects and
which S052 deliberately avoided by placing its platform adapters in the facade.
Splitting the matcher (pure, over a provided signature set, in the shared
`fragcap-profile`) from the store and seed (in `fragcap-targets`) lets both
consumers reach detection through the crate they already depend on, adds zero
edges, and keeps `fragcap-profile` free of any database dependency.

**Alternatives rejected**:
- *Matcher in `fragcap-targets`, new `fragcap-steam -> fragcap-targets` edge*:
  rejected. It fails the deps gate and reverses the S052 decision to keep the two
  sibling crates unlinked; a scaffold that reads the catalog database directly also
  couples profile authoring to the catalog store.
- *Duplicate the matcher in both crates*: rejected. Two matchers drift; a signature
  honored in one path and not the other is exactly the silent inconsistency the
  data-driven table exists to remove.
- *Move the scaffold's technology enrichment into the facade*: rejected as larger
  than needed. Injecting a `&[Signature]` into the existing scaffold entry point is
  a one-parameter change; relocating scaffold logic across crates is not.

## D2. There is no `fragcap catalog update` command; the seed rides the catalog tier

**Decision**: The signature table is seeded through the existing catalog-seed
machinery, not a new top-level `catalog` command. A `seed_signatures(store,
source)` function loads a bundled Appendix B document
(`fragcap-targets/assets/signatures.json`) into the table, mirroring
`seed_catalog`/`seed_engine`. The CLI surface is a `targets seed-signatures`
subcommand (offline from the bundled asset), consistent with the existing
`targets seed` (Tier 1) and `targets seed-engine` (Tier 3) subcommands.

**Rationale**: The issue names `fragcap catalog update` conceptually, but no such
command exists; catalog refresh today is the `targets seed*` family seeding
`catalog.db`. The signature set is one more tier of that same catalog, so it seeds
the same way and refreshes on the same operation. FR-005 ("refreshed as part of the
same update that refreshes the catalog") is satisfied by making signatures a seeded
catalog tier; the spec's `catalog update` wording is the conceptual name for that
refresh.

**Alternatives rejected**:
- *A new `catalog` top-level command*: rejected for this slice. It is a CLI-surface
  redesign that belongs with the S054 command rework, not a detection slice.
  Reusing the `targets seed*` family ships the capability without pre-empting that
  design.
- *Seed lazily from the live network*: rejected. The bundled Appendix B document is
  offline and deterministic, matching `targets seed`'s default; a live signature
  feed is a later `net`-gated addition behind the same `CatalogSource`-style seam.

## D3. PE-version-string matching is a hand-rolled PE version-resource read

**Decision**: PE-version-string matching parses the target binary's PE structure
(DOS header, PE header, section table, `.rsrc` section, `VS_VERSIONINFO` /
`StringFileInfo`) far enough to read the version-resource string fields
(`CompanyName`, `ProductName`, `FileDescription`, and similar) and matches the
signature pattern against them. It is hand-rolled over the file's bytes, adds no
crate, and is confined to `fragcap-profile`. Appendix B seeds no PE-version-string
rows, so a synthetic test signature plus a minimal PE image built by a test helper
proves the path and the "honored with no code change" guarantee for the kind.

**Rationale**: The operator chose to implement this kind. Reading a PE version
resource is byte parsing over an on-disk file, the same shape as the hand-rolled
pcap and pcapng parsers, and needs no `goblin`/`object`/`pelite` dependency for a
bounded read. It calls no Windows API, so it stays portable and testable off
Windows. Because no seeded product needs it yet, its value this slice is making the
kind real, testable, and shippable-as-data later; the cost is one bounded parser
and one generated fixture.

**Alternatives rejected**:
- *Take a PE-parsing crate (`goblin`, `object`, `pelite`)*: rejected. A large graph
  for a bounded read of one resource, against the project's standing preference to
  hand-roll a format it only needs a slice of (the same argument that kept the glob
  matcher and the pcapng writer hand-rolled).
- *Call the Windows version-resource API (`GetFileVersionInfo`)*: rejected. It would
  put a Windows dependency in `fragcap-profile` and make the matcher untestable off
  Windows, for a parse the crate can do itself.
- *Defer the kind entirely*: rejected; the operator chose to include it, and the
  schema-plus-inert approach used for binary-marker would leave the matcher with no
  content-reading path at all, making a later addition larger.

## D4. Fidelity is derived from the signature, local outranks remote

**Decision**: Each signature carries a confidence that maps to the fidelity a match
stamps. A definitive on-disk marker (for example `UnityPlayer.dll` for Unity)
stamps `verified`; a weaker or shape-only signal can stamp lower. A locally
detected engine at `verified` outranks a remote catalog engine attribution, which
stays `heuristic-unverified`; where both exist for one candidate, the local value
is presented (FR-008, FR-009).

**Rationale**: The issue is explicit that local detection is `verified` and remote
catalog data is `heuristic-unverified`, inverting the old detector's blanket
`heuristic-unverified` stamp. Driving fidelity from a per-signature confidence
keeps the strength of evidence a property of the data (a strong marker vs a generic
one), consistent with making detection data-driven, rather than a code constant.

**Alternatives rejected**:
- *Stamp every local match `verified`*: rejected. A generic shape like a lone
  `*.pak` is weaker evidence than a named engine library; a per-signature
  confidence lets the seed say so.
- *Keep the old blanket `heuristic-unverified`*: rejected. It contradicts the P-9
  "local evidence outranks remote" rule this slice implements.

## D5. Schema version 5; the signature table is a catalog-side table

**Decision**: The shared schema advances from version 4 (S052) to version 5 with an
additive `MIGRATE_4_TO_5` creating the `signature` table. The table is conceptually
a `catalog.db` table (the shipped, refreshable catalog), the opposite side of the
S052 `volume_eligibility` table (which is `local.db`). The shared store type carries
the table on both files; `local.db` leaves it empty.

**Rationale**: Signatures are shipped, refreshable catalog data, so they belong with
the catalog the same way the `games`, `launch_entries`, `technologies`, and
`seed_state` tables do. The additive sequential migration follows the exact
discipline S051 (v2->v3) and S052 (v3->v4) used. The two-store split (S050) means the
shared DDL carries the table on both files and each side populates what it owns.

**Alternatives rejected**:
- *Put signatures in `local.db`*: rejected. `local.db` is the user-owned store;
  signatures are shipped catalog data refreshed by a seed, not user state.
- *A non-sequential or destructive migration*: rejected; it breaks the established
  migration chain and the drift discipline the store tests enforce.

## D6. The signature table category is its own enum, distinct from the technologies table

**Decision**: The `signature.category` domain is `engine`, `anti-cheat`, `drm`. This
is distinct from the existing per-appid `technologies` table, whose category CHECK is
`engine`/`anti_cheat`/`sdk`/`framework`/`emulator`/`container`/`runtime`/`launcher`
and which holds remote catalog claims about a specific title. The two tables are
separate concerns: `technologies` is what the remote catalog says about a title
(heuristic-unverified), `signature` is the local-detection pattern set (verified on
match).

**Rationale**: The issue specifies `signature(id, category, kind, pattern, product,
confidence)` with category engine/anti-cheat/drm, and DRM is not a category the
`technologies` table carries. Keeping the enums separate avoids overloading one
table's vocabulary and keeps the local-vs-remote fidelity distinction clean.

**Alternatives rejected**:
- *Reuse the `technologies` category enum*: rejected. It lacks `drm` and conflates
  remote catalog claims with local-detection patterns.

## D7. Removing the vendored SteamDB ruleset cleanly

**Decision**: Remove `fragcap-profile/assets/steamdb/` (`rules.ini`,
`rules.lock.json`, `THIRD_PARTY_NOTICES.md`), the `CompiledRuleset`/`RULES_INI`/
`RULES_LOCK`/`SkippedPattern` machinery in `technologies.rs`, and their re-exports
from `fragcap-profile` and the facade. Repoint the two consumers (`technologies`
CLI, `fragcap-steam` scaffold) at the table-backed matcher. Remove the `sha256`
module if the ruleset lock test was its only user; verify before removing. Update
the dependency-inventory and third-party-notice accounting and the AGENTS.md
current-state narrative.

**Rationale**: The embedded ruleset is depot-authored, path-only, all
`heuristic-unverified`, never validated against a real install, and is the code the
"data not code" goal exists to retire. Leaving it as a fallback keeps a misleading
detector and a conflicting fidelity stamp (spec Clarifications). Its provenance and
license accounting must be removed with it so the license gate stays honest.

**Alternatives rejected**:
- *Keep the asset but stop using it*: rejected. Dead vendored third-party data still
  carries a notice obligation and reads as shipped capability; P-4 and P-11 want the
  removal recorded, not the code left inert.

## D8. Classifier walk vs technologies walk share the match primitive

**Decision**: The matcher exposes a per-directory primitive (given a directory's
immediate listing, and optionally a candidate binary for PE-version-string, return
the products matched). The `technologies` command composes it with a full bounded
tree walk that collects every finding grouped by category. The `SignatureClassifier`
composes it with the S052 shallow, stop-on-hit descent: a directory whose shape
matches an engine signature is a `Hit` (emit one candidate, stamp the engine, stop
descending); anything else is a `Miss` (considered-not-a-game). The two differ only
in walk discipline, not in the matching.

**Rationale**: US1 (classify a directory as a game) and US3 (inventory all
technologies) need the same signature matching but different traversal. Sharing one
primitive keeps a single definition of "does this signature match here" while letting
each caller own its walk, and it satisfies the S052 descent contract (FR-007) without
the classifier enumerating executables first.

**Alternatives rejected**:
- *Two independent matchers*: rejected for the drift reason in D1.
- *One walk serving both*: rejected. The classifier must stop on hit and stay
  shallow; the inventory must descend and collect all. Forcing one walk would either
  slow the classifier or truncate the inventory.

## D9. The neutral-evidence rule is verified across every surface

**Decision**: The `ClassifierVerdict`, the `DetectionFinding`, and every output that
can show a detection carry no status, risk, color token, or gating value. A test
asserts that no detection output path emits wording or a field that frames a
detected anti-cheat or DRM product as a reason not to capture, and that a title with
no online multiplayer mode is still presented as capturable (FR-011, FR-012).

**Rationale**: Section 3.6 makes this load-bearing and absolute. It is enforced by a
test rather than left to prose so a later surface that adds a status column fails
rather than ships.

**Alternatives rejected**:
- *Rely on reviewer vigilance*: rejected. P-4/P-9 discipline in this repo prefers a
  mechanical assertion over a remembered rule.
