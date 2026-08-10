# Specification Quality Checklist: ETW Process Watcher and Tree

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

**Implementation details.** The specification names ETW in its title and its
requirements rather than only in its assumptions, which departs from the pattern
S09 followed for npcap. The departure is deliberate and narrow. ETW is not one
implementation of an abstract capability the way a capture driver is: the whole
argument of specification section 5.3, and therefore this slice's reason to
exist, is that creation-time ancestry is obtainable in exactly one way and every
alternative is wrong rather than merely slower. Naming the mechanism in
requirements FR-001 through FR-004 records a constraint rather than a choice.
The tree requirements, FR-015 through FR-031, name no platform interface at all,
which is the boundary this slice is built around. Windows and the concrete
telemetry crate remain in Assumptions and in the plan phase respectively.

**Non-technical stakeholders.** As in S09, the intended reader of a fragcap
slice specification is an operator or a contributor. Terms of art already
carrying glossary entries are used freely (ETW, process tree, PID recycling,
launcher chain, scripted attributor); terms this slice introduces are listed
under Key Entities and are covered by FR-039.

**Five named deviations.** The command line on the start event, the availability
state that field admits, ancestry provenance on the node, the watcher-owned
report beside `CaptureStats`, and the settlement of `image` as a path all change
or extend what specification sections 8.4, 10.2, and 26.2 declare. Three were
identified while reading section 10 against the S02 types, one (the availability
state) by reading section 10.2's field list against constitution P-1, and one
(the watcher report) during the clarification session. Stating them here rather
than discovering them during implementation is what the deviation rule asks for.

**A requirement about an absence.** FR-038 requires that where this slice has no
discard path, that is a property of the design rather than an uncounted
discard. That reads oddly as a requirement and is testable anyway: the fold
either has a branch that drops an event without counting it or it does not, and
FR-010, FR-023, and FR-024 between them account for every event the tree
receives. It is stated because the natural reading of P-4 is to add a bounded
buffer with a counter, which for this stream would be a defect wearing the
uniform of compliance.

**Re-validated after the 2026-08-09 clarification session.** Five further
answers were integrated, bringing that session to twelve bullets. Requirements
were renumbered to FR-001 through FR-050 so that the seven added requirements
sit in their thematic groups rather than appended out of order, and four success
criteria were added (SC-014 through SC-017, with the former SC-014 becoming
SC-018). All sixteen items still pass and none changed state.

Two of the five closed real holes rather than open decisions, which is worth
recording because the specification did not read as though it had them. The
order of subscription and snapshot was unstated, and the two orders differ by
whether a process created during startup can go unobserved. And FR-023's
resolution by identifier and timestamp had no defined behavior for a node whose
start time the platform does not report, which `ProcessRecord` has admitted as a
possibility since S02.

Items marked incomplete require spec updates before `/speckit-clarify` or
`/speckit-plan`. None are incomplete.
