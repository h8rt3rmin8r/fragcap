# Requirements Quality Checklist: Consistent CLI Result Output

**Purpose**: Review S179's requirements before implementation.

## Completeness

- [x] CHK001 Does the spec cover the affected target, catalog, schema, Doctor, and Capture command families? [Completeness, Spec FR-003 through FR-008]
- [x] CHK002 Does the spec require a declared policy for every remaining accepted global-flag surface? [Completeness, Spec FR-009 and FR-011]
- [x] CHK003 Are authoritative JSON documents and extcap protocol outputs distinguished from ordinary command results? [Completeness, Spec FR-008]

## Clarity and Consistency

- [x] CHK004 Is precedence for JSON plus silent explicit and consistent across stdout and stderr? [Clarity, Spec FR-002 and FR-007]
- [x] CHK005 Is the difference between optional progress, ordinary results, required consent, and errors defined? [Clarity, Spec FR-006]
- [x] CHK006 Do structured diagnostic fields preserve the separate defects described by issue #488? [Consistency, Spec FR-005]
- [x] CHK007 Do JSON result records use observed values rather than re-parsed human text? [Consistency, Spec FR-003 and FR-004]

## Acceptance and Edge Cases

- [x] CHK008 Can the matrix prove exit code, channel, machine syntax, and human suppression for every flag pairing? [Measurability, Spec SC-001]
- [x] CHK009 Are empty results, no-match selections, ambiguous selections, and inaccessible inputs distinguished? [Coverage, Spec Edge Cases]
- [x] CHK010 Are write failures prevented from becoming false success claims? [Coverage, Spec Edge Cases]
- [x] CHK011 Are existing Doctor and Capture machine contracts protected by explicit compatibility assertions? [Coverage, Spec SC-004]
- [x] CHK012 Are privacy and exact JSON escaping requirements stated without altering observed data? [Coverage, Spec Edge Cases]

## Boundaries

- [x] CHK013 Is IGDB #155 excluded as an unrelated deferred feature? [Scope, Spec Assumptions]
- [x] CHK014 Does completion rely on controlled tests and CI rather than a real game or owner evidence? [Measurability, Spec SC-005]
