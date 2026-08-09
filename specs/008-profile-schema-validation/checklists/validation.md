# Profile Validation Completeness Checklist

**Purpose**: Validate that the S05 requirements pin every check specification
section 15.4 names, every check added beyond it, the exact meaning of the
ambiguity decision, and the resolution order well enough that a reviewer can
judge compliance without reading the parser

**Created**: 2026-08-09

**Feature**: [spec.md](../spec.md)

**Depth**: Formal merge gate. Two of this slice's failure modes are silent in
the same way S08's were. A validator that misses a check lets a profile through
that produces a successful run and an empty capture, and section 15.4 exists
because that has already happened to a real title. A validator that reports one
fault at a time is not wrong, only useless, and uselessness is the thing least
likely to be caught by a test.

**Note**: This checklist tests the requirements, not the implementation. An item
passes when the requirement is written well enough for a reviewer to judge
compliance, not when the code works.

## Reporting every problem

- [x] CHK001 Is reporting every problem stated as a requirement rather than left
  to be inferred from section 15.4's prose? [Completeness, Spec §FR-013, §US2]
- [x] CHK002 Is the one exception to accumulation, a TOML syntax fault,
  identified and justified rather than left as an implementation accident?
  [Clarity, Spec §FR-015, §Clarifications]
- [x] CHK003 Is accumulation required across kinds, so that a profile with both
  structural and semantic faults reports both rather than the earlier kind only?
  [Coverage, Spec §US2]
- [x] CHK004 Is diagnostic ordering required to be deterministic, given that a
  set built from a map would otherwise vary between runs? [Gap closed, Spec
  §FR-049, §SC-009]
- [x] CHK005 Is the diagnostic required to carry something a test can assert on
  that is not message prose? [Measurability, Spec §FR-047, §US2]
- [x] CHK006 Is the boundary of the stable surface drawn, so that a later
  reformatting of a location string is not a breaking change? [Ambiguity, Spec
  §FR-050a]
- [x] CHK007 Is the empty-versus-non-empty invariant on the diagnostic set
  stated, ruling out a failure that reports nothing? [Completeness, Spec
  §FR-050]
- [x] CHK008 Is it required that a valid profile cannot be obtained without
  validation, rather than validation being a method callers are trusted to call?
  [Clarity, Spec §FR-011, §Overview]

## The section 15.4 check inventory

- [x] CHK009 Schema version support: is an unsupported version required to
  suppress the other semantic diagnostics, so that a future-schema profile does
  not arrive as a wall of unknown-key faults? [Coverage, Spec §FR-012, §US5]
- [x] CHK010 Required field presence: is every missing field required to be
  reported rather than the first? [Completeness, Spec §FR-013]
- [x] CHK011 Type correctness: is the diagnostic required to name both the
  expected and the found type? [Clarity, Spec §FR-014]
- [x] CHK012 Role name uniqueness: is the diagnostic required to name both
  colliding stages? [Measurability, Spec §FR-016]
- [x] CHK013 At most one terminal stage: is the limit stated as a requirement?
  [Completeness, Spec §FR-017]
- [x] CHK014 `descends_from` resolution: is it required to name a role declared
  in the same profile? [Completeness, Spec §FR-018]
- [x] CHK015 Regular expression compilation: is it required to compile with the
  engine that will later evaluate it, rather than with any engine? [Gap closed,
  Spec §FR-019, §Clarifications]
- [x] CHK016 Glob compilation: is the glob syntax defined completely enough that
  "well-formed" is decidable? [Ambiguity, Spec §FR-020, §Clarifications]
- [x] CHK017 Duration parsing: is the accepted grammar stated exactly, including
  what is refused? [Measurability, Spec §FR-043, §FR-044]
- [x] CHK018 At least one non-service stage: is the requirement stated with the
  reason from section 10.4, so that a later reader does not read it as
  arbitrary? [Traceability, Spec §FR-022, §Edge Cases]
- [x] CHK019 Are the closed value sets for `lifecycle` and `mode` required, and
  are the accepted values enumerated? [Completeness, Spec §FR-023]
- [x] CHK020 Is an empty `match` table refused, given that it would match every
  process on the system? [Edge Case, Spec §FR-024, §Edge Cases]
- [x] CHK021 Is the predicate name set closed, so that a misspelled predicate is
  a fault rather than a silently narrower stage? [Gap closed, Spec §FR-007,
  §FR-010]

## Checks added beyond the section 15.4 list

- [x] CHK022 Is it stated that section 15.4's list is a floor rather than a
  ceiling, so that the additions are not read as scope creep? [Clarity, Spec
  §Clarifications]
- [x] CHK023 Is each addition justified by the same silent-failure class the two
  named checks were added for, rather than by tidiness? [Traceability, Spec
  §Clarifications]
- [x] CHK024 `capture.roles`: is a role naming no declared stage refused, and is
  an empty list refused too? [Coverage, Spec §FR-027, §Edge Cases]
- [x] CHK025 Terminal lifecycle: is `terminal` required to sit on a `session`
  stage, with the consequence of the alternative stated? [Completeness, Spec
  §FR-026, §Edge Cases]
- [x] CHK026 Is the `descends_from` cycle check required, and is every role in a
  cycle required to be named? [Gap closed, Spec §FR-028, §Edge Cases]
- [x] CHK027 Are the additions marked as candidates for promotion into section
  15.4 under the deviation process, rather than left as a local divergence?
  [Traceability, Spec §Clarifications]

## The ambiguous image match decision

- [x] CHK028 Is the decision required to be exact rather than conservative, and
  is the cost of each direction of error recorded? [Clarity, Spec §FR-029,
  §Clarifications]
- [x] CHK029 Is the firing condition stated precisely enough to decide a given
  stage pair, including the "at least one matches on `exe` alone" qualifier?
  [Ambiguity, Spec §FR-030, §Clarifications]
- [x] CHK030 Is the accepted case required, so that the section 15.2 profile for
  the second focal title is not refused by the check meant to protect it?
  [Coverage, Spec §FR-031, §US3]
- [x] CHK031 Is case-insensitivity carried into the intersection decision, and
  not only into matching? [Gap closed, Spec §FR-029, §US3]
- [x] CHK032 Is section 15.4's second clause, an image name the profile
  "elsewhere indicates recurs", accounted for rather than left unimplemented?
  [Completeness, Spec §Clarifications]
- [x] CHK033 Is the diagnostic required to name both stages and the remedy,
  given that the author has to act on it? [Measurability, Spec §FR-032]
- [x] CHK034 Is the runtime half of section 15.4's ambiguity check explicitly
  deferred, rather than silently omitted? [Clarity, Spec §Out of Scope]
- [x] CHK035 Is the cost of the check bounded by limits the crate enforces,
  rather than merely stated, so that a profile cannot exhaust the process
  refusing it? [Gap closed, Spec §FR-003a, §FR-020a, §FR-029a, §SC-018]

## Resolution order

- [x] CHK036 Are all four steps required in order with first match winning?
  [Completeness, Spec §FR-033]
- [x] CHK037 Is the shadowing direction required explicitly, given that section
  15.3's purpose is that a user file beats a bundled one? [Traceability, Spec
  §US4]
- [x] CHK038 Is it required that the search locations come from the caller, so
  that this crate acquires no platform opinion? [Clarity, Spec §FR-034,
  §Clarifications]
- [x] CHK039 Is the distinction between an operator naming a file and a name
  being interpolated into a search path drawn, and is the slug check required on
  the second only? [Ambiguity, Spec §FR-035, §FR-036]
- [x] CHK040 Is a traversal-shaped reference required to be refused before any
  path is joined, rather than after a failed open? [Coverage, Spec §FR-036,
  §SC-007]
- [x] CHK041 Is the difference between a missing directory (skip) and an
  unreadable candidate file (error) stated, given that treating the second as a
  skip would silently select a different profile? [Gap closed, Spec §FR-037,
  §FR-038, §Edge Cases]
- [x] CHK042 Is a successful resolution required to report which source supplied
  the profile, so that an operator can tell which file they got? [Measurability,
  Spec §FR-039, §US4]
- [x] CHK043 Is a failed resolution required to name everywhere it looked?
  [Completeness, Spec §FR-040]
- [x] CHK044 Is a duplicate `game.id` in the bundled set refused, given that
  step four selects on it? [Edge Case, Spec §FR-041, §Edge Cases]
- [x] CHK045 Is the directory-as-reference case specified for both a bare name
  and a name carrying a separator? [Edge Case, Spec §FR-046c, §Edge Cases]

## Duration grammar and its placement

- [x] CHK046 Is the crate that owns duration parsing stated, with the reason,
  rather than left to whichever slice writes it first? [Clarity, Spec §FR-042,
  §Clarifications]
- [x] CHK047 Is the consequence of the alternative placement recorded, so that
  moving it later is a visible reversal? [Traceability, Spec §Clarifications]
- [x] CHK048 Is a bare integer refused, and is the reason given rather than
  asserted? [Completeness, Spec §FR-044, §Clarifications]
- [x] CHK049 Is zero refused, and is it tied to the equivalent S08 decision
  rather than argued afresh? [Consistency, Spec §FR-045, §Clarifications]
- [x] CHK050 Is overflow required to be refused rather than saturated or
  wrapped? [Gap closed, Spec §FR-046]
- [x] CHK051 Is the deliberate narrowness of the grammar recorded with the
  argument that widening later is the compatible direction? [Assumption, Spec
  §Clarifications, §Out of Scope]
- [x] CHK052 Is it stated whether the parsed profile keeps the literal text, so
  that fidelity against constitution P-9 is settled rather than argued at
  review? [Ambiguity, Spec §FR-046a, §Clarifications]

## Dependency decisions

- [x] CHK053 Is each new runtime dependency required to be recorded with its
  license and reason, given that this is the first addition since S02?
  [Completeness, Spec §FR-053, §SC-012]
- [x] CHK054 Is the distinction from the hand-rolled formats argued rather than
  asserted, so that "why not hand-roll TOML too" is answered once?
  [Traceability, Spec §Clarifications]
- [x] CHK055 Is the reason the glob matcher is hand-rolled while the regular
  expression engine is not recorded, given that the pairing looks inconsistent
  at first reading? [Clarity, Spec §Clarifications]
- [x] CHK056 Is the requirement that the validating engine and the evaluating
  engine be the same one stated, rather than left as a coincidence of crate
  choice? [Gap closed, Spec §FR-019, §Clarifications]
- [x] CHK057 Is core's allowlist required to be unchanged, so that a duration
  module in core cannot quietly bring a dependency with it? [Consistency, Spec
  §FR-054, §SC-011]
- [x] CHK075 Is the parser's conformance stated as the constructs a profile can
  contain, rather than as whole-language conformance the chosen crate does not
  have? [Ambiguity, Spec §FR-002, §Clarifications]
- [x] CHK076 Is the known datetime divergence required to be pinned by a test,
  rather than recorded only in prose where a later reader would rediscover it as
  a surprise? [Gap closed, Spec §FR-002, §SC-019]
- [x] CHK077 Is the literal-string form a Windows path needs required and
  tested, given that it is the form an author will actually write and the one a
  subset parser is most likely to get wrong? [Gap closed, Spec §FR-002a,
  §SC-019]

## Input handling and limits

- [x] CHK058 Is a size limit stated as a number rather than as an intention, and
  is it required to apply before the contents are read? [Measurability, Spec
  §FR-046b, §SC-015]
- [x] CHK059 Is the symbolic link position stated as a decision, so that its
  absence is not read as an oversight? [Clarity, Spec §FR-046d,
  §Clarifications]
- [x] CHK060 Is a pathological regular expression required to be refused through
  the engine's own limit rather than a second opinion? [Consistency, Spec
  §FR-046e, §SC-016]
- [x] CHK061 Is the trust posture stated, so that a reviewer knows whether a
  profile is being treated as a security boundary? [Assumption, Spec
  §Clarifications]
- [x] CHK062 Is the `game.id` charset constrained, and is the constraint applied
  at both validation and resolution rather than at one? [Coverage, Spec
  §FR-025, §FR-036, §Clarifications]

## Constitution obligations as they bind this slice

- [x] CHK063 P-2: Is the duration module in core required to add no dependency
  there? [Traceability, Spec §FR-042, §FR-054]
- [x] CHK064 P-4 and P-9: Is the unknown-key rejection tied to the silent
  narrowing it prevents, rather than presented as strictness for its own sake?
  [Clarity, Spec §FR-010, §Clarifications]
- [x] CHK065 P-5: Is strictness paired with the schema version that makes it
  safe, so that a growable format is not traded for a tidy parser? [Consistency,
  Spec §FR-012, §Clarifications]
- [x] CHK066 P-6: Are the terms needing glossary entries named individually,
  rather than left as "every new term"? [Clarity, Spec §FR-051]
- [x] CHK067 P-9: Is the prohibition on normalizing a declared value stated,
  given that case-folding an `exe` pattern or trimming a path would be the
  natural convenience? [Completeness, Spec §FR-009]
- [x] CHK068 P-1: Is it stated that nothing here observes a process, so that a
  "helpful" existence check on an image name is a visible violation? [Gap
  closed, Spec §FR-055]

## Scope discipline

- [x] CHK069 Is predicate evaluation excluded explicitly, rather than left
  ambiguous by the crate's own description naming matching? [Clarity, Spec §Out
  of Scope]
- [x] CHK070 Is the command surface excluded, with the note that this slice
  supplies the values it will print? [Coverage, Spec §Out of Scope]
- [x] CHK071 Are the bundled profiles excluded with a reason, given that section
  15.5 ships them at the same version? [Traceability, Spec §Clarifications,
  §Out of Scope]
- [x] CHK072 Is the `[capture]` key set closed at the five section 15.2 names,
  with the reason a sixth is not added speculatively? [Assumption, Spec
  §FR-005, §Clarifications]
- [x] CHK073 Is the absence of logging stated as a requirement, so that adding a
  logging call is a visible scope change? [Consistency, Spec §FR-052]
- [x] CHK074 Is size-literal parsing excluded, so that it is not added alongside
  duration by association? [Clarity, Spec §Out of Scope]

## Outstanding

None. Eleven items are marked "Gap closed" because the requirement they check
was added after the first draft: CHK004, CHK015, CHK021, CHK026, CHK031, CHK041,
CHK050, CHK056, and CHK068 came from the clarify pass, and CHK076 and CHK077
came from the analyze gate.

CHK075 is the one worth reading twice. The first draft of FR-002 required a
parser that "implements the language rather than a subset of it", and the
analyze gate measured the chosen crate refusing TOML datetimes, which made the
requirement false rather than merely optimistic. The requirement was corrected
to what is both true and sufficient. That is the outcome the gate exists for: a
claim that would have shipped unexamined was measured instead, and the artifact
changed rather than the reading of it.

CHK035 was answered wrongly and is now answered again. It originally recorded
the ambiguity check's cost as a stated bound rather than a capped one, on the
reasoning that the file size limit bounded it. Pull request 11's review showed
that the file limit bounds each factor and not their product, so the pass was
unbounded in practice: two half-megabyte patterns inside a one mebibyte profile
ask for about 10^12 table cells. The requirement now demands enforced limits on
pattern length and stage count, FR-020a, FR-003a, and FR-029a, and the item
passes against that instead.

The lesson is worth carrying to the next checklist: an item that asks whether a
cost is stated is weaker than one that asks whether it is bounded, and for
anything reading a file an operator did not write, only the second is worth
having.
