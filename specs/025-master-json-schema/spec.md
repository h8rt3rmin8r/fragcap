# Feature Specification: Master JSON Schema for Targeting and Attribution

**Feature Branch**: `feat/master-json-schema`

**Created**: 2026-08-12

**Status**: Draft

**Slice**: Post-S24 keystone (GitHub issue #75). Rewrites the format of master
specification section 15 from TOML to JSON and generalizes it beyond the profile
to every machine-readable targeting/attribution artifact. Constitution
principles in play: the honesty posture that forbids presenting a guess as a
fact (the same posture behind `Fidelity::Retained`), the section 15.4 rule that
validation reports every problem at once, and the deliberate refusal of unknown
keys.

**Input**: Build the single, versioned master JSON Schema that governs every
machine-readable target/attribution artifact fragcap produces or consumes,
reused across as much of the targeting and attribution system as possible rather
than re-specified per subsystem. It is the foundational keystone that the
TOML-to-JSON migration (#76), the target resolution cascade (#77), and the
targets hint database (#78) all depend on. Scoping/spec task only.

## Overview

Everything downstream in the targeting redesign speaks the same data. A profile
an author writes, a hint the shipped database offers, a package a contributor
submits, and a row the database exports are all the same kind of thing: a
statement about how to recognize a game's processes and how much to trust that
statement. Today that vocabulary exists in exactly one form (the TOML profile of
section 15) and is about to fork into four subsystems. If each subsystem
re-describes the shape independently, they drift, and drift here is not cosmetic:
a hint that validates in the database but not in the resolver is a capture that
silently targets nothing.

This slice defines one master JSON Schema so the four forms cannot drift, because
they are the same schema. It is composable: a small shared core carries the
process-recognition vocabulary, and each artifact form is a variant of that core
that tightens or loosens which fields are required. A change to the core
propagates to all four consumers by construction rather than by discipline.

Three properties carry the slice.

**Trust is data, not prose.** Every artifact form carries a structured
`fidelity` field with an ordered set of tiers, and the resolver reads it. A
heuristic guess from the database is stamped as a guess; a file the user authored
is stamped as authoritative; a target confirmed against a live capture is stamped
as observed. This is the section-15-era honesty rule generalized: the tool never
lets a guess wear the clothes of a fact. A free-form comment could not do this,
because the machine cannot act on a comment.

**Every problem is reported at once.** This is inherited directly from section
15.4 and is non-negotiable. A file with four schema violations reports four, in
one run, not one violation per edit-run cycle. The population writing and
submitting these files is not the population that can debug a validator, and the
authoring loop is the whole point of shipping a schema they can validate against.

**Structural and semantic validation are separated on purpose, and the seam is
declared.** The schema enforces what a schema can enforce: field types, required
keys, permitted enum values, string shapes. It cannot express the invariants that
make a profile actually work (an acyclic ancestry graph, at most one terminal
stage, every declared role reachable, no ambiguous image match). Those remain
semantic checks the profile-load path owns. This slice is precise about which
layer owns which class of error, so a later reader does not expect the schema to
catch a cycle it structurally cannot see.

The slice stops at the schema and the generic validation surface. Migrating the
profile parser onto JSON is #76, the resolver that reads `fidelity` is #77, and
the database whose export must conform is #78. What this slice owes all three is a
single, published, versioned vocabulary they build on rather than around.

## Clarifications

### Session 2026-08-12

Resolved under autopilot from the spec, the constitution (P-1 through P-9), the
architecture of record, and the slice scope. Recorded here because each
materially shapes the plan, the schema shape, or the dependency justification.

- Q: How does a file declare which of the four artifact forms it is? -> A: An
  explicit top-level `kind` discriminator field, a closed enum
  (`profile`, `hint`, `package`, `export`), driving conditional schema
  selection. Chosen over overloading `fidelity` because an explicit discriminator
  is unambiguous for a validator and for an agent generating a file, and because
  `package` and `profile` share a structural shape that fidelity alone would not
  separate cleanly.
- Q: Which JSON Schema dialect does the master schema target? -> A: Draft
  2020-12. It is the current standard, expresses the conditional
  (`kind`-discriminated) structure cleanly, and is supported by the leading
  Rust validator candidates. Final subject to the MSRV 1.82 gate in planning.
- Q: Which validator crate is the leading candidate? -> A: `boon` (pure Rust, no
  C dependency, small transitive graph, Draft 2020-12 support, collects all
  errors in one pass), with `jsonschema` as the fallback. Final choice and the
  full AGENTS.md dependency justification (MSRV at 1.82, license across the
  graph, graph size, alternatives) is produced in planning research; the slice
  does not proceed to implementation until MSRV 1.82 is confirmed green.
- Q: Which CLI surface does this slice deliver? -> A: Both `fragcap schema
  validate <file>` (structural validation, all errors at once) and `fragcap
  schema print` (emit the embedded schema). Print is required to satisfy
  emit-on-demand and the mechanical drift check, so it is in scope, not deferred.
- Q: Where do the schema and validator live in the crate graph? -> A: The schema
  and the validator dependency live in `fragcap-profile` (which already owns the
  profile schema and validation); the `schema` CLI subcommands live in
  `fragcap-cli`. This keeps `fragcap-core` free of the new dependency, honoring
  P-2 (core stays platform-neutral), since `fragcap-profile` is not core.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Validate any target file and see every mistake at once (Priority: P1)

A profile author, a contributor preparing a submission, or an AI agent
generating a target file runs a one-off validation of a JSON file against
fragcap's schema and receives a complete, precise list of everything wrong with
it in a single pass.

**Why this priority**: This is the minimum viable product. A shipped, versioned
schema is only useful if someone can check a file against it without running a
capture, and the all-errors-at-once behavior is the property that makes the
authoring loop bearable. Ship only this and fragcap already gives every author
and agent a fast, honest feedback loop.

**Independent Test**: Take a JSON file with several deliberate faults (a wrong
type, a missing required field, an unknown key, an out-of-range enum value) and
confirm the validator reports all of them in one invocation, naming each with its
location, and exits non-zero. A clean file reports nothing and exits zero.

**Acceptance Scenarios**:

1. **Given** a JSON file with four independent schema violations, **When** the
   file is validated against the master schema, **Then** all four violations are
   reported in one run, each located precisely, and the run exits non-zero.
2. **Given** a JSON file that conforms to the schema, **When** it is validated,
   **Then** no violations are reported and the run exits zero.
3. **Given** a JSON file containing a key the schema does not define, **When** it
   is validated, **Then** the unknown key is reported as a violation rather than
   silently ignored.
4. **Given** a file that is not syntactically valid JSON, **When** it is
   validated, **Then** the syntax error is reported clearly and distinguished
   from a schema violation.

---

### User Story 2 - One vocabulary across all four artifact forms (Priority: P2)

The same schema describes a full profile, a partial hint record, a user-authored
target package, and a database JSON export, so an artifact that is valid in one
subsystem is valid in the resolver, and a change to the shared core reaches all
of them at once.

**Why this priority**: This is the reason the schema is a keystone rather than a
convenience. Without the shared core, the four downstream subsystems drift and
the drift is invisible until a capture targets nothing. It builds on P1 (the
validation surface already exists) and makes that surface authoritative
everywhere.

**Independent Test**: Take a representative file of each of the four forms and
confirm each validates against the single master schema, that a hint record is
accepted with fewer required fields than a full profile but rejected if it omits
its fidelity or provenance, and that a database export of a hint round-trips to
JSON and validates with no manual adjustment.

**Acceptance Scenarios**:

1. **Given** a full profile, a hint record, a user package, and a database
   export, **When** each is validated against the master schema, **Then** each is
   accepted by the appropriate variant of the shared core.
2. **Given** a hint record that omits a field a full profile requires but
   includes fidelity and provenance, **When** it is validated, **Then** it is
   accepted.
3. **Given** a hint record that omits its fidelity field, **When** it is
   validated, **Then** it is rejected, because a hint that does not declare its
   trust level is exactly the guess-worn-as-fact the schema exists to prevent.
4. **Given** a change to a field in the shared core, **When** the schema is
   rebuilt, **Then** all four artifact forms reflect the change without a
   separate edit per form.

---

### User Story 3 - The schema is discoverable and authoritative (Priority: P3)

A contributor, a downstream tool, or an editor can obtain the exact schema
fragcap enforces, because it is embedded in the binary as the single source of
truth, emitted on demand, published in the repository, and rendered on the
documentation site as a field-level reference.

**Why this priority**: Discoverability turns the schema into an ecosystem. Editors
validate against it live, agents self-check their output, and the submission
pipeline has a canonical target. It depends on P1 and P2 (there must be a schema
worth publishing) and is the layer that lets the outside world build against
fragcap rather than guess.

**Independent Test**: Confirm the schema the binary enforces is byte-identical to
the schema it emits on demand and to the copy published in the repository, and
that the documentation site presents every field with its meaning and
constraints.

**Acceptance Scenarios**:

1. **Given** a running fragcap binary, **When** the schema is requested, **Then**
   it emits the exact schema it enforces during validation.
2. **Given** the published repository copy and the documentation-site reference,
   **When** they are compared to the embedded schema, **Then** they match, and a
   drift between them is caught mechanically rather than by eye.

---

### Edge Cases

- An otherwise valid file that is structurally correct but semantically broken
  (a cyclic ancestry, two terminal stages) passes structural validation; the
  response makes clear this surface checks structure and that semantic validation
  is a separate, profile-load concern, so a passing structural check is not
  mistaken for a working profile.
- An empty file, a file containing only whitespace, and a JSON file that is a
  bare array or scalar rather than an object are each reported with a clear
  message rather than a crash.
- A file whose artifact form cannot be determined (it matches no variant of the
  core) is reported as such rather than silently validated against the wrong
  variant.
- A file declaring a schema version fragcap does not support is rejected with a
  message naming the supported version, not partially validated.
- A fidelity value outside the defined tiers is rejected as an out-of-range enum.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST define a single versioned master schema that
  governs all machine-readable targeting/attribution artifacts, composed of a
  shared core plus per-form variants (full profile, hint record, user package,
  database export).
- **FR-002**: The system MUST provide a one-off command that validates any
  referenced JSON file against the master schema and reports every structural
  violation found in a single run, exiting non-zero when any violation exists and
  zero when none do.
- **FR-003**: Validation MUST NOT stop at the first violation; it MUST accumulate
  and report all structural violations together, each with a precise location
  within the file.
- **FR-004**: The system MUST reject unknown keys as violations rather than
  ignoring them, preserving the deliberate strictness of the current profile
  parser.
- **FR-005**: Every artifact form MUST carry a structured `fidelity` field drawn
  from an ordered, closed set of tiers (authored, verified, heuristic-unverified,
  observed), and a hint record MUST be rejected if it omits fidelity or
  provenance.
- **FR-006**: The schema MUST support a structured human-readable notes field
  distinct from any machine-interpreted field, so human context travels as data
  rather than as a comment.
- **FR-007**: The system MUST distinguish a JSON syntax error from a schema
  violation and report each clearly.
- **FR-008**: The system MUST determine which artifact-form variant a file
  targets from an explicit top-level `kind` discriminator (a closed enum of
  `profile`, `hint`, `package`, `export`) and validate against that variant,
  reporting clearly when the discriminator is absent or names no known variant.
- **FR-009**: The system MUST reject a file whose declared schema version is
  unsupported, naming the supported version.
- **FR-010**: The schema the binary enforces MUST be the single source of truth,
  embedded at build time, and the system MUST be able to emit that exact schema
  on demand via `fragcap schema print`.
- **FR-011**: The published repository copy and the documentation-site
  field-level reference MUST match the embedded schema, and a drift between them
  MUST be caught by an automated check rather than by inspection.
- **FR-012**: The specification MUST declare the boundary between structural
  validation (owned by the schema) and semantic validation (owned by the
  profile-load path: acyclic ancestry, at most one terminal stage, role
  reachability, no ambiguous image match), so neither layer is expected to catch
  the other's class of error.
- **FR-013**: The database export of a hint record MUST validate against the
  master schema with no manual adjustment (round-trip conformance).
- **FR-014**: Any new term the schema introduces (fidelity tier names,
  provenance, artifact-form names) MUST receive a glossary entry in the same
  change.

### Key Entities *(include if feature involves data)*

- **Master schema**: The single versioned vocabulary. A shared core plus
  per-form variants. The source of truth for what a valid targeting/attribution
  artifact is.
- **Shared core**: The process-recognition vocabulary common to all forms: game
  identity, capture defaults, and the stage/match-predicate structure carried
  forward from section 15 (exe, path_contains, path_regex, cmdline_contains,
  descends_from; lifecycle; terminal; role).
- **Full profile**: The strict form. The authoritative description of a game the
  capture pipeline runs against.
- **Hint record**: The loose, partial form emitted by heuristic providers and the
  hint database. May omit fields a full profile requires, but MUST carry fidelity
  and provenance.
- **User package**: The highest-precedence, hand-authored or community-submitted
  form.
- **Database export**: The JSON projection of a hint database row, which MUST
  conform to the master schema.
- **Fidelity**: A structured, ordered trust tier on every artifact (authored,
  verified, heuristic-unverified, observed) that the resolver reads.
- **Provenance**: A structured record of where an artifact came from (for
  example, the source and a seed timestamp for a database hint).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A file containing N independent structural violations produces
  exactly N reported violations in a single validation run, every time, with no
  first-error short-circuit.
- **SC-002**: All four artifact forms (full profile, hint record, user package,
  database export) validate against the one master schema; there is no
  second schema anywhere in the system for any of them.
- **SC-003**: A hint database row exported to JSON validates against the master
  schema with zero manual adjustment.
- **SC-004**: A hint record missing its fidelity or provenance is rejected 100%
  of the time.
- **SC-005**: The schema the binary enforces, the schema it emits on demand, the
  repository copy, and the documentation-site reference are identical, and any
  divergence is reported by an automated check.
- **SC-006**: Describing and validating a new game requires only authoring a JSON
  file that passes validation, never a change to program code; this preserves the
  section 15.1 promise across the format change. End-to-end capture from that
  file (the pipeline consuming JSON) is co-delivered with the profile parser
  migration (#76); this slice delivers the authoring-and-validation half.
- **SC-007**: An author can determine whether a target file is well-formed
  without running a capture.

## Assumptions

- **Scope boundary.** This slice delivers the master schema and a generic
  structural validation surface only. Migrating the profile parser onto JSON
  (#76), the resolver that acts on fidelity (#77), and the hint database and its
  export producer (#78) are separate slices that depend on this one. The
  profile-load semantic checks (acyclic ancestry, single terminal stage, role
  reachability, ambiguous image match) continue to exist and are rewired onto
  JSON by #76; this slice defines the schema they sit above and declares the
  structural/semantic boundary, but does not itself move that semantic code.
- **Validation surface vs profile validation.** The one-off command in this slice
  validates structural conformance to the master schema for any artifact form.
  Full profile validation (structural plus semantic, all errors at once) is the
  profile-load path and remains defined by section 15.4, carried onto JSON by
  #76. Both honor all-errors-at-once within their layer.
- **Form discrimination.** The master schema carries an explicit top-level `kind`
  discriminator (closed enum: `profile`, `hint`, `package`, `export`) so the
  validator selects the correct variant without the caller stating the form
  (resolved in Clarifications). The JSON Schema dialect is Draft 2020-12.
- **Dependency and toolchain constraint.** Introducing a JSON Schema validator is
  a workspace dependency addition and MUST carry the full AGENTS.md justification
  (chosen crate, license across the transitive graph, graph size, alternatives
  considered). The workspace minimum supported toolchain is 1.82 and MUST stay
  green under the minimum-toolchain check; a validator crate or transitive
  dependency that requires a newer toolchain invalidates the JSON direction and
  MUST be caught before implementation proceeds. Planning research (research.md)
  evaluated the `boon` validator crate and rejected it: it adds 42 transitive
  crates (the ICU4X stack via url/idna) for machinery this schema does not need,
  which is irreconcilable with the project's dependency discipline. The resolved
  approach publishes a standard Draft 2020-12 schema for the ecosystem and
  validates internally by hand over serde_json, adding only `serde_json` (already
  a dev dependency, promoted to runtime, 1.82-clean). The schema asset and the
  hand-rolled validator live in `fragcap-profile`; the `schema` CLI subcommands
  live in `fragcap-cli`; `fragcap-core` takes no new dependency (P-2).
- **Reconciliation.** Master specification section 15 is rewritten by this work
  to describe the JSON format and its generalization beyond the profile; that
  edit rides along with the slice.
- **Text hygiene.** All artifacts are UTF-8 without BOM, LF line endings, and
  contain no em-dashes or en-dashes anywhere, including schema description
  strings.
- **Non-technical stakeholders.** Profiles and target packages are authored by
  people who are not the people who can debug a validator; the value of this
  slice is measured by how well it names mistakes, not by how few it permits.
