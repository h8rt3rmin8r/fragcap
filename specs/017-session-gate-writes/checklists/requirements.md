# Specification Quality Checklist: The Session Gates Sink Writes

**Purpose**: Validate specification completeness before planning
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details beyond the architecture-of-record seams this slice
      extends (house style, as in prior slices: the pipeline, the session, the sinks)
- [x] Focused on the fidelity property (the file and the accounting are the same set)
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified (draining/arming discard, packet-bound boundary,
      byte-bound crossing packet, unbounded run, no-gate run, zero bound)
- [x] Scope is clearly bounded (C2 and C3 from the #21 review; the offline unbounded
      goldens do not move; duration is not made a hard byte/packet bound)
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] Every FR has clear acceptance criteria
- [x] User scenarios cover the bound, the watch-time discard, the conservation
      identity, and the golden invariance
- [x] Measurable outcomes defined (SC-001..006)

## Constitution Alignment

- [x] P-3 held: the core seam is generic; the session-aware policy is in the facade
- [x] P-4 held: `gate_dropped` is a named counter in the conservation identity, and
      the reconciliation invariant guards against a double count
- [x] P-9 held: the gate withholds a packet, never alters or fabricates one; the file
      equals the retained set

## Notes

- Follow-up to S14; names the pipeline, the `CaptureSession`, the write gate, and the
  drivers because those are the seams and the unit of traceability, consistent with
  `specs/014-cli-command-surface`. This slice reverses S14's D-c and D-e for the
  watch-time and bound cases and keeps them for the offline unbounded case, recorded
  in research.md D-1..D-7. All items pass.
