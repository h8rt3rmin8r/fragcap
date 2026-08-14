# Feature Specification: Targets Hint Database Tier 3 Seeder (engine attribution)

**Feature Branch**: `036-engine-seeder`

**Created**: 2026-08-13

**Status**: Draft

**Slice**: S036 (issue #78, third of several; follows S034 foundation and S035
Tier 1 catalog seeder)

**Input**: The engine seeder for the targets hint database: fill the Tier 3
(engine attribution) columns (engine name, source, and confidence) from
PCGamingWiki, keyed by Steam application id, reusing the S035 seeder architecture
wholesale, with the fetch abstracted so the whole pipeline is tested offline and
the live source compiled but never run.

## Overview

The S034 foundation built the store and its three-tier column model; S035 wrote
the first seeder, for Tier 1 (the public catalog corpus). This slice writes the
Tier 3 seeder, engine attribution.

Tier 3's job is to record, per title, which game engine the title is built on:
an engine name (Unreal Engine, Unity, and so on), the source that attributed it,
and a confidence grade. The source of record for this tier is PCGamingWiki, whose
community-maintained pages carry an engine field keyed by Steam application id.
The seeder reads that field, maps it into the store's engine columns, and stamps
`engine_source = "pcgamingwiki"` on every row it writes.

The confidence grade is a within-field quality signal, not a trust tier. A row's
overall fidelity stays `heuristic-unverified`, exactly as every hint the database
emits does (P-9): PCGamingWiki says a title uses Unreal, and that is a documented
community claim, never a fact fragcap verified against the running binary. The
confidence column grades how well-attested that one field is; it does not raise
or lower the record's overall trust, and it is emphatically not a fifth fidelity
tier. When PCGamingWiki has no engine for a title, or names an engine
ambiguously, the seeder leaves the engine absent rather than guessing one: a
missing engine is an honest absence, and a guessed engine would be a hint that
looks attributed but is not.

This slice reuses the S035 architecture wholesale rather than inventing a new
one. The engine source is an abstraction with the same shape as S035's catalog
source: an offline, fixture-backed implementation drives every automated test,
and a real implementation performs read-only network reads against PCGamingWiki's
query API in production, behind the same off-by-default network feature S035
introduced and using the same HTTP client that slice already chose and justified.
No new dependency is taken, and the minimum supported toolchain stays green
because the network feature is off in the default and toolchain-check builds. The
seed run returns the same conservation-checked summary S035 defined, and the
store gains a per-tier merge that writes only the engine columns, leaving the
Tier 1 catalog data and any Tier 2 launch data intact, exactly as S035's Tier 1
merge leaves the other tiers untouched.

Tier 2 (launch metadata from Steam PICS/appinfo) and wiring the database into the
live resolver (the precedence-2 hint provider) are later slices. This slice ends
at a store whose engine columns can be filled from PCGamingWiki, offline-tested,
resumable, honestly accounted, and still schema-valid on export.

## Clarifications

### Session 2026-08-13

- Q: Does this slice make live network calls in continuous integration? -> A: No.
  The engine source is a trait; every test drives an offline fixture-backed
  implementation. The real HTTP implementation is compiled under the existing
  `net` feature but never executed in CI, the same posture as S035 and as live
  packet capture. `cargo xtask ci` and `cargo xtask msrv` stay green with no
  network.
- Q: Does this slice add a dependency? -> A: No. It reuses the `http_req` client
  and `net` feature S035 settled for the whole seeder arc. MSRV 1.82 stays
  non-binding because `net` is off in the default and toolchain-check builds.
- Q: How does Tier 3 avoid clobbering Tiers 1 and 2? -> A: The store gains a
  per-tier merge that upserts only the engine columns for an application id,
  leaving name, catalog metrics, launcher_mediated, token_required, launch
  entries, and technologies untouched. This mirrors S035's Tier 1 merge.
- Q: Where does the confidence grade come from, and what is it? -> A: The source
  yields, per title, an optional confidence token; the seeder stores it as the
  engine confidence and stamps the source as pcgamingwiki. It is a within-field
  grade of one heuristic field, never a fifth fidelity tier: the row stays
  heuristic-unverified regardless (P-9). The live source's mapping from a
  PCGamingWiki engine field to a confidence token is a plan decision; the offline
  fixture supplies the token directly so the store path is exercised across all
  confidence values.
- Q: A title PCGamingWiki knows but with no engine, or an ambiguous engine:
  written or excluded? -> A: Excluded, left absent. A missing or ambiguous engine
  is not guessed; the row's engine columns are left unset and the title is
  counted as excluded in the summary, so a store without an engine for a title
  reads as "not attributed", never as a guessed attribution (P-9).
- Q: What does the source yield for the engine to be written? -> A: Per title, an
  application id, an optional engine name, and an optional confidence token. The
  engine is written only when the source resolves a usable, unambiguous engine
  name; source and confidence are always written together (the store's
  both-or-neither engine invariant), with the source fixed to pcgamingwiki and the
  confidence defaulting to a documented token when the source omits it.
- Q: An engine for an application id the store has never seen (not in the Tier 1
  corpus): written or skipped? -> A: Written, as an engine-only row (application
  id plus engine columns, no name). The merge inserts if the application id is
  absent, mirroring the Tier 1 merge; an engine-only row is schema-valid on
  export. The tiers are independent, so an engine hint does not require a prior
  catalog row.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Seed engine attributions into the store, offline-tested (Priority: P1)

A maintainer runs the engine seeder against an engine source; the store's engine
columns are filled for each title the source resolves an engine for
(`engine_source = "pcgamingwiki"`, an engine name, and a confidence grade), and
the run reports how many titles it fetched, wrote, excluded (no or ambiguous
engine), duplicated within the run, and failed to parse. The store then exports
schema-valid JSON in which those titles carry an engine and every record is still
heuristic-unverified.

**Why this priority**: This is the seeder's entire purpose and the MVP: turn a
PCGamingWiki engine source into Tier 3 rows, honestly accounted, testable offline
against fixtures. Every later scenario builds on it.

**Independent Test**: Drive the seeder with an offline fixture-backed source
carrying a mix of titles with clear engines, a title with no engine, a title with
an ambiguous engine, and a malformed entry. Assert the store holds an engine only
for the clear titles, the summary counts every category, the conservation
identity holds, and the export validates.

**Acceptance Scenarios**:

1. **Given** an engine source yielding several titles with clear engines and
   several with no or ambiguous engines, **When** the seeder runs, **Then** the
   store's engine columns are set for the clear titles and unset for the others,
   and the summary reports the written and excluded counts.
2. **Given** a source that yields a malformed entry (missing application id, a
   wrong-typed field, or an out-of-set confidence token) among good ones,
   **When** the seeder runs, **Then** the good titles are written, the bad entry
   is counted as failed (not coerced to an absent engine and reported as
   excluded), and the run does not abort on it.
3. **Given** a freshly seeded store, **When** it is exported, **Then** the output
   validates against the target schema, each engine-bearing record carries the
   engine name, source `pcgamingwiki`, and a confidence, and every record's
   fidelity is heuristic-unverified.

### User Story 2 - Enrich engine without disturbing catalog or launch data (Priority: P1)

A maintainer whose store already carries Tier 1 catalog data (name, metrics) and
Tier 2 launch data for some titles runs the engine seeder; the engine columns
fill in but the name, metrics, launcher flags, and launch entries survive
untouched.

**Why this priority**: The three-tier model's whole promise is independent tiers.
An engine seed that erased the catalog name or the launch array would force a full
re-seed of everything, defeating the design. This exercises the per-tier engine
merge and is the direct analogue of S035's Tier 1 preservation guarantee.

**Independent Test**: Populate a title with a name, launcher flag, and launch
entries (Tiers 1 and 2), run the engine seeder over a source that names that
application id's engine, and assert the engine columns filled while the name,
launcher flag, and launch entries are unchanged.

**Acceptance Scenarios**:

1. **Given** a stored game with a catalog name and launch entries, **When** the
   engine seeder writes that application id's engine, **Then** the engine columns
   are set and the name and launch entries are unchanged.
2. **Given** a stored game absent from the current engine run, **When** the
   seeder runs, **Then** that game's rows are left as they are (the seeder adds
   and updates; it does not prune).

### User Story 3 - Resume a large engine seed without restarting (Priority: P2)

A maintainer seeding engines for a large title universe is interrupted (or seeds
in stages); on the next run the seeder continues from where it left off rather
than re-fetching and re-writing everything.

**Why this priority**: The universe is large; a seed that cannot resume must
complete in one run or waste the work. Resumability is a stated requirement of the
seeding model and is already supported by the store's per-tier seed state, keyed
by the engine tier.

**Independent Test**: Drive a partial seed that records progress, then a second
run against the same store and source; assert the second run continues from the
recorded cursor for the engine tier and the final result equals a single
uninterrupted seed.

**Acceptance Scenarios**:

1. **Given** an engine seed that processed part of the universe and recorded its
   progress under the engine tier, **When** the seeder runs again against the same
   store, **Then** it resumes from the recorded position rather than restarting.
2. **Given** a completed engine seed, **When** the seeder runs again, **Then** it
   refreshes without duplicating rows, and the engine tier's seed state records
   the last run.

### Edge Cases

- An engine source that returns an empty universe: the seed completes writing
  nothing, and the summary reports zero fetched, zero written.
- A title present twice in the source within one run, both with an engine: written
  once (the merge is idempotent per application id), and the repeat is counted as
  a duplicate rather than inflating the written count.
- An entry with a present but wrong-typed field (an application id given as a
  string, or a confidence given as a number): counted as failed, not coerced, so
  the summary does not misattribute why the engine is missing (P-9).
- An entry whose confidence token is a string but outside the schema's confidence
  set: counted as failed, not silently downgraded to a default, so an unknown
  grade is surfaced rather than hidden.
- A title the source names with no engine, or with an engine field that resolves
  ambiguously: excluded, its engine columns left unset, counted as excluded; the
  seeder never guesses an engine (P-9).
- An engine for an application id the store has never seen: written as an
  engine-only row (no name), which is schema-valid on export; the tiers are
  independent.
- A network fetch that fails partway through a large run: the engines already
  written and the recorded resume point survive, so a later run continues rather
  than losing the work.
- The offline fixture source and the live source disagree in shape: the trait
  fixes the shape the seeder consumes, so the seeder cannot depend on a live-only
  detail that a fixture cannot express.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The seeder MUST obtain, per title, an application id, an optional
  engine name, and an optional confidence token from an engine source, and write
  the engine columns (name, `source = "pcgamingwiki"`, confidence) for each title
  the source resolves a usable, unambiguous engine name for.
- **FR-002**: The engine source MUST be an abstraction with at least two
  implementations: an offline, fixture-backed source used by every automated test,
  and a real source that performs read-only network reads against PCGamingWiki in
  production. The seeder's fetch-parse-map logic MUST be exercisable with no
  network access.
- **FR-003**: The real network source MUST sit behind the existing off-by-default
  network feature and reuse the HTTP client S035 already chose; this slice MUST
  add no new dependency, and MUST keep the minimum supported toolchain green (the
  network feature is off in the default and toolchain-check builds).
- **FR-004**: A title the source names with no engine, or with an engine that
  resolves ambiguously, MUST be left with its engine columns unset (not guessed)
  and counted as excluded. The seeder MUST NOT write a placeholder or inferred
  engine (P-9).
- **FR-005**: A seed run MUST produce a summary reporting the counts of titles
  fetched, written, excluded (no or ambiguous engine), duplicated within the run
  (an admitted application id already written, merged once but not double-counted),
  and failed (unparsable or carrying a wrong-typed or out-of-set field). No title
  may be dropped without being counted in one of these categories, and the
  conservation identity (fetched equals written plus excluded plus duplicates plus
  failed) MUST hold (P-4, P-9).
- **FR-006**: A single malformed or unfetchable title MUST NOT abort the run; it
  is counted as failed and the seed continues with the remaining titles.
- **FR-007**: The store MUST provide a per-tier engine merge that writes only the
  engine columns for an application id (insert an engine-only row if the
  application id is absent, update only the engine columns if present), leaving any
  existing name, catalog metrics, launcher flags, launch entries, and technologies
  for that application id unchanged. Source and confidence MUST be written together
  (the store's both-or-neither engine invariant).
- **FR-008**: The seed MUST be resumable: it records progress for the engine tier
  (a resume cursor and a last-run timestamp) in the store's existing per-tier seed
  state, and a resumed run continues from the recorded position rather than
  restarting.
- **FR-009**: The seeder MUST NOT prune: titles present in the store but absent
  from a given run are left unchanged; the seeder only inserts and updates.
- **FR-010**: Every engine the seeder writes MUST be exportable as part of a
  heuristic-unverified record; after any engine seed, the store MUST still export
  schema-valid JSON, and no engine seed may change a record's fidelity (it stays
  heuristic-unverified). The engine confidence MUST be a within-field grade, never
  a fidelity tier (P-9).
- **FR-011**: A command-line surface MUST drive an engine seed from a local
  fixture source with no network, so the pipeline is demonstrable and tested
  offline; the live network seed MUST be the same command against the real source,
  run by the operator.
- **FR-012**: Introducing the engine source MUST NOT make `fragcap-core` depend on
  it; the code lands only in the targets crate (P-2).
- **FR-013**: A present but wrong-typed field (an application id or a confidence of
  the wrong JSON type) or an out-of-set confidence token MUST be counted as failed,
  never coerced to a default and then reported as excluded or written; the summary
  must not misattribute why an engine is or is not present (P-9).

### Key Entities

- **Engine source**: The abstraction the seeder reads from. Yields, per title, an
  application id, an optional engine name, and an optional confidence token. Has an
  offline fixture-backed implementation and a real PCGamingWiki implementation.
- **Engine entry**: One title as the source presents it, before the seeder's
  keep-or-exclude decision: an application id, an optional engine name, and an
  optional confidence token.
- **Engine attribution**: What the seeder writes into the store: an engine name,
  the source `pcgamingwiki`, and a confidence grade, source and confidence always
  together.
- **Seed summary**: The truthful account of a run: fetched, written, excluded,
  duplicated, and failed counts (the same shape S035 defined, reused).
- **Engine seed state**: The resumability record for the engine tier: a resume
  cursor and a last-run timestamp, held in the store's existing per-tier seed
  state under the engine tier.
- **Tier 3 merge**: The store operation that writes only the engine columns for an
  application id without disturbing the other tiers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An offline fixture-backed engine seed run fills the store's engine
  columns for exactly the titles the fixture resolves an engine for and no others,
  and the store exports schema-valid JSON, with no network access.
- **SC-002**: Every title in an engine seed run is accounted for in the summary as
  written, excluded, duplicated, or failed; the counts reconcile with the source's
  total (fetched equals written plus excluded plus duplicates plus failed), so no
  title is silently lost and no repeat inflates the written count (verified by a
  conservation assertion in tests).
- **SC-003**: Seeding engine over a store already carrying catalog and launch data
  for a title fills the engine columns and leaves the name, metrics, launcher flag,
  and launch entries unchanged, in 100% of such cases (verified by a test that
  seeds engine over a catalog-seeded, launch-bearing game and asserts the name
  survives).
- **SC-004**: A resumed engine seed continues from the recorded position and
  yields the same final result as a single uninterrupted seed, with no duplicated
  rows.
- **SC-005**: A default build (without the network feature) compiles no new
  dependency and no transport-security stack, and the full check set, including the
  minimum-supported-toolchain build, passes; the live PCGamingWiki source is
  compiled under the network feature but never run by a test.

## Assumptions

- The engine source yields enough per-title signal to write an engine: an
  application id and a resolvable engine name, plus an optional confidence token.
  Exactly which PCGamingWiki query surface supplies this (the MediaWiki Cargo query
  API over the engine field, keyed by Steam application id) is a plan/research
  decision; the trait insulates the seeder from it, and the offline fixture stands
  in for it in tests.
- The confidence grade is a within-field quality signal supplied per entry by the
  source (the fixture supplies it directly to exercise all values; the live source
  applies a fixed mapping from a resolved engine field). Its exact live mapping and
  default token are set in the plan and recorded; the grade is not a load-bearing
  correctness value and never a fidelity tier.
- "Ambiguous" means a source engine field that does not resolve to a single usable
  engine name (empty, multiple conflicting engines, or an unrecognizable value);
  the precise resolution rule is a plan decision, and the requirement is only that
  an unresolved engine is excluded, not guessed.
- The live network seed's throughput and rate-limit behavior against PCGamingWiki
  is an operator concern; this slice's correctness is judged offline, and the real
  source is a thin adapter over the same trait.
- The resume cursor's granularity (per-page or per-batch) is a plan decision,
  reusing the store's existing per-tier seed-state mechanism; the requirement is
  only that a resumed run does not restart from the beginning.
