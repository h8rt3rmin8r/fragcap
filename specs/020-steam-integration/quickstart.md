# Quickstart / Validation: Steam integration and managed launch

All scenarios below run offline (no Steam installed, no game, no capture driver, no
elevation) except the one explicitly marked tier-2/manual.

## Prerequisites

- The workspace builds: `cargo build --workspace`.
- The gate passes: `cargo xtask ci`.

## Tier-1 (offline, in CI)

### V1 - VDF parser

`cargo test -p fragcap-steam vdf`

Expected: well-formed nested-quoted-block manifests parse; a malformed manifest yields a
positioned error (not a panic, not a silent mis-parse); `//` comments and `\\`/`\"` escapes
are handled.

### V2 - Library discovery against a fixture tree

`cargo test -p fragcap-steam --test discovery`

A tempdir Steam root with `libraryfolders.vdf` pointing at two libraries, each holding
`appmanifest_*.acf` files. Expected: every installed title from both libraries is returned
with app_id and resolved install path; a malformed manifest is skipped and the rest survive;
a duplicate app_id resolves to the first discovered with the collision reported.

### V3 - Scaffolding classifier and self-validation

`cargo test -p fragcap-steam scaffold`

Expected: a launcher-token image is proposed as a launcher stage; the largest non-launcher
image is the client; two proposed stages that would share a basename get a `path_contains`
disambiguator; a degenerate scan still proposes a client. Every rendered scaffold parses
cleanly through `fragcap_profile::Profile::parse` - the guarantee behind FR-008/SC-001.

### V4 - Managed-launch config validation and URL

`cargo test -p fragcap-steam launch` and `cargo test -p fragcap-cli launch`

Expected: a profile without `game.platform` or without `game.app_id` is refused with a named
error before capture; a profile with both yields `LaunchRequest { url:
"steam://run/<app_id>" }`; the launch is sequenced after the watcher is armed (asserted
against the run assembly/ordering, not a live process).

### V5 - Neutral build

`cargo build -p fragcap --no-default-features` (and, where available,
`cargo xtask neutral`).

Expected: the workspace/facade builds with the Steam crate present; the pure Steam modules
compile; no Windows binding is pulled on the neutral target.

## Tier-2 (manual, NOT in CI)

### V6 - Live managed launch

On a Windows machine with Steam and an installed title, with a validated profile declaring
`game.platform = "steam"` and `game.app_id`:

`fragcap run --profile <ref> --launch --out capture.pcapng`

Expected: fragcap arms the watcher, opens the capture handle, then starts the title through
Steam; every process in the launch chain produces an observed start event. This is a manual
verification; it is never asserted as run in CI (P-9).

## End-state check

`cargo xtask ci` is green, and `fragcap steam profile <app_id>` on a machine with the title
installed prints a profile that `fragcap profile validate` accepts unedited.
