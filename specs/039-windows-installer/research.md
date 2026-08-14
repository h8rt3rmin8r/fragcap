# Phase 0 Research: Windows installer and hint-database default

## Installer toolchain

**Decision**: Author the installer as a single WiX v3 `main.wxs`, built with
`cargo-wix`.

**Rationale**: `cargo-wix` is purpose-built for Rust binaries: it scaffolds and
builds a WiX v3 project, derives the MSI `ProductVersion` from the crate version
(so the tagged release version flows through with no scripting), and keeps a
single human-readable `main.wxs` in the repo. WiX v3 (candle/light) is the most
widely documented WiX generation and has stable `WixUI` dialog sets and the
`WixUtilExtension`/`WixShellExec` helpers this slice needs.

**Alternatives considered**:
- WiX v4/v5 as a dotnet tool: newer, but heavier setup on the runner and less
  aligned with `cargo-wix`'s defaults; no capability this slice needs is
  v4-only.
- NSIS / Inno Setup: produce EXE installers, not MSI; the operator specified an
  MSI (Add/Remove integration, per-machine semantics, PATH via the Windows
  Installer `Environment` table).
- Hand-rolled MSI via the `msi` crate: low-level, would reimplement dialog and
  component machinery WiX already provides.

**Runner availability**: WiX v3 is not reliably preinstalled on the
`windows-latest` GitHub runner. The release job installs it explicitly (for
example `choco install wixtoolset`) and installs `cargo-wix` before invoking
`cargo wix`. A missing toolchain is a build failure on the tagged run, never a
silent omission of the installer.

## DB-writability reconciliation

**Decision**: Ship the hint database as a read-only template beside the
executable; the binary bootstraps the writable per-user default
(`%APPDATA%\fragcap\hint.db`) on first `run` by copying the template when present
and creating an empty store otherwise.

**Rationale**: A per-machine install places program files under a directory a
standard user cannot write at runtime, but S038 accumulation writes into the
database on every `run`, so the live store must be per-user. Placing the template
beside the executable makes one bootstrap code path serve both the installer
(template under Program Files) and the portable archive (template beside the
unzipped exe). Making the shipped database empty means copy-template and
create-empty converge today, and a future populated template needs no code
change.

**Alternatives considered**:
- WiX per-user component writing straight into `%APPDATA%`: installer-only (the
  raw-exe/zip user gets nothing), and per-machine-to-per-user seeding through an
  HKCU keypath is fragile and user-scoped in awkward ways.
- Create-empty only (no template): makes the shipped database file decorative,
  and gives the loose-database download and the zip payload nothing to carry.
- Reading hints directly from the read-only Program Files copy: breaks
  accumulation, which must write.

## First-run bootstrap shape

**Decision**: A pure helper `ensure_default_hint_db(default, template)` returning
`io::Result<()>`: no-op if `default` exists; else create the parent directory and
either copy `template` (when `Some` and present) or `Store::open(default)` to
materialize an empty schema. The `run` command resolves the default only when no
explicit `--hint-db`/`FRAGCAP_HINT_DB` is given, tracks that provenance, and calls
the helper only for the defaulted path. Bootstrap failure warns (never fatal).

**Rationale**: Purity makes it tempdir-testable with no `current_exe` faking; the
provenance guard preserves the existing explicit-path semantics (an explicit
absent path stays a non-fatal no-op that never gets created; a present-but-corrupt
one still fails loudly in `build_resolver`). The template path is supplied by the
thin caller from `std::env::current_exe()`'s sibling, keeping the impure lookup
out of the tested unit.

**Alternatives considered**: bootstrapping inside `hint_db_path` (would entangle
pure resolution with filesystem side effects and fire for explicit paths too);
bootstrapping across `watch`/`tap` as well (out of scope this slice, recorded in
clarifications).

## Barebones database production

**Decision**: Commit `assets/hint-seed.json`, a `kind:"export"` document with an
empty `records` array, and build the shipped database at release time with
`fragcap targets import assets/hint-seed.json --db <hint.db>`.

**Rationale**: Reuses the existing offline import path (no new Rust, no network),
is deterministic and CI-friendly, and yields a valid current-schema store that the
existing export path round-trips. The single JSON file is the future switch to a
seeded database with no workflow or code change.

**At-rest single-file assumption**: `rusqlite` opens the store in the default
rollback-journal mode (not WAL), so once `targets import` opens and closes it the
database is a single file with no `-wal`/`-shm` side-car. Copying the template
with `fs::copy` is therefore safe. Spot-check during implementation that no
side-car is produced.

## Defender exclusion custom action

**Decision**: Two deferred, non-impersonated (elevated) WiX custom actions: on
install, `powershell.exe -NoProfile -NonInteractive -Command "Add-MpPreference
-ExclusionPath '<INSTALLDIR>'"`; on uninstall, the matching `Remove-MpPreference`.
Pass `INSTALLDIR` via `CustomActionData` (deferred actions cannot read properties
directly). `Return="ignore"` so a refusal does not fail the install.

**Rationale**: The exclusion is scoped to fragcap's own install directory and runs
with the installer's existing elevation, so it needs no new privilege and touches
no target process (P-1). Tamper Protection or a disabled Defender can reject
`Add-MpPreference` even when elevated; `Return="ignore"` keeps that best-effort.
Removing the exclusion on uninstall leaves no orphaned security change.

**Alternatives considered**: a signed binary custom action (out of scope, and
signing is #79); leaving the exclusion permanent on uninstall (rejected: orphaned
security state); failing the install on a refused exclusion (rejected: hostile on
locked-down machines).

## npcap link surfacing

**Decision**: Use `WixUI_InstallDir` with its optional exit-dialog checkbox:
`WIXUI_EXITDIALOGOPTIONALCHECKBOXTEXT` naming the npcap download page,
`WIXUI_EXITDIALOGOPTIONALCHECKBOX=1`, and a `WixShellExec` custom action opening
`https://npcap.com` when checked.

**Rationale**: Surfaces the one remaining prerequisite at the moment of success
without bundling, downloading, or installing the driver (P-1, no-bundling). The
`WixUI_InstallDir` dialog set also gives the license-acceptance and install-dir
screens appropriate for a per-machine admin installer.

**Alternatives considered**: a Start-menu URL shortcut (less discoverable);
auto-opening the page unconditionally (surprising); bundling the driver
(forbidden).

## Release artifact set and checksums

**Decision**: The `artifacts` job builds the empty `hint.db` and the MSI, adds
`hint.db` to the portable-zip stage beside the exe, copies a loose `hint.db` into
`dist/`, and broadens the checksum step from a `*.zip` filter to cover the zip,
the `.msi`, and the loose `.db`. The `release` job's `gh release create ... dist/*`
uploads all artifacts and their `.sha256` files unchanged.

**Rationale**: Realizes the operator's "three downloads, user chooses" with a
checksum per artifact (the integrity check that matters most for the unsigned
installer), while touching the pinned workflow minimally.

## Unsigned posture

**Decision**: Ship the installer unsigned for this release; document the
unrecognized-publisher / SmartScreen handling and that verifying the checksum is
the integrity check. Signing is issue #79, out of scope.

**Rationale**: Signing is non-blocking and gated on a certificate path the project
does not yet have; withholding the installer until then would deny the whole
distribution improvement. Honesty (P-9): the docs state it is unsigned rather than
implying otherwise.
