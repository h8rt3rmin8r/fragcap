# Feature Specification: Profile Schema, Parsing, and Validation

**Feature Branch**: `feat/profile-schema-validation`

**Created**: 2026-08-09

**Status**: Draft

**Slice**: S05 (specification section 15, with match predicates from section
10.3 and lifecycle classes from section 10.4; constitution P-2, P-4, P-5, P-6,
P-8, P-9)

**Input**: Implement specification section 15 in the `fragcap-profile` crate:
the TOML profile schema of section 15.2, the resolution order of section 15.3,
and the validation of section 15.4, which reports every problem found rather
than stopping at the first. Stage matching against live process events is S12.

## Overview

A profile is the mechanism by which fragcap supports a specific game without
containing knowledge of it. Section 15.1 states the consequence plainly: adding
support for a title requires writing a TOML file and never requires modifying
Rust. That promise is only worth making if the file is read faithfully and its
mistakes are named precisely, because the population writing these files is not
the population that can debug a parser.

This slice is therefore mostly about being wrong well. Three properties carry
it.

**Every problem is reported at once.** Section 15.4 requires it, and the reason
is the authoring loop. A validator that stops at the first fault turns a profile
with four mistakes into four edit-run cycles, and a profile author working
against a game update is the person with the least patience for that. The
implementation consequence is that validation cannot be a chain of `?` operators
over a `Result`; it accumulates diagnostics and hands back all of them.

**A profile that would silently produce nothing is refused.** This is the part
of section 15.4 that is not routine. Two checks exist specifically because the
failures they catch are invisible: a stage bound to the wrong process under a
recurring image name, and a capture default naming something no stage declares.
Both produce a run that completes successfully, exits zero, and writes a
well-formed capture file containing no gameplay. Section 5.4 records the focal
title where this is not hypothetical: three processes share one image name and
only the last holds sockets. Constitution P-4 forbids losing a packet without
counting it; a profile that captures the wrong process loses all of them and
counts nothing, which is the same defect arriving through the configuration
rather than through the pipeline.

**A profile cannot exist in an invalid state.** The parse entry point returns
either a `Profile` or the complete set of diagnostics, and there is no third
option and no way to construct the type past it. Section 15.4's requirement that
"validation runs implicitly before every capture" then costs nothing to honor
and cannot be forgotten by a later caller, which is a stronger guarantee than a
`validate` method every consumer is trusted to call.

The slice stops at the point where a profile becomes a set of facts about
processes that do not exist yet. Evaluating a predicate against a real process
start event, warning at runtime when a recurring image name binds the wrong one,
and everything about session lifecycle belong to S12, which has process events
to evaluate against. What this slice owes S12 is a profile whose predicates are
known to compile and known not to contradict each other.

## Clarifications

### Session 2026-08-09

- Q: Is the TOML parser hand-rolled, as the pcap, pcapng, and JSON Lines
  handling were? -> A: No. It is the first runtime dependency this workspace has
  taken since S02, and the distinction from those three is the direction the
  bytes travel. S04, S06, and S07 read and wrote a format whose exact byte shape
  was the deliverable, produced by fragcap or by a tool, and hand-rolling gave
  verification something independent to judge against. A profile is a file a
  contributor typed. TOML 1.0 has multi-line strings, inline tables, arrays of
  tables, dotted keys, escapes, and datetimes, and a hand-rolled subset would
  reject legal TOML that an author's editor formats or accepts constructs the
  language does not have. Rejecting a valid profile with a syntax error the
  author cannot see in their file is precisely the failure section 15.1's
  promise cannot survive. Crate selection, transitive count, license, and
  minimum supported toolchain are settled in the plan.
- Q: Does the slice take a regular expression engine as well? -> A: Yes, and it
  must be the same engine S12 will evaluate with. Section 15.4 lists regular
  expression compilation as a validation check, so an engine is unavoidable
  unless the check is deferred, and deferring it is not available: the check
  exists so that a bad `path_regex` is refused before a capture rather than
  during one. Validating with one engine and matching with another would let a
  pattern pass validation and fail at capture time, which is a worse defect than
  the one the check prevents. Section 8.2 places matching in this same crate, so
  one dependency serves both slices and there is no second engine to diverge
  from.
- Q: Then why is the `exe` glob matcher hand-rolled rather than taken from a
  crate? -> A: Because the ambiguity check needs a decision no glob crate
  offers. Section 15.4 requires knowing whether two patterns "can match the same
  image name", which is glob intersection rather than glob matching, and every
  glob crate answers only the second question. The intersection decision is a
  product walk over the two patterns, and once it exists, matching a single name
  is the same walk against a literal. Taking a dependency that answers half the
  question and hand-rolling the other half would leave two implementations of
  the same syntax to disagree. The syntax is also small enough to state
  completely: `*` matches any run of characters including none, `?` matches
  exactly one, every other character is a literal, comparison is
  case-insensitive, and there is no escape because no Windows image name can
  contain `*` or `?`.
- Q: Is the ambiguity check exact, or a conservative approximation? -> A: Exact,
  by product automaton over the two patterns. An approximation would have to
  choose which way to be wrong. False negatives return the silent empty capture
  the check exists to prevent, and false positives refuse a legal profile with
  an explanation its author cannot act on. Neither is acceptable when the
  decision is cheaply decidable for the syntax involved: two patterns over `*`,
  `?`, and literals have a non-empty intersection or they do not, and a table
  walk decides it.
- Q: Which stage pairs does the ambiguity check fire on? -> A: A pair fires when
  their `exe` patterns can match a common image name and at least one of the two
  stages matches on `exe` alone, with no other predicate to disambiguate it. Two
  stages that both carry an additional predicate are both pinned and are
  permitted to share an image name, which is exactly the section 15.2 profile
  for the second focal title. Section 15.4's second clause, a stage matching on
  `exe` alone against a name "the profile elsewhere indicates recurs", is the
  same condition read from the other side: the profile indicates recurrence by
  declaring another stage that can match the same name. The diagnostic names
  both stages and the predicate that would fix it.
- Q: Where does duration parsing live, given the command line needs the same
  grammar for `--duration`, `--wait`, and `--ring`? -> A: In `fragcap-core`, as
  a pure module beside `parse`. Section 25.2 lists duration parsing as a tier 0
  concern without placing it, and three consumers are already visible: this
  slice's `capture.duration`, S14's three options, and S16's ring window. Core
  is the crate all three can reach, and a duration grammar is arithmetic over a
  string with no platform surface, which is the same argument that put the
  header parser there in S03. The alternative, keeping it in `fragcap-profile`,
  forces S16 either to depend on a sibling, which section 8.3 forbids, or to
  write a second grammar, and two implementations of "30m" that disagree is a
  defect that would surface as a capture of the wrong length. No new dependency
  is involved, so the core allowlist is untouched.
- Q: What exactly does a duration literal accept? -> A: One unsigned decimal
  integer followed by one required unit suffix from `ms`, `s`, `m`, `h`. A bare
  integer is rejected because its unit would be a guess, and a guess about
  capture length is a guess about how much of a session an operator loses. Zero
  is rejected for the reason S08 rejected a zero-capacity buffer: it is a
  setting whose only possible meaning is a mistake. Compound forms such as
  `1h30m` are rejected in this slice and may be added later, because widening an
  accepted syntax keeps every existing profile valid while narrowing one does
  not, so the conservative direction is the reversible one.
- Q: How does the resolution order of section 15.3 avoid making this crate
  platform-specific? -> A: By taking the search path as an argument rather than
  discovering it. The resolver accepts an ordered list of directories and a set
  of bundled profiles, and implements the four-step order over them; it does not
  ask the operating system where a user's configuration lives. The caller that
  knows, `fragcap-cli` in S14, supplies it. This keeps a `dirs`-style dependency
  out of the workspace, keeps the ordering itself testable against directories a
  test creates, and leaves the platform question at the one layer that is
  allowed to have an opinion about it. Constitution P-2 binds only core, so this
  is a design preference rather than an obligation, but the seam that results is
  the same shape as every other seam in the project.
- Q: Are unknown keys ignored or rejected? -> A: Rejected, with a diagnostic
  naming the key and the accepted set. This is the tightest reading of the
  schema and it is chosen deliberately, because the failure mode of ignoring is
  silent. An author who writes `payloads = false` intending `payload = false`
  gets a capture containing full packet contents they meant to exclude, and
  nothing in the run says so. That is a P-9 problem rather than a typo: the
  instrument was told to narrow what it recorded and did not. `schema` is what
  makes strictness safe, since a profile written for a later schema declares it
  and is refused with a version diagnostic rather than a key diagnostic.
- Q: Which `[capture]` keys exist? -> A: Exactly the five section 15.2 declares:
  `mode`, `duration`, `roles`, `loopback`, `payload`. Section 17.2 lists more
  capture options on the command line, and the temptation is to accept all of
  them here on the theory that a default should exist for each. This slice
  declines, because a profile key with no consumer is a key whose behavior is
  untested and whose meaning is set by whoever first reads it. S14 owns the
  command line and adds the keys it can honor, one schema decision at a time.
- Q: Does section 15.4's list of checks bound what validation may check? -> A:
  It is a floor. Three further checks are added here, each in the same failure
  class the two named checks were added for, which is a successful run that
  captured nothing or captured the wrong thing. A `capture.roles` entry naming
  an undeclared role captures nothing under that role. A `terminal` stage whose
  lifecycle is not `session` ends the capture when a process that was expected
  to exit exits, which for a `transient` launcher is immediately, producing a
  short well-formed file. A `descends_from` cycle is unsatisfiable, so every
  stage in the cycle binds nothing. All three are recorded in the plan as
  additions rather than as readings of section 15.4, and are candidates for
  promotion into section 15.4 under the deviation process.
- Q: Is `game.id` constrained beyond being unique? -> A: Yes, to lowercase ASCII
  alphanumerics, hyphen, and underscore, non-empty. Section 15.2 calls it a slug
  and section 15.3 resolves `<ref>.toml` against directories, which makes the
  identifier a filename component. An id containing a path separator, a parent
  reference, or a drive prefix would let a reference reach outside the search
  directories it was given, and resolution is reached from a command line
  argument. Constraining the charset at validation, and constraining the
  reference itself at resolution, closes it in both places rather than trusting
  one.
- Q: Does resolution follow a reference that is not a bare slug? -> A: Step one
  of section 15.3 is an explicit path and stays an explicit path: a reference
  that names an existing file resolves to it, wherever it is, because the
  operator typed it. Steps two through four join the reference to a directory,
  and there the reference MUST be a valid slug or the resolver refuses it
  without touching the filesystem. The distinction is between an operator naming
  a file and a name being interpolated into a search path.
- Q: What does the parse entry point return? -> A: `Result<Profile,
  ProfileDiagnostics>`, where the error side carries every diagnostic found and
  the success side is a type whose invariants are the validation rules. There is
  no public constructor that bypasses it and no `Profile` that has not been
  validated. The rejected alternative was a parse step returning a document plus
  a separate `validate` returning a report, which reads more conventionally and
  makes "validation runs implicitly before every capture" a convention every
  future caller has to remember. Internally the two phases still exist, because
  collecting every structural fault requires a draft in which a field can be
  absent, but that draft is not public.
- Q: Does a TOML syntax error suppress the semantic diagnostics? -> A: Yes, and
  it is the one place accumulation stops. A file that does not parse has no
  tables to check, and a parser recovering into a guess at the author's intent
  would report faults against a document they did not write. One syntax
  diagnostic, with the line and column the parser supplies, is the honest
  output. Once the document parses, everything after that accumulates: several
  missing fields, several bad types, and every semantic fault are reported
  together in one pass.
- Q: How does a caller distinguish diagnostics without matching on prose? -> A:
  Each diagnostic carries a closed enumeration of codes, a location as a dotted
  key path such as `stage[1].match.descends_from`, an optional line and column
  when the parser supplies one, and a message. Tests assert on codes and
  locations. A message is for the operator and may be reworded; a code is part
  of the crate's surface and is what S14's `profile validate` output and any
  future documentation are keyed to.
- Q: In what order are diagnostics returned? -> A: Deterministically, sorted by
  location and then by code, and never in hash iteration order. The output is
  read by a person comparing two runs and is compared byte for byte by tests, so
  a stable order is a correctness property rather than a nicety.
- Q: Does this slice ship the bundled profiles for the two focal titles? -> A:
  No. Section 15.5 ships them at v0.1.0 and section 15.3 resolves them last, but
  a bundled profile is a claim about a specific game's current process topology,
  and the slices that verify such a claim are S17 for Steam and S14 for the
  command line that would exercise it. This slice ships the resolver's ability
  to consult a bundled set and a bundled set that is empty. The section 15.2
  examples for both focal titles are used as test fixtures, which exercises the
  same parsing without shipping an unverified claim.
- Q: Does this slice provide the `fragcap profile validate` command? -> A: No,
  S14 owns every command surface. This slice provides the diagnostics that
  command prints and the exit-code-2 condition it reports, as library values.
- Q: Which terms need glossary entries? -> A: Profile schema version, lifecycle
  class, terminal stage, match predicate, profile resolution order, ambiguous
  image match, and duration literal. The existing `Stage` and `Game profile`
  entries gain cross-references, and `Game profile` gains the schema version
  because it currently describes the file without mentioning that it is
  versioned.
- Q: What is the trust and resource posture when reading a profile file? -> A: A
  profile is untrusted input to a parser and is not a security boundary, which
  resolves into three concrete rules rather than a general intention. A file
  larger than one mebibyte is refused by size before its contents are read,
  because a profile is tens of lines and no legitimate one approaches the limit,
  while an arbitrarily large file handed to a parser is an ordinary resource
  fault with no upside. A symbolic link is followed exactly as the platform
  would, and this slice adds no link policy: the search directories belong to
  the operator and the distribution, and refusing a link would break the
  configuration managers that create them while protecting against nothing an
  operator could not do by copying the file. A `path_regex` that exceeds the
  regular expression engine's own compiled size limit is reported as a
  compilation failure like any other malformed pattern, which is the honest
  reading: the engine already refuses the pathological case, and reimplementing
  that judgement here would produce a second opinion to keep in step with the
  first.
- Q: Does the parsed profile carry the typed duration, the literal it came from,
  or both? -> A: The typed value only. Carrying both was considered on the
  theory that a diagnostic or a future round-trip would want the text, and it
  does not survive: a literal that fails to parse never yields a profile, so the
  only place the text is needed is a diagnostic that already has it from the
  parser, and no consumer in this slice or the three that follow round-trips a
  profile back to TOML. Constitution P-9 is not in tension with this. A duration
  literal is configuration an operator wrote, not an observation fragcap made,
  and parsing `30m` into a span is reading the value rather than altering one.
- Q: Is there a bound on stage count, and what does the ambiguity check cost? ->
  A: No separate bound, and the cost is stated rather than capped. The check
  decides every unordered pair of stages, and each decision walks a table over
  the two patterns, so it is quadratic in stage count and linear in the product
  of two pattern lengths. Both are bounded already by the file size limit above,
  which is what makes a stage limit unnecessary: an arbitrary maximum would be a
  number with no reason behind it, and a profile author who legitimately needs
  many stages should not meet one.
- Q: What happens when a profile reference names an existing directory? -> A: It
  does not satisfy step one, which section 15.3 states as a path to an existing
  file, and a directory is not one. It also does not fall through into a path
  join, because a reference carrying a path separator is not a valid slug and is
  refused before any directory is joined to it. A bare name that happens to also
  be a directory in the working directory does fall through and is looked up as
  `<ref>.toml`, which is the correct outcome: the operator named a profile and
  not a folder.
- Q: Which parts of a diagnostic are stable public surface? -> A: The code
  enumeration, and not the location string. A code is what S14's output, any
  future documentation, and every test key on, so adding a variant is a surface
  change a reviewer sees. The location is a human-readable locator whose job is
  to point an author at a line in their own file, and committing to it as a
  parseable grammar would freeze a formatting choice for the benefit of a
  consumer that does not exist. Tests may assert on it because they are in-tree
  and change with it.
- Q: The analyze gate measured the chosen parser refusing TOML datetimes. Does
  that break the argument for taking a real parser rather than hand-rolling one?
  -> A: It narrows the argument and the requirement was corrected to match,
  which is why FR-002 now names the constructs a profile can contain rather than
  claiming whole-language conformance. The first draft claimed the parser
  "implements the language rather than a subset", and `toml-span` refuses
  `1979-05-27T07:32:00Z` as an invalid number, so the claim was false as
  written. What survives is the part that mattered: every value type schema
  version 1 has is accepted, including the literal-string form a Windows path
  needs, and a datetime can only appear in a profile that is invalid anyway,
  since no key has that type. The consequence is confined to diagnostic quality.
  An author writing a datetime where a duration belongs gets a syntax diagnostic
  rather than a typed one at `capture.duration`, which is a worse message about
  a profile that is refused either way. The divergence is pinned by a test
  rather than left to be rediscovered, and the alternative was measured and is
  not available: `toml` declares Rust 1.85 against a floor of 1.82, and holding
  it there would require pinning a transitive crate this slice never calls.

- Q: Pull request 11's review found the ambiguity pass unbounded in
  practice. Does that change the decision to state its cost rather than cap it?
  -> A: Yes, and the reversal is worth being precise about because the original
  reasoning was not merely incomplete, it was wrong. The claim was that the one
  mebibyte file limit bounded the pass. It bounds each factor and not their
  product: two `exe` patterns of half a megabyte each fit inside that limit and
  ask the intersection decision for a table of roughly 10^12 cells, which aborts
  the process instead of returning a diagnostic. A profile that has already been
  refused should not be able to end the run that refuses it. Two bounds now
  exist, and each is justified by the domain rather than picked for
  tidiness. An `exe` pattern is capped at 255 characters, because it matches one
  Windows file name component and Windows caps that at 255, so a longer pattern
  is longer than anything it can be compared against. A profile is capped at 64
  stages, because the pairwise pass is quadratic in that count and the focal
  titles of section 5.4 declare two and three, so 64 is two orders of magnitude
  beyond any plausible launcher chain. Together they bound the worst case at
  about 1.3 times 10^8 cell visits and 64 kibibytes of peak table.
- Q: Should the schema version gate run before or after the top level key check?
  -> A: Before, which is a correction rather than a preference. The first
  implementation ran the key check first, so a profile declaring a later schema
  and a key this build does not know came back with two diagnostics where FR-012
  promises one. A new key is the most likely thing a later schema adds, so
  reporting it beside the version fault reports a consequence of that fault as
  though it were a second problem. Found in pull request 11's review; the test
  that was supposed to cover it had placed its unknown key inside `[game]`
  rather than at the top level, and so passed without exercising the path.
- Q: When one entry in `capture.roles` has the wrong type, are the others still
  checked? -> A: Yes. A list of `["ghost", 1]` carries two independent faults:
  the second element's type, and the first naming a role no stage declares.
  Discarding the list on the first fault reports one and hides the other, which
  is what FR-013 forbids. Emptiness is judged on what the author declared rather
  than on what survived parsing, so a list with one bad element is not also
  reported as empty, which would be a wrong diagnostic rather than an extra one.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A profile is read into facts (Priority: P1)

A contributor writes the TOML file section 15.2 shows and fragcap reads back
every field, with the stages in declaration order and the predicates intact.

**Why this priority**: Nothing else in the slice is reachable without it, and
both section 15.2 examples are the acceptance surface for section 15.1's promise
that adding a game means writing a file.

**Independent Test**: Parse both worked examples from section 15.2 and assert
every field, including the three-stage profile whose client stage is
disambiguated by ancestry.

**Acceptance Scenarios**:

1. **Given** the section 15.2 single-title example, **When** it is parsed,
   **Then** the game identity, the capture defaults, and both stages are present
   with their declared values.
2. **Given** the three-stage example, **When** it is parsed, **Then** the client
   stage carries both an `exe` predicate and a `descends_from` predicate naming
   the anticheat role.
3. **Given** a profile declaring only the required fields, **When** it is
   parsed, **Then** it is accepted and the optional fields report their absence
   rather than a substituted default.
4. **Given** a profile whose stages are declared in a given order, **When** it
   is parsed, **Then** that order is preserved.
5. **Given** any parsed profile, **When** its values are compared against the
   file, **Then** no field has been normalized, case-folded, or rewritten.

---

### User Story 2 - Every problem is reported at once (Priority: P1)

A profile author with four mistakes in one file learns about all four from one
run.

**Why this priority**: Section 15.4 requires it explicitly, and it is the
difference between a validator that helps an author and one that rations
information.

**Independent Test**: Parse a profile carrying one of each fault class and
assert the full diagnostic set, by code and location, in one call.

**Acceptance Scenarios**:

1. **Given** a profile missing several required fields, **When** it is parsed,
   **Then** every missing field is reported, each located by its dotted key
   path.
2. **Given** a profile with several fields of the wrong type, **When** it is
   parsed, **Then** every type fault is reported with the expected and the found
   type.
3. **Given** a profile with both structural and semantic faults, **When** it is
   parsed, **Then** both kinds appear in one diagnostic set.
4. **Given** a file that is not valid TOML, **When** it is parsed, **Then** the
   result is exactly one syntax diagnostic carrying the line and column, and no
   speculative semantic diagnostics.
5. **Given** the same invalid profile parsed twice, **When** the diagnostic sets
   are compared, **Then** they are identical in content and order.
6. **Given** any diagnostic, **When** a caller inspects it, **Then** it carries
   a code from a closed enumeration and does not require matching on message
   text.

---

### User Story 3 - A profile that captures nothing is refused (Priority: P1)

An author whose stage would bind to the wrong process under a recurring image
name is told so before the capture rather than after it.

**Why this priority**: This is the slice's reason for existing beyond schema
plumbing. The failure it prevents produces exit code zero and a well-formed file
with no gameplay in it, which is the one class of defect an operator cannot
detect by reading the output.

**Independent Test**: Assert the ambiguity decision directly over pattern pairs,
then assert it in profile form for the section 5.4 case and for the section 15.2
profile that resolves it.

**Acceptance Scenarios**:

1. **Given** two stages matching on `exe` alone whose patterns can match one
   common image name, **When** the profile is parsed, **Then** it is refused and
   both stages are named.
2. **Given** the same two stages where one carries a `descends_from` predicate,
   **When** the profile is parsed, **Then** it is accepted.
3. **Given** the section 15.2 three-stage profile, **When** it is parsed,
   **Then** it is accepted, because its recurring image name is pinned by
   ancestry.
4. **Given** pattern pairs whose intersection is empty, **When** they are
   tested, **Then** no ambiguity is reported, including pairs that share a
   prefix or a suffix without sharing a whole name.
5. **Given** patterns differing only in case, **When** they are tested, **Then**
   they are treated as intersecting, because section 10.3 makes `exe` comparison
   case-insensitive.
6. **Given** a `capture.roles` entry naming a role no stage declares, **When**
   the profile is parsed, **Then** it is refused and the undeclared role is
   named.

---

### User Story 4 - A profile reference resolves predictably (Priority: P2)

An operator who has corrected a bundled profile locally gets their copy, and can
tell from the result which file was used.

**Why this priority**: Section 15.3's shadowing is what lets a drifted bundled
profile be fixed without waiting for a release. Priority P2 because a profile
that resolves to the wrong file is at least visible in the report, unlike a
profile that captures the wrong process.

**Independent Test**: Build a directory layout for each step of the order and
assert which file each reference resolves to, including the shadowing case.

**Acceptance Scenarios**:

1. **Given** a reference naming an existing file, **When** it is resolved,
   **Then** that file is used regardless of the search directories.
2. **Given** a reference present in both a command line directory and the user
   directory, **When** it is resolved, **Then** the command line directory wins.
3. **Given** a reference present in the user directory and in the bundled set,
   **When** it is resolved, **Then** the user file wins.
4. **Given** a reference present only in the bundled set, **When** it is
   resolved, **Then** the bundled profile is used, matched on `game.id`.
5. **Given** a reference matching nothing, **When** it is resolved, **Then** the
   failure names the reference and every location that was searched.
6. **Given** a resolved profile, **When** the result is inspected, **Then** it
   reports which of the four sources supplied it and, for the three file cases,
   the path.
7. **Given** a reference that is not an existing file and not a valid slug,
   **When** it is resolved, **Then** it is refused without any directory being
   joined to it.

---

### User Story 5 - An unsupported schema is refused clearly (Priority: P2)

An operator handed a profile written for a later fragcap gets a version
diagnostic naming the supported version, not a pile of unknown-key faults.

**Why this priority**: It is what makes strict key rejection safe, and it is the
mechanism by which the schema can grow at all.

**Independent Test**: Parse profiles declaring a lower, equal, higher, missing,
and non-integer schema value.

**Acceptance Scenarios**:

1. **Given** a profile declaring `schema = 1`, **When** it is parsed, **Then**
   the version is accepted.
2. **Given** a profile declaring a higher schema version, **When** it is parsed,
   **Then** the only diagnostic is the version diagnostic, which names the
   supported version.
3. **Given** a profile with no `schema` key, **When** it is parsed, **Then** the
   missing key is reported and no version is assumed.
4. **Given** a profile whose `schema` is not an integer, **When** it is parsed,
   **Then** a type diagnostic is reported against `schema`.

---

### Edge Cases

- What happens when a profile declares no stages at all? Refused. Section 15.2
  requires at least one, and section 15.4's non-service requirement cannot be
  satisfied by an empty set. A profile with nothing to match can only produce a
  capture that never starts.
- What happens when every stage is a `service`? Refused, per section 15.4.
  Section 10.4 says a service is never awaited during acquisition, so nothing in
  such a profile can ever trigger it.
- What happens when a stage is marked `terminal` with lifecycle `transient`?
  Refused. Section 10.4 defines a transient exit as normal and expected, so a
  terminal transient ends the capture at the moment a launcher hands off, which
  is the point the whole launcher chain exists to survive.
- What happens when two stages are marked `terminal`? Refused, per the section
  15.2 table. Which one ended the capture would otherwise be an implementation
  detail.
- What happens when `descends_from` names a role that does not exist? Refused,
  per section 15.4. Nothing can descend from a stage that was never declared.
- What happens when `descends_from` names the stage's own role? Refused. A stage
  cannot be its own ancestor, and the general cycle check covers it.
- What happens when `descends_from` forms a cycle between several stages?
  Refused, and every role in the cycle is named. No process assignment can
  satisfy it, so every stage in the cycle would bind nothing.
- What happens when a `match` table is present but empty? Refused. An empty
  predicate set matches every process on the system, which is both a wrong
  binding and the widest possible one.
- What happens when `match` carries an unknown predicate name? Refused and the
  key is named, alongside the five predicates section 10.3 defines. Silently
  ignoring it would leave the stage matching on strictly fewer conditions than
  its author wrote.
- What happens when `path_regex` does not compile? Refused, with the engine's
  own message and the location. This is section 15.4's regular expression check
  and the reason the engine must be the one S12 evaluates with.
- What happens when a `duration` is `"30"`, with no unit? Refused. A guessed
  unit is a guess about how much of a session the operator loses.
- What happens when a `duration` is zero? Refused. A capture bounded at zero can
  only be a mistake, and honoring it would produce an empty file and a
  successful exit.
- What happens when `roles` is present but empty? Refused. An empty role set
  captures nothing, which is the same silent-empty outcome as a role that does
  not exist.
- What happens when `mode` is not one of the three section 17.2 names? Refused,
  and the accepted set is listed.
- What happens when a key appears twice in the file? The TOML parser refuses it
  as a syntax fault, and fragcap does not soften that into a last-one-wins rule.
  Two values for one key means the author's intent is not in the file.
- What happens when `game.id` contains a path separator or a parent reference?
  Refused by the slug charset check, and refused again at resolution if it
  arrives as a reference. Both places, because resolution can be reached with a
  reference that never passed through validation.
- What happens when a search directory in the resolution order does not exist,
  or cannot be read? It is skipped and resolution continues to the next step. A
  missing user configuration directory is the ordinary state of a fresh install,
  not an error, and section 15.3's order is defined over what is present.
- What happens when a resolved file exists but cannot be read, for example
  through permissions? That is an error rather than a skip, and it names the
  path. Section 15.3 selects on presence, so a file that is present and
  unreadable has already won its step, and skipping it would silently fall
  through to a profile the operator did not choose.
- What happens when the same `game.id` appears in two bundled profiles? Refused
  at construction of the bundled set, naming both. Section 15.2 requires the id
  to be unique, and resolution step four selects on it, so a duplicate makes
  step four ambiguous.
- What happens when a profile is valid but describes a game that is not
  installed? Nothing here. A profile is a description, not a claim about the
  current machine, and section 15.5 expects bundled profiles to drift.
- What happens when an `exe` pattern is enormous? Refused, naming the 255
  character limit. Windows caps a file name component there, so a longer pattern
  cannot match anything that exists, and the limit is also what keeps the
  intersection decision's table from being proportional to the file size.
- What happens when a profile declares thousands of stages? Refused, naming the
  64 stage limit. The pairwise ambiguity pass is quadratic in that count, and a
  file that has already been refused should not be able to exhaust the machine
  that refuses it.
- What happens when a candidate file is enormous? It is refused on size before
  its contents are read, naming the limit. A profile is tens of lines, so the
  limit cannot be reached by a legitimate file, and reading an arbitrary
  quantity of bytes because they sat in the right directory is a fault with no
  upside.
- What happens when a reference names an existing directory? It fails step one,
  which requires a regular file. If the reference carries a path separator it is
  refused as an invalid slug before any join; if it is a bare name it falls
  through to step two and is looked up as `<ref>.toml`, because the operator
  named a profile rather than a folder.
- What happens when a candidate path is a symbolic link? It is followed as the
  platform would follow it. The search directories belong to the operator and
  the distribution, and a link there is a choice they made.
- What happens when a `path_regex` compiles in principle but exceeds the
  engine's size limit, for example through nested large repetitions? The engine
  refuses it and that refusal is reported as an ordinary compilation diagnostic.
  fragcap does not form a second opinion about which patterns are too large,
  because two opinions would have to be kept in step.

## Requirements *(mandatory)*

### Functional Requirements

**Schema and parsing, section 15.2**

- **FR-001**: The profile schema MUST live in `fragcap-profile`, and the crate
  MUST NOT depend on a sibling below the facade.
- **FR-002**: Parsing MUST accept every TOML 1.0 construct a schema version 1
  profile can legitimately contain: basic and literal strings in both
  single-line and multi-line forms, integers, booleans, arrays, inline tables,
  dotted and quoted keys, and arrays of tables. A construct the schema has no
  type for MAY be refused, but MUST be refused rather than misread, and the
  known divergence MUST be recorded and pinned by a test.
- **FR-002a**: Parsing MUST accept a Windows path written as a literal string,
  with its backslashes preserved and no escape processing, because that is the
  form a profile author writing `path_contains` will use.
- **FR-003**: Parsing MUST read `schema`, the `[game]` table, the optional
  `[capture]` table, and one or more `[[stage]]` tables. A profile declaring no
  stage MUST be refused.
- **FR-003a**: A profile declaring more than 64 stages MUST be refused with a
  diagnostic naming the limit. The bound exists because the ambiguity check of
  FR-030 is quadratic in stage count and the file size limit does not bound that
  product.
- **FR-004**: `game.id` and `game.name` MUST be required; `game.platform` and
  `game.app_id` MUST be optional and MUST report absence rather than a
  substituted value.
- **FR-005**: The `[capture]` table MUST accept exactly `mode`, `duration`,
  `roles`, `loopback`, and `payload`, each optional.
- **FR-006**: Each stage MUST require `role`, `lifecycle`, and `match`, and MUST
  accept an optional `terminal`.
- **FR-007**: The `match` table MUST accept exactly the five predicates section
  10.3 defines: `exe`, `path_contains`, `path_regex`, `cmdline_contains`, and
  `descends_from`.
- **FR-008**: Stage declaration order MUST be preserved.
- **FR-009**: Parsing MUST NOT alter, case-fold, trim, or normalize any declared
  value, per constitution P-9.
- **FR-010**: An unknown key in any table MUST be reported, naming the key and
  the accepted set for that table.
- **FR-011**: The public entry point MUST return either a validated profile or
  the complete diagnostic set, and MUST NOT expose a way to obtain an
  unvalidated profile.

**Structural validation, section 15.4**

- **FR-012**: A `schema` value other than 1 MUST be reported as an unsupported
  version, naming the supported version, and MUST suppress every other
  diagnostic for that file, including the top level unknown-key check. The
  version gate therefore runs before that check rather than after it: a key this
  build does not know is the most likely thing a later schema added.
- **FR-013**: Every missing required field MUST be reported, and reporting MUST
  NOT stop at the first.
- **FR-014**: Every field whose type does not match the schema MUST be reported
  with the expected and the found type.
- **FR-015**: A TOML syntax fault MUST be reported as exactly one diagnostic
  carrying the line and column, and MUST suppress structural and semantic
  checking.

**Semantic validation, section 15.4**

- **FR-016**: Role names MUST be unique within a profile, and a collision MUST
  name both stages.
- **FR-017**: At most one stage may be `terminal`.
- **FR-018**: Every `descends_from` value MUST name a role declared in the same
  profile.
- **FR-019**: Every `path_regex` MUST compile with the engine that will evaluate
  it, and a failure MUST carry the engine's message.
- **FR-020**: Every `exe` glob MUST be a well-formed pattern in the glob syntax
  this crate defines.
- **FR-020a**: An `exe` pattern longer than 255 characters MUST be refused with
  a diagnostic naming the limit. Windows caps a file name component at 255
  characters, and the limit is also what bounds the intersection decision's
  table.
- **FR-021**: Every duration literal MUST parse.
- **FR-022**: At least one stage MUST have a lifecycle other than `service`.
- **FR-023**: `lifecycle` MUST be one of `transient`, `session`, or `service`,
  and `mode` MUST be one of `file`, `stream`, or `ring`.
- **FR-024**: A `match` table MUST carry at least one predicate.
- **FR-025**: `game.id` MUST be non-empty and MUST contain only lowercase ASCII
  alphanumerics, hyphen, and underscore.

**Additional semantic checks, beyond the section 15.4 list**

- **FR-026**: A `terminal` stage MUST have lifecycle `session`.
- **FR-027**: Every role named in `capture.roles` MUST be declared by a stage,
  and `capture.roles` MUST NOT be empty when present.
- **FR-027a**: An entry in `capture.roles` whose type is wrong MUST NOT suppress
  the checks on the entries that parsed, and emptiness MUST be judged on the
  number of entries declared rather than on the number that parsed.
- **FR-028**: The `descends_from` relation MUST be acyclic, and a cycle MUST
  name every role in it.

**Ambiguous image match, section 15.4**

- **FR-029**: The crate MUST decide exactly whether two `exe` patterns can match
  a common image name, over the glob syntax it defines, case-insensitively.
- **FR-029a**: The decision's memory and time MUST be bounded by constants this
  crate enforces, not by the size of the profile. Exactness is not permitted to
  cost an unbounded allocation.
- **FR-030**: Validation MUST refuse a profile containing a pair of stages whose
  `exe` patterns can match a common image name where at least one of the two
  matches on `exe` alone.
- **FR-031**: Two stages sharing a possible image name MUST be accepted when
  both carry at least one further predicate.
- **FR-032**: The ambiguity diagnostic MUST name both stages and MUST state that
  a further predicate resolves it.

**Resolution, section 15.3**

- **FR-033**: Resolution MUST follow section 15.3's four steps in order, first
  match winning.
- **FR-034**: Resolution MUST take its search directories and its bundled set
  from the caller, and MUST NOT consult the environment or a platform
  configuration location.
- **FR-035**: A reference that names an existing file MUST resolve to it without
  a slug check.
- **FR-036**: A reference used in steps two through four MUST be refused unless
  it is a valid slug, before any path is joined.
- **FR-037**: A search directory that is absent or unreadable MUST be skipped,
  and resolution MUST continue.
- **FR-038**: A candidate file that is present but unreadable MUST be an error
  naming the path, and MUST NOT fall through to a later step.
- **FR-039**: A successful resolution MUST report which source supplied the
  profile, and the path for the three file sources.
- **FR-040**: A failed resolution MUST name the reference and every location
  searched, including a candidate in a supplied directory that does not exist.
  Reporting only the directories that existed would let a failure claim nothing
  was searched when something was supplied.
- **FR-041**: A bundled set containing two profiles with the same `game.id` MUST
  be refused, naming both.

**Duration literals**

- **FR-042**: Duration parsing MUST live in `fragcap-core` and MUST NOT add a
  dependency to that crate.
- **FR-043**: A duration literal MUST be one unsigned decimal integer followed
  by one unit from `ms`, `s`, `m`, `h`.
- **FR-044**: A literal with no unit, an unknown unit, a negative sign, internal
  whitespace, or a fractional part MUST be refused.
- **FR-045**: A zero duration MUST be refused.
- **FR-046**: A literal whose value overflows the duration representation MUST
  be refused rather than wrapped or saturated.
- **FR-046a**: A parsed profile MUST carry the typed duration and MUST NOT keep
  the literal text alongside it.

**Input handling and limits**

- **FR-046b**: A candidate profile file whose size exceeds one mebibyte MUST be
  refused with a diagnostic naming the limit, and its contents MUST NOT be read
  in full first.
- **FR-046c**: Resolution step one MUST require a regular file, and a directory
  MUST NOT satisfy it.
- **FR-046d**: A symbolic link MUST be followed as the platform would, and this
  slice MUST NOT add a link policy of its own.
- **FR-046e**: A `path_regex` that the engine refuses for exceeding its own
  compiled size limit MUST be reported as a compilation failure carrying the
  engine's message, and MUST NOT be special-cased.

**Diagnostics**

- **FR-047**: Every diagnostic MUST carry a code from a closed enumeration, a
  location as a dotted key path, and a message.
- **FR-048**: A diagnostic MUST carry the byte offset the parser supplies for
  the value it concerns, and the one-based line and column derived from that
  offset. The derivation MUST happen in exactly one place.
- **FR-049**: The diagnostic set MUST be ordered deterministically and MUST NOT
  depend on hash iteration order.
- **FR-050**: A diagnostic set MUST NOT be empty when parsing failed, and MUST
  be empty when it succeeded.
- **FR-050a**: The diagnostic code enumeration is stable public surface. The
  location string is a human-readable locator and MUST NOT be specified as a
  parseable grammar for out-of-tree consumers.

**House rules**

- **FR-051**: The glossary MUST gain entries for profile schema version,
  lifecycle class, terminal stage, match predicate, profile resolution order,
  ambiguous image match, and duration literal in this change, and `Stage` and
  `Game profile` MUST gain cross-references, per constitution P-6.
- **FR-052**: The crate MUST NOT emit log output and MUST NOT introduce a
  logging facade.
- **FR-053**: Every new runtime dependency MUST be recorded with its license and
  its reason, and MUST satisfy the constitution's license allowlist.
- **FR-054**: `fragcap-core` MUST still depend on nothing outside its existing
  allowlist.
- **FR-055**: The crate MUST NOT read a process, open a handle, or touch a
  capture driver. Profiles describe processes; nothing here observes one.

### Key Entities

- **Profile**: A validated description of one game: its identity, its capture
  defaults, and its stages. Cannot exist without having passed validation.
- **Schema version**: The declared version of the profile format, currently 1.
  Refusing an unsupported value is what lets strict key checking be safe.
- **Game identity**: The slug, the display name, and the optional platform and
  application identifier. The slug is also a filename component during
  resolution, which is why its charset is constrained.
- **Capture defaults**: The optional `[capture]` table. Values an operator can
  override on the command line, held as declared-or-absent rather than as
  defaults already applied, so that S14 can tell the difference between a
  profile that chose a value and one that did not.
- **Stage**: A named position in the launcher chain, carrying a role, a
  lifecycle class, an optional terminal marker, and a predicate set.
- **Lifecycle class**: `transient`, `session`, or `service`, per section 10.4.
  Governs how an exit is treated and whether acquisition waits.
- **Match predicate set**: The five section 10.3 predicates, all of which must
  hold. Evaluated in S12; here only compiled and checked for contradiction.
- **Image name pattern**: The glob syntax `exe` is written in, with a matcher
  and an intersection decision over it.
- **Duration literal**: An integer and a unit, parsed in `fragcap-core` because
  three slices need the same grammar. A parsed profile keeps the typed span, not
  the text it came from.
- **Diagnostic**: A code, a location, a message, and an optional line and
  column. The unit of being wrong well.
- **Diagnostic set**: Every problem found in one file, deterministically
  ordered.
- **Profile reference**: What an operator names on the command line: a path, a
  filename stem, or a game id.
- **Resolution outcome**: The profile and the source that supplied it, or a
  failure naming everywhere that was searched.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Both worked examples from specification section 15.2 parse, and
  every field is asserted.
- **SC-002**: A profile carrying at least four distinct faults yields all four
  diagnostics from one call, asserted by code and location.
- **SC-003**: Every diagnostic code in the enumeration is produced by at least
  one test.
- **SC-004**: The ambiguity decision is asserted over a table of pattern pairs
  covering intersecting and disjoint cases, including case differences and
  shared prefixes and suffixes.
- **SC-005**: The section 5.4 recurring image name case is refused, and the
  section 15.2 profile that pins it with ancestry is accepted.
- **SC-006**: Each of the four resolution steps is exercised, and both shadowing
  cases are asserted.
- **SC-007**: A reference containing a path separator or a parent reference is
  refused before any filesystem access, asserted by a test that would fail if a
  path were joined.
- **SC-008**: Duration parsing is asserted over accepted and refused literals,
  including the overflow case.
- **SC-009**: Parsing the same invalid profile twice produces byte-identical
  diagnostic output.
- **SC-010**: `cargo xtask ci` passes, including the dependency direction check,
  the license check, and the conventions linter.
- **SC-011**: `fragcap-core` still builds for a target with no capture backend,
  and its dependency allowlist is unchanged.
- **SC-012**: Every runtime dependency added by this slice is named in the plan
  with its license, its transitive count, and the alternative that was rejected.
- **SC-013**: No test in this slice requires a capture driver, elevated
  privilege, a game, or a network interface.
- **SC-014**: The glossary carries the seven new entries and the two
  cross-references, and no term introduced by this slice is absent from it.
- **SC-015**: A file exceeding the size limit is refused by a test, and the test
  would fail if the contents were read first.
- **SC-016**: A pathological `path_regex` is refused by a test, and the
  diagnostic carries the engine's own message.
- **SC-017**: A reference naming an existing directory is exercised, both as a
  bare name and as a name carrying a path separator.
- **SC-018**: The ambiguity check's cost is bounded by a pattern length limit
  and a stage count limit that the crate enforces, each justified in the plan
  against the domain rather than chosen for convenience, and each demonstrated
  by a test that accepts the limit and refuses one past it.
- **SC-020**: A profile declaring a later schema version and an unknown top
  level key yields exactly one diagnostic, asserted by a test that places the
  key at the top level rather than inside a table.
- **SC-021**: A `capture.roles` list mixing a valid entry with a wrongly typed
  one reports both the type fault and the undeclared role, and does not report
  the list as empty.
- **SC-022**: A resolution failure names a supplied directory that does not
  exist, and the rendered message does not claim no directories were given.
- **SC-019**: Every value form a schema version 1 profile can contain is
  exercised by a parser test, including a Windows path as a literal string, and
  the known datetime divergence is pinned by a test that asserts the observed
  refusal rather than leaving it to be rediscovered.

## Assumptions

- The profile format is TOML, as section 15.2 states, and this slice does not
  reconsider it.
- Section 15.2's field table is complete for schema version 1. Where section
  17.2 lists a capture option with no key in that table, the absence is
  deliberate and S14 owns the addition.
- Stage matching, the runtime ambiguous-match warning, and session lifecycle are
  S12's, and S12 evaluates predicates using the same engine this slice validates
  them with.
- The operator, not the profile, decides the final capture parameters. A profile
  expresses defaults, and the command line wins, per section 17.2.
- Bundled profiles are supplied to the resolver by the caller, so this slice
  needs no opinion on where a distribution puts them.
- Tests create their own directories under the directory Cargo provides for
  integration test scratch space, so no temporary-file dependency is needed.

## Out of Scope

- Stage matching against real process start events, the synthetic process tree,
  and the runtime ambiguous-match warning of section 15.4 (S12). This slice
  proves a predicate compiles and does not contradict another; it evaluates
  nothing.
- Session lifecycle and stop conditions, sections 10.5 and 10.6 (S12).
- The `fragcap profile` command surface, its output formatting, and the exit
  code 2 mapping (S14). The diagnostics those depend on are delivered here as
  library values.
- Bundled profiles for the two focal titles, section 15.5 (S14 and S17). The
  resolver consults a bundled set; the set this slice ships is empty.
- Discovery of the user profile directory and any platform configuration
  location (S14).
- Steam library parsing and profile scaffolding, section 16 (S17).
- Applying capture defaults to a capture, which requires a capture (S14).
- Size literals for `--max-bytes` and the ring window. Section 15.2 declares no
  size key, so the grammar arrives with the slice that has a consumer for it.
- Compound duration literals such as `1h30m`. Widening the accepted syntax later
  keeps every profile written against this slice valid.
- Any change to the five behavioral traits in `fragcap-core`, to either writer,
  or to the pipeline.
- Logging and any observability surface. The diagnostic set is the whole
  reporting surface.

## Done When

- [ ] Both section 15.2 worked examples parse, with every field asserted.
- [ ] A profile with several faults reports all of them in one call, and every
  diagnostic code is exercised by a test.
- [ ] The ambiguous image match check refuses the section 5.4 case and accepts
  the profile that pins it by ancestry.
- [ ] All four resolution steps and both shadowing cases are exercised, and a
  traversal-shaped reference is refused before any filesystem access.
- [ ] Duration parsing lives in `fragcap-core`, adds no dependency there, and is
  asserted over accepted, refused, and overflow inputs.
- [ ] Every runtime dependency added is justified in the plan against a named
  alternative, with its license and transitive count.
- [ ] The glossary carries the seven new entries and the two cross-references.
- [ ] `cargo xtask ci` passes in the foreground, watched to completion.
