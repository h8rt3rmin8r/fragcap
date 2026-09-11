# Security Requirements Checklist: Deep Capture Embedded Workflow Help

**Purpose**: Challenge whether the help contract preserves explicit consent, evidence fidelity, and sensitive-artifact boundaries

**Created**: 2026-09-11

**Feature**: [spec.md](../spec.md)

## Consent And Scope

- [x] CHK001 Does the specification leave effect authorization behavior unchanged? [FR-017]
- [x] CHK002 Does the safe journey keep calibration and ordinary Deep Capture separate from target registration? [FR-005]
- [x] CHK003 Are expired, drifted, declined, invalid, and interrupted authorization states included in the refusal inventory?
- [x] CHK004 Does validation explicitly exclude real trust mutation, live sensitive capture, and real-game execution? [FR-018, SC-005]

## Truthful Claims

- [x] CHK005 Is universal decryption explicitly forbidden as a help claim? [FR-012]
- [x] CHK006 Is automatic certificate-pinning bypass explicitly forbidden as a help claim? [FR-012]
- [x] CHK007 Must compatibility remain tied to direct observation rather than inference? [FR-006, FR-012]
- [x] CHK008 Are stale, conflicting, negative, partial, and unknown compatibility states represented in the refusal inventory?

## Sensitive Artifacts And Recovery

- [x] CHK009 Does help distinguish sensitive evidence export from destructive cleanup? [FR-005, FR-011]
- [x] CHK010 Must failed cleanup retain the records required for exact recovery? [FR-011]
- [x] CHK011 Is prior-session residue required to route through the existing recovery authority? [FR-011, FR-017]
- [x] CHK012 Are structured schemas and cleanup authority preserved? [FR-017, SC-006]

## Notes

- The checklist evaluates the security requirements and non-claims, not product behavior.
