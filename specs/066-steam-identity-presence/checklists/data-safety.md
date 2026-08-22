# Requirements Checklist: Schema Migration Safety and Conservation

**Purpose**: Validate that the spec's requirements around the schema migration, the
discovery-account outcome paths, and no-silent-loss are complete, clear, and consistent
before planning proceeds.
**Created**: 2026-08-21
**Feature**: [spec.md](../spec.md)
**Depth**: Standard
**Audience**: Reviewer (pre-plan)
**Focus**: Schema/migration safety; discovery-account conservation (P-4); no-silent-loss

## Requirement Completeness

- [x] CHK001 - Are requirements defined for what happens to a target registered before
  this feature ships (no stored installdir/executable)? [Completeness, Spec §FR-017]
- [x] CHK002 - Are requirements defined for every new path that could omit a discovered
  item from registration (the Music-type exclusion)? [Completeness, Spec §FR-004, FR-018]
- [x] CHK003 - Is a requirement present for what happens when an app's install-directory
  type cannot be determined at all (no appinfo entry, unreadable cache)? [Completeness,
  Spec §Edge Cases]
- [ ] CHK004 - Are requirements defined for what happens if two different candidate
  Music-type exclusions and a genuine game share an ambiguous type signal (a title whose
  appinfo type is missing but not clearly Music)? [Gap]

## Requirement Clarity

- [x] CHK005 - Is "not registered as a capture target" (FR-004) unambiguous about which
  discovery-account outcome bucket absorbs the excluded item, so a reviewer can verify
  conservation without inferring it? [Clarity, Spec §Clarifications]
- [x] CHK006 - Is the missing-install-root detection method ("evaluated fresh at listing
  time", FR-005) explicit enough to rule out a cached or stored presence verdict?
  [Clarity, Spec §FR-005]
- [x] CHK007 - Is "byte-for-byte identical" (FR-009) unambiguous about which output modes
  (color, `NO_COLOR`, piped) it applies to? [Clarity, Spec §FR-009]

## Requirement Consistency

- [x] CHK008 - Do FR-004 (Music apps never produced as candidates) and FR-018 (every new
  discard path counted) agree on which account field absorbs the exclusion, with no
  second, conflicting bucket implied elsewhere in the spec? [Consistency, Spec §FR-004,
  FR-018]
- [x] CHK009 - Do FR-010 (registration never mutated by a presence check) and FR-005
  (presence derived fresh per listing) agree that presence is never persisted back onto
  the stored target entry? [Consistency, Spec §FR-005, FR-010]
- [x] CHK010 - Does FR-017 (pre-feature rows keep resolving by handle/name) avoid
  conflicting with FR-013's expanded selector matching (installdir/exe stem), given a
  pre-feature row has neither field recorded? [Consistency, Spec §FR-013, FR-017]

## Non-Functional Requirements (Data Integrity / No Silent Loss)

- [x] CHK011 - Is a requirement present asserting that every newly introduced discard,
  decline, or non-registration path remains counted so the discovery account's
  conservation invariant is not broken by this feature? [Coverage, Spec §FR-018]
- [x] CHK012 - Is a requirement present that a target's stored fields (display name,
  installdir, executable) are never reconstructed from one another, only recorded
  verbatim when observed? [Coverage, Spec §FR-012]
- [ ] CHK013 - Is there a requirement stating what a store opened by an older build (one
  that predates the new stored fields) must still be able to read without error? [Gap]

## Edge Case Coverage

- [x] CHK014 - Is the reappearance of a previously-missing install root (a reconnected
  drive) addressed, and does the spec confirm no stale verdict survives to the next
  listing? [Edge Case, Spec §Edge Cases]
- [x] CHK015 - Is the case where a selector token matches two different targets by two
  different stored name fields addressed, and does it route through the existing
  ambiguity handling rather than a new one? [Edge Case, Spec §Edge Cases]

## Ambiguities & Conflicts

- [ ] CHK016 - Is there a stated requirement (or an explicit non-requirement) for whether
  a schema migration failure (a corrupt or partially-migrated store) is distinguished from
  an ordinary open failure elsewhere in the CLI surface, or is that left entirely to
  existing store-open error handling? [Ambiguity, Gap]

## Notes

- CHK004, CHK013, and CHK016 are left unchecked deliberately: they surface real,
  low-probability gaps (an ambiguous appinfo type; forward-compatibility of an old build
  against a migrated store; a migration-specific failure mode) that this slice's `plan.md`
  should either address explicitly or document as an accepted, narrow limitation, rather
  than silently leaving the spec to imply an answer. None of the three is high-impact
  enough to block proceeding to `/speckit-plan` under autopilot: the store-open path
  already has a general failure contract (`CliError::failure`) that the migration inherits
  by construction (`ALTER TABLE` inside the same transactional `Store::open`), and the
  "ambiguous appinfo type" case is bounded by the FR-001 fallback (default to `common`).
