# Specification Quality Checklist: Audit and gate rigor

**Purpose**: Validate that the audit-disposition and gate requirements are
themselves complete, unambiguous, and objectively verifiable before planning.
**Created**: 2026-08-22
**Feature**: [spec.md](../spec.md)
**Focus**: audit-finding disposition traceability, gate fail-then-pass rigor,
clarified-decision testability

## Requirement Completeness

- [x] CHK001 - Is a disposition required for every one of the twelve findings
      (4 through 15), not merely "most" or "the significant ones"? [Completeness, Spec §FR-001]
- [x] CHK002 - Does the spec state what a "disposition" must contain (fixed +
      what changed, or closed + reason), rather than leaving the record's shape
      implicit? [Completeness, Spec §FR-001]
- [x] CHK003 - Is the set of six sink schemes and six sink modifiers FR-002 asks
      to be documented enumerated explicitly, rather than left as "every scheme
      the parser accepts" with no fixed list to check against? [Completeness, Spec §FR-002]
- [x] CHK004 - Are all four defaulted options (`--mode`, `--direction`,
      `--roles`, `--wait`) named individually in FR-003, rather than referred to
      as a class that a reader must re-derive from the codebase? [Completeness, Spec §FR-003]

## Requirement Clarity

- [x] CHK005 - Is "states its default" in FR-003 given a concrete acceptable
      form (the `[default: ...]` precedent, or explicit conditional prose), so
      two different fixes could not both claim compliance while looking
      unlike each other? [Clarity, Spec §FR-003]
- [x] CHK006 - Is the direction of the spec/grammar correction in FR-004
      (correct the specification, not the grammar) stated as an explicit
      requirement rather than left inferable only from the Clarifications
      narrative? [Clarity, Spec §FR-004]
- [ ] CHK007 - Is "resolves to a real command or flag" in FR-013 precise about
      whether a cross-reference to a *concept* (e.g. "the title tier") rather
      than a literal command/flag token is exempt, or whether the check's
      backtick-detection could misfire on such a reference? [Ambiguity, Spec §FR-013]

## Requirement Consistency

- [x] CHK008 - Does FR-007's contiguous-field-ordering requirement conflict
      with the existing `#[command(group(...))]` mutual-exclusion declaration
      on `CaptureArgs`, or does the spec make clear the two are independent
      (declaration order versus group membership)? [Consistency, Spec §FR-007, Edge Cases]
- [x] CHK009 - Is the FR-006 (`--json` short/long split) requirement consistent
      with the constraint stated in Edge Cases (a global arg cannot vary help
      text per subcommand), i.e. does FR-006 avoid promising per-command
      specificity the Edge Case explicitly rules out? [Consistency, Spec §FR-006, Edge Cases]

## Acceptance Criteria Quality

- [x] CHK010 - Can SC-002 ("names all seven schemes and all six modifiers") be
      checked by a mechanical diff rather than a subjective read, per the
      Independent Test in User Story 2? [Measurability, Spec §SC-002]
- [x] CHK011 - Does SC-005 commit to the fail-then-pass demonstration named in
      FR-015 for all four gate checks, or could SC-005 be satisfied by a gate
      that only ever shows the passing state? [Measurability, Spec §SC-005, FR-015]
- [x] CHK012 - Is "one line" in User Story 4 / FR-011 given a unit (a
      terminal row at the same `MAX_WIDTH` the existing wrap gate uses, versus
      an unbounded logical line), so the check has one unambiguous
      implementation? [Clarity, Spec §FR-011]

## Scenario Coverage

- [x] CHK013 - Are requirements defined for the case where a defaulted option's
      default is itself conditional (the `--mode` case), distinct from the
      unconditional case (`--scope`), so the gate in FR-012 does not need to
      treat every defaulted option identically? [Edge Case, Spec §FR-003, FR-012, Edge Cases]
- [x] CHK014 - Is the disposition of finding 9 (`--extcap-version`) as
      "closed, not a defect" reflected in a requirement or success criterion,
      so a future reader can distinguish "considered and rejected" from
      "overlooked," rather than only in prose under Clarifications? [Traceability, Spec §OOS-002]
- [ ] CHK015 - Does the spec define what happens to FR-011's one-line check
      when a subcommand's short help is legitimately empty (no doc comment),
      as distinct from a multi-line summary that violates the rule? [Gap, Edge Case]

## Non-Functional / Process Requirements

- [x] CHK016 - Is the requirement to demonstrate each new gate check
      fail-then-pass (FR-015) stated as a MUST rather than an implied nicety,
      given the project's own history of a gate (#67/#178) that passed without
      ever having been shown to fail on the case it existed for? [Clarity, Spec §FR-015]
- [x] CHK017 - Does the spec bound the audit record's required location (FR-001
      references "this feature's `plan.md` or a dedicated file it points to")
      tightly enough that "reviewable per line" (the issue's own acceptance
      bar) is checkable rather than a matter of taste? [Measurability, Spec §Key Entities, FR-001]

## Dependencies & Assumptions

- [x] CHK018 - Is the assumption that no new crate or dependency is needed
      (Assumptions) checked against anything concrete, or does it rest solely
      on the FR list's own contents matching that claim? [Assumption, Spec §Assumptions]
- [x] CHK019 - Is the dependency of FR-004 (spec correction) on FR-014 (the new
      spec-agreement gate check) made explicit, i.e. does the spec state that
      FR-004's fix is what FR-014's gate is expected to newly pass against,
      rather than leaving the two requirements to be read as unrelated? [Consistency, Spec §FR-004, FR-014]

## Ambiguities & Conflicts

- [ ] CHK020 - Does US5's Acceptance Scenario 1 ("no other option's help text
      between the first and the last") give a precise enough boundary for
      "contiguous" when `OfflineArgs` is flattened into the same struct at a
      later field position, or could the flattened hidden fields be read as
      violating or satisfying the rule ambiguously? [Ambiguity, Spec §User Story 5]

## Notes

- Three items (CHK007, CHK015, CHK020) are left unchecked: each names a real
  edge the spec does not pin down (a conceptual cross-reference's exemption
  from FR-013; an empty-short-help case for FR-011; the flattened
  `OfflineArgs` fields' effect on FR-007's contiguity claim). None blocks
  planning, since each is narrow enough to resolve as an implementation-time
  decision recorded in `plan.md` rather than requiring a further
  `/speckit-clarify` round, per the autopilot decision policy (no option is
  clearly best-or-worst enough to be architecture-defining, and a wrong
  implementation-time call is cheap to correct before the gate is demonstrated
  fail-then-pass per FR-015).
- 17 of 20 items pass as written. Recommend carrying the three open items into
  `plan.md` as explicit design decisions rather than blocking on a spec
  rewrite.
