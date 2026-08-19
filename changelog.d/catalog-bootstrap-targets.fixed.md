<!-- spec-impact: none -->

`fragcap targets` (and a bare `fragcap`) now seed the per-user catalog from the
catalog store shipped beside the executable on first run, so a fresh install
discovers and classifies your games immediately instead of listing nothing until
a capture happened to seed it. The first-run copy of the shipped catalog lived
only on the `capture` path, so the documented first command every new user runs
saw no catalog in the per-user location, skipped discovery, and showed an empty
list until the store was seeded by hand; both discovery entry points now resolve
and seed the catalog through one shared step so they cannot drift again.
