**2026-08-14** Added the optional Wireshark extcap registration to the Windows
installer (`crates/fragcap-cli/wix/main.wxs`, release-adjacent and pinned),
implementing the slice 041 decision D-4. Both scopes are offered: per-user by
default, machine-wide for administrators. Per-user is a deferred, user-impersonated
WiX custom action running the installed `fragcap.exe extcap install`, so the target
resolves to the installing user's profile rather than SYSTEM; machine-wide is a
non-impersonated action running `extcap install --dir <WiresharkDir>\extcap`, gated
on a registry search that detects Wireshark. Both mirror the existing
Defender-exclusion pattern (immediate action sets CustomActionData, deferred
WixQuietExec, `Return="ignore"` so a failure never fails the install, paired
rollback, and removal on uninstall) and add no new WiX extension, keeping the
release job's `cargo wix` invocation unchanged. No `fragcap` CLI surface changed;
the installer drives the already-shipped `extcap install` and `--dir`.
