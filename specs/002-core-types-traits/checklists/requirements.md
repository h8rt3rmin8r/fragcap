# Specification Quality Checklist: Core Types and Traits

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

**Clarify session 2026-08-08.** Five further questions were raised by the
ambiguity scan and resolved under the autopilot decision policy: key equality
and hashing, the three attribution states, timestamp resolution, enumerated
versus opaque errors, and separation of backend counters from pipeline counters.
All five were answerable from specification sections 8.4 and 8.5 plus the
constitution, so none was escalated. Each produced at least one requirement;
the spec grew from 27 to 31 requirements and from 9 to 11 success criteria.

One open question in the Edge Cases section, whether the vocabulary
distinguishes "not yet attempted" from "attempted and unresolved", was closed
rather than left as an alternative. It is distinguishable, from the flow key and
attribution read together, and no new field was needed.

Two checklist items need their interpretation recorded, because this slice is a
library vocabulary slice rather than a user-facing feature and the generic
wording does not map cleanly.

**"No implementation details" and "written for non-technical stakeholders".**
The deliverable of this slice is a type and trait vocabulary, so naming the
entities is the requirement rather than a leak. The line held is that the spec
states required properties and prohibitions, and defers concrete backing
choices, specifically what represents a timestamp and a byte payload, to
`plan.md` and `research.md`. The stakeholder is a contributor writing a later
slice, which the Overview states explicitly rather than leaving implied.

Individual requirements name entities from the architecture of record
(specification sections 8.4 and 8.5) by role rather than by their Rust
signature: "a flow key carrying the protocol and two endpoints" rather than the
struct definition. That is deliberate, and it is what keeps the spec reviewable
against the specification rather than against the eventual code.

**Success criteria and technology-agnosticism.** SC-005 through SC-007 reference
the project's own checks by what they establish, not by command name, on the
same reasoning: the outcome that matters is "core is portable and the audit is
meaningful", and the check is the evidence.

Verified against the source sections item by item: every type in specification
section 8.4 and every trait in section 8.5 has a corresponding requirement. Two
gaps in the architecture of record were found during that pass and are recorded
as assumptions rather than silently filled: the specification names `Timestamp`,
`Bytes`, `StageId`, `LinkType`, `Endpoint`, `FilterProgram`, `ProcessEvent`,
and `ProcessRecord` without defining them, and it does not state whether the
behavioral traits must be dyn-compatible. Both are carried into the
Clarifications section with the decision and its rationale.
