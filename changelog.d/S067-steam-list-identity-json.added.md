<!-- spec-impact: 16.3 -->
`fragcap steam list` now honors the global `--json` flag, which it previously
ignored for its own result even though it reached the diagnostic emitter.
`fragcap steam list --json` writes one newline-delimited record per installed
title to standard output: the app id, name, and install directory (which the
human table has never shown), plus the same handle, stable id, and row index
the human table carries when a title is registered, absent entirely rather
than null when it is not. Zero installed titles produces zero records rather
than a sentence describing the empty state, and enumeration warnings keep
reaching standard error through the emitter in either mode, matching the
`doctor --json` precedent.
