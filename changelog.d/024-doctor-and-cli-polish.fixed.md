`fragcap doctor` now reports whether the live capture and socket-table
attribution backends are built into the binary, treats a missing live backend as
a blocking failure rather than reporting a misleading "ready", downgrades a
missing npcap loopback adapter to a warning (it is only needed with
`--loopback`), and shows the detected npcap version. `profile list` and
`profile validate` now honor `--json`, emitting one structured record per
diagnostic plus a summary instead of collapsing everything into one string.
`profile show` and `profile validate` now return the same exit code for a
reference that resolves to nothing. User-facing `--help` no longer exposes
internal roadmap identifiers or argument-parser implementation notes, and
`profile validate` no longer prints the profile path twice.
