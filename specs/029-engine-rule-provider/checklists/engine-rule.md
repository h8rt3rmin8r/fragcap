# Engine-Rule Provider: Requirements Quality Checklist

**Purpose**: Validate that the S029 spec states the engine-rule provider's
requirements completely, clearly, and consistently before planning and
implementation. Each item tests the requirements text, not the eventual code.
**Created**: 2026-08-12
**Feature**: [spec.md](../spec.md)

## Heuristic Correctness (are the rules specified precisely enough to build?)

- [ ] CHK001 - Is the Unreal match signature specified as an exact, testable
      condition (filename suffix plus containing-directory convention) rather
      than an informal description? [Clarity, Spec §FR-002]
- [ ] CHK002 - Is it explicitly stated that the resolved target is the
      `*-Win64-Shipping.exe` child (the socket holder), not the root stub?
      [Clarity, Spec §FR-002]
- [ ] CHK003 - Are the Unity signature (`*_Data` directory plus `UnityPlayer.dll`)
      and the Ren'Py signature (`renpy` directory plus `.rpa` archives) each
      specified with the same precision as the Unreal rule? [Completeness,
      Spec §FR-008]
- [ ] CHK004 - Does the spec define what the provider resolves to for Unity and
      Ren'Py (which executable), not only how it recognizes them? [Gap,
      Spec §FR-008]
- [ ] CHK005 - Is case sensitivity of path/filename matching addressed given the
      Windows target filesystem? [Ambiguity, Spec Assumptions]

## Fidelity and Provenance Honesty (P-9)

- [ ] CHK006 - Is every engine-rule answer required to be stamped
      `heuristic-unverified` and explicitly forbidden from a higher tier?
      [Consistency, Spec §FR-003]
- [ ] CHK007 - Is the provenance source value fixed to `engine-rule` and tied to
      the value already named in the master schema and glossary? [Traceability,
      Spec §FR-003]
- [ ] CHK008 - Does the spec keep targeting fidelity (`heuristic-unverified`)
      distinct from attribution fidelity (`Live`/`Retained`/`None`) so the two
      axes are not conflated? [Consistency, Assumption]

## Passive Observation Guarantee (P-1)

- [ ] CHK009 - Does the spec require identification from filesystem inspection
      only, and explicitly forbid opening a process handle or reading process
      memory? [Coverage, Spec §FR-005]
- [ ] CHK010 - Does the spec explicitly exclude post-run artifacts (per-user
      AppData) and launcher tokens as inputs, with the pre-launch rationale
      stated? [Completeness, Spec §FR-005]
- [ ] CHK011 - Is it clear the provider never launches the target to identify it?
      [Clarity, Spec §FR-005]

## Determinism and Ambiguity Handling

- [ ] CHK012 - Is the outcome required to be independent of filesystem/collection
      iteration order? [Measurability, Spec §FR-006]
- [ ] CHK013 - Is rule evaluation order across engines specified as total and
      order-independent when a layout could match more than one rule?
      [Consistency, Spec §FR-006]
- [ ] CHK014 - For multiple matching candidate executables under one rule, does
      the spec state the provider declines and records the ambiguity rather than
      picking one? [Edge Case, Spec §FR-006]
- [ ] CHK015 - Is the fall-through-to-runtime-observation behavior on ambiguity
      stated as the intended cascade outcome? [Coverage, Spec Clarifications]

## Cascade Integration (S027 resolver)

- [ ] CHK016 - Is the provider required to occupy the engine-rule precedence
      position established by S027, below profile/hint and above platform
      walker/runtime observation? [Consistency, Spec §FR-001]
- [ ] CHK017 - Is it specified that an authored/verified profile answer always
      outranks the engine-rule answer for the same install? [Consistency,
      Spec §SC-004]
- [ ] CHK018 - Is the "no answer" contract (decline vs error vs fabricated
      target) defined for every non-match path? [Completeness, Spec §FR-004]
- [ ] CHK019 - Is the reason for declining or for an unreadable directory
      required to be observable rather than swallowed (no silent loss)? [Coverage,
      Spec §FR-009]

## Composition with the Platform Walker (S030)

- [ ] CHK020 - Is the provider's input contract (install root / launch entry
      point) specified so the S030 walker can feed it without modification?
      [Clarity, Spec §FR-007]
- [ ] CHK021 - Does the spec state the provider declines when its input is
      absent, keeping the cascade well-formed? [Edge Case, Spec Clarifications]

## Data Model (the resolved answer)

- [ ] CHK022 - Is the engine-rule target origin specified as distinct from the
      profile and observed origins, and its carried fields (resolved executable
      name/path plus match predicates) enumerated? [Completeness, Spec §Key
      Entities]

## Terminology (P-6 glossary-first)

- [ ] CHK023 - Does the spec require a full glossary definition of "engine rule"
      (currently only a named provenance example) landing in this slice?
      [Traceability, Spec §FR-010]
- [ ] CHK024 - Are the terms "engine rule", "engine-rule provider", and
      "resolved target (engine-rule origin)" used consistently without unlabeled
      synonyms? [Consistency, Spec §Key Entities]

## Acceptance Measurability

- [ ] CHK025 - Are the success criteria quantified (a sample of at least three
      distinct Unreal layouts; 100% decline on no-match) so acceptance is
      objectively verifiable? [Measurability, Spec §SC-001, §SC-002]
- [ ] CHK026 - Is fixture coverage (twin-exe directory trees in the TempTree
      spirit, plus a no-match tree) required by the acceptance criteria rather
      than left implicit? [Coverage, Spec §SC-001, §SC-003]
- [ ] CHK027 - Is the repository-gate pass (`cargo xtask ci`, including the
      fixture drift check) stated as a done condition? [Completeness,
      Spec §SC-005]

## Scope Boundary

- [ ] CHK028 - Does the spec clearly exclude Steam appinfo/PICS work, hint-DB
      data, and CLI changes from this slice? [Clarity, Spec §Clarifications]
- [ ] CHK029 - Is the Unreal-mandatory / Unity-Ren'Py-may-split boundary stated
      unambiguously so a reviewer knows the minimum acceptance? [Clarity,
      Spec §FR-008]
