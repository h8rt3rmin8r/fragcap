# Checklist: Doctor Output Correctness and Presentation Robustness

**Purpose**: Validate that the requirements for doctor's output are complete,
unambiguous, and consistent before implementation.
**Created**: 2026-08-14
**Feature**: [spec.md](../spec.md)

## Interface Truthfulness

- [x] CHK001 Are the fields a listed interface must carry (name, address,
  up/down, virtual marker) explicitly enumerated? [Completeness, Spec FR-001]
- [x] CHK002 Is the exact condition under which "no interfaces were found" may
  appear stated as only-when-enumeration-is-empty, rather than left implicit?
  [Clarity, Spec FR-002]
- [x] CHK003 Is the behavior when the live capture capability is absent
  distinguished from the behavior when adapters are genuinely missing?
  [Consistency, Spec FR-002, Edge Cases]
- [x] CHK004 Is "a representative address" defined well enough to be
  unambiguous when an interface has several or none? [Ambiguity, Spec FR-001]

## Loopback State

- [x] CHK005 Are the loopback states enumerated as exactly three values
  (supported, not supported, undetermined)? [Completeness, Spec FR-003]
- [x] CHK006 Is it explicit that loopback state must not be derived from an
  unrelated installed component? [Clarity, Spec FR-003]
- [x] CHK007 Is the severity of an undetermined loopback state pinned as
  non-blocking, and its wording constrained away from "not installed"?
  [Consistency, Spec FR-003, Clarifications]

## Identity and Paths

- [x] CHK008 Are all four identity facts (version, binary path, profile dir,
  hint-db path) required, with none optional? [Completeness, Spec FR-004]
- [x] CHK009 Is it specified that a path is shown regardless of whether the
  target exists yet? [Coverage, Spec FR-004, Clarifications]
- [x] CHK010 Is it stated that identity and paths rows never change exit status
  and are informational only? [Consistency, Spec FR-004, Clarifications]
- [x] CHK011 Is the unresolvable-path case defined as an "undetermined" note
  rather than an empty or wrong value? [Edge Case, Spec FR-004]

## Machine-Readable Output

- [x] CHK012 Is the one-record-per-check invariant stated for the
  machine-readable form, including the new identity rows? [Consistency,
  Spec FR-005]
- [x] CHK013 Is it explicit that identity facts appear as ordinary check records
  rather than a separate object or header? [Clarity, Spec FR-005]
- [x] CHK014 Is it required that the machine-readable form is never colorized?
  [Coverage, Spec FR-007, FR-004 acceptance 4]

## Color and Presentation

- [x] CHK015 Are the exact conditions for emitting color (interactive terminal,
  NO_COLOR unset, not the JSON form) all enumerated? [Completeness, Spec FR-006,
  FR-007]
- [x] CHK016 Are the exact conditions for suppressing color (piped, NO_COLOR
  set, JSON) enumerated without overlap or gap against the emit conditions?
  [Consistency, Spec FR-007]
- [x] CHK017 Is the non-overflow requirement quantified with a concrete column
  width? [Measurability, Spec FR-008, Assumptions]
- [x] CHK018 Is the treatment of guidance longer than one line specified (an
  indented continuation) rather than left to interpretation? [Clarity,
  Spec FR-008]
- [x] CHK019 Are section-separation and heading-emphasis requirements stated for
  the human form? [Completeness, Spec FR-006]

## Guidance Strings

- [x] CHK020 Is the driver-absent guidance required to name the official source
  and note that the recommended analyzer's installer also provides it?
  [Completeness, Spec FR-009]
- [x] CHK021 Is the integration guidance required to point at the supported
  registration step and to avoid implying the analyzer lacks the framework?
  [Clarity, Spec FR-010]

## Cross-Cutting

- [x] CHK022 Is the exit-status contract stated as unchanged by the presentation
  and identity additions? [Consistency, Spec FR-011]
- [x] CHK023 Is the boundary between this slice (consume enumeration and driver
  detection) and out-of-scope work (modifying how they are discovered) explicit?
  [Scope, Spec Assumptions]
- [x] CHK024 Are the golden-determinism assumptions (fixed version, color only in
  the presentation layer) recorded so acceptance tests are stable? [Assumption,
  Spec Assumptions]

## Notes

All items pass on review: the spec, its Clarifications, and Assumptions cover
each requirement-quality dimension above. The two items most at risk of
under-specification, the only-when-empty rule (CHK002) and the three-valued
loopback state (CHK005 to CHK007), are pinned in FR-002, FR-003, and the
Clarifications session.
