# Data Model: Consistent User Storage

## StorageContext

The shared path functions resolve the current process token's Windows profile folder and append `.fragcap`. A current-account check rejects service accounts and an interactive session account mismatch. The root is resolved read-only. `FRAGCAP_STORAGE_ROOT` can select an explicit absolute whole-store path for isolated automation; individual path overrides keep their existing precedence.

## StorageLocation

Each Doctor storage observation has `name`, `selected`, `source`, `existence` (`absent`, `present`, or `unavailable`), `effective` when verified from an open handle, `redirected` as a nullable Boolean, and `reason`. An absent path has no asserted effective destination.

## LegacyRoot

A legacy root is selected from the current user's ordinary known folders or an exact product path beneath a bounded package-private `LocalCache` inventory. The transition refuses redirected path components and conflicting databases. Discovery never makes a root eligible for deletion by itself.

## TransitionRecord

Each copied database has a versioned record naming its source, source fingerprint, and `copied-source-retained` disposition. Authored profiles have a separate source fingerprint record. Retained session roots have append-only marker files. The database destination is published without replacement, then its record is written; an interrupted publication without a record refuses a later automatic retry and keeps the legacy source intact.

The database fingerprint covers SQLite schema, selected pragmas, and typed row values rather than physical DB or WAL bytes. Confirmed fresh-start writes `storage-transition-v1-fresh-start.json` in the parent `.fragcap` directory before removing its `roaming` and `local` children. That marker stops later default database and profile import from retained AppData roots; retained legacy sessions remain discoverable for recovery.

## Lifecycle

Read-only resolution -> exact legacy inventory -> conflict classification -> staged consistent copy -> destination verification -> atomic publication -> retained-source discovery. Any failure remains a named state with original data intact. Cleanup examines the same roots but only an explicitly owned, previewed root can be removed.
