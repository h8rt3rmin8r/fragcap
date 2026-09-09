# S137 Research: Explicit Fresh-Start Uninstall

## Decision 1: Preserve S131 by default and make cleanup additive

S131 correctly classifies user databases, captures, profiles, bundles, and independently managed integration as user-owned state that ordinary uninstall preserves. S137 does not reclassify them as installer-owned. It adds a separately confirmed destructive operation whose authority exists only for that invocation. Repair, reinstall, upgrade, rollback, downgrade handling, automatic maintenance, and unopted uninstall remain byte-preserving.

## Decision 2: Keep the MSI thin

The installer already uses the safe WiX pattern of an immediate action that builds `CustomActionData` followed by a hidden deferred action. S137 reuses that pattern to invoke the installed executable. WiX supplies the explicit opt-in, removal-only condition, initiating user's `[AppDataFolder]` and `[LocalAppDataFolder]`, and report location. The executable owns validation, inventory, Deep Capture recovery, deletion, and truth reporting.

Duplicating recursive deletion in WiX, PowerShell, or RemoveFile tables was rejected because those paths cannot share Doctor recovery authority and tend to infer ownership from locations or names.

## Decision 3: Treat initiating-user paths as data, not environment

A per-machine uninstall may execute deferred work with elevated identity. Resolving `%APPDATA%` or `%LOCALAPPDATA%` inside that child could target LocalSystem or an administrator account rather than the person using the uninstall UI. The MSI therefore resolves both properties in the client context and passes exact paths as command arguments. The cleanup command canonicalizes and validates them as one profile pair before producing authority.

Direct current-user CLI use may resolve the same roots from the caller environment, but installer invocation never depends on the deferred process environment.

## Decision 4: Separate all-users preview from static MSI UI

Issue #377 requests a distinct all-users scope and an exact pre-deletion inventory. WiX v3 static dialogs cannot enumerate registry-backed profiles, capture a child process's dynamic output into controls, and bind confirmation to that output without a new native custom-action subsystem. A static all-users checkbox would therefore claim consent without showing the exact roots it authorizes.

S137 deliberately exposes all-users cleanup through the same product command as a separate administrator preview-and-confirm flow. Preview enumerates exact eligible profiles, prints every root and category, and emits a digest. Execution re-enumerates and refuses before deletion unless the digest matches. The MSI dialog identifies this distinct workflow and does not imply that its current-user checkbox covers other profiles. This deviation from a single-dialog interpretation is required by P-9 and the issue's stronger exact-inventory requirement.

Windows CurrentUser certificate trust is bound to the profile identity, not to the elevated administrator enumerating profiles. The all-users flow therefore retains another profile's Deep Capture sessions and reports a partial result instead of replaying Doctor under the wrong identity. The named user runs current-user fresh-start first; a new all-users preview can then remove the remaining canonical ordinary state. This is an explicit P-9 refusal, not a second recovery implementation.

## Decision 5: Bind confirmation to the complete inventory

The inventory identifier is BLAKE3 over a versioned canonical serialization containing scope, profile identity, root kind, canonical root, entry kind, relative path, file type, and observed metadata used by validation. Sorting makes equivalent inventories stable. Execution recomputes the inventory and uses constant-time comparison through the existing `subtle` dependency. A changed path, owner, link status, category, or entry requires a new preview.

A bare `--yes` is rejected for every scope because it authorizes a class of paths rather than the exact current inventory. The deferred MSI action previews the two displayed roots, extracts that inventory identifier, and passes it into the immediately following execution in the same transaction. Its hidden adapter only verifies that the supplied roots equal the impersonated initiating user's canonical roots; it does not bypass digest confirmation.

## Decision 6: Inventory without following redirections

Traversal uses link-aware metadata and never follows a symlink, junction, mount point, or other Windows reparse point. A redirected entry is retained and reported as refused. Roots must be absolute, non-root, distinct, non-overlapping, canonical descendants of the selected profile locations, and free from aliasing. Recursive deletion is implemented as the inverse of the accepted inventory rather than a broad `remove_dir_all` call.

This is intentionally stricter than relying on current standard-library junction behavior because a destructive boundary must remain stable across Windows versions and filesystem implementations.

## Decision 7: Recover Deep Capture effects before deleting authority

`doctor::fix` already owns exact session-owner classification, active-owner checks, recovery locking, journal replay, current-user trust removal by recorded thumbprint, refusal handling, replay verification, and owner-record retirement. S137 exposes a crate-private adapter to that implementation and does not interpret journals or certificate identity itself.

If recovery fails, the `sessions` entry and every associated owner record remain. Other independently owned ordinary categories may still be removed, but the report is partial and names the retained recovery root plus `fragcap doctor --fix` guidance. A complete result is impossible while any approved item remains.

## Decision 8: Use one versioned bounded report

Human preview is concise, but machine output and installer logs carry schema version, inventory identifier, scope, exact roots, categories, per-item outcomes, recovery outcomes, retained paths, overall status, and guidance. Paths are necessary for local remediation and stay in local installer logs or an explicitly chosen report. Payload content, database records, credentials, private keys, and captured traffic are never copied into the report.

## Decision 9: Test destructive behavior only in isolated roots

Rust tests construct scratch profile pairs and inject profile inventories. Windows package certification redirects the candidate executable to certification-owned roots and seeds every owned and excluded class. It never invokes cleanup against the runner's real `%APPDATA%`, `%LOCALAPPDATA%`, profile registry, trust store, or ordinary fragcap state. Real MSI execution remains hidden, finite, and non-interactive.

## Decision 10: Add no dependency or completion claim

Existing `blake3`, `subtle`, `serde_json`, and `windows-sys` capabilities are sufficient. S137 adds no package, schema migration, target instrumentation, persistent task, system-wide proxy action, release, or final Deep Capture completion language.
