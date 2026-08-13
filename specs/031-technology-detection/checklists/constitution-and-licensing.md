# Checklist: Constitution and Licensing Requirements Quality

**Purpose**: Validate that the spec's requirements for the constitution-sensitive
and third-party-vendoring surfaces are complete, clear, consistent, and
measurable before planning. This is a requirements-quality review (unit tests for
the English), not an implementation test.
**Created**: 2026-08-13
**Feature**: [spec.md](../spec.md)
**Focus**: P-1 passive observation, P-4 no-silent-loss, P-9 honest fidelity,
third-party vendoring correctness, master-schema conformance.

## P-1 Passive Observation

- [x] CHK001 - Is "path-only" reading specified precisely enough to exclude any
  file-content read (not just "avoid opening files")? [Clarity, Spec §FR-007]
- [x] CHK002 - Are the prohibited primitives (process handle, process-memory read,
  network access) each named as out of scope, rather than only "passive"?
  [Completeness, Spec §FR-008]
- [x] CHK003 - Is it specified that the detection surface does not interact with a
  detected anti-cheat in any way (detect, never hook/inject/query)? [Clarity, Spec
  §Clarifications, §FR-008]
- [x] CHK004 - Is the boundary between "labels technologies" and "names the socket
  holder" stated so detection cannot be read as a resolution path that might touch
  a process? [Consistency, Spec §FR-015]

## P-4 No Silent Loss

- [x] CHK005 - Is the behavior for a ruleset pattern that cannot compile specified
  as skip-count-surface, with no silent drop? [Completeness, Spec §FR-005]
- [x] CHK006 - Is a conservation property stated (compiled + skipped = total
  patterns) so under-counting is detectable? [Measurability, Spec §FR-006]
- [x] CHK007 - Is the skipped count required to be exposed to the caller (not only
  logged), and is the affected technology identifiable? [Clarity, Spec §FR-005,
  §US3]
- [x] CHK008 - Is an unreadable install directory (or subtree) required to be
  surfaced and distinguishable from a fully-scanned empty result? [Completeness,
  Spec §FR-010]
- [x] CHK009 - Is a partial scan explicitly forbidden from being reported as a
  complete empty result? [Consistency, Spec §FR-010]
- [x] CHK010 - Is the missing-or-hash-mismatched vendored file required to be a
  surfaced error rather than a silent fallback to zero rules? [Edge Case, Spec
  §Edge Cases]

## P-9 Honest Fidelity and Safety Signal

- [x] CHK011 - Is every finding required to carry the `heuristic-unverified`
  targeting fidelity, and is that fidelity distinguished from the attribution
  fidelity (Live/Retained/None)? [Clarity, Spec §FR-011, §FR-014]
- [x] CHK012 - Is anti-cheat detection framed as a user-safety/consent signal, not
  an evasion aid, in the requirements? [Consistency, Spec §Clarifications, §US1]
- [x] CHK013 - Is it specified that a finding names the actual marker path that
  matched (so the claim is auditable, not asserted)? [Measurability, Spec §FR-011]
- [x] CHK014 - Is deduplication defined (one report per technology per category,
  not per matching file) so the report neither inflates nor hides coverage?
  [Clarity, Spec §FR-011]

## Third-Party Vendoring Correctness

- [x] CHK015 - Is verbatim, unmodified vendoring from a pinned upstream commit
  required (no local edits to make patterns compile)? [Completeness, Spec §FR-001,
  §FR-005]
- [x] CHK016 - Are the NOTICE contents specified (MIT license text plus the
  SteamDB copyright attribution)? [Completeness, Spec §FR-002]
- [x] CHK017 - Are the lock record's fields enumerated (source repo, pinned
  commit, SPDX identifier, SHA-256 over vendored bytes)? [Completeness, Spec
  §FR-002]
- [x] CHK018 - Is the hash normalization specified as documented and reproducible
  (so the check is deterministic across platforms/line endings)? [Clarity, Spec
  §FR-002, §FR-003]
- [x] CHK019 - Is the hash mismatch required to be checkable in the repository
  gate (not only at runtime)? [Measurability, Spec §FR-003, §SC-003]
- [x] CHK020 - Is "no new runtime dependency and no new lockfile crate" stated as a
  hard requirement, and is MSRV-green named as a gate? [Consistency, Spec §FR-016,
  §SC-006]
- [x] CHK021 - Is the MIT dependency-license obligation reconciled with the
  Apache-2.0 project license (vendored data under MIT is permitted, attribution
  preserved)? [Assumption, Spec §FR-002]
- [x] CHK022 - Are the vendored asset, NOTICE, lock, and any gate step identified
  as pinned artifacts requiring a dated changelog decision? [Traceability, Spec
  §FR-018]

## Master-Schema Conformance

- [x] CHK023 - Is the `technologies` category vocabulary fully enumerated (engine,
  anti_cheat, sdk, framework, emulator, container, runtime, launcher)?
  [Completeness, Spec §FR-013]
- [x] CHK024 - Is the per-finding shape specified (category, name, marker path,
  fidelity) so schema conformance is testable? [Measurability, Spec §FR-013,
  §FR-014]
- [x] CHK025 - Is the reconciliation between the ruleset's sections and the schema
  categories documented (which categories the ruleset populates, which stay
  defined-but-empty)? [Consistency, Spec §Clarifications, §Assumptions]
- [x] CHK026 - Is the empty/absent `technologies` case defined (present-and-empty
  vs omitted) so a no-detection target is not malformed? [Edge Case, Spec §US2]
- [x] CHK027 - Is it specified that the packet-stream writers (pcapng/JSON Lines)
  stay unchanged, so "output metadata" is unambiguously the target artifact?
  [Clarity, Spec §FR-013a]

## Notes

- All items pass against the current spec: the operator's focus areas were already
  encoded as explicit FRs, clarifications, edge cases, and success criteria. The
  checklist is retained as the reviewable record that the constitution-sensitive
  surfaces were deliberately specified rather than assumed.
- The one item resting on an assumption rather than an explicit FR is CHK021
  (MIT-vendored-data under an Apache-2.0 project); the plan should confirm the
  license allowlist and per-crate license machinery accept a vendored MIT data
  asset with preserved attribution.
