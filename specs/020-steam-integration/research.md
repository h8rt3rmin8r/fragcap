# Phase 0 Research: Steam integration and managed launch

All Technical Context unknowns are resolved. The autopilot decision log in
[plan.md](plan.md) supplied the calls; this file records the findings in the
decision / rationale / alternatives format.

## R1 - Registry and protocol-handler access mechanism

- **Decision**: reuse the workspace-shared `windows-sys` 0.36, adding the additive
  features `Win32_System_Registry` (Steam install-path read) and `Win32_UI_Shell`
  (`ShellExecuteW` for the `steam://` handler). No new package.
- **Rationale**: the repository has twice (S09, S10) chosen not to add a second Windows
  binding tree; additive features on the existing 0.36 pin add nothing to `Cargo.lock` and
  change no resolved version. `fragcap-attr` already carries the `unsafe` FFI pattern.
- **Alternatives**: `winreg` (spec crate table's choice) - more ergonomic, but pulls a
  second binding tree; rejected on the same grounds S10 rejected a second `windows-sys`.
  Reading the registry by shelling out to `reg query` - a wrapper that parses output,
  which P-7 forbids.

## R2 - Steam library layout and manifest formats

- **Decision**: install path from `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath`
  (fall back to `HKCU\SOFTWARE\Valve\Steam\SteamPath`). Libraries from
  `<steam>/steamapps/libraryfolders.vdf`. Per-title metadata from
  `<library>/steamapps/appmanifest_<app_id>.acf`, which carries `appid`, `name`, and
  `installdir` (relative to `<library>/steamapps/common/`).
- **Rationale**: this is the stable, documented on-disk layout; discovery is a bounded walk
  over these files.
- **Alternatives**: the Steamworks API (a runtime dependency and a running Steam client -
  heavier and against the no-bundle posture); rejected.

## R3 - VDF (Valve key-value text) parsing

- **Decision**: hand-rolled parser for the nested-quoted-block subset: `"key" "value"` and
  `"key" { ... }`, with `//` line comments and `\\`/`\"` escapes. Malformed input returns a
  positioned error; callers report-and-skip (FR-004).
- **Rationale**: the format is small and stable; spec 16.2 explicitly says it is not worth a
  dependency. The subset these two manifest kinds use is narrow.
- **Alternatives**: a VDF/keyvalues crate - a dependency larger than the parser it replaces,
  and another supply-chain surface for a format this stable; rejected. Binary VDF
  (`appinfo.vdf`) is out of scope - the manifests read here are text.

## R4 - Scaffold validity guarantee

- **Decision**: build the profile as TOML text, parse it back through
  `fragcap_profile::Profile::parse`, and emit only on success. FR-008/SC-001 hold by
  construction.
- **Rationale**: there is no profile serializer in `fragcap-profile` today, so the scaffold
  builds text regardless; round-tripping it through the real validator is nearly free and
  removes any chance of shipping an invalid skeleton.
- **Alternatives**: emit text and trust it - a P-9 risk (an untested scaffold could fail the
  validation FR-008 promises); rejected. Add a profile serializer to `fragcap-profile` -
  out of scope for this slice and unnecessary for a header-commented skeleton.

## R5 - Managed-launch sequencing point

- **Decision**: issue `ShellExecuteW(steam://run/<app_id>)` after the session reaches
  `Watching` (`session.attach()`, [session.rs:221](../../crates/fragcap/src/session.rs)) and
  after the sinks/capture handle are open. Config validation (platform == steam, app_id
  present) happens earlier, in run assembly, before capture starts.
- **Rationale**: launching only once the watcher is armed is precisely what removes the
  acquisition race (spec 16.4): every process in the chain then produces an observed start
  event, including a launcher shorter-lived than any poll interval.
- **Alternatives**: launch before arming (reintroduces the race - the whole point of the
  slice); rejected. `steam.exe -applaunch <id>` via `std::process::Command` (needs the Steam
  path resolved and a child process fragcap owns) - the protocol handler is what the spec
  names and needs no path; kept as fallback only.

## R6 - Non-Windows behavior and neutral build

- **Decision**: scope `windows-sys` under `[target.'cfg(windows)'.dependencies]`; gate the
  registry read and `ShellExecuteW` with `#[cfg(windows)]`; the non-Windows arm returns a
  structured "Steam integration is only supported on Windows" error. The VDF parser, the
  classifier, and URL construction are portable and unconditionally compiled/tested.
- **Rationale**: satisfies FR-014 and P-2 - the workspace and its neutral build stay green,
  and most of the slice's logic is still exercised on any CI host.
- **Alternatives**: gate the whole crate on `cfg(windows)` - breaks the facade/CLI neutral
  build and hides the portable logic from non-Windows CI; rejected (D2).

## R7 - Verification honesty

- **Decision**: tier-1 offline unit/integration tests cover VDF parsing, classification,
  scaffold self-validation, launch-config validation, and URL construction; a fixture Steam
  tree in a tempdir covers discovery. The live Steam launch is tier-2/manual and is not run
  in CI.
- **Rationale**: P-9 - a launch cannot be honestly asserted as executed on a runner with no
  Steam and no game. The observable, testable content is the decision, not the syscall.
- **Alternatives**: mock `ShellExecuteW` and claim a launch - tests the mock, not the
  behavior, and risks reading as a live-launch claim; rejected.
