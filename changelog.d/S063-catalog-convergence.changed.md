<!-- spec-impact: 15.7, 26.3 -->
Store paths are overrides everywhere, never requirements. Seven subcommands used
to refuse to run without an explicit filesystem path to a store fragcap
installs, manages, and already knows how to find, with nothing in the error
saying what path to type; the whole `catalog` namespace, `technologies`, and
`targets discover` now resolve the same store the rest of the tool does, through
an explicit flag if given, then the environment override, then the per-user
default, and every success line names the store it touched. The three seed verbs
are one: `fragcap catalog seed` takes a repeatable `--tier`, so the launch tier
needs no fourth verb, and a bare `catalog seed` fills every tier that has a
source and names every tier it skipped with the reason. `--from` requires
exactly one `--tier`, because the offline documents are bare JSON arrays that do
not say which tier they are and guessing would write the wrong columns silently.
