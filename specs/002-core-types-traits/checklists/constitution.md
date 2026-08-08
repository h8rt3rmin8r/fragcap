# Constitution Compliance Checklist: Core Types and Traits

**Purpose**: Validate that the S02 requirements specify the constitution's
constraints completely, unambiguously, and measurably, before any code is
written

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

**Note**: This checklist tests the requirements, not the implementation. Each
item asks whether the spec says enough for a later reviewer to judge compliance
mechanically. An item passes when the requirement is written well, not when the
code works.

## P-1 Passive Observation Only

- [x] CHK001 Does the spec confirm no requirement obliges a denylisted
  technique, given that this slice declares traits an implementor must satisfy?
  [Completeness, Spec §Out of Scope]
- [x] CHK002 Are the process watcher requirements free of any obligation to open
  a process handle, so that a later implementor is not pushed toward memory
  rights by the trait's shape? [Clarity, Spec §FR-016]
- [x] CHK003 Is it stated that a dependency providing a prohibited capability
  fails the audit, now that the dependency graph is non-empty for the first
  time? [Coverage, Spec §FR-028]

## P-2 Core Stays Platform-Neutral

- [x] CHK004 Is "no platform-specific dependency, no I/O crate, no capture
  library" stated as a requirement rather than left to the reader to infer from
  the crate's purpose? [Completeness, Spec §FR-027]
- [x] CHK005 Is the evidence for platform neutrality named, so the claim is
  checkable rather than asserted? [Measurability, Spec §SC-005]
- [x] CHK006 Do the requirements say what the timestamp must NOT carry, so that
  output-format knowledge cannot leak into core through it? [Clarity,
  Spec §FR-011]
- [x] CHK007 Are the criteria for admitting an external dependency stated, given
  that this slice is the first to add any? [Gap, Spec §Clarifications]

## P-3 Capture And Attribution Stay Separate

- [x] CHK008 Is the prohibition on combining acquisition with attribution stated
  as a requirement on the traits, not only as a principle reference?
  [Completeness, Spec §FR-020]
- [x] CHK009 Do the requirements forbid the two traits referencing each other,
  which is the form the violation would actually take at this slice?
  [Clarity, Spec §FR-020]

## P-4 No Silent Loss

- [x] CHK010 Is "one named counter per discard cause" stated in a way that rules
  out a single aggregate satisfying it? [Clarity, Spec §FR-023]
- [x] CHK011 Are backend-reported counts and pipeline counts required to stay
  separate, and is the reason recorded so a later slice does not merge them for
  convenience? [Consistency, Spec §FR-024]
- [x] CHK012 Is the relationship between a total and its named counters
  specified, so the two cannot disagree? [Measurability, Spec §FR-025]
- [x] CHK013 Do the requirements state that an unattributed packet is retained
  and marked rather than dropped? [Completeness, Spec §FR-026]
- [x] CHK014 Is "marked" defined concretely enough to be verified, rather than
  left as a word? [Ambiguity, Spec §FR-010]

## P-6 Glossary First

- [x] CHK015 Is a glossary entry required for every term this slice introduces,
  in the same change? [Completeness, Spec §FR-030]
- [x] CHK016 Is there a stated way to tell which terms are new, so the glossary
  requirement is checkable rather than a matter of memory?
  [Measurability, Spec §SC-008]

## P-9 The Instrument Does Not Lie

- [x] CHK017 Is the prohibition written against operations whose purpose is to
  alter, mask, truncate, reorder, or withhold, rather than against a vaguer
  notion of dishonesty? [Clarity, Spec §FR-029]
- [x] CHK018 Do the requirements distinguish permitted scope reduction from
  prohibited alteration, so a snapshot length is not mistaken for a violation?
  [Consistency, Spec §Clarifications]
- [x] CHK019 Is the original on-wire length required to survive truncation of
  the stored bytes? [Completeness, Spec §FR-008]
- [x] CHK020 Is rounding of a timestamp addressed, given that a resolution
  conversion is the most likely accidental route to an altered observation?
  [Edge Case, Spec §FR-011]

## Seam Shape And Dyn Compatibility

- [x] CHK021 Is dyn compatibility stated as a requirement with a named failure
  mode, rather than assumed from the pipeline description?
  [Completeness, Spec §FR-019]
- [x] CHK022 Is the reason dyn compatibility is needed recorded, so a later
  contributor does not remove the constraint as unnecessary?
  [Traceability, Spec §Clarifications]
- [x] CHK023 Are the requirements for the dissector trait explicit that it has
  no implementations on purpose, so its emptiness is not read as an oversight?
  [Clarity, Spec §FR-018]
- [x] CHK024 Is the sink's finishing method specified as consuming an owned
  pointer, which is what makes a boxed trait object usable?
  [Completeness, Spec §FR-017]

## Attribution Key Asymmetry

- [x] CHK025 Is the TCP and UDP asymmetry stated as a property of the platform
  interface rather than a fragcap design choice, so it is not "fixed" later?
  [Traceability, Spec §Clarifications]
- [x] CHK026 Is the prohibition on inventing a UDP remote endpoint expressed as
  a constraint on the type rather than a documented warning?
  [Measurability, Spec §FR-004]
- [x] CHK027 Are wildcard bind address requirements specified, including that
  both the wildcard and the specific address must be matchable?
  [Coverage, Spec §FR-005]
- [x] CHK028 Are the key equality and hashing requirements complete enough to
  prevent a later field that cannot participate in a stable hash?
  [Completeness, Spec §FR-013]

## Dependency Licensing

- [x] CHK029 Is the license allowlist referenced by source rather than restated,
  so the two cannot drift? [Consistency, Spec §FR-028]
- [x] CHK030 Is it stated that the audit must pass against a real graph, making
  explicit that its previous passes were vacuous?
  [Measurability, Spec §SC-006]
- [x] CHK031 Are the consequences for the declared minimum toolchain addressed,
  given that real dependencies may raise it? [Gap, Spec §Assumptions]

## Notes

All thirty-one items pass against the spec as clarified. Three were failing
before the clarify session and are recorded here for traceability:

- CHK014 ("marked" defined concretely) failed while the spec left the
  distinction between "not attempted" and "attempted and unresolved" as an open
  alternative. FR-010 now pins the three-state mapping and requires a test.
- CHK024 (owned pointer on finish) was implied by the architecture of record but
  not stated as a requirement. Now FR-017.
- CHK028 (stable hash participation) had no requirement at all. Now FR-013.

Two items are worth re-reading at the analyze gate rather than treated as
settled:

- CHK007 states the criteria for admitting a dependency in the Clarifications
  prose rather than as a numbered requirement. That is deliberate, because the
  concrete choice belongs to `research.md`, but it means the criteria are not
  independently traceable.
- CHK031 anticipates the declared minimum toolchain rising. The spec records
  this as an expected outcome rather than a requirement, so nothing fails if it
  is forgotten. `tasks.md` must carry it as an explicit step.
