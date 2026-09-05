# Release notes

GitHub release notes are short AI-written summaries of the release highlights. They are not excerpts, reordered entries, or copies of `CHANGELOG.md`.

## Preparation

1. Finish the release changelog and all release validation before writing the summary.
2. Ask an AI to synthesize only the most important user-visible outcomes from the complete release record.
3. Save the result as `release-notes/vX.Y.Z.md`, using `# Highlights`, a brief overview, a small highlight list, and the standard final link to the tagged changelog.
4. Run `cargo xtask notes X.Y.Z` before creating the release tag. The command fails when the file is absent, malformed, shaped like a changelog, longer than 1,400 characters, or longer than 12 non-empty lines.

The tag-triggered release workflow runs the same command and publishes its exact output. There is deliberately no fallback that extracts `CHANGELOG.md`.
