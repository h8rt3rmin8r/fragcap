A technology-detection surface (slice S031) reports the technologies present in a
game's install directory: its game engine, anti-cheat (EasyAntiCheat, BattlEye,
Vanguard, and the rest), SDK, emulator, container, and launcher. It is built on
the open SteamDB `SteamDatabase/FileDetectionRuleSets` ruleset (MIT, (c) 2021
SteamDB), which recognizes technologies from depot file paths alone. The whole
`rules.ini` is vendored verbatim, pinned to upstream commit
`243cf741921d2c8fd6b844f83831edf4692cf788`, carried with its MIT attribution
(`THIRD_PARTY_NOTICES.md`) and integrity-locked by a recorded SHA-256 over its
bytes (`rules.lock.json`), in the same spirit as `skills-lock.json`. Because
fragcap runs against a real install the operator already has on disk, the whole
ruleset applies; the depot-manifest license gate SteamDB itself faces is a
catalog-scale concern for titles nobody owns and does not apply here.

Detection reads directory entries and matches the ruleset's path regexes against
the relative paths it finds, using file names and relative paths only: it opens no
process handle, reads no process memory, reads no file content, launches nothing,
and makes no network call (P-1). A detected anti-cheat is surfaced as a
user-safety and consent signal, so an operator knows what watches a game before
capturing alongside it; fragcap detects it and never interacts with it. Every
finding is stamped `heuristic-unverified` and names the marker path that produced
it, as auditable evidence (P-9). The vendored ruleset is authored for a PCRE-style
engine and contains constructs the project's RE2-family regex engine cannot
compile (atomic groups); each such pattern is skipped, counted, and recorded with
the technology it belonged to, never silently dropped, and `compiled + skipped ==
total` is asserted over the vendored asset (P-4). An unreadable install subtree is
surfaced distinctly from a clean empty scan.

Findings surface two ways. A new `fragcap technologies --path <dir>` command
prints them grouped by category, with a heuristic banner and a note when ruleset
patterns were skipped as incompatible. And the Steam profile scaffold now carries
the detected set into the target artifact it materializes, as a new multi-category
`technologies` structure added to the master target schema (categories `engine`,
`anti_cheat`, `sdk`, `framework`, `emulator`, `container`, `runtime`, `launcher`),
each finding recording its category, name, marker path, and fidelity. This labels
technologies; it does not change which executable the resolver picks as the
socket-holding client, and it does not run inside the live capture loop or alter
the packet-stream output. The detection engine lives in `fragcap-profile` beside
the engine rule and adds no dependency (the existing regex engine matches, and a
hand-rolled SHA-256 locks the asset), so nothing is added to `Cargo.lock` and the
minimum supported toolchain stays green.
