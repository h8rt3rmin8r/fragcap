# Phase 1 Data Model: Steam integration and managed launch

All types live in `fragcap-steam`. None cross into `fragcap-core`, `fragcap-capture`, or
`fragcap-attr` (P-2, P-3). Fields are the conceptual shape; exact Rust spelling is settled
in implementation.

## SteamInstallation

The resolved Steam root and its libraries.

- `root: PathBuf` - the Steam install directory (from the registry).
- `libraries: Vec<SteamLibrary>` - every configured library, including the root library.

Derived by: registry read -> `libraryfolders.vdf` parse. Absent registry key ->
`SteamError::NotInstalled`.

## SteamLibrary

- `path: PathBuf` - the library root (contains `steamapps/`).

## InstalledTitle

The unit discovery yields and scaffolding consumes.

- `app_id: String` - the Steam application identifier. A string even when numeric, matching
  the profile schema's `game.app_id`.
- `name: String` - the human title name (from the manifest).
- `install_dir: PathBuf` - resolved absolute install directory
  (`<library>/steamapps/common/<installdir>`).

Uniqueness: `app_id`. Duplicate `app_id` across libraries -> first discovered wins, the
collision is reported (Edge Case). Malformed manifest -> reported and skipped, discovery of
the rest continues (FR-004).

## ExecutableImage

A candidate stage found by scanning an install directory.

- `path: PathBuf` - full path under the install directory.
- `file_name: String` - the basename (e.g. `eso64.exe`).
- `size: u64` - byte size, the client-selection tiebreak.

## StageProposal

A classified image, ready to render as a profile stage.

- `role: Launcher | Client` - from the heuristic.
- `image: ExecutableImage`
- `exe_pattern: String` - the `exe` match predicate (the basename).
- `path_disambiguator: Option<String>` - a `path_contains` value, set only when another
  proposal shares this basename (D7), so the emitted profile passes the ambiguous-image
  check.

Classification rule (FR-006): an image whose name or path carries a launcher-suggestive
token -> `Launcher`. Of the rest, the largest by `size` -> `Client`. Degenerate cases: no
non-launcher image -> the largest overall becomes `Client` with a weak-guess note; every
image launcher-tokened -> the largest is still promoted to `Client` so a client stage
always exists (Edge Cases).

## ScaffoldedProfile

- `app_id`, `name`, `platform = "steam"` -> the profile's `game` table.
- `stages: Vec<StageProposal>`
- `header_comment: String` - states the classification is heuristic and must be verified
  against an observed session (FR-007).
- Invariant: the rendered TOML parses cleanly through `fragcap_profile::Profile::parse`
  (FR-008). Enforced in-process before emission (D4); a scaffold that fails is a bug, not
  output.

Output sink: standard output (clarification).

## LaunchRequest

The managed-launch decision, produced in run assembly, consumed at the sequencing point.

- `app_id: String` - from `game.app_id`.
- `url: String` - `steam://run/<app_id>` (D5).
- Preconditions (validated before capture starts, FR-011): `game.platform == "steam"` and
  `game.app_id` present. Failure -> a named `CliError::usage` refusal, no capture.
- Sequencing (FR-010): issued only after `Watching` and open sinks.
- Platform: on non-Windows, refused as unsupported before capture (Edge Case).

## SteamError

Structured error for the crate's fallible surface.

- `NotInstalled` - no Steam registry entry / root not found.
- `TitleNotFound { app_id }` - app_id not present in any library (FR-009).
- `Vdf { path, position, detail }` - a manifest failed to parse (report-and-skip at the
  discovery layer).
- `UnsupportedPlatform` - a Windows-only operation called on a non-Windows build.
- `Io { path, source }` - a filesystem error reading metadata or scanning.

No variant carries a process handle, a captured packet, or an attribution - the crate holds
none of those (P-1, P-3).
