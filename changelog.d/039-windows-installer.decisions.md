**2026-08-14** Windows distribution gained an unsigned MSI installer, a bundled
barebones hint database, and a default hint-database location, and several
choices along the way are recorded (issue #96, decision #58).

The pinned release workflow (`.github/workflows/release.yml`) changed. The
`artifacts` job now installs WiX and `cargo-wix`, builds the barebones hint
database offline through `fragcap targets import`, and builds the MSI, and the
release now publishes three downloads with a checksum each: the portable archive
(which now also carries `hint.db`), the unsigned MSI, and the hint database on its
own. This extends the archive contract recorded on 2026-08-12
(`release-packaging.decisions.md`): the archive previously held only the binary,
`LICENSE`, and `NOTICE`, and now also holds the barebones hint database beside the
binary so the first-run bootstrap can seed the per-user copy from it.
Specification section 24.5 is reconciled to match, and sections 20.2 and 15.3 are
amended (the no-bundling obligation binds npcap alone, not fragcap's own data
artifacts; the hint database has a per-user default at `%APPDATA%\fragcap\hint.db`).

The installer is unsigned for this release. Code signing is tracked separately
(issue #79) and is a non-blocking track gated on a certificate path the project
does not yet have; withholding the installer until then would deny the whole
distribution improvement, so it ships unsigned and the documentation explains the
unrecognized-publisher warning and that verifying the checksum is the integrity
check (P-9). The installer carries a stable `UpgradeCode` GUID, generated once and
frozen for the life of the product line (`7F3A2C4E-1D9B-4A6E-9C58-2B0E7D4F6A13`,
in `crates/fragcap-cli/wix/main.wxs`); it is the identity by which a later version
replaces rather than duplicates an install, so it must never change.

The installer best-effort excludes its own install directory from Windows
Defender through a deferred, elevated custom action, removed on uninstall. The
exclusion is scoped to fragcap's own install directory and runs with the rights
the installer already holds; it opens no process handle and touches no target
process, its memory, its traffic, or the network stack, so it is outside the P-1
technique denylist. It is best-effort: Windows Tamper Protection or a disabled
Defender can refuse the change even when elevated, and a refusal must not fail the
install, so the action ignores its own failure. The install path is passed to
PowerShell as a bound parameter of a script block rather than interpolated into
the command text, so a directory name containing a quote or a subexpression is
data and cannot inject code that would then run elevated; a paired rollback action
removes the exclusion if a later install step fails, so a cancelled install leaves
none behind.

The hint database default is on. The MSI installs the database under a
program-files directory a standard user cannot write at runtime, but local
accumulation (slice S038) writes to the database on every run, so the live store
must be per-user. The binary therefore bootstraps a writable per-user default on
first run, copying a read-only template shipped beside the executable when present
and creating an empty store otherwise; one code path serves both the installer and
the portable archive, and a future non-empty template needs no code change. A
consequence is that a plain `fragcap run` with no hint-database option now creates
`%APPDATA%\fragcap\hint.db` and, through S038, learns launch data from the local
Steam appinfo cache by default (local only, no network, no process handle);
sharing that learned data remains a separate opt-in (issue #94).

The shipped hint database is empty. Every hint row is heuristic-unverified, and
shipping specific titles would bake staleness into the artifact and invite a
reader to treat a guess as a fact; the substrate grows from the user's own machine
(S038) and future community sync (#94), and the full curated corpus stays an
out-of-band maintainer artifact rather than this release file. The installer is
authored with `cargo-wix` over WiX v3 (mature, single `main.wxs`, `ProductVersion`
derived from the crate version at tag time); WiX v4/v5 as a dotnet tool was the
alternative and offered nothing this slice needs. The installed-MSI runtime
behavior (the unrecognized-publisher prompt, the per-machine install and path
change, the Defender exclusion, the npcap link, uninstall, and upgrade) is
verified manually and recorded on the pull request, the same honesty posture the
project holds for live capture, because it cannot be exercised by the automated
check set.
