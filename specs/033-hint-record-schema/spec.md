# Feature Specification: Target-Hint-Record Schema Revision

**Feature Branch**: `feat/hint-record-schema`

**Created**: 2026-08-13

**Slice**: S033 (GitHub issue #75 follow-up required by #83, prerequisite for
#78). The master target schema (built by S025, extended by S031) governs every
machine-readable targeting artifact fragcap emits or consumes. The targets hint
database (#78) will emit JSON that must validate against it, and #83's Steam
catalog research sharpened what a hint record must carry: Steam's launch
configuration is an array, not a scalar; some titles are launcher-mediated; and
engine attribution carries its own source and confidence. This slice revises the
loose hint-record subschema to model those, additively and backward-compatibly
within schema version 1, so #78 can emit conformant rows. Constitution principles
in play: an honest instrument, so the launch array is never flattened to a single
"the game binary" at seeding time and the engine's confidence is carried as the
guess it is (P-9); a glossary entry for every new term in the same change (P-6);
and compatibility, so the change is additive and the existing artifacts still
validate (P-5, and the schema's own versioning discipline).

**Input**: Extend the master target schema (the embedded
`crates/fragcap-profile/assets/target-schema.v1.json` and the byte-identical
published `docs/schema/target-schema.v1.json`, guarded by the existing drift
check) so the loose hint variant, and each record inside an export envelope, can
carry three new optional structures: a `launch` array (one entry per Steam launch
configuration, each with `os`, `osarch`, `launch_type`, `beta_branch`, a required
`executable`, `arguments`, and `description`), a `launcher_mediated` boolean, and
an `engine` object (an engine `name`, a `source` from a fixed enum, and a
`confidence` from a fixed enum). The engine confidence is a within-tier gradation
of a heuristic guess, not a fifth record fidelity tier; the record fidelity
vocabulary is unchanged. The strict profile and package variants (the authored
capture format) do not carry these hint-seeding fields. The hand-rolled variant
validator is extended to shape-check the new structures, conformance fixtures are
added, and glossary entries are written. Scope is schema, validator, fixtures, and
documentation only: no SQLite, no seeding pipeline, and no external fetching, all
of which belong to #78. No new dependency; the minimum supported toolchain stays
green.

## Overview

S025 built a single master schema with a shared core and four kind-discriminated
variants: a strict profile and package (the authored, capture-ready format), a
loose hint (a partial, unverified record a heuristic provider or the hint DB
emits), and an export envelope of loose records. The whole point of the shared
core was that a hint is a profile-of-the-core with looser required fields, so the
targeting system does not re-specify itself per subsystem.

The hint DB (#78) is the subsystem that will emit hints at scale, and #83's
research established three facts about what those hints must carry. First, Steam's
`config.launch` is a list of entries, each gated by an OS filter, an architecture
filter, a config type, and a beta-branch selector, and for a class of titles (ESO,
The Division 2) the entry Steam actually invokes is a publisher launcher, not the
socket-holding game binary. The hint must persist that whole array with its
filters intact and must never flatten it to a single "process name" at seeding
time, because reducing the array to the one binary that holds sockets is the
resolver's job (#77), performed at runtime against the real process tree, not a
seeding-time guess. Second, a `launcher_mediated` flag marks exactly that class of
titles, so the resolver has a second signal into the same stub-to-shipping hop the
engine rule already performs. Third, engine attribution at catalog scale comes
from sources of differing trust (a PCGamingWiki lookup, an executable-name
heuristic, or depot-filename rules), so an engine value carries its own `source`
and `confidence`, and a failed lookup simply leaves the engine absent rather than
invalidating the record.

This slice models those three in the schema's loose subschema. It does not build
the database, fetch from any source, or run any detection; it defines the shape
the database will fill and the validator that will police it, so #78 lands against
a schema that is already right.

Two honesty boundaries frame the design. The engine's `confidence` is not a new
rung on the record's fidelity ladder: the record's `fidelity`
(authored, verified, heuristic-unverified, observed) says how much to trust the
record as a whole, while the engine's `confidence` (confirmed, high, medium, low,
unknown) grades one heuristic field within it. Conflating them would let a
low-confidence engine guess quietly promote or demote a record's overall trust,
which is the kind of silent distortion P-9 forbids. And the strict profile and
package variants stay clean: these are hint-seeding fields, and an authored
capture profile that started carrying a Steam launch array and an engine
confidence would blur the line between what an author vouched for and what a
scraper guessed.

## Clarifications

### Session 2026-08-13

- Q: Which variants carry the three new fields? -> A: The loose ones. A hint is a
  single loose record, so it carries them at the top level; an export envelope
  carries them inside each of its records (the envelope top level is just
  provenance plus the records array). The strict profile and package variants,
  and the export envelope's own top level, do not carry them. This mirrors how
  `records` is permitted only on the export variant, and it keeps the authored
  capture format free of hint-seeding metadata. Recorded as a decision.
- Q: Is the launch entry's `executable` the only required field? -> A: Yes. An
  entry with no executable names nothing, so `executable` is required and
  non-empty; every other field (`os`, `osarch`, `launch_type`, `beta_branch`,
  `arguments`, `description`) is an optional filter or label, absent when Steam
  did not specify it. The array itself is optional (a hint may carry none), and
  when present it is never reduced to a single entry by this schema.
- Q: What does the `engine` object require when present? -> A: `source` and
  `confidence`, both from their fixed enums; the engine `name` is optional. A
  record that recorded an engine lookup carries where the value came from and how
  confident it is even when the lookup did not settle on a name (a `confidence` of
  `unknown` with no `name`); a record that never attempted engine attribution
  omits the `engine` object entirely. A failed lookup leaves the object absent,
  never a present-but-lying value (P-9).
- Q: Do the engine `confidence` and `source` vocabularies change the record
  fidelity model? -> A: No. The record `fidelity` enum is unchanged
  (authored, verified, heuristic-unverified, observed). The engine `confidence`
  (confirmed, high, medium, low, unknown) is a within-tier gradation of the one
  engine field, and the engine `source` (pcgamingwiki, exe_heuristic,
  depot_filename_rules) is distinct from the record's provenance `source` (a free
  string naming where the whole record came from). They are separate fields on
  separate objects and do not interact.
- Q: Is this a schema version bump? -> A: No. Every addition is an optional
  property on a variant that already exists, so an artifact that predates them
  still validates and an artifact that carries them now validates where the closed
  property set would previously have rejected it. This is the same additive,
  backward-compatible extension of schema version 1 that S031's `technologies`
  structure was. The two schema copies (embedded and published) stay
  byte-identical.
- Q: Is any database, fetch, or detection built here? -> A: No. This slice is the
  schema, the hand-rolled validator, conformance fixtures, and documentation. The
  SQLite database, the three-tier seeding pipeline, the PCGamingWiki and PICS
  lookups, and the depot-filename detection are all #78. This slice defines the
  shape #78 will emit and validate against.
- Q: Are the launch-entry filter fields (`os`, `osarch`, `launch_type`,
  `beta_branch`) constrained enums or free strings? -> A: Free strings (`type:
  string`, optional). Steam's launch-filter vocabularies are external and evolve
  (new OS tokens, new config types, arbitrary beta-branch names), so constraining
  them to a fixed enum would reject valid Steam data the moment Steam adds a value,
  which is a correctness cost for no honesty benefit. Only the two vocabularies
  this project's research actually fixes are enums: the engine `source` and the
  engine `confidence`. The engine `name` is likewise a free string. Recorded as a
  decision.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A hint record carries the full launch array, launcher flag, and engine attribution (Priority: P1)

The hint database (a future consumer) emits a hint for a title: its Steam launch
array with every entry's filters intact, a `launcher_mediated` flag, and an
`engine` object naming the engine, where the attribution came from, and how
confident it is. The record validates against the master schema.

**Why this priority**: This is the whole slice. Without it #78 has no conformant
shape to emit, and the launch array would be flattened or the engine confidence
lost. Delivered alone, it is the shape the database fills.

**Independent Test**: A hint fixture carrying a multi-entry launch array (each
with OS/arch/type/beta filters and a required executable), `launcher_mediated:
true`, and an `engine` object with a valid source and confidence validates with no
diagnostics.

**Acceptance Scenarios**:

1. **Given** a hint with a `launch` array of several entries each carrying an
   `executable` plus optional filters, **When** it is validated, **Then** it
   conforms and the array is preserved as an array (no flattening imposed by the
   schema).
2. **Given** a hint with `launcher_mediated: true` and an `engine` object whose
   `source` and `confidence` are in their enums, **When** it is validated,
   **Then** it conforms.
3. **Given** an export envelope whose records each carry these fields, **When** it
   is validated, **Then** every record conforms.

---

### US2 - The vocabularies are reconciled and honest (Priority: P1)

An engine confidence of `low` grades the engine field only; it does not change the
record's overall `fidelity`. A malformed engine `source` or `confidence`, or a
launch entry with no executable, is rejected with a diagnostic that names the
problem.

**Why this priority**: This is the P-9 guarantee for the slice. If a bad engine
enum passed validation, or if engine confidence bled into record fidelity, the
schema would let a guess masquerade as something it is not.

**Independent Test**: (a) A record with `fidelity: heuristic-unverified` and an
`engine.confidence: low` validates, and the two are independent fields. (b) An
`engine.source` outside its enum, an `engine.confidence` outside its enum, and a
launch entry missing `executable` are each rejected with a named diagnostic.

**Acceptance Scenarios**:

1. **Given** a hint with a record-level `fidelity` and an independent
   `engine.confidence`, **When** it is validated, **Then** both are accepted as
   separate fields and neither is required to match the other.
2. **Given** an `engine.source` or `engine.confidence` not in its enum, **When**
   it is validated, **Then** it is rejected naming the invalid value.
3. **Given** a launch entry object with no `executable`, **When** it is validated,
   **Then** it is rejected naming the missing required field.

---

### US3 - The strict authored format is unchanged (Priority: P1)

An authored profile or package does not carry the hint-seeding fields. A profile
that tries to declare a `launch` array, a `launcher_mediated` flag, or an `engine`
object is rejected, because those are hint-DB metadata and the authored capture
format stays clean. Every existing artifact still validates.

**Why this priority**: This is the boundary that keeps the shared core honest: the
loose subschema grows, the strict one does not. A profile silently accepting
scraper metadata would erode the authored-versus-guessed distinction the fidelity
model exists to protect.

**Independent Test**: (a) The existing valid profile, package, hint, and export
fixtures still validate unchanged. (b) A profile fixture carrying a `launch` array
(or `launcher_mediated`, or `engine`) is rejected as an unknown key.

**Acceptance Scenarios**:

1. **Given** every pre-existing conformance fixture, **When** the corpus is
   validated, **Then** each fixture's expected outcome is unchanged.
2. **Given** a profile or package carrying any of the three new fields, **When**
   it is validated, **Then** it is rejected (the strict variant does not accept
   hint-seeding fields).

---

### Edge Cases

- A hint with no `launch` array, no `launcher_mediated`, and no `engine` (the
  common minimal hint) still validates, exactly as today; the fields are optional.
- An empty `launch` array validates (a title for which no launch entries were
  found, distinct from a title never looked up); the schema does not require at
  least one entry.
- An `engine` object with a `confidence` of `unknown` and no `name` validates: the
  attribution was attempted and did not settle, which is honest to record.
- A launch entry carrying only `executable` (no filters) validates; the filters
  are optional.
- An unknown key inside a launch entry, an engine object, or the record is
  rejected (the closed property set is preserved, per the S05 rationale).
- The `engine.source` and the record `provenance.source` share a field name across
  two objects but are different vocabularies; a value valid for one is not
  required to be valid for the other.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The master schema MUST define a launch-entry structure with fields
  `os`, `osarch`, `launch_type`, `beta_branch` (optional free-string filters, not
  enums), `executable` (required, non-empty), `arguments`, and `description`
  (optional strings), and a closed property set (unknown keys rejected).
- **FR-002**: The schema MUST define an optional `launch` array of launch entries,
  carried by the loose hint variant and by each record in an export envelope, and
  MUST NOT require the array to be reduced to a single entry (the array is
  preserved; reduction is the resolver's runtime job, #77).
- **FR-003**: The schema MUST define an optional `launcher_mediated` boolean,
  carried by the same loose variants.
- **FR-004**: The schema MUST define an optional `engine` object with an optional
  `name` (string), a required `source` from the enum
  {`pcgamingwiki`, `exe_heuristic`, `depot_filename_rules`}, and a required
  `confidence` from the enum {`confirmed`, `high`, `medium`, `low`, `unknown`},
  with a closed property set, carried by the same loose variants.
- **FR-005**: The record `fidelity` enum MUST be unchanged
  (authored, verified, heuristic-unverified, observed); the engine `confidence`
  MUST be a separate field and MUST NOT be added to, or required to match, the
  record fidelity. The engine `source` MUST be separate from the record's
  provenance `source`.
- **FR-006**: The three new fields MUST NOT be accepted on the strict profile or
  package variant, nor on the export envelope's own top level; they are carried
  only where a single loose record lives (the hint top level and each export
  record).
- **FR-007**: Every addition MUST be additive and backward compatible within
  schema version 1 (no version bump): every pre-existing artifact MUST still
  validate, and the change MUST be applied identically to the embedded and the
  published schema copies (the drift check stays green).
- **FR-008**: The hand-rolled variant validator MUST be extended to shape-check
  the new structures wherever they are permitted, rejecting a launch entry with no
  executable, an out-of-enum engine source or confidence, and an unknown key in
  any new object, each with a named diagnostic; and it MUST reject the new fields
  on the strict variants.
- **FR-009**: Conformance fixtures MUST cover: a valid hint with a launch array,
  `launcher_mediated`, and an engine object; an out-of-enum engine source; an
  out-of-enum engine confidence; a launch entry missing `executable`; and a strict
  profile carrying a new field (rejected). The existing fixtures MUST keep their
  outcomes.
- **FR-010**: New terms (the launch array/entry, launcher-mediated, engine
  attribution with its source and confidence) MUST gain glossary entries in the
  same change, and the specification MUST document the revised hint-record
  subschema and the confidence-versus-fidelity reconciliation (P-6).
- **FR-011**: The slice MUST add no new runtime dependency and MUST keep the
  minimum supported toolchain green; all emitted JSON and edited prose MUST be
  UTF-8 without BOM, LF, with no em or en dashes.

### Key Entities *(include if feature involves data)*

- **Launch entry**: one Steam launch configuration. Attributes: `os`, `osarch`,
  `launch_type`, `beta_branch` (optional filters), `executable` (required),
  `arguments`, `description` (optional). The invoked binary for launcher-mediated
  titles is a publisher launcher, not the socket holder; the entry records what
  Steam declares, not what holds sockets.
- **Launch array**: the ordered list of launch entries for one title, carried
  whole and never flattened at seeding time.
- **Launcher-mediated flag**: a boolean marking a title whose invoked launch entry
  starts a publisher launcher that then starts the real client.
- **Engine attribution**: an engine `name`, the `source` it came from
  (pcgamingwiki, exe_heuristic, depot_filename_rules), and the `confidence` in it
  (confirmed, high, medium, low, unknown). A within-record grading of one
  heuristic field, distinct from the record's fidelity.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A hint carrying a multi-entry launch array (with filters and
  required executables), `launcher_mediated`, and a valid engine object validates
  with no diagnostics, in a conformance test.
- **SC-002**: An out-of-enum engine `source`, an out-of-enum engine `confidence`,
  and a launch entry missing `executable` are each rejected with a named
  diagnostic, in conformance tests.
- **SC-003**: A profile or package carrying any of the three new fields is
  rejected, and every pre-existing fixture keeps its expected outcome, in the
  conformance corpus.
- **SC-004**: The embedded and published schema copies remain byte-identical (the
  drift check passes), and no schema version bump is made.
- **SC-005**: The record `fidelity` enum is unchanged and the engine `confidence`
  is a separate field, demonstrated by a fixture that carries both independently.
- **SC-006**: The full repository gate (`cargo xtask ci`) passes and the minimum
  supported toolchain (`cargo xtask msrv`) stays green, with no new dependency.

## Assumptions

- The master schema (S025), its four kind variants, the hand-rolled variant
  validator bound to it by the conformance corpus, and the additive-extension
  pattern S031 used for `technologies` are the contract this slice extends.
- The authoritative field shapes are the Steam catalog research gist referenced by
  #83 (its proposed `launch_entries` table and `games` engine columns); this slice
  models those fields in JSON, adjusting only for the JSON object shape (a `launch`
  array of entry objects rather than a foreign-keyed table, and an `engine` object
  rather than three flat columns).
- The confidence and source vocabularies come from that research
  (`confirmed|high|medium|low|unknown` and
  `pcgamingwiki|exe_heuristic|depot_filename_rules`); reconciling them with the
  fidelity model means carrying them as separate fields, not remapping them onto
  the fidelity enum.
- The database, the seeding pipeline, and all external fetching are out of scope
  and belong to #78; this slice is the schema those will target.
- Fixtures are committed JSON under the profile crate's test fixtures, validated
  by the existing conformance corpus test; no new test harness is introduced.
