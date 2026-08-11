### Added

- **Steam integration: library discovery, profile scaffolding, and managed
  launch** (specification section 16, roadmap slice S17). The `fragcap-steam`
  crate, previously a skeleton, now reads Steam's local installation metadata to
  enumerate installed titles, scaffolds a profile from one, and starts a title
  under capture. It contains no capture and no attribution logic, and
  `fragcap-core` gains no notion of Steam.
- **`fragcap steam profile <app_id>` scaffolds a validating profile.** It locates
  the Steam installation through its Windows registry entry, resolves the app_id
  to an installed title, scans the install directory for executable images,
  proposes launcher-suggestive images as launcher stages and the largest
  remaining image as the client, and prints a profile skeleton to standard
  output. The skeleton is built as TOML and parsed back through the section 15.4
  validator before it is emitted, so a scaffold that would fail validation is a
  bug caught in-process rather than shipped. A header comment states the
  classification is heuristic and must be verified against an observed session.
  This replaces the earlier `steam` stub.
- **`fragcap run --profile <ref> --launch` starts a title without the acquisition
  race.** The title is started through Steam's protocol handler
  (`steam://run/<app_id>`) only after the session is watching and the sinks are
  open, so every process in the launch chain, including a launcher shorter-lived
  than any poll interval, produces a start event fragcap observes. Managed launch
  requires `game.platform` and `game.app_id`; absent either, or on a non-Windows
  build, `--launch` is refused as a named configuration error before capture
  starts. This replaces the earlier "not yet supported" refusal.
- **A hand-rolled VDF parser** for the Valve key-value text format covers the
  subset `libraryfolders.vdf` and `appmanifest_*.acf` use. A malformed manifest
  is reported and skipped rather than aborting discovery of the well-formed ones,
  and a duplicate app_id across libraries keeps the first and reports the
  collision.
- **The integration opens no process handle.** Section 16.5 (environment
  inheritance), which would require a handle carrying memory-read rights, is
  deferred; it is a corroborating signal only, and section 10 ancestry already
  attributes reliably. The `OpenProcess`/`ReadProcessMemory`/`WriteProcessMemory`
  lint stays green.
