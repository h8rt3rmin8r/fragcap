# Specification Quality Checklist: Socket Table Attributor

**Purpose**: Validate specification completeness and quality before proceeding
to planning

**Created**: 2026-08-09

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

Four items warrant comment rather than a bare mark.

**Implementation details.** Windows and the IP Helper API are named in
Assumptions and in the Overview, not in the requirements. FR-034 through FR-036
are stated against "the platform's table interface" and "the object-model
projection", which is the distinction specification section 11.2 draws and the
one that matters; the concrete interface names are a record of the choice
rather than the requirement itself. This follows the pattern S04, S06, and S09
established.

Two requirements do name project types: FR-026 names `AttributionState` and
FR-019 names `Fidelity::Live` and `Fidelity::Retained`. These are the
specification's own vocabulary from sections 13.4 and 8.4 rather than
implementation choices this slice makes, and stating the requirement without
them would make it less testable, not more abstract.

**Non-technical stakeholders.** As with S09, the intended reader is an operator
or a contributor. Terms of art that appear (socket table, endpoint, flow key,
fidelity, retention window, snapshot) all carry glossary entries or acquire one
under FR-039.

**No clarification markers.** Ten questions that would otherwise have been
marked were resolved in the Clarifications session and recorded with their
reasoning: the creation timestamp, dual-stack matching, how an `&self` lookup
requests a refresh, where time comes from, whether the pipeline's attributor
mutex belongs to this slice, the total order over competing matches, where the
cadence configuration lives, whether image names resolve eagerly or lazily, and
what origin the retention window is measured from. Each had a defensible answer
available from the specification, Appendix D, S05's own reasoning about closed
key sets, or the constitution, so none was left open.

The tie-break question is the one that would have caused real trouble if left
implicit. "Prefer the more exact match" reads as settled and is not: with a
wildcard bind and a specific bind on the same port, or two sockets on a reused
port, an implementation that iterates the platform's rows produces an answer
that depends on row order and therefore changes between runs over identical
traffic. FR-008 through FR-008b and SC-014 make that a property under test
rather than an accident.

**Success criteria and technology.** SC-010 through SC-012 name
`cargo xtask ci`, `deps`, and `lint`. These are the repository's own gates
rather than implementation detail, and stating them by name is what makes them
checkable. The same pattern appears in S09's SC-009 through SC-013.
