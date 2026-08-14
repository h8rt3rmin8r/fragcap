# Network Dependency Checklist: Tier 1 Catalog Seeder

**Purpose**: Validate that the requirements around the HTTP-client dependency,
offline testability, and MSRV/feature discipline are complete and testable before
planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Offline testability

- [x] CHK016 Is the catalog source required to be an abstraction with an offline fixture-backed implementation that drives every automated test? [Clarity, Spec §FR-002]
- [x] CHK017 Is it stated that the seeder's fetch-parse-map-gate logic is fully exercisable with no network access? [Completeness, Spec §FR-002, §SC-001]
- [x] CHK018 Is the "live network never runs in CI" posture stated (the real source is compiled but not executed there, like live capture)? [Consistency, Spec §Clarifications]
- [x] CHK019 Is the shape the seeder consumes fixed by the trait, so the seeder cannot depend on a live-only detail a fixture cannot express? [Clarity, Spec §Edge Cases]

## Dependency justification & MSRV

- [x] CHK020 Is the HTTP-client dependency required to be justified to the project's rubric (unique property, rejected alternatives, exact Cargo.lock delta including build-deps)? [Completeness, AGENTS.md rubric]
- [x] CHK021 Is MSRV 1.82 stated as a hard gate for the client and its whole transitive graph, verified via the minimum-toolchain build? [Measurability, Spec §FR-003, §SC-005]
- [x] CHK022 Is the transport-security (TLS) choice called out as part of the dependency decision, with its graph and MSRV impact in scope? [Completeness, Spec §Assumptions]
- [x] CHK023 Is the license constraint stated (restricted to the constitution's allowed set) for the whole HTTP+TLS graph? [Completeness, Constitution Licensing]

## Feature gating & dependency direction (P-2)

- [x] CHK024 Is the network source and its client required to be optional at build time, so a default build and the offline test suite compile neither the client nor a TLS stack? [Clarity, Spec §FR-003, §SC-005]
- [x] CHK025 Is it explicit that the client lands only in the targets crate and never makes fragcap-core depend on it? [Consistency, Spec §FR-012, Constitution P-2]
- [x] CHK026 Does the spec/plan require the dependency-direction allowlist to remain satisfied (no new core edge; the crate's edges unchanged except as needed)? [Completeness, Constitution P-2]

## Process artifacts

- [x] CHK027 Is the requirement to record the dependency in the AGENTS.md inventory and as a dated changelog decision present? [Traceability, Spec §DONE WHEN]
- [x] CHK028 Is a P-1 statement present (a read-only public HTTP GET is permitted; no process handle, capture, or injection)? [Coverage, Spec §Constitution-critical]

## Notes

- Check items off as completed: `[x]`
- These items test whether the REQUIREMENTS are complete and testable, not whether the build works.
- The governing rubric is the AGENTS.md "Dependency inventory" section and the constitution's P-1/P-2, Licensing, and Verification sections.
