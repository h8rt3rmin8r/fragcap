# Contract: Volume eligibility (the allowlist)

The cross-volume known-roots walk (FR-008) enumerates known roots on every
*eligible* fixed volume. Eligibility is a persistent, user-editable allowlist in
`local.db`, keyed on a stable volume identity. This is the layer-3 volume safety
of §7.4; deep scanning and the other §7.4 hazards are deferred (recorded below).

## Identity

A volume is keyed on a stable identity that survives drive-letter reassignment
(the volume GUID path; the volume serial is an acceptable fallback). The drive
letter is stored as a mutable display attribute only. Keying on the drive letter
is forbidden: a reassigned letter must not inherit a prior volume's eligibility.

## Lifecycle (FR-016a, clarified 2026-08-17)

1. **First run seeds permissively.** On the first discovery against an empty
   table, every fixed volume then present is recorded `eligible = true,
   reason = seeded-first-run`. Out-of-box discovery therefore walks the machine's
   existing fixed volumes (SC-001).
2. **Thereafter it is an allowlist.** A volume not recorded eligible is not
   walked. A fixed volume that first appears after seeding, or a mount that
   misreports as fixed, is *unseen*, hence not walked, until an explicit user
   opt-in records it `eligible = true, reason = user-added`.
3. **Exclusion is durable.** A user may record a volume `eligible = false,
   reason = user-excluded`; it is never walked until the user re-includes it. No
   automatic transition re-includes a volume.

## Guarantees

- An ineligible or unseen volume is enumerated zero times by any tier-2 walk
  (SC-003); the skip is counted `volume_skipped` in the account, never silent
  (FR-017).
- Each per-volume decision is statable: the row carries its `reason`, so "why was
  this volume walked / not walked" has a recorded answer.
- The table is `local.db` only; the catalog store leaves it empty.

## Deferred to v0.6.0 (recorded, not implemented; FR-018)

These §7.4 hazards are named here so a later slice does not rediscover them:

- **Cloud placeholder hydration.** A file with `FILE_ATTRIBUTE_RECALL_ON_OPEN` or
  `RECALL_ON_DATA_ACCESS` (OneDrive/cloud placeholders) must not be forced to
  hydrate by the walk. Deep scanning that would open such files is deferred.
- **Reparse-point loops.** Junctions/symlinks can form cycles; a deep recursive
  walk needs loop detection. The v0.5.0 walk is shallow (known roots, one level,
  stop-on-hit) and does not need it yet.
- **Within-volume skip list.** A per-volume list of directories to skip (system,
  temp, package caches) belongs with deep scanning, not the shallow known-roots
  walk.

Deep filesystem scanning is out of scope for this slice; the eligibility
machinery ships now only because the known-roots walk already crosses fixed
volumes.
