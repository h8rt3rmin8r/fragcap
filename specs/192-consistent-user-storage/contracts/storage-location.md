# Storage Location Contract

## Default paths

For a Windows user with verified profile path `P`, the persistent root is `P\.fragcap`. The current roaming data root is `P\.fragcap\roaming` and contains `catalog.db`, `local.db`, `profiles`, `sessions`, and the session registry under their existing relative names. The current local data root is `P\.fragcap\local` and contains `captures`. These names classify data but do not claim that Windows automatically roams the profile-root directory. Explicit database, profile, session, and bundle paths keep their current precedence and are never silently migrated.

`FRAGCAP_STORAGE_ROOT` selects an explicit absolute whole-store root for isolated automation. When it is present, the ordinary default transition is skipped. A relative value is refused.

## Observation

Doctor's human and JSON representations expose the configured path, source, existence, effective destination when opened, redirection state, and a stable reason when verification is unavailable. A selected string or successful write does not by itself establish effective placement. Session terminal results report the verified retained bundle path separately from the nominal selected path.

## Transition

The first default data operation inventories the exact current-user ordinary AppData roots and discoverable package-private Fragcap roots before creating new default databases. A single unambiguous source is copied through a consistent SQLite snapshot. Existing destination bytes are never replaced. Conflicts and active/unverifiable sources halt automatic transition with their paths and reason. Legacy session bundles remain in place and are indexed until an exact later migration preserves every recovery reference and recipient right.

## Cleanup

Retention and fresh-start use the same root inventory. A discoverable legacy root is not automatically cleanup eligible. Preview and execution bind exact current-user roots and exclude custom paths, other users, and unrelated package contents.
