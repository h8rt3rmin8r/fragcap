# Tasks: Profile Schema, Parsing, and Validation

**Slice**: S05

**Branch**: `feat/profile-schema-validation`

**Created**: 2026-08-09

**Input**: [spec.md](spec.md), [plan.md](plan.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/profile-schema.md](contracts/profile-schema.md),
[quickstart.md](quickstart.md)

Tests are included and are not optional. The deliverable of this slice is a set
of refusals, and an untested refusal is a refusal nobody has seen happen.

Four notes on the shape.

**Phases 2 and 3 are foundational and independent of everything else.** The
duration grammar and the glob intersection decision are pure functions over
strings with no profile, no diagnostics, and no filesystem involved. Both are
also the two places in the slice where the logic is intricate enough to get
quietly wrong, so both come first and both are table-driven.

**Phase 4 precedes the schema.** Diagnostics are the shape every later phase
pushes into, and the ordering and location invariants are properties of the set
rather than of any individual check. Building the container before its contents
means the accumulation discipline is in place before the first `?` operator has
a chance to be reached for.

**Phase 6 and Phase 7 are split on the structural and semantic boundary section
15.4 itself draws.** The draft type that passes between them is what makes
accumulation possible, and separating them keeps each phase's tests able to fail
for one reason.

**Phase 10 is the phase that justifies the slice.** Everything before it proves
a check works in isolation. Phase 10 parses the two profiles the architecture of
record actually publishes and asserts every field, which is the check that the
schema agrees with the document it claims to implement.

## Phase 1: Setup

- [X] T001 Add `toml-span = "0.7"` and `regex = { version = "1.13",
  default-features = false, features = ["std", "unicode"] }` to
  `[workspace.dependencies]` in `Cargo.toml`, each with a comment naming the
  slice and the research entry that justifies it, matching the existing comment
  style for `bytes` and `serde_json`
- [X] T002 Add both as `workspace = true` dependencies in
  `crates/fragcap-profile/Cargo.toml`. Confirm with `cargo tree -e normal -p
  fragcap-profile` that the graph is exactly the one
  [quickstart.md](quickstart.md) predicts, and that neither `aho-corasick` nor
  `serde` appears. The crate takes no sibling dependency (FR-001), and this
  parser choice is what FR-002 is satisfied by
- [X] T003 Create `crates/fragcap-profile/src/{diagnostic,schema,parse,validate,
  glob,resolve}.rs` with SPDX headers and module documentation naming the
  specification sections each implements, and declare them in
  `crates/fragcap-profile/src/lib.rs`
- [X] T004 Write the `fragcap-profile` crate documentation in
  `crates/fragcap-profile/src/lib.rs`: what a profile is, that a `Profile`
  cannot exist unvalidated, that nothing here observes a process (FR-055), and
  why the two dependencies are here when the pcap, pcapng, and JSON Lines
  handling were hand-rolled
- [X] T005 Create `crates/fragcap-core/src/duration.rs` with an SPDX header and
  module documentation naming specification section 25.2 and plan decision D-6,
  and declare `pub mod duration;` in `crates/fragcap-core/src/lib.rs` (FR-042)
- [X] T006 Extend the `fragcap-core` crate documentation to name `duration` as
  the third module that is behavior rather than vocabulary, with the reason it
  is here rather than in `fragcap-profile` (three consumers, no sibling edge)

## Phase 2: The duration grammar

Foundational. Nothing in it depends on anything else in the slice.

- [X] T007 Write the duration tests first in
  `crates/fragcap-core/src/duration.rs`, table-driven over accepted and refused
  literals: `500ms`, `30s`, `30m`, `2h` accepted; `30`, `30 m`, `1.5h`, `-5m`,
  `0s`, `1h30m`, `30d`, empty, `m`, and an overflowing value refused. Per FR-043
  through FR-046 and the contract's grammar, and SC-008
- [X] T008 Define `DurationError` with one variant per refusal reason, so a
  caller can distinguish a missing unit from an unknown one, and implement
  `Display` and `std::error::Error`
- [X] T009 Implement the parser returning `Result<std::time::Duration,
  DurationError>`. Reject zero (FR-045). Use checked arithmetic and refuse on
  overflow rather than saturating (FR-046)
- [X] T010 Confirm `cargo xtask deps` still passes, which is the mechanical
  proof that the new core module brought no dependency with it (FR-054)

## Phase 3: The glob syntax and the ambiguity decision

Foundational and self-contained. This is the most intricate code in the slice.

- [X] T011 Write the matcher tests first in
  `crates/fragcap-profile/src/glob.rs`: literals, `?`, `*` at each position,
  multiple `*`, `*` matching empty, and case differences, per the contract's
  glob syntax
- [X] T012 Write the intersection tests first, table-driven over pattern pairs.
  Intersecting: identical patterns, `*Launcher.exe` against `ESOLauncher.exe`,
  `*` against anything, `?so64.exe` against `eso64.exe`, and pairs differing
  only in case. Disjoint: `a*.exe` against `b*.exe`, patterns sharing a prefix
  but not a whole name, patterns sharing a suffix but not a whole name, and
  `?.exe` against `ab.exe`. Per FR-029 and SC-004
- [X] T013 Define `ImagePattern` holding the author's text verbatim, with a
  constructor that refuses the empty pattern (FR-020) and no normalization of
  what it stores (FR-009)
- [X] T014 Implement the reachability walk that decides whether two patterns
  have a non-empty intersection, per research R-2. Case folding is applied to
  copies at comparison time
- [X] T015 Implement `matches` for one image name as the same walk against a
  literal, so there is one implementation of the syntax rather than two
- [X] T016 Add a test asserting `matches` and the intersection decision agree:
  for every pattern pair in the table and a set of concrete names, a name
  matching both patterns implies the pair intersects. This is the property that
  would catch the two walks drifting apart if they are ever separated

## Phase 4: Diagnostics

- [X] T017 Define `DiagnosticCode` in `crates/fragcap-profile/src/diagnostic.rs`
  with every variant from [data-model.md](data-model.md), non-exhaustive to
  callers, and document that it is the stable surface while `location` and
  `message` are not (FR-050a)
- [X] T018 Define `Position` and the byte-offset-to-line-and-column conversion,
  one-based, in this module and nowhere else (plan D-4). Test it against offsets
  at a line start, mid-line, at a line end, on the first line, and past a
  multi-byte character
- [X] T019 Define `Diagnostic` with code, location, offset, position, and
  message per FR-047 and FR-048
- [X] T020 Define `Diagnostics` as an ordered set, sorted by offset then code
  (FR-049), with an invariant that it is never empty when returned as an error
  (FR-050). Test that building the same set from a different insertion order
  yields the same sequence
- [X] T021 Implement `Display` for `Diagnostics` producing one line per
  diagnostic, and a test asserting byte-identical output across two runs
  (SC-009)

## Phase 5: The schema types

- [X] T022 Define `GameId` in `crates/fragcap-profile/src/schema.rs` with a
  fallible constructor enforcing the slug charset (FR-025), and test the
  accepted charset alongside refusals for empty, uppercase, a path separator, a
  parent reference, and a drive prefix
- [X] T023 Define `Lifecycle` and `CaptureMode` as closed enumerations with
  parsing from their TOML spellings (FR-023)
- [X] T024 Define `PathRegex` holding the source text and the compiled
  expression, with `Debug` and `PartialEq` defined on the source (per
  [data-model.md](data-model.md))
- [X] T025 Define `MatchPredicates` with the five section 10.3 predicates and
  the `pinned()` method the ambiguity check consumes (FR-007)
- [X] T026 Define `Stage`, `Game`, `CaptureDefaults`, and `Profile` with private
  fields, accessor methods, no public constructor, and no `Default` (FR-011).
  Every capture field is an `Option` carrying no substituted default (FR-004),
  and `duration` holds the typed span with no literal text retained beside it
  (FR-046a)

## Phase 6: Parsing and structural checks

- [X] T027 Define the private draft types in
  `crates/fragcap-profile/src/parse.rs`: the same shape as the schema types with
  every field optional, so that a fault in one field does not prevent checking
  another (research R-4)
- [X] T028 Implement the spanned-document walk producing the draft, pushing a
  diagnostic per fault and never returning early. No `?` on a diagnostic path
- [X] T029 Implement the closed key sets for each of the five tables, emitting
  `UnknownKey` naming the key and the accepted set (FR-010), which is where the
  five accepted `[capture]` keys (FR-005) and the stage key set (FR-006) are
  enforced. Test one unknown key per table
- [X] T030 Implement `MissingField` for every required key and `WrongType` for
  every type mismatch, each carrying its dotted location (FR-013, FR-014). Test
  a profile missing several required fields and assert all of them are reported
- [X] T031 Implement the schema version check: exactly 1 accepted, any other
  integer yielding `UnsupportedSchema` and suppressing the semantic set
  (FR-012). Test a lower, higher, missing, and non-integer value
- [X] T032 Implement the TOML syntax path: one `Syntax` diagnostic carrying the
  parser's span, suppressing everything else (FR-015). Test a malformed document
  and a duplicate key, which `toml-span` refuses for us
- [X] T033 Implement `Profile::parse` as the only public entry, returning
  `Result<Profile, Diagnostics>` (FR-011)
- [X] T033a Add the value-form tests the analyze gate asked for: every form a
  schema version 1 profile can contain, including a Windows path as a literal
  string with backslashes preserved (FR-002, FR-002a, SC-019)
- [X] T033b Pin the known datetime divergence with a test asserting the observed
  refusal, and reference research R-1 in a comment so the next reader finds the
  decision rather than a surprise (FR-002, SC-019)

## Phase 7: Semantic checks

Each task is one check with its own test. Every check must be reachable
independently, because the multiple-fault test in Phase 10 asserts they
accumulate.

- [X] T034 Role name uniqueness, naming both stages (FR-016)
- [X] T035 At most one terminal stage (FR-017)
- [X] T036 `terminal` implies lifecycle `session`, per FR-026 and the transient
  hand-off reasoning in the spec's edge cases
- [X] T037 `descends_from` names a declared role (FR-018)
- [X] T038 `descends_from` is acyclic, naming every role in a cycle, including
  the self-reference case (FR-028)
- [X] T039 At least one stage is not a service (FR-022), and at least one stage
  exists, which is the `NoStages` code and FR-003's closing clause
- [X] T040 Every `match` table carries at least one predicate (FR-024)
- [X] T041 `capture.roles` is non-empty when present and every entry is declared
  by a stage (FR-027)
- [X] T042 Every `path_regex` compiles, with the engine's message carried into
  the diagnostic (FR-019). Include the pathological-pattern case and assert the
  diagnostic is `InvalidRegex` rather than a special case (FR-046e, SC-016)
- [X] T043 Every duration literal parses, delegating to the core grammar
  (FR-021)
- [X] T044 Wire the ambiguity check: for every unordered stage pair whose `exe`
  patterns intersect, refuse unless both stages are pinned (FR-030, FR-031). The
  diagnostic names both roles and the remedy (FR-032)
- [X] T045 Add the section 5.4 regression test: two stages on `exe` alone that
  can match one name are refused, and the section 15.2 three-stage profile is
  accepted (SC-005)

## Phase 8: Loading

- [X] T046 Define `LoadError` with `TooLarge`, `Read`, and `Invalid` variants,
  implementing `Display` and `std::error::Error`
- [X] T047 Implement `load` reading the file's metadata and refusing above one
  mebibyte before reading the contents (FR-046b). Test with a file over the
  limit and assert the refusal names the limit; the test must be written so that
  it would fail if the contents were read first (SC-015)
- [X] T048 Implement the read path and the delegation to `Profile::parse`,
  mapping a diagnostic set into `LoadError::Invalid`

## Phase 9: Resolution

- [X] T049 Define `SearchPath`, `BundledSet`, `ProfileSource`, `Resolved`, and
  `ResolveError` in `crates/fragcap-profile/src/resolve.rs` per
  [data-model.md](data-model.md). `BundledSet`'s constructor refuses a duplicate
  `game.id`, naming both (FR-041)
- [X] T050 Implement step one: a reference naming an existing regular file
  resolves to it, with no slug check (FR-035). A directory does not satisfy it
  (FR-046c)
- [X] T051 Implement the slug gate for steps two through four, refusing before
  any path is joined (FR-036). The test asserts `InvalidReference` rather than
  `NotFound`, which is what distinguishes a check from a failed open (SC-007)
- [X] T052 Implement steps two and three over the caller-supplied directories,
  first match winning, skipping a directory that is absent or unreadable
  (FR-033, FR-037). Nothing here reads an environment variable or a platform
  configuration location (FR-034)
- [X] T053 Implement step four against the bundled set, matching on `game.id`
- [X] T054 Implement the present-but-unreadable case as an error naming the
  path, never a fall-through (FR-038)
- [X] T055 Implement `ProfileSource` reporting on success (FR-039) and
  `NotFound` carrying every searched location on failure (FR-040)

## Phase 10: Integration tests

- [X] T056 `crates/fragcap-profile/tests/examples.rs`: parse the section 15.2
  single-title profile and assert every field, including that `platform` and
  `app_id` are present and that absent optional fields report absence rather
  than a default (SC-001, FR-004)
- [X] T057 Parse the section 15.2 three-stage profile and assert the client
  stage carries both `exe` and `descends_from`, and that stage order is the
  declaration order (SC-001, FR-008)
- [X] T058 Assert no field was normalized: compare the stored `exe` pattern,
  role, and name against the file's text exactly (FR-009)
- [X] T059 `crates/fragcap-profile/tests/diagnostics.rs`: a profile carrying at
  least four distinct faults yields all four, asserted by code and location
  (SC-002)
- [X] T060 Assert every `DiagnosticCode` variant is produced by at least one
  test in this file, with a comment naming which case produces each (SC-003)
- [X] T061 `crates/fragcap-profile/tests/resolution.rs`: exercise all four steps
  and both shadowing cases against directories built under
  `CARGO_TARGET_TMPDIR`, asserting on `ProfileSource` rather than only on
  contents (SC-006)
- [X] T062 Add the traversal cases: a reference with a path separator and a bare
  name that is also an existing directory (SC-007, SC-017)

## Phase 11: Documentation and house rules

- [X] T063 Add seven glossary entries to `docs/glossary.md` in the appropriate
  categories: profile schema version, lifecycle class, terminal stage, match
  predicate, profile resolution order, ambiguous image match, duration literal.
  Follow the section 4.3 entry structure with a `{: .matters }` note and
  cross-references (FR-051, SC-014, P-6)
- [X] T064 Add cross-references to the existing `Stage` and `Game profile`
  entries, and note on `Game profile` that the file is versioned (FR-051)
- [X] T065 Write `changelog.d/S05-profile-schema-validation.added.md` describing
  the schema, the validation set, and the resolver
- [X] T066 Write `changelog.d/S05-profile-schema-validation.decisions.md` dated
  2026-08-09 recording: the two runtime dependencies with the measurements that
  chose them, the duration grammar's placement in `fragcap-core`, the three
  checks added beyond section 15.4 as candidates for promotion, and the
  caller-supplied search path. Each dependency is recorded with its license and
  its reason (FR-053, SC-012), and the plan already carries the transitive
  count, the rejected alternative, and the ambiguity check's complexity bound
  (SC-018)
- [X] T067 Update `AGENTS.md`: the current-state paragraph, the dependency
  inventory table (three runtime dependencies now, with the reason each was
  taken), and the note that S05 is no longer outstanding

## Phase 12: Verification

- [X] T068 Run `cargo xtask ci` in the foreground and watch it to completion.
  Formatting, Clippy with warnings denied, the workspace tests, the conventions
  linter, the dependency direction check, and the license check (SC-010). No
  test in the slice may require a capture driver, elevated privilege, a game, or
  a network interface, which is SC-013 and is a property of what was written
  rather than a step to perform
- [X] T069 Run `cargo xtask msrv` and watch it. This is the check that the
  dependency selection was made for; a failure here means a direct or transitive
  crate raised its declared minimum above 1.82
- [X] T070 Run `cargo xtask neutral` and watch it. The proof that the duration
  module added no platform surface to `fragcap-core`, which with T010 is SC-011
- [X] T071 Run `cargo tree -e normal -p fragcap-profile` and compare against the
  graph [quickstart.md](quickstart.md) predicts, so that a transitive addition
  is noticed here rather than at the next release

## Dependencies between phases

```text
Phase 1 (setup)
  ├── Phase 2 (duration)      independent
  ├── Phase 3 (glob)          independent
  └── Phase 4 (diagnostics)
        └── Phase 5 (schema types)   needs Phase 2 and Phase 3
              └── Phase 6 (parse, structural)
                    └── Phase 7 (semantic)   needs Phase 3 for T044
                          ├── Phase 8 (loading)
                          │     └── Phase 9 (resolution)
                          └── Phase 10 (integration tests)
                                └── Phase 11 (documentation)
                                      └── Phase 12 (verification)
```

Phases 2, 3, and 4 can be done in any order. Everything from Phase 5 is a chain.

## What would make this slice fail review

Worth stating, because each of these passes its own tests.

- A `?` operator on a diagnostic path, which turns "report every problem" into
  "report the first problem" while every individual check still works.
- A conservative ambiguity check, which passes every disjoint case and admits
  the silent empty capture.
- A slug check performed after the path join, which passes today because nothing
  is at the traversal target.
- A capture default that substitutes a value for absence, which destroys the
  distinction S14 needs.
- A normalized `exe` pattern, which is the natural convenience and a P-9
  violation.
- A dependency appearing in the manifest without appearing in the decisions
  fragment.
- A logging call anywhere in the crate, which FR-052 forbids and which no task
  adds, so its absence has to be preserved rather than achieved.
- A symbolic link policy, which FR-046d deliberately declines. There is no task
  because the requirement is to add nothing; a link check appearing in review is
  the violation.
