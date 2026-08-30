# Research Integrity Checklist: Smaller Native Proxy Fallback Spike

**Purpose**: Validate that S100 can close the fallback comparison without weakening product or research boundaries
**Created**: 2026-08-30
**Feature**: [spec.md](../spec.md)

## Evidence Completeness

- [x] CHK001 Are requirements defined for every protocol, lifecycle, certificate, key-log, dependency, license, advisory, toolchain, build, and cleanup proof point? [Completeness, Spec FR-003 through FR-015]
- [x] CHK002 Is the S099 matrix required for the fallback rather than allowing candidate-specific traffic to masquerade as comparison? [Consistency, Spec FR-005 and FR-016]
- [x] CHK003 Are partial, empty, bounded, truncated, unsupported, failed, and not-measured outcomes distinguished from complete observations? [Clarity, Spec FR-006]
- [x] CHK004 Are environment, version, command, input, and limitation fields sufficient to reproduce each conclusion? [Completeness, Spec FR-015]

## Fidelity and Safety

- [x] CHK005 Does the specification require length and digest accounting while forbidding silent loss or transformation? [Consistency, Spec FR-006, FR-007, and SC-003]
- [x] CHK006 Are requirements explicit that only proxy-owned key material is in scope and target key extraction remains prohibited? [Security, Spec FR-011 and FR-018]
- [x] CHK007 Are loopback scope, explicit routing, trust separation, validation, and system-proxy refusal stated independently? [Security, Spec FR-003 and FR-009]
- [x] CHK008 Are private keys, raw traffic, operator paths and addresses, credentials, tokens, and ephemeral ports excluded from committed evidence? [Privacy, Spec FR-019]
- [x] CHK009 Are cleanup and residue requirements defined for listeners, connections, certificate state, keys, and temporary output? [Coverage, Spec FR-004 and FR-010]

## Audit Quality

- [x] CHK010 Does the audit cover active and inactive target paths, direct and transitive packages, root stores, licenses, sources, and advisories? [Completeness, Spec FR-012]
- [x] CHK011 Are Rust 1.82 parsing, checking, and building stated as separate results rather than one compatibility claim? [Clarity, Spec FR-013]
- [x] CHK012 Are build timing, package-count, and size requirements reproducible and separated into clean and warm measurements? [Measurability, Spec FR-014 and SC-006]
- [x] CHK013 Does the specification require proof that the released graph remains free of fallback dependencies? [Boundary, Spec FR-002 and FR-020]

## Decision Discipline

- [x] CHK014 Is the three-way comparison authority explicit and consistent across the stories, requirements, and assumptions? [Consistency, Spec FR-016 and SC-008]
- [x] CHK015 Does the specification prevent parity or adoption when a deciding proof point is missing or inconclusive? [Coverage, User Story 2 and User Story 4]
- [x] CHK016 Is exactly one backend outcome required without another speculative issue tree? [Scope, Spec FR-017]
- [x] CHK017 Are the non-shipping boundary and prohibited product changes consistent across stories, requirements, success criteria, and assumptions? [Consistency, Spec FR-002, FR-018, FR-020, and SC-007]

## Notes

- This is a formal pull-request review checklist because S100 closes the only fallback path selected by S099.
- All 17 requirement-quality checks pass before planning.
