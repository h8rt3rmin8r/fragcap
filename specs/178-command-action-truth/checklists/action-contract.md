# Requirements Checklist: CLI Action Truth

**Purpose**: Review the clarity and completeness of S178 action, side-effect, and next-step requirements.

**Created**: 2026-10-21

**Feature**: [spec.md](../spec.md)

## Destructive Action Scope

- [x] CHK001 Are eligible, retained, unresolved, and empty collection cases separately defined? [Completeness, Spec FR-001 to FR-003]
- [x] CHK002 Is the condition for offering Apply objective, including the zero-work and incomplete cases? [Clarity, Spec FR-002]
- [x] CHK003 Are preview identity, explicit retained selection, exact authorization, and eventual deletion distinct in the requirements? [Consistency, Spec FR-001 and FR-014]
- [x] CHK004 Are missing and unowned path errors specified with a non-destructive next step for collect, cleanup, and export? [Coverage, Spec FR-002 to FR-004]

## Selector and Seed Preflight

- [x] CHK005 Is conflicting positional versus stable-ID selection defined at argument validation with no partial output? [Clarity, Spec FR-005]
- [x] CHK006 Are export-all, one-selector, and missing-selector cases defined without altering existing resolution semantics? [Completeness, Spec FR-006 and FR-007]
- [x] CHK007 Are unsupported tiers, absent sources, repeated tiers, and default multi-tier results separately specified? [Coverage, Spec FR-008 to FR-011]
- [x] CHK008 Is the rule for whether a new store may be created after failed preflight measurable? [Acceptance, Spec SC-003]

## Side Effects and Boundaries

- [x] CHK009 Does the spec distinguish candidate registration from eligibility seeding and store creation? [Clarity, Spec FR-012]
- [x] CHK010 Are output and filesystem/store effects both required in controlled acceptance evidence? [Acceptance, Spec FR-013]
- [x] CHK011 Is the global JSON/silent redesign explicitly excluded while requiring equivalent collection reasons? [Consistency, Spec Edge Cases]
- [x] CHK012 Are owner evidence, release claims, and unrelated IGDB scope excluded or bounded? [Boundary, Spec Assumptions]
