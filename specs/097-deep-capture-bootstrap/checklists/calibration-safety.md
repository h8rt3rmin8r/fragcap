# Requirements Checklist: Calibration Safety And Evidence

**Purpose**: Validate that S097 requirements fully define authorization, scope, observation fidelity, side-effect ordering, cleanup, and reuse before implementation

**Created**: 2026-08-28

**Feature**: [spec.md](../spec.md)

## Authorization And Scope Completeness

- [x] CHK001 Are the target, launch-case, and phase selection requirements explicit and singular? [Completeness, Spec FR-001]
- [x] CHK002 Is calibration clearly separated from ordinary Deep Capture eligibility rather than described as a gate bypass? [Consistency, Spec FR-002]
- [x] CHK003 Are the initially supported and explicitly refused real launch cases enumerated? [Boundary, Spec FR-003]
- [x] CHK004 Are all planned side effects required to appear before confirmation? [Completeness, Spec FR-006]
- [x] CHK005 Are declined, unavailable, and preconfirmed interaction paths defined without allowing the plan to disappear? [Coverage, Spec FR-007]
- [x] CHK006 Is system-wide proxy mutation excluded without exceptions or fallback wording? [Security, Spec FR-008]
- [x] CHK007 Is every constitution-denied technique explicitly excluded from this feature? [Security, Spec FR-021]

## Phase And Ordering Clarity

- [x] CHK008 Are reachability and TLS defined as separate declared phases with different trust permissions? [Clarity, Spec FR-004]
- [x] CHK009 Is the prerequisite evidence for entering the TLS phase exact about target, launch case, freshness, and routing result? [Clarity, Spec FR-005]
- [x] CHK010 Does the specification require unsupported or insufficient requests to stop before every mutation class? [Acceptance Criteria, Spec SC-002]
- [x] CHK011 Are finite deadlines required for launch, observation, proxy shutdown, and cleanup rather than only for the whole run? [Completeness, Spec FR-009]
- [x] CHK012 Are interruption points before observation, after partial observation, during trust, and during cleanup covered? [Scenario Coverage, Spec Edge Cases]

## Observation Fidelity

- [x] CHK013 Are all routing outcomes mutually distinguishable without an aggregate compatibility verdict? [Clarity, Spec FR-010]
- [x] CHK014 Are trust behavior, protocol behavior, and inspectability represented as separate observation dimensions? [Consistency, Spec FR-011]
- [x] CHK015 Are launch case and final-owner handoff required to remain separate facts? [Consistency, Spec FR-012]
- [x] CHK016 Are provenance, time, version, backend, launch, owner, and freshness requirements complete for every stored observation? [Completeness, Spec FR-013]
- [x] CHK017 Does the specification prohibit inferred positive and negative facts on failure or interruption? [Fidelity, Spec FR-014]
- [x] CHK018 Are repeated, stale, conflicting, partial, unknown, and malformed observations all addressed? [Edge Case Coverage, Spec Edge Cases]
- [x] CHK019 Can the nine minimum distinct measurement outcomes be objectively counted? [Measurability, Spec SC-003]

## Audit And Cleanup

- [x] CHK020 Is the local evidence bundle required to name plan, outcome, observations, omissions, updates, and cleanup? [Completeness, Spec FR-015]
- [x] CHK021 Are credentials, publication, private local evidence, and silent sensitivity changes excluded? [Privacy, Spec FR-015, FR-019]
- [x] CHK022 Must every resource and write have an explicit performed, skipped, failed, or not-applicable result? [Accounting, Spec FR-016]
- [x] CHK023 Are partial cleanup and cleanup failure first-class outcomes rather than generic failure? [Recovery, Spec Edge Cases]
- [x] CHK024 Can complete resource reconciliation be measured for every run? [Measurability, Spec SC-005]

## Existing-System Consistency

- [x] CHK025 Is reuse of the existing target identity, fact store, detail view, and ordinary safety gate mandatory? [Consistency, Spec FR-013, FR-017, FR-018]
- [x] CHK026 Does the specification prohibit a second fact persistence or target-resolution path? [Architecture, Spec FR-017]
- [x] CHK027 Is non-aggregating target presentation preserved after calibration? [Consistency, Spec FR-018]
- [x] CHK028 Are controlled verification boundaries explicit about accounts, trust stores, system proxy, and private artifacts? [Testability, Spec FR-019]
- [x] CHK029 Are specification, glossary, help, compatibility guidance, and event documentation all named as synchronized surfaces? [Documentation, Spec FR-020]
- [x] CHK030 Are the separate #252, #253, and #254 boundaries recorded so S097 does not absorb library extraction, native backend adoption, or direct launch? [Scope, Spec Assumptions]

## Review Result

All 30 requirements-quality checks pass. The specification is ready for technical planning and adversarial analysis.
