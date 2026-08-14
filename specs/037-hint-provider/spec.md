# Feature Specification: Hint database resolution provider

**Feature Branch**: `037-hint-provider`

**Created**: 2026-08-13

**Status**: Draft

**Input**: User description: "S037: Wire the targets hint database into the live
resolution cascade as the precedence-2 (HintDatabase) provider. This is the
final wiring slice of issue #78."

## Clarifications

### Session 2026-08-13

Resolved under autopilot from the architecture of record, the constitution, and
the established provider precedents (S029 engine rule, S030 platform walker).

- Q: How does a hint database become available to resolution (there is no
  discovery convention today)? → A: Only when the operator supplies its path
  explicitly, via a new `--hint-db <path>` option on the capture command that
  resolves a Steam title, plus a `FRAGCAP_HINT_DB` environment override (mirroring
  `FRAGCAP_PROFILE_DIR`) that tests use to point at a scratch database. This slice
  introduces no automatic database-discovery location. Neither set, or the path
  absent, leaves precedence 2 empty; a set path that cannot be opened is an error.
- Q: When a row's launch array names more than one executable, which is the
  capture identity? → A: Consider only launch entries applicable to the capture
  platform (an entry whose operating-system filter is unset or names Windows);
  reduce them to the set of distinct executable file names (the file-name
  component, compared case-insensitively). Exactly one distinct name resolves;
  zero is a decline (no usable executable); two or more is an ambiguity decline
  with a recorded note. Repeated identical executables across configurations count
  as one name, not several.
- Q: What provenance source does a hint answer carry? → A: The same `hint-db`
  label the database's export projection already stamps, so the database has one
  honest name across its read (resolution) and write (export) surfaces, and never
  names a source it did not read (P-9).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A seeded title resolves from the hint database (Priority: P1)

An operator points fragcap at a Steam title by its application id. No authored
profile exists for it, but the shipped hint database carries a launch executable
for that title. Instead of falling through to a filesystem heuristic or waiting
for the process to appear, the cascade answers from the hint database: it names
the executable to capture, marks the answer heuristic-unverified, and records
that the answer came from the hint database.

**Why this priority**: This is the whole point of the hint database and the last
wiring step of issue #78. Without it the database is built, seeded, and inert;
with it, a title the community has documented resolves without an operator
authoring a profile first.

**Independent Test**: Seed an in-memory store with one game row that carries a
launch executable, ask the cascade to resolve that application id, and assert it
returns a heuristic-unverified target naming that executable, with provenance
that names the hint database.

**Acceptance Scenarios**:

1. **Given** a hint database holding a row for application id `A` with a single
   launch executable `game.exe`, **When** the cascade resolves a request carrying
   application id `A`, **Then** it returns a target at the heuristic-unverified
   tier whose capture identity matches `game.exe` and whose provenance names the
   hint database.
2. **Given** the same row also records `launcher_mediated = true` and an engine,
   **When** the row resolves, **Then** those facts are carried on the answer and
   not discarded.
3. **Given** an authored profile also matches the same request, **When** the
   cascade resolves, **Then** the profile answer wins because it outranks the
   hint database, and the hint database is not consulted for the final answer.

---

### User Story 2 - The database defers rather than guesses (Priority: P1)

The hint database holds many rows that are too sparse to name a process: a
catalog-only row that has just an application id and a name, or an engine-only
row that records the engine but no launch executable. For those, the hint
provider must not invent a capture identity. It declines, and the cascade
continues to the filesystem engine rule, the platform walker, and finally
runtime observation, exactly as it does today.

**Why this priority**: Answering with a guessed identity would arm a capture
against the wrong process, or against nothing, and count no loss for it. That is
the silent-loss failure the project forbids, and the honest-instrument principle
forbids claiming an answer the row does not support. A wrong hint is worse than
no hint, because a lower provider could have answered correctly.

**Independent Test**: Seed a store with a catalog-only row and an engine-only
row, resolve each application id, and assert the hint provider declines (returns
no answer) so a lower provider is consulted.

**Acceptance Scenarios**:

1. **Given** a row with only an application id and a name, **When** the cascade
   resolves that application id, **Then** the hint provider declines and the
   answer, if any, comes from a lower provider.
2. **Given** a row with an engine recorded but no launch executable, **When** the
   cascade resolves that application id, **Then** the hint provider declines.
3. **Given** a row whose launch entries name several distinct executables that
   cannot be reduced to one, **When** the cascade resolves that application id,
   **Then** the hint provider declines and records why (an ambiguity note), so a
   not-resolved outcome can explain itself.
4. **Given** a request that carries no application id, **When** the cascade
   resolves, **Then** the hint provider declines because it has nothing to look
   up.

---

### User Story 3 - No database is not an error (Priority: P1)

The hint database is optional. A build may exclude it entirely, or a user may run
without a database file present. In every such case the tool behaves exactly as
it did before this feature: the precedence-2 slot is simply empty, the cascade
skips it, and no error or warning is produced by its absence.

**Why this priority**: The database is an optional enhancement, not a
requirement. Making its absence an error, or changing the no-database behavior in
any observable way, would regress every existing capture path.

**Independent Test**: Resolve a request through the cascade with no database
available and assert the outcome is byte-identical to the pre-feature cascade
(same target, or the same not-resolved outcome).

**Acceptance Scenarios**:

1. **Given** the hint database is not compiled into this build, **When** any
   request is resolved, **Then** the result is identical to today's cascade with
   the no-answer stub.
2. **Given** the database is available in the build but no database file is
   present, **When** a request is resolved, **Then** the hint provider is not
   registered, no error is raised for the missing file, and the cascade skips
   precedence 2.

---

### User Story 4 - A live process still overrides a stale hint (Priority: P2)

A hint is a documented guess, not an observation. When the real process is
running and can be matched, the tool must be able to prefer the live truth. The
hint database sits above runtime observation in the cascade, so a hint arms the
capture; but because the answer is marked heuristic-unverified and carries the
identity to match, the live capture path can still refine or correct the match
against the running process rather than treating the hint as ground truth.

**Why this priority**: The honest-instrument principle requires that an inferred
answer never masquerade as an observed one, and that a live observation can
always correct a heuristic. This protects against a stale or wrong hint silently
misdirecting a capture.

**Independent Test**: Resolve a request where both a hint and a live observation
could contribute, arm the capture from the hint, and confirm the fidelity stamp
is heuristic-unverified (never observed) and the carried identity is what the
live path re-matches against.

**Acceptance Scenarios**:

1. **Given** a hint-resolved target, **When** it is used to arm a capture, **Then**
   its fidelity is heuristic-unverified, never observed or authored.
2. **Given** the hint carries the executable identity, **When** the target
   process starts, **Then** the live capture path binds it by that identity, so a
   correct live match refines the hint rather than being blocked by it.

---

### Edge Cases

- A database file exists but is unreadable or of an unexpected schema version:
  opening it fails. This is a real error the operator must see (a corrupt or
  wrong-version database is not the same as an absent one), surfaced through the
  wiring layer, not swallowed as a silent decline.
- A row's launch entries repeat the same executable across several
  configurations (operating systems, arguments, beta branches). This is one
  distinct executable, not several: it resolves, it is not ambiguous.
- A launch executable is recorded as a path fragment rather than a bare file
  name. The capture identity is keyed on the executable's file-name component so
  it matches the same way an authored profile's executable predicate does.
- The hint database and an authored profile disagree. The authored profile wins
  unconditionally because it is the higher-precedence, higher-fidelity source.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The tool MUST resolve a capture target from the hint database when
  a request carries a Steam application id and the database holds a row for it
  that names a usable launch executable.
- **FR-002**: A hint-database answer MUST be stamped at the heuristic-unverified
  fidelity tier, never authored, verified, or observed.
- **FR-003**: A hint-database answer MUST carry a provenance whose source is the
  `hint-db` label the database's export already uses, and MUST NOT name a data
  source the provider did not read.
- **FR-004**: The hint provider MUST occupy precedence 2 in the cascade: below
  authored and curated profiles, and above the engine rule, the platform walker,
  and runtime observation.
- **FR-005**: When a matching authored or curated profile exists for the same
  request, the profile answer MUST win over the hint database.
- **FR-006**: When the hint database declines and a lower provider (engine rule,
  platform walker, runtime observation) can answer, the lower provider's answer
  MUST be used.
- **FR-007**: The hint provider MUST decline (return no answer) when the row is
  absent, when the row carries no usable launch executable, when the request
  carries no application id, or when the row is launcher-mediated (its launch
  executable is the publisher launcher rather than the socket-holding client, so
  resolving it would record the launcher as the game).
- **FR-008**: After restricting a row's launch entries to those applicable to the
  capture platform (operating-system filter unset or naming Windows) and reducing
  them to the set of distinct executable file names (file-name component, compared
  case-insensitively), the hint provider MUST decline when that set holds more than
  one name, and MUST record an ambiguity note so a not-resolved outcome can explain
  itself. A set of exactly one name resolves; an empty set declines under FR-007.
- **FR-009**: The `launcher_mediated` flag and the engine attribution MUST be
  carried on a hint answer when the row records them, so they are not silently
  lost.
- **FR-010**: The concrete hint provider MUST reside in a component that is
  permitted to depend on both the resolver contract and the targets database, and
  MUST NOT introduce a dependency from the resolver's home component onto the
  targets database.
- **FR-011**: The resolver-home component's no-answer hint stub MUST be removed,
  so precedence 2 is served by exactly one implementation (the concrete provider,
  when registered) and never by two.
- **FR-012**: The tool MUST register the concrete hint provider only when the
  targets database is available in the build and the operator has supplied a
  database path (through the `--hint-db` option or the `FRAGCAP_HINT_DB`
  environment override) that points at a present file; otherwise it MUST leave
  precedence 2 empty.
- **FR-013**: The absence of a hint database MUST NOT be reported as an error, and
  MUST NOT change any observable resolution or capture behavior relative to a
  build without this feature.
- **FR-014**: A database file that is present but cannot be opened (corrupt or of
  an unexpected schema version) MUST surface as an error to the operator, not be
  silently treated as absent.
- **FR-015**: The Steam application id MUST reach the hint provider through the
  resolution request, alongside the request's other inputs, so a single request
  can offer the application id to the hint provider and an install location to
  the lower providers without the providers interfering.
- **FR-016**: The entire feature MUST be testable offline, with no network access
  and no running game, using a store the test seeds directly.
- **FR-017**: This slice MUST NOT add any seeding capability, and MUST NOT read or
  populate the deferred Tier 2 (launch-array) data beyond what a store already
  holds.

### Key Entities *(include if feature involves data)*

- **Hint answer**: A resolved capture target derived from a hint-database row. It
  carries a capture identity (the executable to match), a heuristic-unverified
  fidelity stamp, a provenance naming the hint database, and the carried facts
  (`launcher_mediated`, engine) the row recorded. It is distinct from a profile:
  it names a process to match, not a set of authored stages.
- **Hint-database row**: A stored game record keyed by Steam application id. It
  may be sparse (application id and name only), engine-annotated, or carry a
  launch array. Only a row with a usable launch executable produces a hint
  answer; sparser rows produce a decline.
- **Resolution request input**: The Steam application id, carried alongside the
  existing request inputs (profile reference, identity and process tree, install
  location), so the hint provider can be offered its input without disturbing the
  other providers.
- **Ambiguity note**: A record the hint provider leaves when it recognizes a row
  but cannot reduce it to one executable, so a fully unresolved outcome can state
  why the hint database did not answer.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A title that has a launch executable in the hint database and no
  authored profile resolves to a named, heuristic-unverified capture target with
  no operator-authored profile and no running game.
- **SC-002**: For every row the hint provider cannot turn into a single
  executable, resolution falls through to a lower provider or a named
  not-resolved outcome; in no case does it arm a capture against a guessed
  process.
- **SC-003**: With no hint database available, 100% of resolution outcomes are
  identical to the pre-feature tool, and no error is attributable to the missing
  database.
- **SC-004**: Every hint answer is distinguishable from an authored, verified, or
  observed answer by its fidelity stamp and provenance, so an operator can always
  tell a documented guess from an observed fact.
- **SC-005**: The full behavior is demonstrated by tests that run with no network
  and no game, including the profile-outranks-hint and hint-outranks-engine-rule
  orderings.
- **SC-006**: The dependency-direction check passes: no dependency is introduced
  from the resolver's home component onto the targets database.

## Assumptions

- The concrete provider lives in the targets-database component, which already
  depends on the resolver contract, mirroring the established precedent where the
  platform walker lives in the storefront component for the same
  dependency-direction reason. The resolver's home component keeps the trait and
  the precedence position but not the concrete database-reading provider.
- The tool assembles and injects the provider at its command layer, which is the
  only surface permitted to depend on both the resolver contract and the targets
  database.
- The hint database's location is supplied to the tool explicitly, through a
  `--hint-db <path>` option on the capture command and a `FRAGCAP_HINT_DB`
  environment override (mirroring `FRAGCAP_PROFILE_DIR`), the same spirit in which
  the existing database subcommands take a database path. This slice introduces no
  automatic database-discovery convention; a database is registered for resolution
  only when the operator makes one available.
- "Usable launch executable" means a non-empty executable file name recorded on a
  row's launch array. The deferred Tier 2 seeder is what will populate that array
  at scale; until then, tests seed it directly, and production rows that lack it
  simply decline, which is correct behavior, not a defect.
- Selecting among several distinct executables by any coincidental signal (size,
  order) is disallowed; several distinct executables is an ambiguity decline,
  matching the discipline of the engine rule and the platform walker.
- Reusing the established resolution-request, provider-trait, and fidelity/
  provenance machinery is preferred over introducing new mechanisms; this slice
  is wiring, not new architecture.
