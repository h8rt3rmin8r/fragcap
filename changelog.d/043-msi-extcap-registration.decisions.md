**2026-08-14** Added the optional Wireshark extcap registration to the Windows
installer (`crates/fragcap-cli/wix/main.wxs`, release-adjacent and pinned),
implementing the slice 041 decision D-4. Both scopes are offered: per-user by
default, machine-wide for administrators. Per-user is a deferred, user-impersonated
WiX custom action running the installed `fragcap.exe extcap install`, so the target
resolves to the installing user's profile rather than SYSTEM; machine-wide is a
non-impersonated action running `extcap install --dir <WiresharkDir>\extcap`, gated
on a registry search that detects Wireshark. Both use the Defender-exclusion command
pattern (immediate action sets CustomActionData, deferred WixQuietExec,
`Return="ignore"` so a failure never fails the install) and add no new WiX
extension, keeping the release job's `cargo wix` invocation unchanged. No `fragcap`
CLI surface changed; the installer drives the already-shipped `extcap install` and
`--dir`.

Registration is deliberately forward-only: unlike the Defender exclusion, it has no
rollback and no unregister-on-uninstall. extcap registration is user-managed,
idempotent state (a user may register or unregister independently with `fragcap
extcap install` / `uninstall`), so an installer-owned undo would delete a
registration this install does not own, on uninstall, on a major upgrade, or on
rollback of an unrelated failure. Users unregister with `fragcap extcap uninstall`.
This reverses the "paired rollback and removal" half of D-4, which assumed the
Defender-exclusion symmetry applied; it does not, because registration state is
shared with the user and the CLI. `fragcap doctor` probes the per-user extcap
directory only, so it confirms the per-user scope; a machine-wide-only registration
is confirmed by Wireshark listing fragcap as a source. Teaching `doctor` to also
recognize the system extcap directory is a separate follow-up.
