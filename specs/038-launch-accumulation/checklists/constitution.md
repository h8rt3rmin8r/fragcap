# Checklist: Constitutional and Correctness Requirements Quality

**Purpose**: Validate that the spec's requirements are written well enough to
guarantee the constitutional and correctness properties this slice depends on,
before planning. This tests the requirements, not the implementation.
**Created**: 2026-08-14
**Feature**: [spec.md](../spec.md)

## P-1 Passivity (no handle, no network, local read only)

- [ ] CHK001 - Is the sole permitted source of launch data stated unambiguously as the user's own local file, excluding any remote or shipped source? [Clarity, Spec FR-001]
- [ ] CHK002 - Is the prohibition on opening any network connection stated as a requirement rather than left implicit? [Completeness, Spec FR-012]
- [ ] CHK003 - Is the prohibition on opening any process handle stated explicitly, including that only a file already written by the platform is read? [Completeness, Spec FR-013]
- [ ] CHK004 - Are the passivity requirements written so they can be objectively verified without running a live capture? [Measurability, Spec SC-006]

## P-4 / P-9 Conservation and honesty

- [ ] CHK005 - Are the outcome categories for a considered application enumerated exhaustively and made mutually exclusive? [Completeness, Spec FR-007]
- [ ] CHK006 - Is the reconciliation identity (outcomes sum to applications considered) stated as a testable requirement? [Measurability, Spec FR-007, SC-004]
- [ ] CHK007 - Is the distinction between a parse failure and an application that simply yields nothing to write specified so the two cannot be conflated? [Clarity, Spec FR-008, FR-009]
- [ ] CHK008 - Is "stored verbatim" defined against concrete prohibited transformations (no reducing, reordering, normalizing, or filtering)? [Clarity, Spec FR-005]
- [ ] CHK009 - Is it specified that a single unparseable application must not abort the walk, so partial coverage is impossible to mistake for complete? [Completeness, Spec FR-008]
- [ ] CHK010 - Are progress-surfacing requirements defined well enough that a slow first run cannot be misread as a hang or as completion? [Clarity, Spec FR-010, SC assumptions]

## Zero new dependency / MSRV

- [ ] CHK011 - Is the zero-new-dependency constraint stated as a requirement with an objective check (no new lockfile entry)? [Measurability, Spec FR-014, SC-007]
- [ ] CHK012 - Is the requirement that the minimum supported toolchain and full check set stay green stated measurably? [Measurability, Spec SC-008]
- [ ] CHK013 - Is the "no net feature required or compiled" requirement distinct from and additional to "no network connection opened"? [Consistency, Spec FR-012]

## Store migration (first v1 to v2)

- [ ] CHK014 - Is the need for a schema-version migration stated explicitly, correcting the earlier no-migration expectation rather than leaving a contradiction? [Consistency, Spec Clarifications, Assumptions]
- [ ] CHK015 - Is the migration required to be additive and backward-safe for existing v1 stores (existing rows' new column left null)? [Completeness, Spec Clarifications]
- [ ] CHK016 - Is it specified that writing launch data must not disturb the public catalog or engine columns, nor the launcher-mediated or token-required attributes? [Completeness, Spec FR-006, FR-015, FR-015a]
- [ ] CHK017 - Is the staleness rule defined precisely enough (recorded change-number comparison) to be tested for both the skip and the refresh path? [Clarity, Spec FR-011a, SC-002, SC-003]

## Crate boundaries and scope

- [ ] CHK018 - Does the spec keep crate placement out of the requirements while the plan owns it, so no sibling-dependency rule is implied by the spec text? [Consistency, Spec Assumptions]
- [ ] CHK019 - Is the considered set bounded to the installed library (not every application in the cache), so scale and progress requirements are well-founded? [Clarity, Spec FR-002]
- [ ] CHK020 - Is the trigger condition for accumulation (only when a hint database is configured, writing to that same store) specified without ambiguity about a second store? [Clarity, Spec FR-011]

## Out-of-scope boundaries

- [ ] CHK021 - Is community pooling explicitly excluded and traced to its separate tracking item? [Completeness, Spec Assumptions]
- [ ] CHK022 - Are the excluded attributes (launcher-mediated, token-required) stated as deliberately unpopulated rather than forgotten? [Clarity, Spec FR-015, FR-015a]

## Notes

- Every item interrogates whether the requirement is written well (present,
  clear, consistent, measurable), not whether code behaves. Items marked against
  a spec section check an existing requirement; items citing Clarifications or
  Assumptions check a decision recorded during clarify.
