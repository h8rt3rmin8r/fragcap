# Lifecycle and Evidence Requirements Checklist: S187

**Purpose**: Review whether timing, authority, and evidence requirements are complete before design.
**Audience**: PR reviewer.

## Timing Contract

- [x] Are accepted explicit and defaulted launch and observation windows stated separately? [FR-001 to FR-004]
- [x] Are unsupported values required to fail before effects, including direct facade callers? [FR-004]
- [x] Are owner release, shutdown, cleanup, and cancellation separated from observation? [FR-005]

## Process Authority

- [x] Are declared roles, observed descendants, and controllable process objects kept distinct? [FR-006 to FR-008]
- [x] Does process-control authority require exact instance, path, account, session, and ancestry rather than image name? [FR-007]
- [x] Are late and same-name unrelated processes covered as negative cases? [FR-010 to FR-011]

## Evidence and Recovery

- [x] Is trace omission required to be bounded and counted? [FR-008]
- [x] Are exact, ambiguous, and unavailable connection ownership outcomes all defined from first observation? [FR-009]
- [x] Does timeout require finite proxy continuity and an exact unresolved recovery path? [FR-010]
- [x] Is the #542 protocol-failure investigation excluded from claims made by this slice? [Assumptions]
