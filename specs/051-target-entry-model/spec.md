# Feature Specification: The target entry model (handles, stable ids, selector resolution, cascade collapse)

**Feature Branch**: `051-target-entry-model`

**Created**: 2026-08-16

**Status**: Draft

**Slice**: S051 (GitHub issue #138, milestone v0.5.0). Depends on S050 (merged).

**Input**: fragcap v0.5.0 UX Handoff Plan sections 3.2, 3.5, 5, 6, and Appendix A.

## Clarifications

### Session 2026-08-16 (autopilot)

- Q: Does "retire the AppData profile directory and `--profile <path>`" mean
  migrate existing profile files into `local.db`, or drop the file-based path
  outright? -> A: Drop it outright. v0.5.0 ships no deprecation shims (the
  precedent set in S050), and the whole point of the slice is that a target is a
  row, not a file. Existing loose profile files are neither read nor
  auto-imported by this slice; a user re-registers a target through the target
  entry model. Migration tooling, if ever wanted, is a separate concern and out
  of scope here.
- Q: When an unanchored target is later matched to an anchor and gains the
  anchored identifier, is the old random value discarded or kept? -> A: Kept as a
  superseded alias on the same entry, so any export or machine-facing reference
  that captured the old value still resolves to the merged entry. The entry's
  active identifier becomes the anchored one; the random value is retained as
  history, never reissued.
- Q: Does a bare-integer selector index into a stable ordering or an ephemeral
  one? -> A: Ephemeral. A bare integer is a row-index selector over the current
  listing and carries no durability guarantee across mutations; `--id <n>` is the
  durable, machine-facing contract. This is why a purely numeric handle is
  forbidden: it would collide with the row-index namespace.

### Session 2026-08-16 (operator)

- Q: How is the anchored identifier hashed -> A: With BLAKE3, truncated to 63
  bits, matching the handoff plan literally. The spec stays algorithm-agnostic
  ("a cryptographic hash"); the algorithm choice and its dependency are settled
  in the plan.
- Q: Issue #138 collapses the resolver from five provider positions to three;
  does the removal of the engine-layout and platform-walker providers happen in
  this slice -> A: No. This slice introduces the fidelity-ordered resolution over
  the two stores plus runtime promotion, and preserves the four hint declines as
  fidelity-aware query conditions. The engine-layout and platform-walker
  providers remain in the cascade for this slice and become `TargetSource`
  implementations that write entries in S052, which is where the provider list is
  reduced to exactly the three positions. Deferring the removal avoids a
  transitional window in which installed-but-unregistered titles stop resolving.

### Session 2026-08-16 (analyze remediation)

- Q: The scope retires `profile validate`; what becomes of `profile list` and
  `profile show`, whose subject was profile files -> A: The `profile` command is
  retired in full. Its subject (files) no longer exists; listing and showing
  targets is served by the `targets` command. This reconciles the spec with the
  plan and tasks, which already retire the whole command (FR-023).
- Q: `schema validate` is kept and exports now carry the entry fields; does the
  published schema describe the exported entry -> A: Yes. The exported entry is
  the JSON target document the published master schema describes, so an exported
  entry validates under `schema validate` and re-imports cleanly. The schema is
  extended with the entry fields; derived fields (handle, identifier) are
  optional on input because they are computed when absent. There is one document
  shape for export, import, and validation (P-10). A schema-version bump, if the
  extension requires one, is an implementation detail (FR-014).

### Session 2026-08-16 (implementation)

- Q: `run`/`tap`/`watch` select their capture target with `--profile`, which
  resolves a name through the file provider and the AppData profile directory.
  Retiring that directory and selector in this slice removes the only capture
  entry point before its replacement exists -> A: Split the retirement. This slice
  retires the standalone `profile` MANAGEMENT command (`validate`/`list`/`show`)
  and the `--profile-dir` management flag, because those stand alone and their
  subject is gone. It does NOT remove the `--profile` capture selector or the
  Profile file provider the capture commands resolve through: reworking the
  capture command surface is S054's explicit scope, and removing the selector
  before S054 provides its handle/`--id` replacement breaks capture with nothing
  in its place. This is the same transitional-window reasoning that keeps the
  engine and platform-walker providers until S052. FR-022 therefore lands
  partially in S051 (the management command and directory flag) and completes in
  S054 (the capture-path directory read and `--profile` selector). FR-023
  (retire the `profile` command, keep `schema validate`) lands in full here.
- Q: A local `targets` row resolves at its own fidelity; how is it turned into a
  resolver `Target` without a new origin -> A: Reuse
  `TargetOrigin::HintDatabase` stamped with the entry's own `FidelityTier`
  (`Target::new` takes the fidelity separately), rather than adding a new
  `TargetOrigin` variant to `fragcap-profile`. Minimal, and the fidelity carries
  the authored/verified distinction the origin would otherwise imply.

### Session 2026-08-16 (implementation deferrals, operator-approved)

Two of the five stories proved to be the leading edge of later slices once
implemented, and are deferred with the operator's approval. The durable core
(the entry model, handles, identity and merge, selectors, and fidelity-aware
resolution) ships in S051; the two deferred pieces are handed to the slices that
own them.

- Q: Should the JSON export/import of target entries and their validation under
  `schema validate` (US2, FR-014) ship in S051 -> A: No; defer to S055/S057. The
  `schema validate` master schema describes a profile document (game plus stages
  plus match rules), which is structurally different from a target entry (handle,
  stable identifier, classification, anchor). Unifying them under one document
  shape is the "profiles stop being documents" transition S055 (the `targets`
  hero command and interactive authoring) and S057 (the docs and getting-started
  rewrite) own. A separate entry-only export format would be a third document
  shape, which P-10 and the earlier one-shape clarification forbid. What ships in
  S051 is the load-bearing half of US2: the anchor-deterministic identifier, the
  supersede-and-alias transition, and merge-on-anchor, all at the store and CLI
  level and tested (SC-004). FR-014's JSON-export/import and schema-validate
  clauses complete in S055/S057.
- Q: Should the profile-file surface be retired in S051 (US5, FR-022, FR-023) ->
  A: No; defer the whole surface to S054. The `profile` command, the AppData
  profile directory, the file provider, and the `run`/`tap`/`watch` `--profile`
  capture selector are one coherent surface that S054's capture rework retires
  together. `--profile` is the only capture entry point, so removing any part in
  isolation either breaks capture or leaves an incoherent partial state (the
  command gone while the directory it read remains). The `targets` command added
  in S051 is the replacement surface; the old surface is retired when S054 wires
  its handle/`--id` replacement into capture. This supersedes the earlier
  "FR-023 lands in full here" note. FR-022 and FR-023 both complete in S054.

### Session 2026-08-17 (code review, PR #147)

- Q (P1): The anchored identifier is documented as a "63-bit BLAKE3 truncation",
  but the implementation reserved a locality bit, making it a 62-bit value that
  disagrees with anything computing the low 63 bits -> A: The anchored identifier
  is now the full low 63 bits of BLAKE3 over the canonical anchor (the durable,
  cross-implementation contract). The locality bit is removed; whether an entry is
  anchored is read from its `anchor` field, which always exists, so no bit is
  reserved. FR-012 and the identifier docs are revised to match.
- Q: A CLI-supplied anchor with a non-canonical prefix (`STEAM:620`) produced a
  different identifier and failed to resolve against `steam:620` -> A: The anchor
  is canonicalized before hashing and before storage: the platform prefix is
  lowercased and surrounding whitespace trimmed, so logically identical anchors
  produce identical identifiers (FR-010).
- Q: `targets show` returned exit 1 for every no-match, losing the selector kind
  -> A: The no-match exit code now follows the section 5.4 contract: a handle or
  name miss is a clean 0, an unknown `--id` or an out-of-range row index is a
  usage error (2). An ambiguous name still exits 2.

## User Scenarios & Testing *(mandatory)*

The actors are the user, who registers the games they want to capture and later
refers to them on the command line; ShruggieTech, whose shipped `catalog.db`
seeds heuristic knowledge about titles the user has not registered; and the tool
itself, which normalizes names into handles, assigns stable identities, and
resolves a selector to exactly one target or refuses to guess.

### User Story 1 - Register a target and refer to it by a human handle (Priority: P1)

Today a target is a loose JSON profile file in an AppData subdirectory: to
capture a game the user has to understand a file format, a directory search
order, and a schema before anything is captured. After this slice a target is a
row in `local.db` with a human-readable `handle` derived automatically from its
name, and the user refers to it by that handle (or by its name) on the command
line. Registering "Tom Clancy's(TM) The Division 2" yields the handle
`tom_clancys_the_division_2` with no file authored and no schema learned.

**Why this priority**: This is the reason the slice exists. Removing the file
format, the directory search order, and the up-front schema is what lets a user
capture a title by naming it, and every later v0.5.0 slice (discovery sources,
the `targets` hero command, detection signatures) writes into this same entry
model.

**Independent Test**: Register a handful of titles by name, confirm each gets the
expected handle from the Appendix A table, and confirm each is thereafter
selectable by that handle with no file on disk.

**Acceptance Scenarios**:

1. **Given** a title name, **When** a target is registered from it, **Then** a
   `local.db` row exists carrying a handle produced by the normalization
   algorithm, and no profile file is written anywhere.
2. **Given** every Appendix A input name, **When** its handle is computed, **Then**
   the handle equals the expected value in the table.
3. **Given** a registered target, **When** the user names its handle on the
   command line, **Then** that single target resolves.

---

### User Story 2 - Independent registrations of one title merge instead of duplicating (Priority: P1)

> **Partially deferred.** The identity and merge (anchor-deterministic
> identifier, supersede-and-alias, merge-on-anchor) ship in S051 and are tested.
> The JSON export/import of entry documents and their `schema validate`
> conformance are deferred to S055/S057 (see Clarifications, deferrals session).

A title can be registered more than once: the user authors it, a platform walker
finds it, and the shipped catalog already knows it. Without a stable identity
these produce three rows for one game. After this slice an anchored target (one
with a platform anchor such as `steam:2221490`) derives its identifier
deterministically from that anchor, so two independent registrations of the same
title compute the same identifier and merge into one entry rather than
duplicating. An unanchored target gets a random identifier that is replaced by
the anchored one, and retains the random value as a superseded alias, when it is
later matched to an anchor.

**Why this priority**: Duplicate entries for one game are the merge-and-precedence
problem the single store exists to prevent (P-10). Identity keyed on the anchor
is what makes registration idempotent across sources.

**Independent Test**: Construct two entries independently from the same Steam
anchor and confirm their identifiers are identical; construct one from a
different anchor and confirm it differs.

**Acceptance Scenarios**:

1. **Given** two entries built independently from the same anchor
   `steam:2221490`, **When** their identifiers are computed, **Then** the
   identifiers are identical.
2. **Given** two entries built from different anchors, **When** their identifiers
   are computed, **Then** the identifiers differ.
3. **Given** an unanchored entry with a random identifier, **When** it is later
   matched to an anchor, **Then** its active identifier becomes the anchored one
   and the former random value is retained as a superseded alias.
4. **Given** two entries that carry the same anchor, **When** they are stored,
   **Then** they collapse to one entry rather than two rows.

---

### User Story 3 - An ambiguous selector refuses to guess (Priority: P2)

The user selects a target by naming it. When a name matches more than one
registered target, the tool must not pick one: it prints the matches with their
handles and identifiers and exits non-zero, so the user disambiguates by handle
or `--id`. A selector must resolve to exactly one target or to none, never to a
silently chosen one.

**Why this priority**: Silently attributing a capture to the wrong one of two
same-named targets is the class of error P-9 forbids: a run that exits zero and
captures the wrong game. Refusing to guess is what keeps the instrument honest.

**Independent Test**: Register two targets that normalize to the same name,
select by that name, and confirm the tool lists both and exits non-zero without
capturing.

**Acceptance Scenarios**:

1. **Given** two targets sharing a case-insensitive name, **When** the user
   selects by that name, **Then** the tool prints both with handles and
   identifiers and exits with a non-zero status, resolving nothing.
2. **Given** a unique handle, **When** the user selects by it, **Then** exactly
   that target resolves.
3. **Given** an `--id <n>`, **When** the user selects by it, **Then** the target
   with that identifier resolves regardless of any name or handle collision.
4. **Given** a bare integer, **When** the user selects by it, **Then** it is
   treated as an ephemeral row index over the current listing, not as an
   identifier.

---

### User Story 4 - Resolution is ordered by fidelity, and the declines are preserved (Priority: P2)

Resolution over the two stores becomes fidelity-ordered, and the highest fidelity
wins: in `local.db`, among competing rows `authored` beats `verified` beats
`heuristic-unverified` beats `observed`; `catalog.db` always answers
`heuristic-unverified`; live runtime observation may promote a match to
`verified`. The four cases the store read correctly declines to answer (a sparse
catalog-only row, an engine-only row with no launch executable, a
launcher-mediated row, and a row naming more than one distinct Windows
executable) remain declines, now expressed as fidelity-aware query conditions
rather than incidental provider code, so the cascade continues past them.

The engine-layout and platform-walker providers stay in the cascade for this
slice; reducing the provider list to exactly the three positions
(`local.db`, `catalog.db`, runtime) completes in S052, when those providers
become sources that write entries into `local.db` rather than resolving at
request time.

**Why this priority**: This is where fidelity stops being a convention and
becomes a column the resolver reads (P-10), and preserving the declines is what
keeps the resolver from naming a launcher as the game or guessing among several
clients (P-9). It is P2 because it depends on the entry model of US1 and the
identity of US2 being in place.

**Independent Test**: Seed the same title into `local.db` and `catalog.db` at
different fidelities and confirm the highest-fidelity answer wins; feed each of
the four declined shapes and confirm none resolves from the store read.

**Acceptance Scenarios**:

1. **Given** one title present in `local.db` as `authored` and in `catalog.db` as
   `heuristic-unverified`, **When** it is resolved, **Then** the `local.db`
   `authored` entry wins.
2. **Given** competing `local.db` rows for one title at different fidelities,
   **When** it is resolved, **Then** the highest-fidelity row wins in the order
   `authored` > `verified` > `heuristic-unverified` > `observed`.
3. **Given** a sparse catalog-only row, an engine-only row with no launch
   executable, a launcher-mediated row, or a row naming more than one distinct
   Windows executable, **When** the store read is consulted, **Then** it declines
   and the cascade continues.
4. **Given** a live runtime observation of a title, **When** it is resolved,
   **Then** the match may be promoted to `verified`.

---

### User Story 5 - Profiles stop being files (Priority: P3)

> **Deferred to S054.** The profile-file surface (the `profile` command, the
> AppData directory, the file provider, and the `--profile` capture selector) is
> one coherent surface S054's capture rework retires together; removing any part
> in isolation breaks capture or leaves a partial state. The `targets` command
> added in S051 is its replacement. See Clarifications, deferrals session. The
> narrative below describes the eventual S054 end state.

The AppData profile directory and the `--profile <path>` file selector are
retired: a target is a row, not a file, and there is no directory to search and
no path to pass. The `profile validate` command is removed; `schema validate`,
which validates a target document against the published schema, is kept.

**Why this priority**: This is the user-visible cleanup that follows from the
entry model. It is P3 because the value is realized by US1 through US4; this story
removes the superseded surface so two ways to name a target do not coexist
(P-10).

**Independent Test**: Confirm `--profile` and `profile validate` no longer exist,
that no profile directory is created or read, and that `schema validate` still
runs.

**Acceptance Scenarios**:

1. **Given** the retired surface, **When** a user passes `--profile <path>` or
   runs any `profile` subcommand (`validate`, `list`, or `show`), **Then** the
   option or command is not recognized.
2. **Given** a fresh install, **When** the tool runs, **Then** no AppData profile
   directory is created or consulted.
3. **Given** a target document and the published schema, **When** the user runs
   `schema validate`, **Then** validation runs and reports conformance.

### Edge Cases

- **Purely numeric name** (`2048`): the normalized handle would be purely numeric,
  which is forbidden because it collides with the bare-integer row-index
  namespace; the handle falls back to the executable stem, then to `target_<n>`.
- **Whitespace-only or empty name**: normalization yields nothing usable; the
  handle falls back to the executable stem, then to `target_<n>`. Fallback never
  errors and never loops.
- **Handle collision** (`Portal 2` registered twice): the second registration
  auto-increments the *new* item to `portal_2_2` (then `_3`, ...), leaving the
  existing `portal_2` untouched.
- **Over-length name** (90 characters): the handle is truncated to 64 characters
  and any trailing underscore left by the cut is trimmed, so no handle ends in
  `_`.
- **User-supplied handle override**: accepted only if it satisfies the same
  validity rules (unique, not purely numeric, normalized shape); an override that
  collides auto-increments like any other collision.
- **Symbol-only decoration** (trademark, registered, degree, fraction, roman
  numeral): decorative symbol and format characters are stripped or decomposed
  before handle formation so `Rock Band 360(deg)` yields `rock_band_360` and
  `1/2 Life` yields `1_2_life`.
- **Name matching zero targets**: resolution returns no match (distinct from the
  ambiguous case, which lists matches and exits non-zero).
- **Anchor absent then acquired**: an entry created unanchored and later matched
  to an anchor adopts the anchored identifier and keeps its former random value
  as a superseded alias.

## Requirements *(mandatory)*

### Functional Requirements

**Target entry**

- **FR-001**: The system MUST store each target as a single entry in `local.db`
  carrying: a numeric primary key; a unique text handle; a display name; a
  classification drawn from the closed set {game, launcher, tool, mod, emulator,
  unknown}; a classification source drawn from {catalog, engine-signature,
  platform, user, unset}; a fidelity drawn from {authored, verified,
  heuristic-unverified, observed}; a provenance record; a nullable anchor; the
  launch entries carried whole; a nullable install root; and an evidence record.
- **FR-002**: The classification MUST be an enum that includes `unknown` as a
  first-class value, because "unknown" is a frequent, real state and forcing a
  binary guess is the kind of fabricated certainty P-9 forbids.
- **FR-003**: The fidelity value MUST be constrained to its four-value set at the
  storage layer (a rejected value is a storage error, not a silently accepted
  one), so that fidelity is enforceable rather than conventional.

**Handle normalization**

- **FR-004**: The system MUST derive a handle from a name by applying these steps
  in exactly this order: strip Unicode symbol-other, symbol-modifier, and format
  characters; apply compatibility decomposition (NFKD); strip combining marks;
  lowercase; delete apostrophes and quotation marks outright; replace each run of
  characters outside `[a-z0-9]` with a single underscore; trim leading and
  trailing underscores; truncate to 64 characters and then trim any trailing
  underscore.
- **FR-005**: Every handle MUST be unique within `local.db`.
- **FR-006**: A handle MUST NOT be purely numeric; a name that would normalize to
  a purely numeric handle falls back to the handle-fallback chain (FR-007).
- **FR-007**: When normalization yields an empty or invalid handle, the system
  MUST fall back to the executable stem, and if that too is unusable to
  `target_<n>`. The fallback MUST always terminate: it never errors and never
  loops.
- **FR-008**: On a handle collision, the system MUST auto-increment the *new*
  item by appending `_2`, then `_3`, and so on, and MUST leave the pre-existing
  entry's handle unchanged.
- **FR-009**: The system MUST let a user override a handle, subject to the same
  validity rules as a derived handle (unique, not purely numeric, normalized
  shape, collision auto-increment).

**Stable identifier**

- **FR-010**: An anchored target MUST receive a 63-bit identifier computed as a
  truncation of a cryptographic hash over a canonical anchor string, where the
  canonical form is the platform-prefixed anchor (for example `steam:2221490`,
  `epic:<catalogItemId>`, `gog:<productId>`) with the platform prefix lowercased
  and surrounding whitespace trimmed, so a non-canonical input such as `STEAM:620`
  resolves to the same identifier. The computation MUST be deterministic so that
  independent registrations of one title collide on identity and merge.
- **FR-011**: The identifier MUST derive only from the anchor, never from the
  name, the handle, or the install path.
- **FR-012**: An unanchored target MUST receive a random 63-bit identifier.
  Whether an entry is anchored is read from its `anchor` field, not from a bit of
  the identifier, so no bit is reserved and an anchored identifier is the full
  63-bit truncation of FR-010. (Revised from a "designated locality bit" during
  code review; see the P1 remediation in the code-review clarifications.)
- **FR-013**: When an unanchored target is later matched to an anchor, it MUST
  adopt the anchored identifier as its active identifier and MUST retain the
  former random value as a superseded alias; the superseded value is never
  reissued.
- **FR-014**: The stable identifier MUST be the merge key: two registrations of
  one anchor MUST collapse to one entry, and a superseded identifier MUST remain
  resolvable as an alias. (Delivered in S051 at the store and CLI level.) The JSON
  export/import of entry documents and their validation under `schema validate`
  are DEFERRED to S055/S057, where the profile and entry document shapes are
  unified into the one shape `schema validate` describes; a separate entry-only
  export format is explicitly not introduced in S051 (it would be a third document
  shape, against P-10). See Clarifications, deferrals session.

**Selector resolution**

- **FR-015**: The system MUST resolve a selector by these forms: a bare integer is
  an ephemeral row index over the current listing; a token is matched as an exact
  handle, then as a case-insensitive exact name; and `--id <n>` selects by
  identifier as an explicit, durable, machine-facing contract.
- **FR-016**: For a non-`--id` selector the resolution order MUST be exact handle
  first, then case-insensitive exact name.
- **FR-017**: When a name selector matches more than one entry, the system MUST
  print the matching entries with their handles and identifiers and exit with a
  non-zero status, resolving nothing; it MUST NOT choose one.
- **FR-018**: When a selector matches zero entries, the system MUST report no
  match, distinctly from the ambiguous case.

**Resolution cascade**

- **FR-019**: The system MUST resolve over the two stores and runtime in fidelity
  order: `local.db` first (highest-fidelity row wins), then `catalog.db` (always
  `heuristic-unverified`), with live runtime observation able to promote a match
  to `verified`. Reducing the resolver to exactly these three provider positions
  by removing the engine-layout and platform-walker providers is deferred to S052
  (they become sources then); this slice keeps them in the cascade.
- **FR-020**: Within and across the store reads the highest fidelity MUST win,
  ordered `authored` > `verified` > `heuristic-unverified` > `observed`. A
  `catalog.db` answer is always `heuristic-unverified`; a live runtime
  observation MAY promote a match to `verified`.
- **FR-021**: The system MUST preserve, as fidelity-aware query conditions, the
  four cases the prior provider correctly declined: a sparse catalog-only row, an
  engine-only row with no launch executable, a launcher-mediated row, and a row
  naming more than one distinct Windows executable. Each MUST remain a decline so
  the cascade continues to the next position rather than resolving a launcher or
  guessing among clients.

**Retirement (deferred to S054)**

The profile-file surface (the `profile` command, the AppData profile directory,
the file provider, and the `run`/`tap`/`watch` `--profile` capture selector) is
one coherent surface that S054's capture rework retires as a unit. It is NOT
retired in S051: `--profile` is the only capture entry point, so removing any part
in isolation either breaks capture or leaves an incoherent partial state. The
`targets` command added in S051 is the replacement surface. See Clarifications,
deferrals session.

- **FR-022**: (Deferred to S054.) After S054 the AppData profile directory is not
  created, searched, or read, and `--profile <path>` is not a recognized option.
- **FR-023**: (Deferred to S054.) After S054 the `profile` command is retired and
  `schema validate` is kept under the separate `schema` command.
- **FR-024**: Retirement MUST NOT auto-migrate existing loose profile files into
  `local.db`; this slice ships no migration path and no deprecation shim.

### Key Entities

- **Target entry**: the single stored representation of one capture target, held
  as a row in `local.db`. Carries identity (identifier, handle, name),
  classification and its source, fidelity, provenance, an optional anchor, the
  launch entries, an optional install root, and evidence.
- **Handle**: a unique, human-readable, URL-safe-ish slug derived deterministically
  from a name by the normalization algorithm; the primary human selector for a
  target. Never purely numeric.
- **Anchor**: a platform-scoped title reference (Steam app id, Epic catalog item
  id, GOG product id) rendered as a canonical prefixed string; the sole input to
  an anchored identifier.
- **Stable identifier**: a 63-bit value identifying a target across registrations
  and exports; the low 63 bits of the anchor hash when anchored, random when not,
  and mergeable on import.
- **Fidelity**: the confidence stamp on a resolution, ordered `authored` >
  `verified` > `heuristic-unverified` > `observed`; the column the resolver reads
  to choose among competing answers.
- **Classification**: what a target is (game, launcher, tool, mod, emulator,
  unknown), paired with the source that assigned it.
- **Selector**: a user-supplied reference resolving to at most one target: a
  bare-integer row index, a handle, a case-insensitive name, or an explicit
  `--id`.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Every Appendix A handle test vector produces its expected handle,
  verified as a unit test (13 vectors, including the fallback and truncation
  cases).
- **SC-002**: A name that normalizes to a purely numeric handle is rejected and
  falls back, so no stored handle is purely numeric.
- **SC-003**: Registering a colliding name a second time produces a `_2`-suffixed
  handle on the new item and leaves the existing entry's handle byte-identical.
- **SC-004**: Two entries constructed independently from the same Steam anchor
  produce identical identifiers; two from different anchors produce different
  identifiers.
- **SC-005**: Store resolution is fidelity-ordered (`authored` > `verified` >
  `heuristic-unverified` > `observed`, `catalog.db` always
  `heuristic-unverified`, runtime may promote to `verified`), and each of the
  four preserved declines is exercised by a test that confirms the store read
  declines and the cascade continues.
- **SC-006**: A name selector that matches more than one entry resolves nothing,
  lists every match with handle and identifier, and exits non-zero; a unique
  handle or an `--id` resolves exactly one.
- **SC-007**: (Deferred to S054, with FR-022/FR-023.) After S054 no profile
  directory is created or read, `--profile` and `profile validate` are
  unrecognized, and `schema validate` still runs.

## Assumptions

- Retiring the profile file path means dropping it, not migrating it: existing
  loose profile files are neither read nor auto-imported, consistent with the
  v0.5.0 no-deprecation-shims stance established in S050.
- An unanchored identifier reserves no bit: an anchored identifier is the full
  low-63-bit truncation of the anchor hash (the durable contract), and whether an
  entry is anchored is read from its `anchor` field. (Revised from an earlier
  "locality bit" during code review; see the code-review clarifications.)
- The canonical anchor strings for platforms beyond Steam (`epic:<catalogItemId>`,
  `gog:<productId>`) are specified now for forward compatibility even though
  Steam is the only platform a source populates today; the identifier scheme must
  be stable before a second platform arrives.
- "Distinct Windows executable" counts case-insensitively and preserves first-seen
  casing, matching the existing decline logic being carried forward.
- The published schema referenced by `schema validate` is the master target
  schema introduced in earlier slices; its role (validate a JSON target document)
  is unchanged, but this slice extends it with the entry fields so the one
  document shape covers export, import, and validation (FR-014). A schema-version
  bump, if the extension requires one, is settled in implementation.
- A bare-integer row index is ephemeral by design and carries no cross-invocation
  durability; `--id` is the only durable machine-facing selector.
