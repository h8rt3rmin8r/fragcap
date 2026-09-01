# Security Requirements Checklist: Warm-To-Cold Restart

**Purpose**: Test whether S113 requirements constrain identity, consent, process control, effects, and failure truthfully

## Identity And Scope

- [x] Does the spec forbid treating an image-name match as selected-target identity? [Spec FR-002, FR-005]
- [x] Does cold detection cover every declared direct, platform, and publisher image? [Spec FR-007]
- [x] Must changed target or launch facts cause fresh resolution or refusal? [Spec FR-008 through FR-010]

## Consent And Effects

- [x] Is consent explicit before waiting and again before session effects? [Spec FR-004, FR-011, FR-012]
- [x] Are JSON and noninteractive behavior deterministic? [Spec FR-004]
- [x] Are all process-control mechanisms and fallbacks prohibited? [Spec FR-005, FR-017]
- [x] Are bundle, proxy, trust, routing, launch, and compatibility effects after second authorization? [Spec FR-012]

## Failure And Audit

- [x] Are decline, timeout, inventory failure, changed state, and preparation failure distinct? [Spec FR-008]
- [x] Are wait deadlines finite and enforceable? [Spec FR-006]
- [x] Can later launch and cleanup failures remain visible? [Spec FR-014, FR-015]
- [x] Are both human and structured outcomes required? [Spec FR-013]

## Notes

- All security requirements are complete. The deliberate absence of an automated shutdown operation is a product safety boundary, not an omitted implementation.
