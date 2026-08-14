# Feature Specification: exe FileVersion stamp and extcap scope flags (clear #104)

**Feature Branch**: `048-extcap-version-and-scope`

**Created**: 2026-08-14

**Status**: Draft

**Input**: GitHub issue #104. Its three acceptance criteria (register without a
manual copy; doctor reports the integration installed; not-registered guidance
names the supported mechanism) are already met by shipped slices #110, #114,
#121, #123. Two items from the issue body remain: the exe's PE FileVersion reads
`0.0.0.0` (never stamped), and `extcap install`/`uninstall` express scope only
through `--dir` rather than the `--user`/`--system` flags the issue's suggested
fix named. This slice does both, then #104 is closed.

## Clarifications

### Session 2026-08-14

Resolved from the issue, the approved plan, and the design research:

- Q: How is the FileVersion stamped without breaking MSRV 1.82? -> A: A Windows
  build-dependency `winresource` with `default-features = false` (drops its
  optional `toml`, which declares Rust 1.85), stamping the PE version resource in
  `crates/fragcap-cli/build.rs` from `CARGO_PKG_VERSION`. Verified under 1.82; if
  it fails, fall back to a hand-rolled `.rc` compiled via the SDK resource
  compiler (no dependency). Never ship an MSRV-breaking dependency.
- Q: On which builds is the resource stamped? -> A: Every Windows-MSVC build,
  independent of the `live` feature (the npcap linker args stay `live`-gated).
- Q: What do the scope flags mean, and what is the default? -> A: `--user`
  registers into the per-user Wireshark extcap directory, `--system` into the
  machine-wide one (reusing the existing `paths::extcap_dir()` /
  `paths::system_extcap_dir()`); `--dir` remains an explicit override. At most one
  of the three may be given. No flag defaults to per-user, preserving today's
  behavior.
- Q: Does the MSI change? -> A: No. It keeps `extcap install --dir
  "[WIRESHARK_DIR]\extcap"` for machine-wide, so no pinned artifact changes.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The binary reports a real version (Priority: P1)

A user who inspects `fragcap.exe` (via `Get-Command fragcap`, Explorer file
properties, or an inventory tool) sees the real version, matching what `fragcap
--version` prints, instead of `0.0.0.0`.

**Why this priority**: `0.0.0.0` is an outright-wrong metadata value (the #104
aside), mildly suspicious to some AV/inventory heuristics, and cheap to fix at the
build.

**Independent Test**: On a Windows-MSVC release build,
`(Get-Item target\release\fragcap.exe).VersionInfo.FileVersion` equals the crate
version (e.g. `0.3.0.0`), and `fragcap --version` still prints the same version.

**Acceptance Scenarios**:

1. **Given** a Windows-MSVC build, **When** the exe's PE FileVersion is read,
   **Then** it equals the crate version rather than `0.0.0.0`.
2. **Given** the crate version is bumped (one `[workspace.package] version`
   edit), **When** rebuilt, **Then** the stamped FileVersion tracks it with no
   other change.
3. **Given** a non-Windows or non-MSVC build, **When** compiled, **Then** the
   build succeeds unchanged (no resource step runs), and the 1.82 MSRV build
   compiles.

### User Story 2 - Registering the integration by scope (Priority: P2)

A user registers or unregisters the extcap integration by naming the scope:
`fragcap extcap install --user` (the per-user default) or `--system`
(machine-wide), without having to know and type the Wireshark extcap directory
path. `--dir <PATH>` still works as an explicit override.

**Why this priority**: It is the issue's suggested-fix ergonomics; the underlying
capability already exists via `--dir`, so this is a clarity/usability layer.

**Independent Test**: `extcap install --system` writes the binary into the
machine-wide directory (driven by `FRAGCAP_SYSTEM_EXTCAP_DIR` in tests); `--user`
into the per-user one; giving two scopes is a usage error.

**Acceptance Scenarios**:

1. **Given** `extcap install --system`, **When** run, **Then** the binary is
   registered into the machine-wide extcap directory.
2. **Given** `extcap install --user` (or no scope flag), **When** run, **Then**
   the binary is registered into the per-user directory (today's behavior).
3. **Given** `extcap install --user --system` (or `--dir` with a scope flag),
   **When** run, **Then** it is a usage error, not an ambiguous registration.
4. **Given** `extcap uninstall --system`, **When** run, **Then** it removes the
   machine-wide registration.

### Edge Cases

- The chosen scope's directory cannot be determined (`None`): the same
  "could not determine ... pass --dir" error the current `--dir`-less path emits.
- A dev machine without the Windows resource compiler: the build emits a
  `cargo:warning` and links unstamped rather than failing.
- The doctor guidance strings are unchanged, so no doctor golden regenerates.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: On Windows-MSVC builds, `fragcap.exe` MUST carry a PE FileVersion
  and ProductVersion stamped from `CARGO_PKG_VERSION` (the workspace version), so
  the exe no longer reports `0.0.0.0`.
- **FR-002**: The version resource MUST be added via a Windows-only
  build-dependency that compiles under the 1.82 MSRV and whose license is in the
  allowlist; `winresource` with `default-features = false` is the chosen crate
  (no `toml`, two MIT/Apache-2.0 packages). `Cargo.lock` MUST be updated and
  committed.
- **FR-003**: The version-resource step MUST run only on Windows-MSVC and MUST
  NOT break non-Windows builds, the `cargo xtask neutral` build, or the
  `cargo xtask msrv` (1.82) build. The npcap `/DELAYLOAD` linker args MUST remain
  gated on the `live` feature.
- **FR-004**: `extcap install` and `extcap uninstall` MUST accept `--user` and
  `--system` scope flags, resolving to `paths::extcap_dir()` and
  `paths::system_extcap_dir()` respectively; `--dir <PATH>` MUST remain an
  explicit override; at most one of `{--user, --system, --dir}` MUST be accepted
  (a clap conflict otherwise); no flag MUST default to per-user.
- **FR-005**: The MSI (`wix/main.wxs`) MUST be unchanged; `--dir` remains the
  mechanism it uses. No pinned artifact is modified by this slice.
- **FR-006**: The CLI reference docs MUST document `--user`/`--system` and show
  `--system` as the machine-wide form, keeping `--dir` as the override. The doctor
  guidance is unchanged.
- **FR-007**: The `AGENTS.md` dependency inventory MUST gain a row for
  `winresource`; changelog fragments MUST be added. All edited text MUST be
  UTF-8, LF, and free of em and en dashes.

### Key Entities

- **Version resource**: the PE VERSIONINFO stamped into `fragcap.exe`, sourced
  from `CARGO_PKG_VERSION`.
- **Registration scope**: per-user vs machine-wide vs explicit directory; selected
  by `--user`/`--system`/`--dir` and resolved by the existing path helpers.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A Windows-MSVC release build's `fragcap.exe` FileVersion equals the
  crate version (not `0.0.0.0`), while `fragcap --version` is unchanged.
- **SC-002**: `extcap install --system` / `--user` register into the correct
  directories (unit/integration tested via the env override); a double scope is a
  usage error.
- **SC-003**: `cargo xtask ci` is green and `cargo xtask msrv` (1.82) compiles the
  new build-dependency; `cargo deny`/`cargo xtask license` accept it.
- **SC-004**: The `doctor-ready` golden and all existing extcap/doctor tests are
  unchanged/green.

## Assumptions

- The release and dev builds are Windows-MSVC; cross-compiling from a non-Windows
  host would skip the stamp (winresource is a `cfg(windows)` build-dep), which is
  acceptable since the project builds natively on Windows.
- `paths::system_extcap_dir()` and `FRAGCAP_SYSTEM_EXTCAP_DIR` (added S044) are the
  correct machine-wide resolver and test override.
- Adding a Cargo build-dependency is not a pinned-artifact change, so no dated
  decision is required; the dependency is recorded in the AGENTS.md inventory.
