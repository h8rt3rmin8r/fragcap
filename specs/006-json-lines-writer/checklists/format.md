# Stream Correctness and Cross-Format Agreement Checklist

**Purpose**: Validate that the S07 requirements pin JSON validity, numeric
exactness, agreement with the pcapng profile, loss accounting, and determinism
well enough that a reviewer can judge compliance without reading the writer

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

**Depth**: Formal merge gate. This is the second of two formats that carry the
same facts. A defect here is either a wrong stream or, worse, a stream that
disagrees with the pcapng file about the same packet, which is the failure mode
the shared derivation exists to prevent.

**Note**: This checklist tests the requirements, not the implementation. An
item passes when the requirement is written well enough for a reviewer to judge
compliance, not when the code works.

## Stream shape

- [x] CHK001 Is the absence of an enclosing array required rather than implied
  by the format name? [Completeness, Spec §FR-001]
- [x] CHK002 Is the one-object-per-packet rule stated separately from the
  one-line rule, given that a serializer can satisfy one and not the other?
  [Clarity, Spec §FR-002, §FR-008]
- [x] CHK003 Are the header and trailer required, and required to be
  distinguishable from packet records by a mechanism a consumer can use before
  parsing? [Completeness, Spec §FR-003 to §FR-006]
- [x] CHK004 Is line termination specified for the final line, where writers
  most often differ? [Edge Case, Spec §FR-009]
- [x] CHK005 Is the absence of a trailer defined as meaningful rather than as
  an error? [Coverage, Edge Cases]

## Numeric exactness

- [x] CHK006 Is the timestamp representation specified beyond "a number", given
  that the obvious implementation loses precision? [Clarity, Spec §FR-011]
- [x] CHK007 Is the prohibition on a floating point path stated as a
  requirement rather than left to the implementer's judgement? [Completeness,
  Spec §FR-012]
- [x] CHK008 Is the rounding direction fixed? [Clarity, Spec §FR-013]
- [x] CHK009 Is fixed-width fractional output required, so a whole second does
  not render differently from a fractional one? [Consistency, Spec §FR-011,
  §US3]
- [x] CHK010 Is an unrepresentable timestamp an error rather than a clamped
  value, consistently with the pcapng writer? [Consistency, Spec §FR-014]

## Cross-format agreement

- [x] CHK011 Is reuse of the S06 derivation a requirement rather than an
  expectation? [Completeness, Spec §FR-022]
- [x] CHK012 Is agreement between the two formats stated as a testable property
  over all packets rather than as an intention? [Measurability, Spec §FR-023,
  §SC-002]
- [x] CHK013 Are the deliberate divergences between the formats enumerated, so
  a reviewer can tell an intended difference from a defect? [Clarity,
  Clarifications: `iface`, hex case]
- [x] CHK014 Is the reason `iface` differs from the pcapng rule recorded, given
  that it looks like an inconsistency? [Traceability, Clarifications]

## Endpoint naming

- [x] CHK015 Is the conflict between section 13.5's `src` and `dst` and the
  normalized `local` and `remote` of the flow key resolved explicitly rather
  than left to the implementer? [Conflict, Spec §FR-019a, §FR-019b]
- [x] CHK016 Is guessing wire order prohibited when direction is unknown?
  [Completeness, Spec §FR-019b]
- [x] CHK017 Is emitting both key pairs prohibited, so a consumer can dispatch
  on which is present? [Clarity, Spec §FR-019c]
- [x] CHK018 Is the no-flow-key case distinguished from the unknown-direction
  case? [Coverage, Spec §FR-019, Edge Cases]

## Escaping and validity

- [x] CHK019 Are the characters requiring escape enumerated rather than
  described as "special characters"? [Clarity, Spec §FR-026]
- [x] CHK020 Is the escape form for control characters without a short escape
  specified? [Completeness, Spec §FR-027]
- [x] CHK021 Is the treatment of non-ASCII specified, rather than left to
  produce either UTF-8 or escapes depending on the implementation?
  [Consistency, Spec §FR-028]
- [x] CHK022 Is validation required against a parser that is not this writer?
  [Coverage, Spec §FR-029, §FR-037]
- [x] CHK023 Is the prohibition on embedded newlines stated, given that it is
  what makes the format line-oriented at all? [Completeness, Spec §FR-008]

## Loss accounting

- [x] CHK024 Is every section 12.4 counter required in the trailer, rather than
  a selection? [Coverage, Spec §FR-030]
- [x] CHK025 Is presence-when-zero required, so a consumer can distinguish "no
  loss" from "not reported"? [Clarity, Spec §FR-031]
- [x] CHK026 Is retention of an unattributed packet required? [Completeness,
  Spec §FR-032]
- [x] CHK027 Is a refused record required to surface as an error rather than a
  silent skip? [Completeness, Spec §FR-033]

## Payload mode

- [x] CHK028 Is omission of the key required, rather than an empty value?
  [Clarity, Spec §FR-024]
- [x] CHK029 Is a zero-length payload distinguishable from a suppressed one?
  [Edge Case, Spec §US5]
- [x] CHK030 Is the scope of the mode bounded to exactly one key?
  [Measurability, Spec §FR-025, §SC-006]

## Determinism and goldens

- [x] CHK031 Is key order fixed, given that a map would not supply one?
  [Completeness, Spec §FR-010]
- [x] CHK032 Is hex case fixed? [Clarity, Clarifications]
- [x] CHK033 Is byte-identical output required across runs? [Measurability,
  Spec §FR-038]
- [x] CHK034 Is golden coverage across the corpus decided rather than left to
  a representative sample? [Clarity, Spec §FR-040]
- [x] CHK035 Is failure output required to locate the divergence?
  [Measurability, Spec §FR-039]
- [x] CHK036 Are the sources of nondeterminism this writer could acquire
  enumerated, as S06 was required to do? [Gap, see notes]

## Constitution and scope

- [x] CHK037 Is the no-runtime-dependency constraint stated as a requirement?
  [Completeness, Spec §FR-036]
- [x] CHK038 Is the dev-dependency confined to test code by a requirement
  rather than by convention? [Clarity, Spec §FR-037]
- [x] CHK039 Is the session anchor gap recorded rather than silently omitted?
  [Traceability, Out of Scope]
- [x] CHK040 Is a glossary entry required for terms this slice introduces?
  [Completeness, Done When]

## Notes

Forty items. Thirty-nine pass. One fails, and it is the same class of finding
the S06 checklist caught, which is why it was looked for.

**CHK036.** FR-038 requires byte-identical output and FR-039 pins it with
goldens, but unlike S06 the spec does not enumerate what could make this writer
nondeterministic. Two candidates exist and neither is covered by an existing
requirement. The header object declares an interface set, and if that set were
iterated from an unordered collection the header would vary between runs. And
the trailer is the only record whose content comes from outside the packet
stream, so if any counter were sampled rather than supplied it would vary.
Neither is likely given the design, but "unlikely given the design" is exactly
what was said about the Interface Statistics Block timestamp in S06 before it
was written from a clock.

**Resolved, same day.** FR-038a states the general rule that the writer reads
no ambient input. FR-038b requires the header interface set to be emitted in
declaration order from an ordered collection. FR-038c requires every trailer
counter to come from the supplied snapshot rather than being sampled. Forty of
forty items now pass.

The value of this item was not that a defect existed, but that the spec was
relying on the implementer making the same choices S06 had to be told to make.
