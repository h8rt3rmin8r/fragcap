# Contract: Fresh-Start Data Cleanup

## Current-user preview

`fragcap fresh-start --scope current-user --preview`

Prints schema version, current-user scope, the exact canonical roaming and local roots, category summary, per-entry eligibility, and an inventory identifier. It performs no recovery or deletion.

Installer invocation supplies `--roaming-root <exact-path>` and `--local-root <exact-path>` from the initiating MSI client. These adapter options are hidden from ordinary help and never inferred from the elevated child environment.

## Current-user execution

`fragcap fresh-start --scope current-user --confirm <inventory-id> --yes [--report <path>]`

Re-inventories both roots and refuses before deletion if the identifier changed. The MSI uses a hidden `--installer-confirmed` adapter instead of `--confirm`: its dialog already displays the same two exact roots, and the immediate action passes those roots unchanged to the deferred command in one uninstall transaction. That adapter is valid only for current-user scope with `--yes` and both explicit roots. A complete exit is zero; refused or partial cleanup is nonzero and retains an actionable report.

## All-users preview

`fragcap fresh-start --scope all-users --preview`

Requires administrator authority. Enumerates Windows profiles from the operating system profile authority, not by searching the filesystem. Prints each exact eligible or refused profile identity and canonical root. It performs no recovery or deletion.

## All-users execution

`fragcap fresh-start --scope all-users --confirm <inventory-id> --yes --report <path>`

Requires administrator authority, an exact current preview identifier, and a new local report path outside every cleanup root. It re-enumerates every profile and refuses all deletion when any inventory fact differs. Current-user execution never includes another profile. If a profile still has Deep Capture session evidence, all-users execution retains that profile's sessions and reports a partial result because an elevated process cannot safely act as that profile's Windows CurrentUser certificate authority. Run current-user cleanup in the named profile first, then repeat the all-users preview.

## Silent MSI properties

`msiexec /x <package-or-product> /qn FRAGCAP_FRESH_START=1 FRAGCAP_FRESH_START_SCOPE=current-user`

Both exact values are required. Missing, empty, differently cased, unknown, or all-users scope preserves data. All-users removal uses the direct preview-bound command before ordinary MSI uninstall.

## Ownership contract

The only implicit roots are canonical `%APPDATA%\fragcap` and `%LOCALAPPDATA%\fragcap` for an exact selected profile. Everything below an accepted root is fresh-start data, including captures and sensitive session evidence, except a redirection that is retained and refused. Paths from flags, environment overrides, exports, third-party tools, Wireshark configuration, Npcap, and independent extcap registration are outside authority.

## Recovery contract

Before removing current-user `sessions` or its owner registry, execution calls Doctor's shared exact recovery implementation. Active, ambiguous, unsupported, malformed, wrong-store, mismatched, or failed actions retain their evidence and make the result partial or refused. Cross-profile recovery is retained for execution in that profile's identity. No second recovery planner exists.

## Report contract

Schema version 1 records scope, inventory identifier, exact roots, categories, recovery and deletion outcomes, retained paths, overall status, and guidance. A direct-command destination must not already exist and is reserved outside the cleanup roots before deletion begins. The MSI may atomically replace only its own fixed temporary report after proving it is an ordinary writable file. The report is bounded and contains no payload content, database rows, credentials, secrets, private keys, or captured traffic.
