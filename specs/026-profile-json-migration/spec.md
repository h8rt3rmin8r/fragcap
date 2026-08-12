# Feature Specification: Profile Format Migration from TOML to JSON

**Feature Branch**: `feat/profile-json-migration`

**Created**: 2026-08-12

**Status**: Draft

**Slice**: Post-S025 (GitHub issue #76). Moves the runtime profile-load path onto
the master JSON Schema delivered by S025 (#75). Reverses the S05 decision to parse
profiles with toml-span. Constitution principles in play: the section 15.4 rule
that validation reports every problem at once (P-4-adjacent, no silent loss), the
honesty posture that a heuristic is labeled as one (P-9), and the platform
neutrality of the profile crate (P-2).

**Input**: Migrate the runtime game profile / target definition format from TOML
to JSON, governed by the master JSON Schema merged in #75. Profile::parse stays
the only constructor. Loading validates structurally against the master schema
and semantically for the checks a schema cannot express, reporting every problem
in one pass. The Steam scaffold emits JSON with a machine-readable heuristic
warning. Examples, fixtures, resolution, the spec, and the glossary follow. The
toml-span dependency is removed. NOT YAML. Scoping/spec task only.

## Overview

The profile is the one file an operator or contributor writes by hand, and until
now it has been TOML. Slice S025 published a master JSON Schema that already
describes a profile as one of its four artifact forms, embedded a validator that
reports every structural violation at once, and made JSON a first-class input to
fragcap. What it deliberately did not do was move the runtime profile-load path:
the pipeline still parses TOML. This slice closes that gap so that the format an
author writes, the format the schema describes, and the format the pipeline loads
are the same one.

The migration is mostly a substitution with one genuinely new seam. The
substitution: the parser reads JSON instead of TOML, the resolution order finds
`.json` instead of `.toml`, the examples and fixtures are reauthored, and the
Steam scaffold emits JSON. The new seam is how two validation layers compose into
one report. Structural conformance (types, required keys, enums, unknown-key
refusal, the discriminators) is what the S025 schema and its validator already
express. The semantic invariants that a schema cannot express stay in this crate:
an acyclic ancestry graph, at most one terminal stage, at least one non-service
stage, every declared role reachable, no ambiguous image match, and the
compilation of every regex, glob, and duration literal. Loading a profile runs
both layers and reports every problem across both in a single pass. That
single-pass, all-errors property is the section 15.4 promise, and it is
non-negotiable: an author working against a game update is the person with the
least patience for a validator that stops at the first fault.

Three properties are load-bearing and must survive the format change unchanged.

**Profile::parse stays the only constructor.** There is no way to obtain a
Profile except through the function that validates it, so section 15.4's
requirement that validation run before every capture cannot be forgotten by a
later caller. The format underneath changes; this guarantee does not.

**A heuristic stays labeled as a heuristic.** The Steam scaffold's output carries
a warning that its stage classification is a guess and must be verified against a
live capture. Today that warning is a TOML comment, which a machine cannot act
on. After the migration it is structured data: a fidelity of
heuristic-unverified and a notes field. This is not cosmetic; it is the honesty
posture of the whole targeting redesign applied at the point a guess is written
down, and losing it in translation would be a regression.

**The promise that a profile is data, not code, is preserved.** Adding support
for a game is still writing a file and never editing Rust. The file's syntax
changes from TOML to JSON; the promise does not.

The slice stops at the profile-load path and its immediate satellites (the
scaffold, resolution, examples, fixtures, the spec, the glossary). It does not
build the resolver that ranks providers by fidelity (#77) or the hint database
(#78); it makes the profile a JSON citizen those slices can build on.

## Clarifications

### Session 2026-08-12

Resolved under autopilot from the spec, the constitution, the S025 code already
on main, and the existing profile-load architecture (a lenient all-optional
`Draft` built from the parsed tree, with `validate::check` running the semantic
checks on it, both sharing one `Diagnostics`/`DiagnosticCode` type).

- Q: How is the structural layer implemented, given S025 already ships a
  validator? -> A: The profile-load path reuses `jsonschema::validate_json` for
  structural conformance (so there is one structural implementation, bound to the
  published schema, with no third thing to keep in sync), and maps its
  `SchemaDiagnostics` into this crate's existing `Diagnostics`. `Profile::parse`
  keeps returning `Diagnostics`, so downstream consumers and tests that match on
  `DiagnosticCode` are unaffected.
- Q: How do the structural and semantic layers combine into one single-pass
  report without double-reporting? -> A: Split responsibilities cleanly.
  `jsonschema::validate_json` owns pure-structural faults (types, required keys,
  enum ranges, unknown keys, the kind and schema discriminators). A lenient
  fragcap pass owns only what a schema cannot express: compiling the glob, the
  regex, and the duration literals, and the semantic graph checks (acyclic
  descends_from, single terminal stage, at least one non-service stage, role
  reachability, ambiguous image match). Both accumulate into one `Diagnostics`;
  the two responsibility sets do not overlap, so nothing is reported twice.
- Q: What notation do diagnostic locations use after the migration? -> A: JSON
  pointers (RFC 6901, for example `/stage/1/match/exe`) throughout profile
  loading, replacing the TOML dotted-key path. Byte-offset line and column
  positions are dropped because serde_json exposes no per-value spans; this is a
  documented precision tradeoff (the pointer still names the exact value). Tests
  asserting on the old positions or dotted paths are updated.
- Q: Does the `DiagnosticCode` set change? -> A: Reuse the existing enum. Its
  `Syntax` variant is repurposed from "not valid TOML" to "not valid JSON"; no
  code is removed and no check is lost. New structural distinctions the schema
  adds (missing kind, missing fidelity, invalid fidelity) map onto existing codes
  (`MissingField`, and an added code only if an existing one does not fit).
- Q: What fidelity and kind do migrated profiles declare? -> A: Every JSON
  profile carries `kind: "profile"` and a `fidelity`. Hand-authored examples and
  bundled profiles use `verified` (they were validated in development); the Steam
  scaffold uses `heuristic-unverified`.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Author and load a game profile as JSON (Priority: P1)

An operator or contributor writes a game profile as a JSON file and fragcap loads
it, validating it before capture exactly as it validated the TOML form, with no
loss of any check.

**Why this priority**: This is the migration's core. If a JSON profile cannot be
authored and loaded with the same validation guarantees the TOML profile had, the
slice has not delivered. It is the minimum viable product: everything else
(scaffold, examples, docs) serves this.

**Independent Test**: Author a valid JSON profile for a game with a launcher and
a client, load it, and confirm it produces the same in-memory profile the
equivalent TOML produced, drives the same capture behavior, and that
Profile::parse is the only way to construct it.

**Acceptance Scenarios**:

1. **Given** a valid JSON profile conforming to the master schema's profile
   variant, **When** it is loaded, **Then** it is accepted and yields a profile
   with the same game identity, capture defaults, and stages the equivalent TOML
   profile yielded.
2. **Given** a JSON profile with a mix of structural and semantic mistakes,
   **When** it is loaded, **Then** every problem across both validation layers is
   reported in one pass, and the load is refused.
3. **Given** any code path that needs a profile, **When** it obtains one, **Then**
   it can only do so through the validating constructor; there is no unvalidated
   construction path.
4. **Given** a JSON profile that is structurally valid but semantically broken
   (a cyclic ancestry, or two terminal stages), **When** it is loaded, **Then**
   the semantic problem is reported, not silently accepted.

---

### User Story 2 - Resolve and validate profiles from the command line as JSON (Priority: P2)

An operator points fragcap at a profile by path, by name in a profile directory,
by name in the user directory, or by game id for a bundled profile, and the
resolution finds and validates JSON profiles, reporting every problem at once.

**Why this priority**: A profile that can be loaded programmatically but not
resolved and validated the way operators actually reference it is only half
migrated. It builds directly on US1's load path.

**Independent Test**: Place JSON profiles in a profile directory and the user
directory, reference one by name and one by path, and confirm each resolves,
validates, and reports diagnostics identically to the former TOML behavior; and
that a reference resolving to nothing fails as an expected, distinct outcome.

**Acceptance Scenarios**:

1. **Given** a JSON profile referenced by an explicit path, **When** it is
   resolved and validated, **Then** it validates and the path is reported once.
2. **Given** JSON profiles in a profile directory and the user directory, **When**
   a reference is resolved by name, **Then** the resolution order is honored and
   the correct source is reported.
3. **Given** an invalid JSON profile, **When** it is validated from the command
   line, **Then** every diagnostic is reported in one pass and the exit outcome
   marks a configuration error.

---

### User Story 3 - Scaffold a JSON profile whose heuristic warning is machine-readable (Priority: P3)

A user scaffolds a starter profile for an installed Steam title and receives a
JSON profile that validates against the master schema, is stamped
heuristic-unverified, and carries the warning that its stage classification must
be verified against a live capture as structured data rather than as a comment.

**Why this priority**: The scaffold is the on-ramp for non-technical users and
the point where a guess is first written down. Preserving the warning as data
(not a stripped comment) is the honesty property the redesign depends on. It
builds on US1 (the scaffold's output must be a loadable profile).

**Independent Test**: Scaffold a profile for an installed title, confirm the
output validates as a profile, carries fidelity heuristic-unverified and a notes
field containing the verification warning, and that loading it succeeds.

**Acceptance Scenarios**:

1. **Given** an installed title, **When** a profile is scaffolded, **Then** the
   output is JSON that validates against the master schema's profile variant.
2. **Given** a scaffolded profile, **When** it is inspected, **Then** it carries a
   fidelity of heuristic-unverified and a notes field stating the classification
   is heuristic and must be verified against a live capture.
3. **Given** a scaffolded profile, **When** it is loaded, **Then** it loads
   successfully through the validating constructor.

---

### Edge Cases

- A leftover TOML profile (a `.toml` file, or JSON-invalid content) is refused
  with a clear message rather than half-parsed; the tool does not silently accept
  the old format.
- A JSON profile that omits the now-required `kind` or `fidelity` top-level keys
  is refused with a diagnostic naming the missing key, in the same pass as any
  other problem.
- A profile whose only faults are semantic (a role cycle) still reports in one
  pass, proving the semantic layer runs even when structural validation passes.
- A profile file larger than the accepted maximum is refused as a whole-file
  problem, as it was under TOML.
- Duration, glob, and regex literals that do not compile are reported as
  semantic faults, not accepted and deferred to capture time.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A game profile MUST be authored and loaded as JSON conforming to
  the master schema's profile variant; the TOML profile format is replaced, not
  kept alongside.
- **FR-002**: Loading a profile MUST run structural validation against the master
  schema and the semantic validations a schema cannot express (acyclic
  descends_from, at most one terminal stage, at least one non-service stage,
  declared roles reachable, no ambiguous image match, and regex/glob/duration
  compilation).
- **FR-003**: Loading MUST report every problem found across both the structural
  and semantic layers in a single pass, never stopping at the first.
- **FR-004**: The validating constructor MUST remain the only way to obtain a
  profile; there MUST be no unvalidated construction path.
- **FR-005**: Profile resolution MUST resolve JSON profiles by explicit path, by
  name in a profile directory, by name in the user directory, and by game id for
  a bundled profile, honoring the existing resolution order.
- **FR-006**: The Steam scaffold generator MUST emit JSON that validates against
  the master schema's profile variant.
- **FR-007**: The scaffold output MUST carry a fidelity of heuristic-unverified
  and a notes field containing the warning that the stage classification is
  heuristic and must be verified against a live capture; this warning MUST be
  structured data, not a comment.
- **FR-008**: A file that is not valid JSON, or is the former TOML format, MUST be
  refused with a clear message rather than partially accepted.
- **FR-009**: Example profiles and test fixtures MUST be migrated from TOML to
  JSON, and the committed goldens or expectations updated to match.
- **FR-010**: The toml-span dependency MUST be removed from the profile crate, and
  the dependency inventory updated to record the removal.
- **FR-011**: Master specification section 15 and the glossary Game profile entry
  MUST be reconciled to describe the JSON format.
- **FR-012**: The in-memory profile representation and every downstream consumer
  (stage matching, capture) MUST behave identically to the pre-migration
  behavior for an equivalent profile; the change is the input format, not the
  semantics.

### Key Entities *(include if feature involves data)*

- **Game profile (JSON)**: The migrated artifact. A JSON document conforming to
  the master schema's profile variant: schema version, kind, fidelity, game
  identity, capture defaults, and the stage array with match predicates.
- **Load diagnostics**: The combined report from the structural and semantic
  validation layers, every problem in one pass.
- **Scaffold output**: A generated JSON profile at heuristic-unverified fidelity
  with a structured verification warning.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A JSON profile containing N mistakes (structural and semantic mixed)
  produces exactly N reported problems in a single load, every time.
- **SC-002**: Adding support for a new game requires only authoring a JSON file
  that loads cleanly; no program code changes.
- **SC-003**: Every validation the TOML path performed is performed on the JSON
  path; no check is lost in the migration (verified by parity of the diagnostic
  set on equivalent invalid inputs).
- **SC-004**: A scaffolded profile validates against the master schema and its
  heuristic warning is retrievable as structured data 100% of the time.
- **SC-005**: A profile that is not valid JSON, or is the former TOML format, is
  refused 100% of the time rather than partially accepted.
- **SC-006**: An equivalent profile drives byte-identical capture output before
  and after the migration (the format changed, not the behavior).

## Assumptions

- **Reuse over reinvention.** The structural layer reuses the S025 validator
  (jsonschema::validate_json) rather than re-deriving structural checks; the
  semantic layer is the existing profile validation logic, retained. How the two
  diagnostic representations combine into one single-pass report is a design
  decision for the planning phase, constrained by the all-errors-at-once and
  house-style requirements.
- **Required top-level keys.** Because the master schema requires kind and
  fidelity on every artifact, a migrated JSON profile carries kind (profile) and
  fidelity; the examples, fixtures, and scaffold set these. An authored profile's
  fidelity is authored or verified; the scaffold's is heuristic-unverified.
- **No dual-format period.** The migration replaces TOML rather than supporting
  both; a bundled or user TOML profile is not silently read. This avoids the
  ambiguity of two formats resolving to the same game id.
- **Scope boundary.** This slice migrates the profile-load path and its immediate
  satellites. It does not build the fidelity-ranking resolver (#77) or the hint
  database (#78); it makes the profile a JSON citizen those slices build on. The
  standalone `fragcap schema validate/print` command already exists from S025 and
  is not re-created here.
- **Toolchain.** The workspace minimum supported toolchain stays 1.82 and MUST
  remain green; the migration removes a dependency (toml-span) and adds none
  (serde_json is already a runtime dependency from S025).
- **Text hygiene.** All artifacts are UTF-8 without BOM, LF line endings, and
  contain no em-dashes or en-dashes, including JSON string values and comments in
  code.
- **Not YAML.** The target format is JSON. YAML is explicitly excluded.
