`fragcap doctor` output is easier to read and now identifies itself: it opens
with the fragcap version and the paths to the running binary, the user profile
directory, and the hint database, separates its sections with blank lines,
colors the status flags when writing to a terminal, and wraps long lines to a
normal terminal width. Redirected output, output with `NO_COLOR` set, and the
`--json` form stay plain.
