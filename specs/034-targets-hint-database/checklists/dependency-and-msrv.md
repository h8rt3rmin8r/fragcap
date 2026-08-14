# Dependency & MSRV Checklist: Targets Hint Database (foundation)

**Purpose**: Validate that the requirements around adding the embedded-database
dependency (rusqlite) are complete and testable against the project's
dependency-justification rigor and MSRV floor, before planning.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)

## Dependency Justification Rigor

- [x] CHK016 Does the spec/plan require the new dependency to be justified with the unique property it supplies that hand-rolling or existing in-tree assets cannot? [Completeness, AGENTS.md rubric]
- [x] CHK017 Is the requirement to record rejected alternatives (hand-rolled format, staying on embedded JSON) with concrete grounds present? [Completeness, AGENTS.md rubric]
- [x] CHK018 Is the exact Cargo.lock package delta required to be recorded, explicitly including build-dependencies (e.g. the C build via cc), not just direct deps? [Clarity, AGENTS.md rubric]
- [x] CHK019 Is the license requirement stated (crate MIT; bundled database source public-domain) with the deny.toml/NOTICE handling captured as a decision? [Completeness, Spec Assumptions, Constitution Licensing]
- [x] CHK020 Is the requirement to add a row to the AGENTS.md dependency inventory (Crate | Kind | Added by | Why) present, with Kind marked runtime, optional? [Traceability, AGENTS.md]

## MSRV

- [x] CHK021 Is the MSRV floor (1.82) stated as a hard gate, with the requirement to verify it via the project's minimum-toolchain build (cargo xtask msrv through rustup run 1.82)? [Measurability, Spec §DONE WHEN, SC-005]
- [x] CHK022 Is it required that the whole transitive set (including build-deps) be checked for any member declaring a rust-version above 1.82, and pinned if so? [Coverage, AGENTS.md rubric]
- [x] CHK023 Is a not-run minimum-toolchain check required to be treated as a failure (exit 2), never as a pass? [Clarity, Constitution Verification]

## Feature Gating & Dependency Direction (P-2)

- [x] CHK024 Is the requirement that the database capability is optional at build time (a `targets` feature) so a default build compiles neither the engine nor a C toolchain for it, stated measurably? [Clarity, Spec §FR-013, SC-005]
- [x] CHK025 Is it explicit that introducing the crate must not make fragcap-core depend on it or on the database dependency, keeping core to its existing allowlist? [Consistency, Spec §FR-014, Constitution P-2]
- [x] CHK026 Does the spec/plan require the dependency-direction allowlist (deps.rs EXPECTED edges and the sibling rule) to be updated to admit the new crate's edges and nothing more? [Completeness, Spec §DONE WHEN]
- [x] CHK027 Is the new crate's placement (depends only on fragcap-profile, is not depended on by any sibling) specified so the mechanical deps check can pass? [Clarity, Spec §DONE WHEN]

## Process Artifacts & Records

- [x] CHK028 Is the requirement to add changelog fragments (added + decisions) for this slice present, with the dependency addition recorded as a dated decision? [Traceability, Spec §DONE WHEN]
- [x] CHK029 Is the new publishable crate's obligation to carry LICENSE, NOTICE, and README (byte-checked by the license gate) captured as a requirement? [Gap, Constitution Licensing]

## Notes

- Check items off as completed: `[x]`
- These items test whether the REQUIREMENTS are complete and testable, not whether the build works.
- The governing rubric is the AGENTS.md "Dependency inventory" section and the constitution's Licensing and Verification sections.
