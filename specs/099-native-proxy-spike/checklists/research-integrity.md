# Research Integrity Checklist: Native Proxy Backend Spike

**Purpose**: Validate that S099 requirements can produce a reviewable backend decision without weakening product or research boundaries
**Created**: 2026-08-29
**Feature**: [spec.md](../spec.md)

## Evidence Completeness

- [x] CHK001 Are requirements defined for every lifecycle, protocol, certificate, cache, key-log, dependency, license, toolchain, build, and cleanup proof point? [Completeness, Spec FR-003 through FR-014]
- [x] CHK002 Is the controlled scenario matrix required for both the candidate and baseline rather than allowing backend-specific traffic to masquerade as parity? [Consistency, Spec FR-005 and FR-015]
- [x] CHK003 Are unavailable, unsupported, failed, and not-measured outcomes distinguished from successful observations? [Clarity, Spec FR-006 and FR-015]
- [x] CHK004 Are environment and version fields sufficient to reproduce each material conclusion? [Completeness, Spec FR-014]

## Fidelity and Safety

- [x] CHK005 Does the specification require complete body accounting and forbid silent loss or transformation? [Consistency, Spec FR-006]
- [x] CHK006 Are requirements explicit that only proxy-owned key material is in scope and target key extraction remains prohibited? [Security, Spec FR-010 and FR-018]
- [x] CHK007 Are loopback scope, explicit routing, trust separation, and system-proxy refusal stated independently? [Security, Spec FR-003 and FR-008]
- [x] CHK008 Are private keys, raw traffic, operator paths, addresses, credentials, and tokens excluded from committed evidence? [Privacy, Spec FR-019]
- [x] CHK009 Are cleanup and residue requirements defined for listeners, connections, cache state, certificates, keys, and temporary outputs? [Coverage, Spec FR-004 and FR-009]

## Audit Quality

- [x] CHK010 Does the dependency audit cover active, inactive target-conditional, direct, transitive, root-store, license, and source-provenance cases? [Completeness, Spec FR-011]
- [x] CHK011 Are Rust 1.82 failure and feature-gated MSRV exclusion treated as explicit evidence and policy choices rather than accidental omissions? [Clarity, Spec FR-012]
- [x] CHK012 Are build timing and size requirements reproducible and separated into clean and warm measurements? [Measurability, Spec FR-013 and SC-006]
- [x] CHK013 Does the specification require proof that the released graph remains free of spike-only dependencies? [Boundary, Spec FR-002 and FR-020]

## Decision Discipline

- [x] CHK014 Are the four permitted decision outcomes mutually exclusive and exhaustive for this spike? [Clarity, Spec FR-016]
- [x] CHK015 Does the specification prevent an adoption recommendation when a deciding proof point remains implicit or inconclusive? [Coverage, User Story 4]
- [x] CHK016 Is exactly one bounded follow-up required, preventing parallel speculative backend work? [Scope, Spec FR-017]
- [x] CHK017 Are the non-shipping boundary and prohibited product changes consistent across stories, requirements, success criteria, and assumptions? [Consistency, Spec FR-002, FR-020, SC-007]

## Notes

- This is a formal pull-request review checklist because S099 closes an architecture decision and audits a large optional dependency graph.
- All 17 requirement-quality checks pass before planning.
