# Feature Specification: Targets Hint Database Tier 1 Seeder (public catalog)

**Feature Branch**: `035-catalog-seeder`

**Created**: 2026-08-13

**Status**: Draft

**Slice**: S035 (issue #78, second of several; follows S034 foundation)

**Input**: The first seeder for the targets hint database: fill the Tier 1
(public catalog) columns (appid + name + catalog metrics) from the Steam public
Web API's app-list universe, scoped by a review-count / type gate, resumably, with
the fetch abstracted so the whole pipeline is tested offline.

## Overview

The S034 foundation built the targets hint database's store and its three-tier
column model but shipped no seeder: the store is only populated offline from a
hand-authored fixture. This slice writes the first real seeder, for Tier 1, the
public catalog.

Tier 1's job is to establish the corpus: which titles the database knows about at
all, by application id and name. The universe comes from the Steam public Web
API's app list, which is large and mostly noise (tools, videos, dead entries,
unreleased apps). Seeding all of it would bury the useful hints, so the seeder
scopes the corpus by a gate: a title is kept only if it is a game and clears a
configurable popularity threshold (a review count, working figure a few hundred).
Everything the gate excludes, and everything that fails to fetch or parse, is
counted and reported, because a corpus that silently drops what it could not
handle reads as complete when it is not (P-4, P-9).

Two structural pieces make this honest and testable. First, the catalog source is
an abstraction: the seeder's fetch-parse-map-gate logic is driven in tests by an
offline source backed by committed fixtures, and in production by a source that
performs read-only HTTP GETs against the public Web API. The network source is
optional at build time, so a default build and the whole offline test suite pull
neither an HTTP client nor a TLS stack; the live fetch is exercised by the
operator, not by continuous integration, exactly as live packet capture is. This
is also the slice where the project's HTTP-client dependency is chosen and
justified once, for every later seeder.

Second, the store gains a per-tier merge. The foundation's write path replaces a
whole game, which would erase any launch or engine data a later tier had already
written. Tier 1 must be able to update only its own columns for an application id
and leave the other tiers intact, so a catalog refresh over an enriched store does
not undo Tier 2 or Tier 3.

Tiers 2 (launch metadata) and 3 (engine) are later slices, as is wiring the
database into the live resolver. This slice ends at a populated, corpus-gated,
resumable, schema-valid Tier 1.

## Clarifications

### Session 2026-08-13

- Q: Does this slice make live network calls in continuous integration? -> A: No.
  The catalog source is a trait; every test drives an offline fixture-backed
  implementation. The real HTTP implementation is compiled under its feature but
  never executed in CI, the same posture as live packet capture. `cargo xtask ci`
  and `cargo xtask msrv` stay green with no network.
- Q: Is the HTTP client always compiled? -> A: No. The network source and its
  HTTP-client dependency sit behind an off-by-default feature, so a default build
  and the offline tests compile neither the client nor a TLS stack. The
  dependency is justified to the project's rubric and must keep MSRV 1.82 green.
- Q: How does Tier 1 avoid clobbering Tiers 2 and 3? -> A: The store gains a
  per-tier merge that upserts only the Tier 1 columns (name and catalog metrics)
  for an application id, leaving any existing launch entries, engine attribution,
  and technologies untouched. This is distinct from the foundation's whole-game
  replace.
- Q: What defines the corpus? -> A: A configurable gate: a title is kept only if
  it is classified a game and meets a review-count threshold (default a few
  hundred). Excluded and failed titles are counted and surfaced in a seed
  summary; they are never silently omitted.
- Q: What exactly does "the catalog source provides" for the gate? -> A: The
  precise Web API surface (the app-list endpoint plus how a title's type and
  review count are obtained) is a plan/research decision. The spec fixes only that
  the source yields, per title, an application id, a name, and enough signal to
  apply the gate, behind the trait.
- Q: An in-corpus title whose source name is empty or missing: written or
  excluded? -> A: Written, with the name omitted (stored as absent, never as an
  empty string, per S034). Corpus membership is by application id, and the gate
  (game classification plus the review threshold) has already filtered junk, so a
  known appid is kept even if its name is unusable. The seed summary counts it as
  written.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Seed the corpus into the store, offline-tested (Priority: P1)

A maintainer runs the catalog seeder against a catalog source; the store gains one
Tier 1 row per in-corpus title (application id and name, plus catalog metrics when
the source supplies them), and the run reports how many titles it fetched, wrote,
excluded by the gate, and failed. The store then exports schema-valid JSON.

**Why this priority**: This is the seeder's entire purpose and the MVP: turn a
catalog source into a corpus of Tier 1 rows, honestly accounted. Every later
scenario builds on it, and the whole pipeline is testable offline against
fixtures.

**Independent Test**: Drive the seeder with an offline fixture-backed source
carrying a mix of in-corpus games, out-of-corpus entries (non-games, below
threshold), and a malformed entry. Assert the store holds exactly the in-corpus
titles, the summary counts every category, and the export validates.

**Acceptance Scenarios**:

1. **Given** a catalog source yielding several in-corpus games and several
   out-of-corpus entries, **When** the seeder runs, **Then** the store holds one
   Tier 1 row per in-corpus game and none for the excluded entries, and the
   summary reports the written and excluded counts.
2. **Given** a source that yields a malformed or unfetchable entry among good
   ones, **When** the seeder runs, **Then** the good titles are written, the bad
   entry is counted as failed in the summary, and the run does not abort on it.
3. **Given** a freshly seeded store, **When** it is exported, **Then** the output
   validates against the target schema and each record carries the application id,
   the name, fidelity heuristic-unverified, and no launch or engine data.

### User Story 2 - Resume a large seed without restarting (Priority: P2)

A maintainer seeding a large catalog is interrupted (or seeds in stages); on the
next run the seeder continues from where it left off rather than re-fetching and
re-writing the whole universe.

**Why this priority**: The real catalog is large; a seed that cannot resume is a
seed that must complete in one run or waste the work. Resumability is a stated
requirement of the seeding model (#83) and depends on US1's write path.

**Independent Test**: Drive a partial seed that records progress, then a second
run against the same store and source; assert the second run continues from the
recorded cursor and the final corpus equals a single uninterrupted seed.

**Acceptance Scenarios**:

1. **Given** a seed that processed part of the catalog and recorded its progress,
   **When** the seeder runs again against the same store, **Then** it resumes from
   the recorded position rather than restarting.
2. **Given** a completed seed, **When** the seeder runs again, **Then** it
   recognizes completion (or refreshes) without duplicating rows, and the seed
   state records the last run.

### User Story 3 - Refresh Tier 1 without disturbing other tiers (Priority: P2)

A maintainer whose store already carries Tier 2 launch data and Tier 3 engine data
for some titles re-runs the Tier 1 seeder; the catalog columns update but the
launch entries and engine attributions survive untouched.

**Why this priority**: The three-tier model's whole promise is independent tiers.
A Tier 1 refresh that erased later tiers would force a full re-seed of everything,
defeating the design. This exercises the per-tier merge.

**Independent Test**: Populate a title with an engine (Tier 3) and launch entries
(Tier 2), run the Tier 1 seeder over a catalog that includes that application id
with an updated name, and assert the name updated while the engine and launch
entries are unchanged.

**Acceptance Scenarios**:

1. **Given** a stored game with an engine attribution and launch entries, **When**
   the Tier 1 seeder writes that application id with a new name, **Then** the name
   updates and the engine and launch entries are unchanged.
2. **Given** a stored game absent from the current catalog run, **When** the seeder
   runs, **Then** that game's rows are left as they are (the seeder adds and
   updates; it does not prune).

### Edge Cases

- A catalog source that returns an empty universe: the seed completes writing
  nothing, and the summary reports zero fetched, zero written.
- A title present twice in the source within one run: it is written once (the
  merge is idempotent per application id), and the repeat is counted as a duplicate
  rather than inflating the written count.
- An entry with a present but wrong-typed field (for example a review count given as
  a string): counted as failed, not coerced to an absent value and then reported as
  excluded, so the summary does not misattribute why the title is missing (P-9).
- A title that is in corpus but carries no name: written with the name omitted
  (never as an empty string, per S034) and counted as written; corpus membership
  is by application id.
- A network fetch that fails partway through a large run: the titles already
  written and the recorded resume point survive, so a later run continues rather
  than losing the work (no partial-corpus-presented-as-complete).
- The gate threshold set to zero: every game-classified title is kept; the gate
  still excludes non-games, and the summary still reports exclusions.
- The offline fixture source and the live source disagree in shape: the trait
  fixes the shape the seeder consumes, so the seeder cannot depend on a
  live-only detail that a fixture cannot express.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The seeder MUST obtain the app-list universe (per title: an
  application id, a name, and the signal needed to apply the corpus gate) from a
  catalog source, and write each in-corpus title's Tier 1 columns (application id,
  name, and catalog metrics such as review count, owners, or peak concurrent
  players when the source supplies them) into the store.
- **FR-002**: The catalog source MUST be an abstraction with at least two
  implementations: an offline, fixture-backed source used by every automated test,
  and a real source that performs read-only network reads in production. The
  seeder's fetch-parse-map-gate logic MUST be exercisable with no network access.
- **FR-003**: The real network source and its network-client dependency MUST be
  optional at build time (behind a feature), so a default build and the offline
  test suite compile neither the client nor a transport-security stack. The
  dependency MUST be justified to the project's dependency rubric and MUST keep the
  minimum supported toolchain green.
- **FR-004**: The corpus gate MUST keep only titles classified as games that meet a
  configurable popularity threshold (a review count), and MUST exclude the rest.
  The threshold MUST be configurable, with a documented default.
- **FR-005**: A seed run MUST produce a summary reporting, at minimum, the counts
  of titles fetched, written, excluded by the gate, duplicated within the run (an
  admitted appid already written, merged once but not double-counted), and failed
  (unfetchable or unparsable). No title may be dropped from the corpus without being
  counted in one of these categories, and no repeated appid may inflate the written
  count (P-4, P-9).
- **FR-006**: A single malformed or unfetchable title MUST NOT abort the run; it is
  counted as failed and the seed continues with the remaining titles.
- **FR-007**: The store MUST provide a per-tier merge that writes only the Tier 1
  columns for an application id (insert if absent, update only the Tier 1 columns
  if present), leaving any existing launch entries, engine attribution, and
  technologies for that application id unchanged.
- **FR-008**: The seed MUST be resumable: it records progress for the catalog tier
  (a resume cursor and a last-run timestamp) in the store's per-tier seed state,
  and a resumed run continues from the recorded position rather than restarting.
- **FR-009**: The seeder MUST NOT prune: titles present in the store but absent from
  a given run are left unchanged; the seeder only inserts and updates.
- **FR-010**: Every Tier 1 row the seeder writes MUST be exportable as a
  heuristic-unverified record; after any seed, the store MUST still export
  schema-valid JSON.
- **FR-011**: A command-line surface MUST drive a seed from a local fixture source
  with no network, so the pipeline is demonstrable and tested offline; the live
  network seed MUST be the same command against the real source, run by the
  operator.
- **FR-012**: Introducing the network client MUST NOT make `fragcap-core` depend on
  it; the dependency lands only in the targets crate (P-2).
- **FR-013**: An empty name MUST NOT be stored (consistent with S034): a title
  whose source name is empty or missing is written with the name omitted (stored
  as absent), not with an empty string, and is counted as written. Corpus
  membership is by application id, so a nameless in-corpus title is kept, not
  excluded.

### Key Entities

- **Catalog source**: The abstraction the seeder reads from. Yields, per title, an
  application id, a name, and the gate signal. Has an offline fixture-backed
  implementation and a real network implementation.
- **Catalog entry**: One title as the source presents it, before the gate: an
  application id, a name, a type classification, and a popularity signal.
- **Corpus gate**: The rule that decides whether a catalog entry is kept: a game
  meeting the configurable threshold.
- **Seed summary**: The truthful account of a run: fetched, written, excluded, and
  failed counts.
- **Catalog seed state**: The resumability record for the catalog tier: a resume
  cursor and a last-run timestamp, held in the store's existing per-tier seed
  state.
- **Tier 1 merge**: The store operation that writes only the catalog columns for an
  application id without disturbing other tiers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An offline fixture-backed seed run fills the store with exactly the
  in-corpus titles from the fixture and no others, and the store exports
  schema-valid JSON, with no network access.
- **SC-002**: Every title in a seed run is accounted for in the summary as written,
  excluded, duplicated, or failed; the counts reconcile with the source's total
  (fetched equals written plus excluded plus duplicates plus failed), so no title is
  silently lost and no repeat inflates the written count (verified by a conservation
  assertion in tests).
- **SC-003**: Seeding Tier 1 over a store already carrying engine and launch data
  for a title updates the catalog columns and leaves the engine and launch data
  unchanged, in 100% of such cases.
- **SC-004**: A resumed seed continues from the recorded position and yields the
  same final corpus as a single uninterrupted seed, with no duplicated rows.
- **SC-005**: A default build (without the network feature) compiles neither the
  network client nor a transport-security stack, and the full check set, including
  the minimum-supported-toolchain build, passes with the feature enabled.

## Assumptions

- The catalog source yields enough per-title signal to apply the gate (a type
  classification and a review count). Exactly which Web API endpoints supply this
  (a single app-list call, or an app-list call plus per-title detail, or a bulk
  third-party mirror) is a plan/research decision; the trait insulates the seeder
  from it, and the offline fixture stands in for it in tests.
- The default popularity threshold is a small positive number (a few hundred
  reviews), tunable by the operator; the exact default is set in the plan and
  recorded, and is not a load-bearing correctness value.
- The live network seed's throughput and rate-limit behavior against the real Web
  API is an operator concern; this slice's correctness is judged offline, and the
  real source is a thin adapter over the same trait.
- Catalog metrics beyond appid and name (review count, owners, peak concurrent
  players) are written when the source provides them and left absent otherwise;
  they are secondary enrichment, not required for an in-corpus row.
- The resume cursor's granularity (per-page, per-batch, or per-title) is a plan
  decision; the requirement is only that a resumed run does not restart from the
  beginning.
- The network client and its transport-security choice are selected in the plan to
  keep the minimum supported toolchain green and the dependency graph small,
  following the project's smallest-graph-that-does-the-job practice.
