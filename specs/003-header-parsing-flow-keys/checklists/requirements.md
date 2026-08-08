# Specification Quality Checklist: Header Parsing and Flow Keys

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

Eight questions were raised by the initial pass and five more by the clarify
ambiguity scan. All thirteen were resolved under the autopilot decision policy
rather than escalated, because all thirteen were answerable from specification
sections 12.5 and 12.6, sections 8.2 through 8.4, and the constitution. They
are recorded in the spec's Clarifications section with their rationale.

The clarify session closed five gaps the first pass left as unquantified
requirements: the rule assigning local and remote when direction is
undetermined, the two numeric bounds FR-015 and FR-024 asked for without
stating, the parser's caller-facing shape, the defence against a stale fragment
identity, and whether the captured packet type gains a rejection cause field.
The spec grew from 39 to 41 requirements. One answer changed existing behavior
rather than only adding to it, and is called out below.

Three checklist items need their interpretation recorded, because this slice
produces a library capability rather than a user-facing feature.

**"No implementation details" and "written for non-technical stakeholders".**
The deliverable is a parsing capability inside a library, so naming the
protocols and header fields it reads is the requirement rather than a leak. The
line held is that requirements state observable behavior and prohibitions, and
defer the concrete shape, meaning what the result type looks like and how the
fragment table is structured, to `plan.md`. The one deliberate exception is
FR-001, which names the crate: placement is the decision most likely to be got
wrong by a later reader, and it is architecturally load-bearing under
specification section 8.3 rather than an implementation detail.

**Success criteria and technology-agnosticism.** SC-003 references measurement
under a counting allocator and SC-009 references the portability build. Both
name the evidence rather than the mechanism, on the same reasoning S02 used:
the outcome that matters is "parsing costs nothing per packet" and "core is
still portable", and the check is how that is established rather than what is
being claimed.

**Priority ordering.** Five user stories, three at P1. That is unusual but
correct here: the flow key, the honest failure reporting, and the honest
direction reporting are one capability viewed three ways, and shipping any two
without the third produces a parser that is wrong in a way the operator cannot
see. Fragments and the allocation property are genuinely separable and are
ranked P2.

A separate pass verified every clause of specification sections 12.5 and 12.6
against a requirement. Two gaps in the architecture of record were found and
are recorded rather than silently filled.

**Section 12.5 does not say where a subsequent fragment's remembered flow key
lives.** It states that subsequent fragments are attributed by their fragment
identifier and address pair, which presupposes a memory it does not describe.
FR-021 through FR-026 define one, bounded and evicting, and the decision is
recorded for promotion to specification section 29.

**Section 12.6 does not say what direction is reported when the interface
address set matches neither endpoint.** It defines the local-source and
local-destination cases and the loopback case, and is silent on the fourth.
FR-030 makes it a separate counted outcome, and additionally produces no flow
key for it, on the reasoning that section 8.4 defines the key's local field as
the endpoint on the capturing host and no such endpoint exists in that case.
Both parts are recorded for promotion.

That decision revised the first pass, which had this case producing a flow key
with no direction. The revision is the only one in this session that removed
behavior rather than adding it, and the acceptance scenario, edge case, and
success criterion that assumed the earlier answer were rewritten rather than
left alongside it.

One defect in the existing code was found during the same pass and is fixed
here rather than worked around: the documentation on the link type constant for
code 0 describes the encapsulation belonging to code 101. FR-011 corrects it.

**A known limitation is stated rather than resolved.** A sixteen bit IPv4
fragment identifier can be reused before its table entry is removed, producing
a wrong flow key for a subsequent fragment. It is not detectable from the
capture, so it cannot be counted, which puts it outside what P-4 can enforce.
The spec carries it as a named limitation with the mitigations that narrow it
and the reason the obvious further mitigation, an expiry timer, is refused.
