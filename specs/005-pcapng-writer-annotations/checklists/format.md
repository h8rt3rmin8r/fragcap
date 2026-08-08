# Format Correctness and Output Fidelity Checklist

**Purpose**: Validate that the S06 requirements pin structural validity,
annotation grammar, fidelity marking, loss accounting, and determinism well
enough that a reviewer can judge compliance without reading the writer, and
that no later slice inherits an undecided question about the file format

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

**Depth**: Formal merge gate. This slice fixes the on-disk format. A format
defect is not contained by the slice that introduces it: it is inherited by
every capture ever written, and by S07 and S08, which restate its rules.

**Note**: This checklist tests the requirements, not the implementation. An
item passes when the requirement is written well enough for a reviewer to judge
compliance, not when the code works.

## Structural validity, as an unmodified reader sees it

- [x] CHK001 Are the required block types enumerated rather than left to the
  reader to infer from the format? [Completeness, Spec §FR-001, §FR-004,
  §FR-007, §FR-008]
- [x] CHK002 Is block ordering constrained, specifically that an interface is
  declared before any packet referencing it? [Completeness, Spec §FR-004]
- [x] CHK003 Is the leading and trailing length agreement stated as a
  requirement rather than assumed from the format? [Clarity, Spec §FR-011]
- [x] CHK004 Are alignment and padding requirements explicit about what padding
  is excluded from? [Clarity, Spec §FR-010]
- [x] CHK005 Is option list termination required? [Completeness, Spec §FR-012]
- [x] CHK006 Is byte order fixed rather than left to the host, and is the
  reason recorded? [Completeness, Spec §FR-013, Clarifications]
- [x] CHK007 Is interface identifier assignment defined, including its starting
  value and its ordering rule? [Clarity, Spec §FR-006]
- [x] CHK008 Is validation required to run by a path independent of the
  writer's own encoding code, so the writer cannot grade itself? [Coverage,
  Spec §FR-039]
- [x] CHK009 Is the content of the Section Header Block comment specified
  beyond naming what it declares, given that a golden comparison pins its
  exact bytes? [Gap, Spec §FR-003]
- [x] CHK010 Is the timestamp carried in the Interface Statistics Block
  specified, given that a wall clock reading there would break byte-identical
  output? [Gap, Spec §FR-008, §FR-037]

## Annotation grammar

- [x] CHK011 Is the sentinel required explicitly rather than shown only in an
  example? [Completeness, Spec §FR-015]
- [x] CHK012 Is key casing constrained? [Clarity, Spec §FR-016]
- [x] CHK013 Is the presence rule for every key in the section 13.3 table
  covered by a requirement? [Coverage, Spec §FR-017 to §FR-021]
- [x] CHK014 Are the characters requiring percent-encoding enumerated rather
  than described as "special characters"? [Clarity, Spec §FR-022, §FR-023]
- [x] CHK015 Is the widening beyond the three characters the specification
  names justified and recorded as a deviation? [Traceability, Clarifications]
- [x] CHK016 Is a decoder required, so the grammar has a second independent
  implementation to disagree with? [Completeness, Spec §FR-024]
- [x] CHK017 Is round-trip fidelity stated as a requirement over all inputs
  rather than over the example? [Measurability, Spec §FR-024, §SC-005]
- [x] CHK018 Is deriving the key set separated from rendering it, so section
  13.5's writer cannot restate the rules differently? [Consistency, Spec
  §FR-025]
- [x] CHK019 Is the order of keys within an annotation fixed, given that
  byte-identical output requires it and a map iteration would not supply it?
  [Gap, Spec §FR-037]
- [x] CHK020 Is the case of percent-encoded hexadecimal digits fixed, given
  that both cases are valid and only one produces a stable golden? [Gap, Spec
  §FR-022, §FR-037]
- [x] CHK021 Is the encoding of an empty value defined, given that a process
  name is a string and the type does not forbid an empty one? [Edge Case, Gap]

## Fidelity marking

- [x] CHK022 Is each fidelity value given a meaning tied to how attribution was
  obtained rather than to how confident the writer is? [Clarity, Spec §FR-026
  to §FR-028]
- [x] CHK023 Is inference, upgrading, and defaulting of a fidelity value
  prohibited rather than merely undescribed? [Completeness, Spec §FR-029]
- [x] CHK024 Is the absence of identity keys required for an unattributed
  packet, rather than leaving an empty or placeholder value permissible?
  [Clarity, Spec §FR-017, §FR-028]
- [x] CHK025 Is retention of an unattributed packet required, so P-4 is
  satisfied by a requirement and not by an implementation habit?
  [Completeness, Spec §FR-030]
- [x] CHK026 Is the contradiction between the specification's three direction
  values and the two-variant optional type in core resolved explicitly rather
  than left for the implementer? [Conflict, Spec §FR-019a, Clarifications]
- [x] CHK027 Is the unreachability of `local` in this slice recorded as a known
  gap rather than left to look like an untested path? [Traceability, Out of
  Scope]
- [x] CHK028 Is the independence of `role` and `stage` resolved against the
  specification's paired presentation? [Conflict, Spec §FR-018,
  Clarifications]

## Loss accounting

- [x] CHK029 Is every counter of section 12.4 required to be recoverable from
  the file, rather than only the three the standard fields cover? [Coverage,
  Spec §FR-031, §SC-006]
- [x] CHK030 Is misreporting a fragcap loss as an upstream loss prohibited
  explicitly? [Completeness, Spec §FR-032]
- [x] CHK031 Is the carrier for the non-standard counters specified, rather
  than left as "somewhere in the file"? [Clarity, Spec §FR-031]
- [x] CHK032 Is a packet the writer itself refuses required to surface as an
  error rather than a silent discard? [Completeness, Spec §FR-033]
- [x] CHK033 Is the behavior of an unfinished writer defined, so a truncated
  capture is bounded rather than undefined? [Edge Case, Spec §US4]

## Determinism and golden coverage

- [x] CHK034 Is byte-identical output required across runs? [Measurability,
  Spec §FR-037]
- [x] CHK035 Is byte-identical output required across architectures, not only
  across runs on one machine? [Coverage, Spec §FR-013, §SC-007]
- [x] CHK036 Is golden coverage across the corpus decided, rather than left as
  "representative fixtures"? [Clarity, Spec §FR-038a]
- [x] CHK037 Is a drift check required in the ordinary gate, so a hand-edited
  golden fails rather than passing quietly? [Completeness, Spec §FR-038a]
- [x] CHK038 Is failure output required to locate the divergence, rather than
  only reporting that files differ? [Measurability, Spec §FR-038]
- [x] CHK039 Are the sources of nondeterminism a writer could accidentally
  acquire identified, rather than covered by a general instruction to be
  deterministic? [Gap, see CHK010, CHK019, CHK020]

## Constitution gates

- [x] CHK040 Is the dependency direction stated as a requirement, so P-2 is
  checked rather than assumed? [Completeness, Spec §FR-036]
- [x] CHK041 Is the compatibility-over-richness constraint traceable to a
  requirement a reviewer can apply, rather than only to the overview prose?
  [Traceability, Spec §FR-014, §SC-001, §SC-002]
- [x] CHK042 Is the lossy narrowing of timestamps declared rather than hidden,
  and confined to one inspectable site? [Clarity, Spec §FR-009a, Edge Cases]
- [x] CHK043 Are the alternatives to failing on an unrepresentable timestamp
  rejected on the record, rather than left as an implementer's choice?
  [Completeness, Spec §FR-009b, Clarifications]
- [x] CHK044 Is a glossary entry required for terms this slice introduces?
  [Completeness, Done When]
- [x] CHK045 Are the deviations from the master specification enumerated for
  promotion to section 29, rather than resolved silently? [Traceability, Done
  When]

## Scope boundaries

- [x] CHK046 Is the boundary against section 13.5 drawn in a way that still
  obliges this slice to make the derivation reusable? [Clarity, Out of Scope,
  Spec §FR-025]
- [x] CHK047 Is the session anchor gap recorded rather than silently omitted?
  [Traceability, Out of Scope]
- [x] CHK048 Is pcapng reading scoped out while still permitting the structural
  validation FR-039 requires? [Consistency, Out of Scope, Spec §FR-039]

## Notes

Forty-eight items. Forty-three pass. Five fail, and four of the five are the
same defect wearing different clothes.

**CHK009, CHK010, CHK019, CHK020 are one finding.** FR-037 requires
byte-identical output across runs and FR-038 pins it with committed goldens,
but four inputs to those bytes are unspecified: the text of the Section Header
Block comment, the timestamp in the Interface Statistics Block, the order of
keys within an annotation, and the case of percent-encoded hexadecimal digits.
Each is individually a small omission and each is individually enough to make
the golden comparison either impossible or falsely green. The Interface
Statistics Block timestamp is the dangerous one, because the obvious
implementation reads the wall clock, which would make every golden fail on the
second run and invite the fix of deleting the goldens. S04 met the same class
of defect from the reading side and answered it the same way: name the ambient
inputs rather than instruct the implementer to be deterministic.

**CHK021 is separate and smaller.** An empty attribution value is
representable in the type system and undefined in the grammar. It is unlikely
to occur, which is precisely why it should be decided now rather than by
whichever code path first encounters it.

All five are fixable in the specification without changing the slice's scope,
and are addressed before the plan phase rather than deferred.

**Resolved, same day.** FR-003 now fixes the Section Header Block comment as
`fragcap:profile=0.1.0`. FR-008a derives the Interface Statistics Block
timestamp from the last packet written rather than from a clock, and says the
writer reads no clock at all. FR-016a fixes key order to the section 13.3
table order. FR-023a fixes percent-encoding to uppercase on output while
requiring the decoder to accept either case, since it reads files other tools
wrote. FR-023b defines an empty value as written rather than omitted, because
omitting `proc` reports that the packet was not attributed, which is a
different fact.

Forty-eight of forty-eight items now pass. The determinism finding is the
reason this checklist earned its place: nothing in the four unspecified inputs
would have failed a test written from the spec as it stood, and the Interface
Statistics Block timestamp in particular would have passed every test on the
first run and failed every test thereafter.
