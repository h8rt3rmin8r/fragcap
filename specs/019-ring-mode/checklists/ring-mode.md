# Ring Mode Checklist: Ring mode and triggers

**Purpose**: Validate that the requirements for the rolling retained window, its
duration and size bounds, the terminating-trigger dump, the configuration
refusals, conservation accounting, and the terminology collision are complete,
clear, consistent, and measurable before planning.
**Created**: 2026-08-10
**Feature**: [spec.md](../spec.md)

## Retention Semantics

- [x] CHK001 Is the retained set defined as the most recent packets within the
  window, with the oldest evicted as new ones arrive, stated unambiguously rather
  than as a qualitative "rolling window"? [Clarity, Spec §FR-001]
- [x] CHK002 Is a duration window's origin specified exactly (measured back from
  the newest retained packet's capture instant, not from a wall clock or the stop
  instant)? [Clarity, Spec §FR-002, §Clarifications]
- [x] CHK003 Is a size window's unit pinned to a single, named quantity (captured
  length, matching `--max-bytes`) rather than left to encoded on-disk size?
  [Measurability, Spec §FR-002, §Clarifications]
- [x] CHK004 Is the degenerate case of a window smaller than one packet resolved
  so the dump is never empty when traffic was captured? [Completeness, Spec §Edge
  Cases]
- [x] CHK005 Is out-of-order arrival addressed so the retained set is always the
  recent tail by capture instant? [Edge case, Spec §Edge Cases]

## Trigger and Dump

- [x] CHK006 Is the dump specified to fire for every one of the six session stop
  conditions, by one shared path, rather than only on interrupt? [Completeness,
  Spec §FR-003, §SC-003]
- [x] CHK007 Is the dumped file's structure defined by an objective criterion
  (accepted in full by an unmodified pcapng parser: a Section Header Block, one
  Interface Description Block per declared interface, then the retained packets in
  capture order)? [Measurability, Spec §FR-004, §SC-001]
- [x] CHK008 Is the whole-input case (window larger than the input) required to
  equal a plain file capture's packet record sequence, giving a byte-level
  regression anchor? [Consistency, Spec §FR-012, §SC-002]
- [x] CHK009 Is an unwritable `--out` required to fail the same way a plain file
  capture's does, rather than being discovered only at dump time? [Edge case, Spec
  §Edge Cases]

## Configuration and Refusals

- [x] CHK010 Is ring mode required to refuse a missing `--out` and a missing
  `--ring` before capture starts, each naming the missing flag? [Completeness,
  Spec §FR-005, §SC-004]
- [x] CHK011 Is the combination of ring mode with a volume stop bound
  (`--max-bytes`/`--max-packets`) required to be refused, with the reason
  (a rolling window does not stop on volume) stated? [Consistency, Spec §FR-006]
- [x] CHK012 Is a ring window supplied without ring mode required to be refused,
  so it is never silently ignored? [Completeness, Spec §FR-007]
- [x] CHK013 Is mode resolution (command line over the profile `[capture]`
  default, and a profile `mode = "ring"`) specified so the refusals fire on the
  effective mode, not only an explicit `--mode`? [Clarity, Spec §FR-008]
- [x] CHK014 Is `--duration` explicitly retained as valid in ring mode, so the
  refusal set does not accidentally forbid the one bound that is meaningful?
  [Consistency, Spec §FR-010]

## Accounting

- [x] CHK015 Is a ring eviction explicitly excluded from capture-loss counting,
  with the pipeline conservation invariant required to hold for a ring capture as
  for a file capture? [Measurability, Spec §FR-009, §SC-005]
- [x] CHK016 Is the ring sink required to accept every delivered packet (return
  success), so the sink is never retired for its own retention decisions?
  [Consistency, Spec §FR-009]

## Terminology

- [x] CHK017 Is a glossary entry for ring mode required in the same change, and
  is it required to distinguish ring mode from the internal ring buffer of
  specification section 12.4 (constitution P-6)? [Consistency, Spec §FR-011]

## Notes

- Every item resolves against the current spec; none is outstanding. The checklist
  exists to keep the analyze gate anchored to the ring-specific risks (eviction
  correctness, dump validity, conservation, refusals, and the terminology
  collision) rather than only the generic requirements-quality set.
