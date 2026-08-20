<!-- spec-impact: none -->
`--help` now wraps. Every one of the twenty-nine help pages previously emitted
at least one line wider than 100 columns, the worst at 449, because clap was
taken without its `wrap_help` feature and the wrapping function was compiled out
entirely; 82 over-long lines across the surface become none, continuation text
aligns under the description column, and a narrow terminal still shrinks. Help
also no longer prints internal development vocabulary: slice identifiers,
specification section numbers, appendix letters, build-feature names, and bare
tier numbers are gone from all fifteen pages that carried them, and where the
provenance is useful to a maintainer it moved to a comment clap does not
publish. Two help lines that described behavior the tool does not have are
corrected: `capture --launch` no longer reads as an instruction to pass a Steam
app id to `--target`, where a bare integer is always a listing row number, and
`targets list` now names the four columns it actually prints and says that it
registers newly discovered titles rather than only reading. A selector that
matches no row and looks like a number now says so: it names the interpretation
it used, how many rows the listing holds, and how to register a Steam app id
instead of guessing at it.
