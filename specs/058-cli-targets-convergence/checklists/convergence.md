# Checklist: CLI targets convergence requirements quality (S058)

**Purpose**: Unit-test the requirements for S058 (default `--db` + extcap-to-target convergence) before implementation.
**Created**: 2026-08-18
**Feature**: [spec.md](../spec.md)

## Requirement Completeness

- [ ] CHK001 - Is the exact set of subcommands that gain an optional `--db` enumerated (add/show/remove/export/import/list), and the ones deliberately excluded (discover's two-store pair) named? [Completeness, Spec §FR-001, FR-005]
- [ ] CHK002 - Is the default-store resolution order specified (explicit flag, then env override, then per-user application-data default)? [Completeness, Spec §FR-001, FR-002]
- [ ] CHK003 - Is the behavior for `add`/`import` against a defaulted, not-yet-created store specified? [Completeness, Spec §FR-004, Edge Cases]
- [ ] CHK004 - Are the extcap target inputs the convergence adds (selector + store-path overrides) and the renamed selection arg specified? [Completeness, Spec Clarifications, §FR-006]
- [ ] CHK005 - Is "no documentation page describes extcap capture as a legacy profile-file path" stated as a concrete removal (the S057 callout)? [Completeness, Spec §FR-011, SC-004]

## Requirement Clarity

- [ ] CHK006 - Is the no-resolvable-store failure specified precisely (a named error, not a panic or silent no-op) and distinguished from the `list` degrade-to-empty case? [Clarity, Spec §FR-003, Clarifications]
- [ ] CHK007 - Is "the same resolution `capture` uses" defined concretely enough to test (the shared seam resolving a selector to a synthesized profile)? [Clarity, Spec §FR-006, FR-009]
- [ ] CHK008 - Is the preserved extcap wire contract stated precisely (interfaces, link types, config block as arg lines, FIFO streaming unchanged; only the one selection arg's meaning changes)? [Clarity, Spec §FR-008]

## Requirement Consistency

- [ ] CHK009 - Do the `--db` default requirements use the same resolution mechanism the bare hero command and the `scan` variant already use (no new path helper)? [Consistency, Spec §FR-001, Assumptions]
- [ ] CHK010 - Is the extcap selector surface consistent with `capture`'s selector surface (handle/name/row-index/`--id`)? [Consistency, Spec §FR-006, Key Entities]
- [ ] CHK011 - Is the "single shared implementation, no duplicated body" requirement consistent with the fragcap-cli-only boundary (extraction stays within the crate)? [Consistency, Spec §FR-009, FR-010]

## Acceptance Criteria Quality

- [ ] CHK012 - Is each success criterion objectively verifiable (test result, handler inspection, grep for profile-file resolver, `Cargo.lock` diff, ci exit)? [Measurability, Spec §SC-001..SC-005]
- [ ] CHK013 - Is "no `Cargo.lock` delta / no new dependency" stated as a checkable constraint? [Measurability, Spec §FR-010, SC-005]

## Scenario & Edge Coverage

- [ ] CHK014 - Are the precedence (explicit `--db` wins) and the no-store-resolvable failure both covered as scenarios? [Coverage, Spec §US1, FR-002, FR-003]
- [ ] CHK015 - Is the parallel-test env-var race for `FRAGCAP_LOCAL_DB` called out so the added default-store tests isolate their paths? [Edge Case, Spec Edge Cases]
- [ ] CHK016 - Is the extcap "no profile-file resolution remains" requirement covered by both a behavioral test and a handler-inspection criterion? [Coverage, Spec §FR-007, SC-002]

## Dependencies, Assumptions, Governance

- [ ] CHK017 - Is the seam-extraction assumption (the named private functions moved, behavior preserved for `capture`) documented? [Assumption, Spec Assumptions, §FR-009]
- [ ] CHK018 - Is the forward dependency recorded (this slice's clean seam is what S059 extends)? [Dependency, Spec Dependencies]
- [ ] CHK019 - Is the spec/docs reconciliation gated (extcap section reconciled, `cargo xtask spec` lockstep, P-6 glossary for any new term)? [Governance, Spec §FR-011, FR-012]
- [ ] CHK020 - Is "`cargo xtask ci` green" stated as a hard completion gate, and is the fragcap-cli-only boundary explicit? [Completion, Spec §FR-010, FR-013, SC-005]

## Notes

- Items are requirements-quality tests, resolved by confirming the spec says enough, not by running the tool.
- The slice is expected to introduce no new user-facing term; CHK019 guards P-6 in case one appears (e.g. any label for the shared seam is internal, not user-facing).
