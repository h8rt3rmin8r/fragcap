# Fixture Fidelity and Determinism Checklist

**Purpose**: Validate that the S04 requirements pin determinism, accounting,
privacy, and reviewability well enough that every later slice can build tests
on this substrate without re-examining it

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

**Depth**: Formal merge gate. Everything from S06 onward is tested through this
corpus, so a weakness here is inherited rather than contained.

**Note**: This checklist tests the requirements, not the implementation. An
item passes when the requirement is written well enough for a reviewer to judge
compliance, not when the code works.

## Determinism, and whether it can carry golden comparison

- [x] CHK001 Is determinism stated as a requirement on the source rather than
  left implied by reading a file? [Completeness, Spec §FR-019]
- [x] CHK002 Is determinism required across platforms, not only across runs on
  one machine? [Coverage, Spec §FR-019, §SC-002]
- [x] CHK003 Are the ambient inputs a generator could accidentally depend on
  enumerated, rather than covered by a general instruction to be
  deterministic? [Clarity, Spec §FR-032a]
- [x] CHK004 Is the timestamp base required to be a constant, given that a
  clock reading is the likeliest way a generator becomes nondeterministic?
  [Gap, Spec §FR-032a]
- [x] CHK005 Is determinism required for every fixture rather than demonstrated
  on a representative one? [Measurability, Spec §SC-002]
- [x] CHK006 Are byte order and timestamp resolution required to be properties
  of the file rather than of the reading host, so the same capture reads the
  same way anywhere? [Consistency, Spec §FR-002, §FR-003, §SC-003]

## Reader accounting under P-4

- [x] CHK007 Is every way the reader can decline to deliver a record as-is
  given its own named counter rather than an aggregate?
  [Completeness, Spec §FR-012]
- [x] CHK008 Is a truncated final record distinguished from a record declaring
  more bytes than the file holds, given that one is a damaged file and the
  other a lying record? [Clarity, Spec §FR-008, §FR-009]
- [x] CHK009 Is it stated, for each counted cause, whether the record is still
  delivered or not? [Gap, Spec §FR-008, §FR-009, §FR-010, §FR-011]
- [x] CHK010 Are the reader's counters required to stay out of the shared
  source statistics type, so a backend's report is not mixed with fragcap's
  accounting? [Consistency, Spec §FR-012, §FR-016a]
- [x] CHK011 Is any total over the reader counters required to be derived
  rather than stored? [Measurability, Spec §FR-013]
- [x] CHK012 Is reachability of every counted cause required as an outcome, so
  a counter nothing can advance would be caught?
  [Measurability, Spec §SC-004]
- [x] CHK013 Is failing to open a file distinguished from opening a file that
  holds no packets? [Edge Case, Spec §FR-005, §Edge Cases]

## P-9 fidelity across a file-backed source

- [x] CHK014 Is the reader forbidden from altering a record it finds unusual,
  as opposed to merely required to count it?
  [Completeness, Spec §FR-007]
- [x] CHK015 Is an out-of-order timestamp required to be delivered unreordered,
  given that sorting is the obvious and wrong convenience?
  [Edge Case, Spec §FR-007, §Edge Cases]
- [x] CHK016 Is the reader forbidden from reconciling a record whose captured
  length exceeds its on-wire length, rather than left free to repair it?
  [Gap, Spec §FR-010]
- [x] CHK017 Is the original on-wire length required to stay separate from the
  captured length, so truncation recorded in a fixture survives into the
  packet? [Completeness, Spec §FR-006]
- [x] CHK018 Is a resolution conversion forbidden from rounding, given that
  microsecond and nanosecond files must both round-trip?
  [Clarity, Spec §FR-003]
- [x] CHK019 Is a zero-length record required to be delivered rather than
  treated as absence? [Edge Case, Spec §Edge Cases]
- [x] CHK020 Is a file declaring a link type fragcap cannot parse required to
  be read anyway, so the parser's own rejection path stays reachable through
  the corpus? [Coverage, Spec §Edge Cases]

## P-3 separation across two new backends

- [x] CHK021 Is the replay source required to contain no attribution logic,
  stated rather than assumed from where it lives?
  [Completeness, Spec §FR-018]
- [x] CHK022 Is the scripted attributor required to contain no packet
  acquisition, given that it is the one component that could plausibly read the
  fixture itself? [Completeness, Spec §FR-026]
- [x] CHK023 Are the two required to live in the crates specification section
  8.2 assigns them, rather than wherever is convenient?
  [Traceability, Spec §FR-014, §FR-020]

## The scripted attributor, and whether it can disagree with S10

- [x] CHK024 Is the script's flow identity required to match the shape a socket
  table can actually answer, rather than an independent notion?
  [Consistency, Spec §FR-021a]
- [x] CHK025 Is it made impossible to script a UDP attribution that requires a
  remote endpoint, which the real attributor could never make?
  [Completeness, Spec §FR-021a]
- [x] CHK026 Is the wildcard bind allowance required of the double, so a test
  passing against it is a test the real attributor must also pass?
  [Consistency, Spec §FR-021b]
- [x] CHK027 Is "no owner" distinguished from "flow not mentioned", or is the
  spec explicit that both resolve to nothing?
  [Clarity, Spec §FR-022]
- [x] CHK028 Is the time source specified, given that the seam carries no
  timestamp? [Gap, Spec §FR-022a, §Clarifications]
- [x] CHK029 Is widening the seam explicitly refused rather than left open, so
  a later contributor does not add a parameter to it for convenience?
  [Traceability, Spec §FR-022a, §SC-006b]
- [x] CHK030 Is the behavior of a script with no time windows defined, so a
  caller that never sets a clock is not undefined?
  [Edge Case, Spec §FR-022a]
- [x] CHK031 Are script times required to share a base with the fixture's
  packet timestamps, so the two cannot disagree about when something happened?
  [Consistency, Spec §FR-022b]
- [x] CHK032 Is an ambiguous script required to fail rather than resolve
  silently? [Completeness, Spec §FR-024]

## Corpus privacy, stated testably

- [x] CHK033 Is the set of permitted addresses enumerated rather than described,
  so the rule is mechanical? [Clarity, Spec §FR-029]
- [x] CHK034 Does the permitted set actually admit every fixture the corpus
  requires, including the loopback one? [Conflict, Spec §FR-029]
- [x] CHK035 Is "contains no account identifier or session token" expressed as
  something a test can evaluate, rather than as a judgment?
  [Measurability, Spec §FR-029a]
- [x] CHK036 Are link layer addresses covered, or only network layer ones?
  [Coverage, Spec §FR-029]
- [x] CHK037 Is the prohibition on real captured traffic stated as a
  requirement rather than inherited from the contributing guide?
  [Completeness, Spec §FR-028]

## Reviewability and drift

- [x] CHK038 Is the generator required to be the readable record of what each
  fixture contains, rather than the fixture being its own documentation?
  [Completeness, Spec §FR-032, §User Story 5]
- [x] CHK039 Is drift detection required to run in the ordinary gate rather
  than as a command someone remembers? [Measurability, Spec §FR-033]
- [x] CHK040 Does drift detection cover the attribution scripts and not only
  the capture files? [Gap, Spec §FR-033]
- [x] CHK041 Is a fixture without a script, and a script without a fixture,
  required to be reported? [Coverage, Spec §FR-034]
- [x] CHK042 Is the size ceiling a number rather than a judgment, and does it
  cover the corpus as well as each file?
  [Measurability, Spec §FR-031, §SC-010]
- [x] CHK043 Is each fixture's stated condition required to be asserted, so a
  fixture that stops exercising it fails here rather than in a later slice?
  [Completeness, Spec §FR-035, §SC-008]
- [x] CHK044 Is the generator required to stay out of every published crate, so
  test scaffolding is not shipped to consumers?
  [Completeness, Spec §FR-032]

## Dependencies and assumptions

- [x] CHK045 Is the constraint that no dependency is added stated for both the
  script format and the generator?
  [Completeness, Spec §FR-025, §FR-032]
- [x] CHK046 Is the repository's existing exclusion of capture files, and its
  re-inclusion under the fixture directory, recorded as something to verify
  rather than assume? [Assumption, Spec §Assumptions]
- [x] CHK047 Is the divergence from section 25.3 over the burst fixture
  recorded for promotion rather than silently applied?
  [Traceability, Spec §Clarifications]
- [x] CHK048 Are the fixtures this slice builds but does not consume identified,
  so their lack of a caller is not read as an oversight?
  [Clarity, Spec §Assumptions]

## Notes

All forty-eight items pass against the spec as it now stands. Five were failing
when the checklist was drafted, and each was a real gap rather than a wording
problem.

- **CHK034** failed on a genuine conflict. FR-029 required every address to
  come from the ranges reserved for documentation, and `loopback.pcap` cannot
  exercise direction ambiguity without a loopback address, which is not one of
  those ranges. The requirement forbade a fixture the same document mandates.
  FR-029 now enumerates both sets and says why loopback is admissible.
- **CHK035** failed because "no payload resembling an account identifier or
  session token" is not something a test can evaluate; no assertion recognizes
  what a session token looks like. FR-029a inverts it: payloads are a documented
  filler pattern, and anything else fails. The property is now mechanical.
- **CHK016** failed because FR-010 counted a record whose captured length
  exceeds its on-wire length without saying whether it is delivered, leaving
  "repair the lengths so they agree" open. That is precisely the well-intentioned
  alteration P-9 names. FR-010 now requires delivery with both lengths as
  recorded.
- **CHK040** failed because FR-033 said the check regenerates "the corpus"
  without stating whether scripts were included. A script that has drifted from
  its fixture misattributes as quietly as a drifted fixture. Now explicit.
- **CHK010** partly failed: nothing said what the replay source reports in the
  backend drop counters. Reporting its own skips there would fold fragcap's
  accounting into a backend's observation, which is what S02 kept the two types
  apart to prevent. FR-016a now requires them to be zero.

Two items pass on a reading worth restating so the analyze gate does not have
to rediscover it.

- **CHK028** and **CHK029** together cover the one architectural pressure in
  this slice. The attributor seam has no timestamp, a scripted attributor needs
  one, and the obvious fix is to widen the seam. The spec refuses that and puts
  the clock on the double, and SC-006b asserts the seam is unchanged after the
  slice. If a later slice does widen it, that assertion is where it will be
  noticed.
- **CHK020** passes because the spec requires an unparseable link type to be
  read rather than rejected. That looks permissive and is deliberate: the
  parser counts unsupported link types, and if the reader refused such files
  that counter would be unreachable through the corpus.
