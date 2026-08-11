# Contract: `fragcap-steam` crate API

Public surface. Signatures are conceptual; exact spelling settles in implementation. The
crate contains no capture and no attribution logic (P-3).

## `fn discover() -> Result<SteamInstallation, SteamError>`

Locate Steam via the registry, read `libraryfolders.vdf`, read every
`appmanifest_*.acf` across every library, and return the installation with its installed
titles.

- Malformed individual manifests are reported and skipped, not fatal (FR-004).
- No Steam registry entry -> `SteamError::NotInstalled`.
- Non-Windows -> `SteamError::UnsupportedPlatform`.

## `fn scaffold(title: &InstalledTitle) -> Result<String, SteamError>`

Scan the title's install directory, classify executables, render a profile skeleton, and
return it as a validated TOML string.

- The returned string parses cleanly through `fragcap_profile::Profile::parse` (FR-008);
  an internal validation failure is a bug surfaced as an error, never returned as output.
- Portable: the classifier and renderer do not touch the OS beyond reading the directory,
  so this is unit-testable on any host against a fixture directory tree.

## `fn launch_request(profile: &Profile) -> Result<LaunchRequest, SteamError>`

Validate managed-launch preconditions against a loaded profile and construct the request.

- `game.platform != "steam"` or missing `game.app_id` -> a typed refusal the CLI renders as
  a named usage error before capture starts (FR-011). Portable and unit-tested.

## `fn launch(request: &LaunchRequest) -> Result<(), SteamError>`  *(cfg(windows))*

Issue the `steam://run/<app_id>` protocol handler via `ShellExecuteW`. Called only after
the session is `Watching` and the sinks are open (FR-010). Windows-only; the non-Windows
build exposes a stub returning `SteamError::UnsupportedPlatform`. Tier-2/manual - not
asserted as run in CI (D5).

## `mod vdf`

`fn parse(input: &str) -> Result<VdfValue, VdfError>` for the nested-quoted-block subset.
Portable, hand-rolled, unit-tested (well-formed, malformed-position, escapes, comments,
nested blocks). `VdfError` carries a byte position.

## Types

Re-exported per [data-model.md](../data-model.md): `SteamInstallation`, `SteamLibrary`,
`InstalledTitle`, `ExecutableImage`, `StageProposal`, `ScaffoldedProfile`, `LaunchRequest`,
`SteamError`.
