fragcap now ships an unsigned Windows MSI installer alongside the portable
archive. The installer places the binary per-machine, adds its directory to the
system path so `fragcap` resolves in any new terminal, ships the barebones hint
database beside the binary, best-effort excludes its own install directory from
Windows Defender (removed on uninstall), and links the npcap download page on
completion. It is unsigned by design for this release, so the documentation
explains the expected unrecognized-publisher warning and points at the checksum as
the integrity check; code signing is tracked separately.

Every release now publishes three downloads, each with its own checksum: the
portable archive, the installer, and a barebones targets hint database. The hint
database is an empty store that the local launch-data accumulation fills from the
user's own machine over time.

The hint database now has a per-user default location, `%APPDATA%\fragcap\hint.db`,
created on first `run` when no `--hint-db` option or `FRAGCAP_HINT_DB` environment
variable names one. On first use fragcap seeds it from the database shipped beside
the executable when present, and otherwise creates an empty store, so hint
resolution and local accumulation work with no configuration for both the
installer and the portable archive.
