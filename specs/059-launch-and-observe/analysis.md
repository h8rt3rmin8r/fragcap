# Cross-Artifact Analysis: Launch-and-observe promotion (S059)

**Date**: 2026-08-18 | Non-destructive consistency check across spec.md, plan.md,
tasks.md, contracts, and the constitution. Gate: MUST pass, no CRITICAL/HIGH.

## Requirement -> task coverage

| Requirement | Covered by | Status |
| --- | --- | --- |
| FR-001 unresolved target capturable | T011, T016 | OK |
| FR-002 valid observe profile, not wildcard | T012 (validated by `Profile::parse`) | OK |
| FR-003 binds holder (self or child) | T012, T015, T016 | OK |
| FR-004 deterministic per-image aggregation | T002, T004, T008 | OK |
| FR-005 additive (no golden/counter change) | T003, T005, T006 | OK |
| FR-006 promote on observation | T013, T016 | OK |
| FR-007 leave on no observation | T013, T017 | OK |
| FR-008 shared seam; extcap no write-back | T010, T014 | OK |
| FR-009 no direct-exe launcher | boundary honored (no task adds one); plan Constitution Check | OK |
| FR-010 offline verifiable; Tier 2 labeled | T016, T017, T021 | OK |
| FR-011 glossary (3 terms) | T018 | OK |
| FR-012 spec 17.2/17.7 reconcile | T018 | OK |
| FR-013 ci green, no dep delta | T020, T022 | OK |

Every task maps to a requirement or is setup (T001) / polish (T020-T022). No orphan
tasks, no uncovered requirements.

## Consistency findings

- **C1 (resolved)** The observe-mode profile validates. Verified against
  `crates/fragcap-profile/src/validate.rs`: `descends_from_resolves` (client names the
  declared `launcher` role), `descends_from_is_acyclic` (launcher has no ancestor),
  `unique_roles` (launcher != client), the empty-predicate check (both stages carry a
  predicate), and `ambiguous_image_match` (only the launcher carries `exe`). No
  CRITICAL.
- **C2 (resolved)** The tally is golden-safe. `CaptureStats` is not blanket-Serialized;
  the JSON Lines and pcapng writers and `build_summary` read named fields only, so a
  new `BTreeMap` field cannot change any committed golden. Additive invariant is a
  dedicated core test (T005).
- **C3 (resolved)** No-observe exit is exit 0, matching US1 scenario 2. Confirmed by
  the existing `a_zero_packet_bound...` test's reasoning ("acquired a target, so it
  exits zero"). T017 acquires (launcher binds) but attributes nothing, so
  `dominant_holder` is `None` and no promotion runs. Distinct from the
  `an_acquisition_timeout...` exit-1 case (no process script).
- **C4 (resolved)** The child-holder path is offline-testable. The offline substrate
  drives a `--process-script` tree and a `--attr-script` ownership map through the
  `RoleStampingAttributor`, so a launcher+child fixture (T015) exercises
  `descends_from` binding without a live driver.
- **C5 (noted)** `CaptureStats::absorb` folds the tally; the drop counters
  (`buffer_dropped`/`sink_dropped`/`gate_dropped`) stay output-owned and untouched, so
  the conservation identity is unchanged (P-4).

## Constitution alignment

- P-1: observe-only; reads the already-attributed image; no handle/injection; live
  launch stays Steam-only (FR-009). PASS.
- P-4: additive tally, no drop-counter change. PASS.
- P-6: three glossary terms in the same change (T018). PASS.
- P-9: promote only on observation; no fabrication on no observation (FR-007, T017);
  Tier 2 `steam://run` labeled. PASS.
- P-11: spec 17.2/17.7 reconciled; `cargo xtask spec` gated (T018). PASS.
- Architecture: core field is std-only (`BTreeMap<Arc<str>, u64>`); no platform dep
  into core; `cargo xtask deps` unaffected. PASS.

## Verdict

No CRITICAL or HIGH findings. All requirements covered, all consistency risks
resolved against the real code. Ready for `/speckit-implement`.
