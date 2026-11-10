# Data Model: Signed Direct Route Foundation

S199 adds package identity, not session or target state.

## WinDivert component identity

One immutable release row has upstream tag, selected binary-asset URL and SHA-256, corresponding-source URL and SHA-256, x64 DLL member path/hash/size, x64 driver member path/hash/size, driver signer thumbprint and exact installed filenames. Each field is required for acquisition or certification. Any drift stops the candidate.

## Package entries

The package contract's shared entry set grows from six to ten: the existing six; `WinDivert.dll`; `WinDivert64.sys`; `WinDivert-LICENSE.txt`; and `WinDivert-source.zip`. The two new executable-format entries have different signature requirements. The license and source entries are retained byte-for-byte from the pinned official archives. Both package formats carry the same bytes at the same relative paths.

## Readiness observation

Doctor's new component check is read-only. It reports `ready`, `absent`, `mismatch`, `invalid-signature`, `wrong-architecture`, `access-denied`, or `unavailable` as observed. A `ready` result means static package identity passed; it does not claim a successful driver open. `blocked` is reported only when a platform-observed block error exists in a later effectful route, not inferred from file checks.

## Lifecycle evidence

The existing package report records exact installed-file hashes across clean install, repair, same-version reinstall, upgrade and uninstall. Add component-specific signed-driver evidence and acknowledge that the service may persist until reboot after files are removed. No new installer action opens or removes the driver service.
