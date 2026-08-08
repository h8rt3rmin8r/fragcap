# Specification Quality Checklist: Replay Source and Fixture Corpus

**Purpose**: Validate specification completeness and quality before proceeding
to planning

**Created**: 2026-08-08

**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

Eight questions were raised on the first pass and five more by the clarify
ambiguity scan. All thirteen were resolved under the autopilot decision policy
rather than escalated, because all thirteen were answerable from specification
sections 8.2, 8.4, 25.1, and 25.3 plus the constitution. They are in the spec's
Clarifications section with their rationale.

The clarify session closed one gap that would have forced an architectural
change if it had been found during implementation instead. The flow attributor
seam takes a flow key and no timestamp, because a real attributor reads a
socket table that is already current. A scripted attributor has no implicit
now, and port reuse is precisely the case that depends on it. Widening the seam
was refused, since S02 fixed those five traits as the surface intended to reach
1.0.0 unchanged; the clock is a method on the double instead. The spec grew
from 38 to 44 requirements and from 14 to 16 success criteria, one of which
asserts the seam is still unchanged after this slice.

Three checklist items need their interpretation recorded, because this slice
delivers test infrastructure rather than a user-facing feature.

**"No implementation details" and "written for non-technical stakeholders".**
The deliverable is a substrate a contributor builds tests on, so naming the
capture file format and the two crates is the requirement rather than a leak.
FR-001 names classic pcap because section 25.3 does and because choosing
differently would silently change what the corpus is. FR-014 and FR-020 name
crates because placement is load-bearing under specification section 8.2 and is
the decision most likely to be got wrong by a later reader. Everything else
states observable behavior and defers shape to `plan.md`.

**"Success criteria are technology-agnostic".** SC-003 references byte order
and timestamp resolution, and SC-009 references regenerating the corpus. Both
name the evidence rather than the mechanism: the outcomes that matter are "the
same capture reads the same way however it was written" and "the committed
bytes are what the generator claims", and these are how that is established.

**Priority ordering.** Four stories at P1 and two at P2. The four are one
capability seen from four sides: packets arrive, they arrive identically every
time, failures to deliver are counted, and owners can be scripted. Shipping any
three leaves a substrate that is quietly wrong in the fourth dimension, which is
worse here than elsewhere because every later slice inherits it. Reviewability
and corpus coverage are genuinely separable and are P2.

A pass over both source sections against the requirements found three gaps in
the architecture of record, recorded rather than silently filled.

**Section 25.3 does not say what an attribution script looks like.** It requires
one per fixture and states what it must express. FR-021 through FR-025 define a
format, and the choice not to reuse the profile format TOML is recorded with its
reasoning, because it is the kind of decision a later reader would otherwise
assume was an oversight.

**Section 25.3's `burst.pcap` is self-contradictory.** It must exceed a 65,536
packet buffer and also be small. The spec resolves it by making the fixture
carry the rate and the test supply the capacity, and records the narrowing for
promotion to section 29.

**Neither section says what a replay source does with a filter.** The seam
requires the method. FR-017 records it and applies nothing, with the reasoning
that failing would break a pipeline that filters unconditionally and silent
acceptance would mislead a test.

One repository fact was verified rather than assumed while writing this.
`.gitignore` excludes `*.pcap` globally and re-includes `fixtures/**/*.pcap` at
lines 42 to 44. This slice is the first to depend on that re-inclusion, so the
Assumptions section requires it be checked rather than trusted.
